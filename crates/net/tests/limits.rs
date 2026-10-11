mod common;
use an_net::node::{MAX_UNAPPROVED, Me, Node, PAIRING_WINDOW};
use an_net::proto::{MAX_FRAME, MAX_NAME_CHARS, Response};
use an_net::tls;
use common::hello;
use rustls::pki_types::ServerName;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;

#[tokio::test]
async fn oversize_frame_rejected() {
    let b = common::start("big-b").await;
    let id = common::bare_identity();
    let tcp = TcpStream::connect(b.addr).await.unwrap();
    let mut s = TlsConnector::from(tls::client_config(&id, None))
        .connect(ServerName::IpAddress(b.addr.ip().into()), tcp)
        .await
        .unwrap();
    s.write_all(&((MAX_FRAME + 1) as u32).to_be_bytes())
        .await
        .unwrap();
    s.flush().await.unwrap();
    let mut out = Vec::new();
    // 본문을 기다리지 않고 바로 끊는다(수신 연결 상한까지 붙잡지 않는다).
    let read = tokio::time::timeout(
        std::time::Duration::from_millis(1500),
        s.read_to_end(&mut out),
    )
    .await;
    assert!(read.is_ok(), "길이만 보고 바로 끊어야 한다");
    assert!(out.is_empty(), "응답 없이 끊어야 한다: {out:?}");
    assert!(b.node.peers().unwrap().is_empty());

    // 상한 정확히까지는 읽는다(경계 반대쪽): 64 KiB 이름의 hello 는 받아들이고 이름은 잘린다.
    let name = "가".repeat((MAX_FRAME - 120) / 3);
    let req = hello(&name, "h", 1, b.open());
    let body = serde_json::to_vec(&req).unwrap();
    assert!(
        body.len() <= MAX_FRAME && body.len() > MAX_FRAME - 200,
        "{}",
        body.len()
    );
    let (resp, _) = common::start("big-a")
        .await
        .node
        .send(b.addr, None, &req)
        .await
        .unwrap();
    assert!(matches!(resp, Response::Hello { .. }), "{resp:?}");
    assert_eq!(b.only_peer().name.chars().count(), MAX_NAME_CHARS);
}

#[tokio::test]
async fn remote_name_sanitized() {
    let b = common::start("name-b").await;
    let a = common::start("name-a").await;
    let req = hello(
        "  이\u{0}도\n윤\u{1b}[31m  ",
        "\u{7}\t",
        a.addr.port(),
        b.open(),
    );
    a.node.send(b.addr, None, &req).await.unwrap();
    let p = b.only_peer();
    assert_eq!(p.name, "이도윤[31m");
    assert_eq!(p.host, "?", "제어 문자만 있으면 ?");

    // 받는 쪽(주소 추가한 쪽)도 상대 이름을 정제한다.
    let evil_db = std::env::temp_dir().join(format!("an-net-evil-{}.db", std::process::id()));
    let _ = std::fs::remove_file(&evil_db);
    let evil = Node::new(
        common::bare_identity(),
        an_core::db::open(&evil_db).unwrap(),
        Me {
            name: "x\u{0}".repeat(100),
            host: "ok".into(),
            port: 1,
        },
    );
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0))
        .await
        .unwrap();
    let evil_addr = listener.local_addr().unwrap();
    tokio::spawn(evil.clone().serve(listener));
    let code = evil.open_window(PAIRING_WINDOW);
    let p = a.node.add(evil_addr, Some(code)).await.unwrap();
    assert_eq!(p.name, "x".repeat(MAX_NAME_CHARS));
}

#[tokio::test]
async fn unapproved_peer_cap() {
    let b = common::start("cap-b").await;
    let first = common::start("cap-first").await;
    first
        .node
        .send(b.addr, None, &hello("n", "h", 1, b.open()))
        .await
        .unwrap();
    for _ in 1..MAX_UNAPPROVED {
        let n = Node::new(
            common::bare_identity(),
            an_core::db::Connection::open_in_memory().unwrap(),
            Me {
                name: "n".into(),
                host: "h".into(),
                port: 1,
            },
        );
        let (resp, _) = n
            .send(b.addr, None, &hello("n", "h", 1, b.open()))
            .await
            .unwrap();
        assert!(matches!(resp, Response::Hello { .. }), "{resp:?}");
    }
    assert_eq!(b.node.peers().unwrap().len() as i64, MAX_UNAPPROVED);

    // 33번째 처음 보는 키 → busy(맞는 초대 코드가 있어도 상한이 먼저), 기록 안 늘어남.
    let late = common::start("cap-late").await;
    let (resp, _) = late
        .node
        .send(b.addr, None, &hello("n", "h", 1, b.open()))
        .await
        .unwrap();
    assert_eq!(resp, Response::error("busy"));
    assert_eq!(b.node.peers().unwrap().len() as i64, MAX_UNAPPROVED);

    // 이미 아는 키는 상한 · 코드와 무관하게 hello 가 통한다.
    let (resp, _) = first
        .node
        .send(b.addr, None, &hello("n", "h", 1, None))
        .await
        .unwrap();
    assert!(matches!(resp, Response::Hello { .. }), "{resp:?}");

    // 하나를 승인하면 자리가 난다.
    let id = b.node.peers().unwrap()[0].id;
    an_core::peers::set_my_ok(&b.db(), id).unwrap();
    let (resp, _) = late
        .node
        .send(b.addr, None, &hello("n", "h", 1, b.open()))
        .await
        .unwrap();
    assert!(matches!(resp, Response::Hello { .. }), "{resp:?}");
}
