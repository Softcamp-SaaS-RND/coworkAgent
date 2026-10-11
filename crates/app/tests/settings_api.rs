mod common;
use an_app::notify::{self, Notifier};
use an_app::server;
use std::path::{Path, PathBuf};
use std::sync::Arc;

fn db_path(tag: &str) -> PathBuf {
    let p = std::env::temp_dir().join(format!("an-app-settings-{tag}-{}.db", std::process::id()));
    let _ = std::fs::remove_file(&p);
    p
}

fn notifier(path: &Path) -> Arc<Notifier> {
    Notifier::new(an_core::db::open(path).unwrap(), |_| {}).unwrap()
}

/// 운영과 같은 `router_with` + `notify::routes` 웹UI 서버. 돌려주는 값: (포트, 쿠키 헤더).
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
    (port, ("Cookie", format!("{}={token}", server::COOKIE)))
}

fn body(raw: &str) -> serde_json::Value {
    serde_json::from_str(raw.split("\r\n\r\n").nth(1).unwrap()).unwrap()
}

#[tokio::test]
async fn notify_default_off() {
    let (port, cookie) = web(notifier(&db_path("off"))).await;
    let (code, raw) =
        common::send(port, "GET", "/api/settings", &[common::host(port), cookie]).await;
    assert_eq!(code, 200, "{raw}");
    assert_eq!(body(&raw)["notify_inbound"], false);
    // guard 뒤 — 토큰 없으면 401.
    let (code, _) = common::send(port, "GET", "/api/settings", &[common::host(port)]).await;
    assert_eq!(code, 401);
}

#[tokio::test]
async fn notify_toggle_persists() {
    let path = db_path("toggle");
    let (port, cookie) = web(notifier(&path)).await;
    let json = ("Content-Type", "application/json".to_string());
    let (code, raw) = common::send_body(
        port,
        "PUT",
        "/api/settings",
        &[
            common::host(port),
            common::origin(port),
            cookie.clone(),
            json.clone(),
        ],
        r#"{"notify_inbound":true}"#,
    )
    .await;
    assert_eq!(code, 200, "{raw}");
    let (_, raw) = common::send(
        port,
        "GET",
        "/api/settings",
        &[common::host(port), cookie.clone()],
    )
    .await;
    assert_eq!(body(&raw)["notify_inbound"], true);
    // 재시작(같은 DB 를 새로 연 Notifier)에도 유지.
    assert!(notifier(&path).enabled());
    // 형식이 틀린 본문은 4xx, 값은 그대로.
    let (code, _) = common::send_body(
        port,
        "PUT",
        "/api/settings",
        &[common::host(port), common::origin(port), cookie, json],
        r#"{"notify_inbound":"yes"}"#,
    )
    .await;
    assert!((400..500).contains(&code), "{code}");
    assert!(notifier(&path).enabled());
}

async fn put(port: u16, cookie: &(&'static str, String), body: &str) -> (u16, String) {
    common::send_body(
        port,
        "PUT",
        "/api/settings",
        &[
            common::host(port),
            common::origin(port),
            cookie.clone(),
            ("Content-Type", "application/json".to_string()),
        ],
        body,
    )
    .await
}

#[tokio::test]
async fn prefs_patch_validated_and_persisted() {
    let path = db_path("prefs");
    let (port, cookie) = web(notifier(&path)).await;
    let (_, raw) = common::send(
        port,
        "GET",
        "/api/settings",
        &[common::host(port), cookie.clone()],
    )
    .await;
    let v = body(&raw);
    assert_eq!(
        (
            v["notify_pair"].as_bool(),
            v["notify_relay"].as_str(),
            v["notify_preview"].as_bool(),
            v["relay_cap_default"].as_i64()
        ),
        (Some(true), Some("start_end"), Some(true), Some(10)),
        "기본값"
    );
    let (code, raw) = put(
        port,
        &cookie,
        r#"{"notify_request":false,"notify_relay":"off","notify_preview":false,"relay_cap_default":3}"#,
    )
    .await;
    assert_eq!(code, 200, "{raw}");
    let v = body(&raw);
    assert_eq!(v["notify_request"], false);
    assert_eq!(v["notify_inbound"], false, "준 항목만 바뀐다");
    // 틀린 값이 하나라도 있으면 4xx · 아무것도 안 바뀐다.
    for bad in [
        r#"{"notify_reply":false,"relay_cap_default":51}"#,
        r#"{"notify_inbound":true,"relay_cap_default":51}"#,
        r#"{"notify_reply":false,"relay_cap_default":0}"#,
        r#"{"notify_reply":false,"notify_relay":"x"}"#,
        r#"{"notify_reply":false,"unknown":1}"#,
    ] {
        let (code, raw) = put(port, &cookie, bad).await;
        assert!((400..500).contains(&code), "{bad} → {raw}");
    }
    // 재시작(같은 DB 를 새로 연 Notifier)에도 유지.
    let n = notifier(&path);
    let p = n.prefs();
    assert_eq!(
        (
            p.notify_request,
            p.notify_reply,
            p.notify_relay.as_str(),
            p.notify_preview
        ),
        (false, true, "off", false)
    );
    assert_eq!(n.relay_cap_default(), 3);
    assert!(!n.enabled(), "틀린 본문의 전체 켜짐도 저장되지 않는다");
}
