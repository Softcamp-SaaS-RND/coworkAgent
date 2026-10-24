//! 웹UI 인증 — 탭 메모리 토큰 + `Authorization: Bearer`(쿠키 없음) · 정적 자산 공개 · 보안 헤더(1a A2 · A3).
mod common;
use an_app::server;
use axum::Router;
use axum::routing::get;
use common::{host, origin, send, start};

fn bearer(token: &str) -> (&'static str, String) {
    ("Authorization", format!("Bearer {token}"))
}

/// `/api/events` 를 가진 서버 — 경로별 토큰 규칙만 본다(알림 스트림 대신 200 을 주는 대역).
async fn with_events() -> (u16, String) {
    let listener = server::bind().await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let token = server::new_token();
    let status = server::Status {
        version: "t".into(),
        data_dir: "d".into(),
        db: "ok".into(),
    };
    let extra = Router::new().route("/api/events", get(|| async { "stream" }));
    let app = server::router_with(port, token.clone(), status, || Ok(()), extra);
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    (port, token)
}

#[tokio::test]
async fn entry_serves_page_without_cookie() {
    let r = start().await;
    let (code, raw) = send(r.port, "GET", &format!("/?t={}", r.token), &[host(r.port)]).await;
    assert_eq!(code, 200, "{raw}");
    assert!(raw.contains("coworkAgent"), "{raw}");
    assert!(!raw.to_ascii_lowercase().contains("set-cookie"), "{raw}");
}

#[tokio::test]
async fn entry_wrong_token_401() {
    let r = start().await;
    let (code, _) = send(
        r.port,
        "GET",
        &format!("/?t={}", server::new_token()),
        &[host(r.port)],
    )
    .await;
    assert_eq!(code, 401);
}

#[tokio::test]
async fn static_assets_public() {
    let r = start().await;
    for path in ["/", "/app.css", "/favicon.ico"] {
        let (code, _) = send(r.port, "GET", path, &[host(r.port)]).await;
        assert_eq!(code, 200, "{path}");
    }
    for (name, _) in server::SCRIPTS {
        let (code, raw) = send(r.port, "GET", &format!("/js/{name}"), &[host(r.port)]).await;
        assert_eq!(code, 200, "{name}");
        assert!(raw.to_ascii_lowercase().contains("content-type: text/javascript"), "{name}");
    }
}

/// 화면 모듈은 `SCRIPTS` 이름과 정확히 같을 때만 토큰 없이 — 표 밖 `/js/…` · 옛 경로는 토큰이 있어도 404.
#[tokio::test]
async fn scripts_exact_names_only() {
    let r = start().await;
    for path in ["/js/nope.js", "/js/core/", "/js/logic/../app.js", "/app.js", "/view.js"] {
        let (code, _) = send(r.port, "GET", path, &[host(r.port)]).await;
        assert_eq!(code, 401, "토큰 없이 {path}");
        let (code, _) = send(r.port, "GET", path, &[host(r.port), bearer(&r.token)]).await;
        assert_eq!(code, 404, "토큰으로 {path}");
    }
}

/// 화면 글꼴(#1p) — 토큰 없이 · woff2 · 1년 캐시, 목록 밖 이름은 공개가 아니다.
#[tokio::test]
async fn fonts_public_and_cached() {
    let r = start().await;
    for (path, mime) in [
        ("/fonts/IBMPlexSansKR-Regular.woff2", "font/woff2"),
        ("/fonts/IBMPlexSansKR-Medium.woff2", "font/woff2"),
        ("/fonts/IBMPlexSansKR-SemiBold.woff2", "font/woff2"),
        ("/fonts/LICENSE.txt", "text/plain"),
    ] {
        let (code, raw) = send(r.port, "GET", path, &[host(r.port)]).await;
        let head = raw.to_ascii_lowercase();
        assert_eq!(code, 200, "{path}");
        assert!(head.contains(&format!("content-type: {mime}")), "{path}");
        assert!(
            head.contains("cache-control: public, max-age=31536000, immutable"),
            "{path}"
        );
    }
    let (code, _) = send(r.port, "GET", "/fonts/other.woff2", &[host(r.port)]).await;
    assert_eq!(code, 401, "목록 밖 글꼴 이름은 토큰이 있어야 닿는다");
}

#[tokio::test]
async fn api_needs_bearer() {
    let r = start().await;
    let (code, _) = send(r.port, "GET", "/api/status", &[host(r.port)]).await;
    assert_eq!(code, 401);
    let (code, _) = send(
        r.port,
        "GET",
        "/api/status",
        &[host(r.port), bearer(&server::new_token())],
    )
    .await;
    assert_eq!(code, 401);
    let (code, _) = send(
        r.port,
        "GET",
        "/api/status",
        &[host(r.port), bearer(&r.token)],
    )
    .await;
    assert_eq!(code, 200);
}

/// 쿠키는 포트를 가리지 않아 같은 PC 의 다른 포트로도 실린다 — 쿠키만으로는 아무것도 안 된다.
#[tokio::test]
async fn cookie_only_401() {
    let r = start().await;
    let cookie = ("Cookie", format!("an_token={}", r.token));
    let (code, _) = send(
        r.port,
        "GET",
        "/api/status",
        &[host(r.port), cookie.clone()],
    )
    .await;
    assert_eq!(code, 401);
    let (code, _) = send(
        r.port,
        "POST",
        "/api/open",
        &[host(r.port), origin(r.port), cookie],
    )
    .await;
    assert_eq!(code, 401);
    assert_eq!(r.opened(), 0);
}

/// `EventSource` 는 헤더를 못 실어 `/api/events` 만 `?t=` 를 받는다.
#[tokio::test]
async fn events_accepts_query_token() {
    let (port, token) = with_events().await;
    let (code, raw) = send(
        port,
        "GET",
        &format!("/api/events?t={token}"),
        &[host(port)],
    )
    .await;
    assert_eq!(code, 200, "{raw}");
    let (code, _) = send(
        port,
        "GET",
        &format!("/api/events?t={}", server::new_token()),
        &[host(port)],
    )
    .await;
    assert_eq!(code, 401);
    let (code, _) = send(port, "GET", "/api/events", &[host(port)]).await;
    assert_eq!(code, 401);
}

#[tokio::test]
async fn other_api_rejects_query_token() {
    let r = start().await;
    let (code, _) = send(
        r.port,
        "GET",
        &format!("/api/status?t={}", r.token),
        &[host(r.port)],
    )
    .await;
    assert_eq!(code, 401);
}

#[tokio::test]
async fn wrong_host_403_even_public() {
    let r = start().await;
    for path in ["/", "/js/app.js", "/api/status"] {
        let headers = [
            ("Host", format!("evil.example:{}", r.port)),
            bearer(&r.token),
        ];
        let (code, _) = send(r.port, "GET", path, &headers).await;
        assert_eq!(code, 403, "{path}");
    }
}

/// CLI · 핸드오프는 Origin 없이 Bearer 로 상태를 바꾼다.
#[tokio::test]
async fn bearer_post_without_origin_ok() {
    let r = start().await;
    let (code, _) = send(
        r.port,
        "POST",
        "/api/open",
        &[host(r.port), bearer(&r.token)],
    )
    .await;
    assert_eq!(code, 204);
    assert_eq!(r.opened(), 1);
}

#[tokio::test]
async fn security_headers_on_every_response() {
    let r = start().await;
    let cases = [
        ("GET", "/", vec![host(r.port)], 200),
        ("GET", "/js/app.js", vec![host(r.port)], 200),
        ("GET", "/api/status", vec![host(r.port)], 401),
        (
            "GET",
            "/api/status",
            vec![("Host", "evil.example".to_string())],
            403,
        ),
        (
            "POST",
            "/api/open",
            vec![
                host(r.port),
                bearer(&r.token),
                ("Origin", "http://evil.example".into()),
            ],
            403,
        ),
    ];
    for (method, path, headers, want) in cases {
        let (code, raw) = send(r.port, method, path, &headers).await;
        assert_eq!(code, want, "{method} {path}");
        let raw = raw.to_ascii_lowercase();
        for h in [
            "x-frame-options: deny",
            "x-content-type-options: nosniff",
            "referrer-policy: no-referrer",
            "frame-ancestors 'none'",
        ] {
            assert!(raw.contains(h), "{method} {path} 에 {h} 없음:\n{raw}");
        }
    }
    assert_eq!(r.opened(), 0);
}

#[tokio::test]
async fn csp_and_no_store_on_page() {
    let r = start().await;
    let (_, raw) = send(r.port, "GET", "/", &[host(r.port)]).await;
    let raw = raw.to_ascii_lowercase();
    assert!(
        raw.contains(&format!(
            "content-security-policy: {}",
            server::CSP.to_ascii_lowercase()
        )),
        "{raw}"
    );
    assert_eq!(
        server::CSP,
        "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; \
         connect-src 'self'; frame-ancestors 'none'; object-src 'none'"
    );
    assert!(raw.contains("cache-control: no-store"), "{raw}");
}
