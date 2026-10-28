mod common;
use an_app::notify::{self, Notifier};
use an_app::server;
use an_net::node::PeerEvent;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

async fn web(n: Arc<Notifier>) -> (u16, (&'static str, String)) {
    let listener = server::bind().await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let token = server::new_token();
    let status = server::Status {
        version: "t".into(),
        data_dir: "d".into(),
        db: "ok".into(),
    };
    let app = server::router_with(port, token.clone(), status, || Ok(()), notify::routes(n));
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    (port, ("Authorization", format!("Bearer {token}")))
}

fn notifier(tag: &str, marks: Arc<Mutex<Vec<usize>>>) -> Arc<Notifier> {
    let path = std::env::temp_dir().join(format!("an-app-events-{tag}-{}.db", std::process::id()));
    let _ = std::fs::remove_file(&path);
    Notifier::new(an_core::db::open(&path).unwrap(), move |c| {
        marks.lock().unwrap().push(c)
    })
    .unwrap()
}

/// 스트림에서 `needle` 이 나올 때까지(최대 3초) 읽는다.
async fn read_until(s: &mut tokio::net::TcpStream, needle: &str) -> String {
    let mut got = Vec::new();
    let mut buf = [0u8; 1024];
    let until = tokio::time::Instant::now() + Duration::from_secs(3);
    while !String::from_utf8_lossy(&got).contains(needle) {
        let n = tokio::time::timeout_at(until, s.read(&mut buf))
            .await
            .expect("시간 초과")
            .unwrap();
        assert!(n > 0, "스트림이 닫혔다: {}", String::from_utf8_lossy(&got));
        got.extend_from_slice(&buf[..n]);
    }
    String::from_utf8_lossy(&got).into_owned()
}

#[tokio::test]
async fn events_stream_delivers_without_clearing() {
    let marks = Arc::new(Mutex::new(Vec::new()));
    let n = notifier("sse", marks.clone());
    n.set_enabled(true).unwrap();
    n.on_event(&PeerEvent::PairRequest {
        name: "a".into(),
        host: "b".into(),
        fingerprint: "f".into(),
    }); // 웹UI 가 붙기 전 연결 요청 1건 → 트레이 1
    let (port, (ck, cv)) = web(n.clone()).await;
    let mut s = tokio::net::TcpStream::connect(("127.0.0.1", port))
        .await
        .unwrap();
    s.write_all(
        format!("GET /api/events HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\n{ck}: {cv}\r\nAccept: text/event-stream\r\n\r\n")
            .as_bytes(),
    )
    .await
    .unwrap();
    let head = read_until(&mut s, "\r\n\r\n").await;
    assert!(head.starts_with("HTTP/1.1 200"), "{head}");
    assert!(head.contains("text/event-stream"), "{head}");
    // GET 은 상태를 바꾸지 않는다 — 트레이 "새 요청" 은 그대로.
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert_eq!(*marks.lock().unwrap(), vec![1]);
    n.on_event(&PeerEvent::PairRequest {
        name: "이도윤".into(),
        host: "DYOON-PC".into(),
        fingerprint: "7F3A".into(),
    });
    let body = read_until(&mut s, "DYOON-PC").await;
    assert!(body.contains("data: {"), "{body}");
    assert!(body.contains("\"kind\":\"pair_request\""), "{body}");
}

/// 변경 신호는 `event: change` 로 흐른다 — 알림이 꺼져 있어도(1u).
#[tokio::test]
async fn events_stream_change_event() {
    let n = notifier("sse-chg", Arc::new(Mutex::new(Vec::new())));
    let (port, (ck, cv)) = web(n.clone()).await;
    let mut s = tokio::net::TcpStream::connect(("127.0.0.1", port))
        .await
        .unwrap();
    s.write_all(
        format!("GET /api/events HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\n{ck}: {cv}\r\nAccept: text/event-stream\r\n\r\n")
            .as_bytes(),
    )
    .await
    .unwrap();
    read_until(&mut s, "\r\n\r\n").await;
    tokio::time::sleep(Duration::from_millis(100)).await; // 구독이 붙을 때까지
    n.on_event(&PeerEvent::Approved {
        name: "a".into(),
        host: "b".into(),
    });
    let body = read_until(&mut s, "\"kind\":\"peer\"").await;
    assert!(body.contains("event: change"), "{body}");
    assert!(!body.contains("pair"), "알림은 꺼져 있다: {body}");
}

#[tokio::test]
async fn events_need_token() {
    let n = notifier("sse401", Arc::new(Mutex::new(Vec::new())));
    let (port, _) = web(n).await;
    let (code, _) = common::send(port, "GET", "/api/events", &[common::host(port)]).await;
    assert_eq!(code, 401);
}

#[tokio::test]
async fn seen_post_clears() {
    let marks = Arc::new(Mutex::new(Vec::new()));
    let n = notifier("seen", marks.clone());
    n.set_enabled(true).unwrap();
    // 트레이 건수는 받은 요청 · 연결 요청 · 내 요청 상태 변경만 센다.
    n.on_event(&PeerEvent::PairRequest {
        name: "a".into(),
        host: "b".into(),
        fingerprint: "f".into(),
    });
    let (port, cookie) = web(n.clone()).await;
    let (code, raw) = common::send(
        port,
        "POST",
        "/api/seen",
        &[common::host(port), common::origin(port), cookie],
    )
    .await;
    assert_eq!(code, 204, "{raw}");
    assert_eq!(*marks.lock().unwrap(), vec![1, 0]);
    assert_eq!(n.pending(), 0);
}
