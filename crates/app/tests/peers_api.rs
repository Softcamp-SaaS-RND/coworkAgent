mod common;
use an_app::{peers_api, server};
use an_core::{db, paths::Paths};
use an_net::identity::Identity;
use an_net::node::{Me, Node};
use std::sync::Arc;

/// 임시 루트의 실제 신원 · DB 로 127.0.0.1 임의 포트에 피어 수신하는 노드.
async fn node(tag: &str) -> (Arc<Node>, std::net::SocketAddr) {
    let root = std::env::temp_dir().join(format!("an-app-peers-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let paths = Paths::under(&root);
    paths.ensure().unwrap();
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0))
        .await
        .unwrap();
    let addr = listener.local_addr().unwrap();
    let node = Node::new(
        Identity::load_or_create(&paths.identity_file()).unwrap(),
        db::open(&paths.db_file()).unwrap(),
        Me {
            name: format!("user-{tag}"),
            host: format!("PC-{tag}"),
            port: addr.port(),
        },
    );
    tokio::spawn(node.clone().serve(listener));
    (node, addr)
}

/// 운영과 같은 `router_with` + `peers_api::routes` 웹UI 서버. 돌려주는 값: (포트, 쿠키 헤더).
async fn web(node: Arc<Node>) -> (u16, (&'static str, String)) {
    let listener = server::bind().await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let token = server::new_token();
    let status = server::Status {
        version: "t".into(),
        data_dir: "d".into(),
        db: "ok".into(),
    };
    let app = server::router_with(
        port,
        token.clone(),
        status,
        || Ok(()),
        peers_api::routes(node),
    );
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    (port, ("Cookie", format!("{}={token}", server::COOKIE)))
}

fn json(raw: &str) -> serde_json::Value {
    serde_json::from_str(raw.split("\r\n\r\n").nth(1).unwrap()).unwrap()
}

#[tokio::test]
async fn me_shows_fingerprint_and_port() {
    let (a, a_addr) = node("me-a").await;
    let (port, cookie) = web(a.clone()).await;
    let (code, raw) = common::send(
        port,
        "GET",
        "/api/me",
        &[common::host(port), cookie.clone()],
    )
    .await;
    assert_eq!(code, 200, "{raw}");
    let me = json(&raw);
    assert_eq!(me["name"], "user-me-a");
    assert_eq!(me["host"], "PC-me-a");
    assert_eq!(me["fingerprint"], a.identity().fingerprint());
    assert_eq!(me["port"], a_addr.port());
    assert!(me["addrs"].is_array());
    // "페어링 받기" 는 초대 코드(ABCD-EFGH)와 창 길이를 준다.
    let (code, raw) = common::send(
        port,
        "POST",
        "/api/pairing-window",
        &[common::host(port), common::origin(port), cookie],
    )
    .await;
    assert_eq!(code, 200, "{raw}");
    let w = json(&raw);
    assert_eq!(w["seconds"], 300);
    assert_eq!(w["code"].as_str().unwrap().len(), 9);
    let invite = an_net::invite::Invite::parse(w["invite"].as_str().unwrap()).unwrap();
    assert_eq!(invite.code, w["code"].as_str().unwrap());
    assert_eq!(invite.addr.port(), a_addr.port());
    // guard 뒤에 있다 — 토큰 없으면 401.
    let (code, _) = common::send(port, "GET", "/api/me", &[common::host(port)]).await;
    assert_eq!(code, 401);
}

#[tokio::test]
async fn add_then_list_peer() {
    let (a, _) = node("list-a").await;
    let (b, b_addr) = node("list-b").await;
    let (port, cookie) = web(a).await;
    let json_type = ("Content-Type", "application/json".to_string());
    let (cookie2, json_type2) = (cookie.clone(), json_type.clone());

    let code = b.open_window(an_net::node::PAIRING_WINDOW);
    let body = format!(r#"{{"addr":"{b_addr}","code":"{code}"}}"#);
    let (code, raw) = common::send_body(
        port,
        "POST",
        "/api/peers",
        &[
            common::host(port),
            common::origin(port),
            cookie.clone(),
            json_type.clone(),
        ],
        &body,
    )
    .await;
    assert_eq!(code, 201, "{raw}");
    let (code, raw) = common::send(
        port,
        "GET",
        "/api/peers",
        &[common::host(port), cookie.clone()],
    )
    .await;
    assert_eq!(code, 200, "{raw}");
    let list = json(&raw);
    let list = list.as_array().unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0]["fingerprint"], b.identity().fingerprint());
    assert_eq!(list[0]["name"], "user-list-b");
    // 코드가 맞아 상대가 승인했으므로 붙인 쪽 승인까지 한 번에 — 직접 추가도 자동 등록(Task #15 D8 추가 수용 · 적대적 평가 라운드 1).
    assert_eq!(list[0]["paired"], true);

    // 형식 아닌 주소 → 400, JSON 아닌 본문(폼 CSRF 형태) → 415.
    let (code, _) = common::send_body(
        port,
        "POST",
        "/api/peers",
        &[
            common::host(port),
            common::origin(port),
            cookie.clone(),
            json_type,
        ],
        r#"{"addr":"pc-b"}"#,
    )
    .await;
    assert_eq!(code, 400);
    let (code, _) = common::send_body(
        port,
        "POST",
        "/api/peers",
        &[
            common::host(port),
            common::origin(port),
            cookie,
            ("Content-Type", "application/x-www-form-urlencoded".into()),
        ],
        &format!("addr={b_addr}"),
    )
    .await;
    assert_eq!(code, 415);

    // 메신저로 받은 초대 텍스트 통째 → 지문 대조 · 추가 · 내 승인까지.
    let (c, _) = node("list-c").await;
    let text = c
        .open_invite(
            std::net::Ipv4Addr::LOCALHOST.into(),
            an_net::node::PAIRING_WINDOW,
        )
        .to_text("c", "PC-c");
    let (code, raw) = common::send_body(
        port,
        "POST",
        "/api/peers",
        &[
            common::host(port),
            common::origin(port),
            cookie2.clone(),
            json_type2.clone(),
        ],
        &serde_json::json!({ "invite": format!("받은 메시지:
{text}") })
        .to_string(),
    )
    .await;
    assert_eq!(code, 201, "{raw}");
    let added = json(&raw);
    assert_eq!(added["fingerprint"], c.identity().fingerprint());
    assert_eq!(added["my_ok"], true);
    // 토큰 없는 텍스트 → 400.
    let (code, _) = common::send_body(
        port,
        "POST",
        "/api/peers",
        &[
            common::host(port),
            common::origin(port),
            cookie2,
            json_type2,
        ],
        r#"{"invite":"안녕하세요"}"#,
    )
    .await;
    assert_eq!(code, 400);
}
