mod common;
use common::{host, send, start};

#[tokio::test]
async fn wrong_token_query_401() {
    let r = start().await;
    let (code, _) = send(
        r.port,
        "GET",
        &format!("/?t={}", an_app::server::new_token()),
        &[host(r.port)],
    )
    .await;
    assert_eq!(code, 401);
}

#[tokio::test]
async fn status_page_served() {
    let r = start().await;
    let (code, body) = send(
        r.port,
        "GET",
        "/",
        &[
            host(r.port),
            ("Authorization", format!("Bearer {}", r.token)),
        ],
    )
    .await;
    assert_eq!(code, 200);
    assert!(body.contains("coworkAgent"), "{body}");
}
