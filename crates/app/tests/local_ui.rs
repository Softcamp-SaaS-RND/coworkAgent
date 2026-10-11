mod common;
use an_app::server;
use common::{host, send, start};

#[tokio::test]
async fn loopback_only() {
    let listener = server::bind().await.unwrap();
    assert_eq!(listener.local_addr().unwrap().ip().to_string(), "127.0.0.1");
}

#[tokio::test]
async fn no_token_401() {
    let r = start().await;
    let (code, _) = send(r.port, "GET", "/api/status", &[host(r.port)]).await;
    assert_eq!(code, 401);
}

#[tokio::test]
async fn wrong_token_401() {
    let r = start().await;
    let other = server::new_token();
    let (code, _) = send(
        r.port,
        "GET",
        "/api/status",
        &[host(r.port), ("Cookie", format!("an_token={other}"))],
    )
    .await;
    assert_eq!(code, 401);
}

#[tokio::test]
async fn wrong_host_403() {
    let r = start().await;
    let headers = [
        ("Host", format!("evil.example:{}", r.port)),
        ("Cookie", format!("an_token={}", r.token)),
    ];
    let (code, _) = send(r.port, "GET", "/api/status", &headers).await;
    assert_eq!(code, 403);
}

#[tokio::test]
async fn cross_origin_403() {
    let r = start().await;
    let headers = [
        host(r.port),
        ("Authorization", format!("Bearer {}", r.token)),
        ("Origin", "http://evil.example".to_string()),
    ];
    let (code, _) = send(r.port, "POST", "/api/open", &headers).await;
    assert_eq!(code, 403);
    assert_eq!(r.opened(), 0);
}

#[tokio::test]
async fn valid_200() {
    let r = start().await;
    let (code, body) = send(
        r.port,
        "GET",
        "/api/status",
        &[host(r.port), ("Cookie", format!("an_token={}", r.token))],
    )
    .await;
    assert_eq!(code, 200);
    // JSON 안에서 역슬래시는 이스케이프된다(Windows 경로).
    assert!(body.contains(r#""data_dir":"C:\\tmp\\a""#), "{body}");
}
