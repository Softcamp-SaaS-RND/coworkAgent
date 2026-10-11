mod common;
use common::{host, send, start};

#[tokio::test]
async fn cookie_name_prefix_is_not_token() {
    let r = start().await;
    let cookie = format!("an_token_x={}; other=1", r.token);
    let (code, _) = send(
        r.port,
        "GET",
        "/api/status",
        &[host(r.port), ("Cookie", cookie)],
    )
    .await;
    assert_eq!(code, 401);
}

#[tokio::test]
async fn token_among_multiple_cookies_ok() {
    let r = start().await;
    let cookie = format!("a=1; an_token={}; b=2", r.token);
    let (code, _) = send(
        r.port,
        "GET",
        "/api/status",
        &[host(r.port), ("Cookie", cookie)],
    )
    .await;
    assert_eq!(code, 200);
}

#[tokio::test]
async fn static_assets_need_token() {
    let r = start().await;
    for path in ["/", "/app.js", "/app.css"] {
        let (code, _) = send(r.port, "GET", path, &[host(r.port)]).await;
        assert_eq!(code, 401, "{path}");
    }
}

#[tokio::test]
async fn same_origin_post_allowed() {
    let r = start().await;
    let headers = [
        host(r.port),
        ("Authorization", format!("Bearer {}", r.token)),
        ("Origin", format!("http://127.0.0.1:{}", r.port)),
    ];
    let (code, _) = send(r.port, "POST", "/api/open", &headers).await;
    assert_eq!(code, 204);
    assert_eq!(r.opened(), 1);
}

#[tokio::test]
async fn localhost_host_rejected() {
    let r = start().await;
    let headers = [
        ("Host", format!("localhost:{}", r.port)),
        ("Cookie", format!("an_token={}", r.token)),
    ];
    let (code, _) = send(r.port, "GET", "/api/status", &headers).await;
    assert_eq!(code, 403);
}
