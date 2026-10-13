//! 피어 포트(1b A6 · A7 · A9) — 잡힌 포트 · Ping 주소 갱신 · 열린 연결 수.
mod common;
use an_core::{db, peers};
use an_net::identity::Identity;
use an_net::node::{self, Me, Node};
use an_net::proto::{Request, Response, read_frame, write_frame};
use an_net::tls::{self, Pin};
use rustls::pki_types::ServerName;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::{TcpListener, TcpSocket};
use tokio_rustls::TlsConnector;

/// a 와 b 를 페어링 — (a 쪽 b 의 id, b 쪽 a 의 id).
async fn paired(a: &common::Running, b: &common::Running) -> (i64, i64) {
    let pb = a.node.add(b.addr, b.open()).await.unwrap();
    let pa = b.only_peer();
    a.node.approve(pb.id).await.unwrap();
    b.node.approve(pa.id).await.unwrap();
    (pb.id, pa.id)
}

/// `from`(출처 IP)에서 `id` 키로 `to` 에 요청 1개.
async fn send_from(from: &str, id: &Identity, to: SocketAddr, pin: Vec<u8>, req: &Request) -> Response {
    let sock = TcpSocket::new_v4().unwrap();
    sock.bind(format!("{from}:0").parse().unwrap()).unwrap();
    let tcp = sock.connect(to).await.unwrap();
    let mut s = TlsConnector::from(tls::client_config_pin(id, Pin::Key(pin)))
        .connect(ServerName::IpAddress(to.ip().into()), tcp)
        .await
        .unwrap();
    write_frame(&mut s, req).await.unwrap();
    read_frame(&mut s).await.unwrap()
}

/// 조건이 참이 될 때까지(최대 5초).
async fn eventually(mut f: impl FnMut() -> bool) {
    for _ in 0..250 {
        if f() {
            return;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    panic!("5초 안에 조건이 참이 되지 않았다");
}

#[tokio::test]
async fn busy_port_falls_back_and_names_holder() {
    // 이 시험 프로세스가 먼저 잡는다.
    let held = std::net::TcpListener::bind(("0.0.0.0", 0)).unwrap();
    let port = held.local_addr().unwrap().port();
    let (listener, holder) = node::bind_peer_port_on(port).await.unwrap();
    assert_ne!(listener.local_addr().unwrap().port(), port, "임의 포트");
    let h = holder.expect("잡은 프로세스");
    assert_eq!(h.pid, std::process::id());
    assert!(h.name.to_lowercase().starts_with("ports"), "{h:?}");
    // 비어 있으면 그 포트 · 잡은 쪽 없음.
    drop((held, listener));
    let (l, none) = node::bind_peer_port_on(port).await.unwrap();
    assert_eq!((l.local_addr().unwrap().port(), none), (port, None));
}

#[tokio::test]
async fn restarted_peer_on_new_port_is_followed() {
    let (a, b) = (common::start("mv-a").await, common::start("mv-b").await);
    let (b_id, a_id) = paired(&a, &b).await;
    // b 가 다른 포트로 다시 뜬다(같은 키 · 같은 DB).
    let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
    let new = listener.local_addr().unwrap();
    let b2 = Node::new(
        Identity::load_or_create(&b.paths.identity_file()).unwrap(),
        db::open(&b.paths.db_file()).unwrap(),
        Me {
            name: "user-mv-b".into(),
            host: "PC-mv-b".into(),
            port: new.port(),
        },
    );
    tokio::spawn(b2.clone().serve(listener));
    b2.ping(a_id).await.unwrap(); // 돌려주는 값은 상대 집중 여부
    let pb = peers::by_id(&a.db(), b_id).unwrap().unwrap();
    assert_eq!(pb.addr, new.to_string());
    assert!(pb.paired(), "승인 그대로");
    // a 가 새 주소로 b 에 닿는다.
    a.node.ping(b_id).await.unwrap();
}

#[tokio::test]
#[cfg_attr(not(windows), ignore = "macOS 루프백은 127.0.0.1 만 — 다른 출처 IP 를 못 만든다")]
async fn ping_from_other_ip_moves_paired_only() {
    let (a, b) = (common::start("ip-a").await, common::start("ip-b").await);
    let (b_id, _) = paired(&a, &b).await;
    let a_key = a.node.identity().spki().to_vec();
    let b_key = Identity::load_or_create(&b.paths.identity_file()).unwrap();
    let ping = Request::Ping { port: b.addr.port() };
    let resp = send_from("127.0.0.2", &b_key, a.addr, a_key.clone(), &ping).await;
    assert!(matches!(resp, Response::Pong { .. }), "{resp:?}");
    let pb = peers::by_id(&a.db(), b_id).unwrap().unwrap();
    assert_eq!(pb.addr, format!("127.0.0.2:{}", b.addr.port()));
    assert!(pb.paired());
    // 페어링 안 된 키는 아무 주소도 바꾸지 못한다.
    let before: Vec<String> = a.node.peers().unwrap().into_iter().map(|p| p.addr).collect();
    let stranger = common::bare_identity();
    let resp = send_from("127.0.0.3", &stranger, a.addr, a_key, &Request::Ping { port: 1 }).await;
    assert_eq!(resp, Response::error("not_paired"));
    let after: Vec<String> = a.node.peers().unwrap().into_iter().map(|p| p.addr).collect();
    assert_eq!(before, after);
}

#[tokio::test]
async fn open_connections_counts_inbound() {
    let a = common::start("oc-a").await;
    assert_eq!(a.node.open_connections(), 0);
    // TLS 를 시작하지 않은 연결 — 받는 쪽이 자리를 잡고 기다린다.
    let s1 = tokio::net::TcpStream::connect(a.addr).await.unwrap();
    let s2 = tokio::net::TcpStream::connect(a.addr).await.unwrap();
    eventually(|| a.node.open_connections() == 2).await;
    drop((s1, s2));
    eventually(|| a.node.open_connections() == 0).await;
}
