mod common;
use common::{host, send, start};

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
        ("Authorization", format!("Bearer {}", r.token)),
    ];
    let (code, _) = send(r.port, "GET", "/api/status", &headers).await;
    assert_eq!(code, 403);
}
