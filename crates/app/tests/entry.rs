mod common;
use common::{host, send, start};

#[tokio::test]
async fn token_query_sets_cookie_and_redirects() {
    let r = start().await;
    let (code, raw) = send(r.port, "GET", &format!("/?t={}", r.token), &[host(r.port)]).await;
    assert_eq!(code, 303);
    let raw = raw.to_ascii_lowercase();
    assert!(
        raw.contains(&format!(
            "set-cookie: an_token={}; httponly; samesite=strict; path=/",
            r.token
        )),
        "{raw}"
    );
    assert!(raw.contains("location: /\r\n"), "{raw}");
}

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
async fn status_page_served_with_cookie() {
    let r = start().await;
    let (code, body) = send(
        r.port,
        "GET",
        "/",
        &[host(r.port), ("Cookie", format!("an_token={}", r.token))],
    )
    .await;
    assert_eq!(code, 200);
    assert!(body.contains("coworkAgent"), "{body}");
}

#[tokio::test]
async fn token_query_redirects_even_with_cookie() {
    let r = start().await;
    let headers = [host(r.port), ("Cookie", format!("an_token={}", r.token))];
    let (code, _) = send(r.port, "GET", &format!("/?t={}", r.token), &headers).await;
    assert_eq!(
        code, 303,
        "주소창에서 토큰을 지우려면 쿠키가 있어도 리다이렉트해야 한다"
    );
}
