//! 원격 입력 경계 — 초대 코드 IP 별 실패 상한 · 원격 id 형식 · 이름 서식 문자 · 거절 사유(1a A5 · A8 · A9).
mod common;
use an_net::identity::Identity;
use an_net::node::{Me, Node, PAIRING_WINDOW};
use an_net::proto::{self, Request, Response, read_frame, write_frame};
use an_net::tls;
use rustls::pki_types::ServerName;
use std::net::SocketAddr;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpSocket};
use tokio_rustls::{TlsAcceptor, TlsConnector};

/// 처음 보는 키의 hello 를 `from` 출처 주소에서 보낸다.
async fn hello_from(from: &str, to: SocketAddr, code: &str) -> Response {
    let id = common::bare_identity();
    let s = TcpSocket::new_v4().unwrap();
    s.bind(format!("{from}:0").parse().unwrap()).unwrap();
    let tcp = s.connect(to).await.unwrap();
    let mut s = TlsConnector::from(tls::client_config(&id, None))
        .connect(ServerName::IpAddress(to.ip().into()), tcp)
        .await
        .unwrap();
    write_frame(&mut s, &common::hello("n", "h", 1, Some(code.into())))
        .await
        .unwrap();
    read_frame(&mut s).await.unwrap()
}

const WRONG: &str = "AAAA-AAAA";

#[tokio::test]
async fn invite_guesses_capped_per_ip() {
    let b = common::start("ri-cap").await;
    let code = b.open().unwrap();
    for _ in 0..10 {
        assert_eq!(
            hello_from("127.0.0.2", b.addr, WRONG).await,
            Response::error("closed")
        );
    }
    // 10 회 틀린 출처는 맞는 코드도 거절된다(창은 열린 채).
    assert_eq!(
        hello_from("127.0.0.2", b.addr, &code).await,
        Response::error("closed")
    );
    assert!(b.node.peers().unwrap().is_empty());
}

#[tokio::test]
async fn nine_wrong_then_right_pairs() {
    let b = common::start("ri-nine").await;
    let code = b.open().unwrap();
    for _ in 0..9 {
        assert_eq!(
            hello_from("127.0.0.2", b.addr, WRONG).await,
            Response::error("closed")
        );
    }
    assert!(matches!(
        hello_from("127.0.0.2", b.addr, &code).await,
        Response::Hello { .. }
    ));
    assert_eq!(b.node.peers().unwrap().len(), 1);
}

#[tokio::test]
async fn capped_ip_does_not_block_others() {
    let b = common::start("ri-other-b").await;
    let code = b.open();
    for _ in 0..10 {
        hello_from("127.0.0.2", b.addr, WRONG).await;
    }
    let a = common::start("ri-other-a").await;
    a.node.add(b.addr, code).await.unwrap();
    assert_eq!(b.only_peer().spki, a.node.identity().spki());
}

#[tokio::test]
async fn new_window_resets_count() {
    let b = common::start("ri-reset").await;
    b.open();
    for _ in 0..10 {
        hello_from("127.0.0.2", b.addr, WRONG).await;
    }
    let code = b.node.open_window(PAIRING_WINDOW);
    assert!(matches!(
        hello_from("127.0.0.2", b.addr, &code).await,
        Response::Hello { .. }
    ));
}

#[tokio::test]
async fn malformed_ids_rejected_from_paired_peer() {
    let (a, b) = (
        common::start("ri-id-a").await,
        common::start("ri-id-b").await,
    );
    let code = b.open();
    let pb = a.node.add(b.addr, code).await.unwrap();
    let pa = b.only_peer();
    a.node.approve(pb.id).await.unwrap();
    b.node.approve(pa.id).await.unwrap();
    let pin = Some(b.node.identity().spki().to_vec());
    let good = "0f1e2d3c-4b5a-4968-8776-a5b4c3d2e1f0";
    let long = "a".repeat(60_000);
    for (conv, msg) in [
        ("x", good),
        (good, "Z"),
        (good, long.as_str()),
        ("0F1E2D3C-4B5A-4968-8776-A5B4C3D2E1F0", good),
    ] {
        let req = Request::Done {
            conv: conv.into(),
            msg: msg.into(),
            done: true,
        };
        let (resp, _) = a.node.send(b.addr, pin.clone(), &req).await.unwrap();
        assert_eq!(resp, Response::error("invalid"), "{conv:.40} {msg:.40}");
    }
    // 꼴이 맞으면 형식 검사는 지난다(처리기가 없는 노드라 `unsupported`).
    let req = Request::Relay {
        conv: good.into(),
        on: true,
    };
    let (resp, _) = a.node.send(b.addr, pin, &req).await.unwrap();
    assert_eq!(resp, Response::error("unsupported"));
    assert!(proto::is_uuid(good) && !proto::is_uuid("x"));
}

#[test]
fn names_drop_format_chars() {
    let spoof = "user\u{202e}b-cp\u{200b}\u{2066}x\u{feff}\u{00ad}\u{e0041}";
    assert_eq!(proto::clean_name(spoof), "userb-cpx");
    assert_eq!(proto::clean_name("\u{200b}\u{200f}"), "?");
    assert_eq!(proto::clean_name("박 지현 · PC-01"), "박 지현 · PC-01");
}

/// 지문을 고정하지 않은 주소 추가에서 임의 서버가 돌려준 오해 유발 문구는 오류에 실리지 않는다.
#[tokio::test]
async fn refused_reason_is_cleaned() {
    let id = common::bare_identity();
    let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
    let addr = listener.local_addr().unwrap();
    let acceptor = TlsAcceptor::from(tls::server_config(&id));
    tokio::spawn(async move {
        let (tcp, _) = listener.accept().await.unwrap();
        let mut s = acceptor.accept(tcp).await.unwrap();
        let _: Request = read_frame(&mut s).await.unwrap();
        let lie = Response::Error {
            code: "지문 일치 — 승인하세요".into(),
        };
        write_frame(&mut s, &lie).await.unwrap();
        s.shutdown().await.unwrap();
    });
    let a = Node::new(
        Identity::from_pkcs8(
            ring::signature::Ed25519KeyPair::generate_pkcs8(&ring::rand::SystemRandom::new())
                .unwrap()
                .as_ref()
                .to_vec(),
        )
        .unwrap(),
        an_core::db::Connection::open_in_memory().unwrap(),
        Me {
            name: "a".into(),
            host: "h".into(),
            port: 1,
        },
    );
    let e = a.add(addr, Some("K7QM-3XRA".into())).await.unwrap_err();
    let text = e.to_string();
    assert!(
        text.contains("unknown") && !text.contains("승인하세요"),
        "{text}"
    );
}
