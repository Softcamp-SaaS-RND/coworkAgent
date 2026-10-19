mod common;
use an_agent::chat::{Chat, Run};
use an_agent::runner::{RunInput, RunOutput, Session};
use an_app::notify::Notifier;
use an_app::{chat_api, server};
use an_core::{agents, db, paths::Paths};
use an_net::identity::Identity;
use an_net::node::{Me, Node, PAIRING_WINDOW};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// 실행기 대역 — `claude -p` 대신 "답: {입력}".
fn echo() -> Run {
    Arc::new(|input: RunInput| {
        let session = match &input.session {
            Session::New(id) | Session::Resume(id) => id.clone(),
        };
        Ok(RunOutput {
            text: format!(
                "답: {}",
                // 실행 입력의 원격 본문(`<원격 본문>` 구분자 안)만 되돌린다.
                input
                    .text
                    .rsplit_once("<원격 본문>\n")
                    .and_then(|(_, r)| r.strip_suffix("\n</원격 본문>"))
                    .unwrap_or(&input.text)
            ),
            session,
            is_error: false,
            turns: Some(2),
            duration_ms: Some(1500),
            ctx_tokens: None,
        })
    })
}

struct Side {
    node: Arc<Node>,
    chat: Arc<Chat>,
    addr: std::net::SocketAddr,
    paths: Paths,
}

async fn side(tag: &str) -> Side {
    let root = std::env::temp_dir().join(format!("an-app-chat-{tag}-{}", std::process::id()));
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
    let chat = Chat::new(db::open(&paths.db_file()).unwrap(), node.clone(), echo());
    chat.attach();
    tokio::spawn(node.clone().serve(listener));
    Side {
        node,
        chat,
        addr,
        paths,
    }
}

/// 운영과 같은 `router_with` + `chat_api::routes`. 돌려주는 값: 요청에 붙일 헤더(Host · Origin · 쿠키 · JSON).
async fn web(chat: Arc<Chat>) -> (u16, Vec<(&'static str, String)>) {
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
        chat_api::routes(chat),
    );
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let headers = vec![
        common::host(port),
        common::origin(port),
        ("Authorization", format!("Bearer {token}")),
        ("Content-Type", "application/json".to_string()),
    ];
    (port, headers)
}

async fn call(
    port: u16,
    h: &[(&str, String)],
    method: &str,
    path: &str,
    body: &str,
) -> (u16, serde_json::Value) {
    let (code, raw) = common::send_body(port, method, path, h, body).await;
    let text = raw.split("\r\n\r\n").nth(1).unwrap_or("");
    (
        code,
        serde_json::from_str(text).unwrap_or(serde_json::Value::Null),
    )
}

#[tokio::test(flavor = "multi_thread")]
async fn agents_crud_api() {
    let a = side("crud").await;
    let (port, h) = web(a.chat.clone()).await;
    let folder = a.paths.root().display().to_string();
    let body = serde_json::json!({ "name": "review", "folder": folder }).to_string();
    let (code, made) = call(port, &h, "POST", "/api/agents", &body).await;
    assert_eq!(code, 201, "{made}");
    assert!(made.get("tools").is_none(), "도구 칸 없음(#1d) {made}");
    let (code, list) = call(port, &h, "GET", "/api/agents", "").await;
    assert_eq!(code, 200);
    assert_eq!(list.as_array().unwrap().len(), 1);
    assert_eq!(list[0]["name"], "review");
    // 같은 이름 409, 없는 폴더 · 상대 경로 · 빈 이름 400.
    assert_eq!(call(port, &h, "POST", "/api/agents", &body).await.0, 409);
    for bad in [
        serde_json::json!({ "name": "x", "folder": format!("{folder}/없는폴더") }),
        serde_json::json!({ "name": "x", "folder": "." }),
        serde_json::json!({ "name": " ", "folder": folder }),
    ] {
        let (code, r) = call(port, &h, "POST", "/api/agents", &bad.to_string()).await;
        assert_eq!(code, 400, "{bad} → {r}");
    }
    let path = format!("/api/agents/{}", made["id"]);
    assert_eq!(call(port, &h, "DELETE", &path, "").await.0, 204);
    assert_eq!(call(port, &h, "DELETE", &path, "").await.0, 404);
    let (_, list) = call(port, &h, "GET", "/api/agents", "").await;
    assert!(list.as_array().unwrap().is_empty());
}

/// 대화의 메시지가 `n` 개가 될 때까지(최대 5초).
async fn messages(port: u16, h: &[(&str, String)], conv: i64, n: usize) -> serde_json::Value {
    let until = Instant::now() + Duration::from_secs(5);
    loop {
        let (code, ms) = call(
            port,
            h,
            "GET",
            &format!("/api/conversations/{conv}/messages"),
            "",
        )
        .await;
        assert_eq!(code, 200, "{ms}");
        if ms.as_array().unwrap().len() >= n {
            return ms;
        }
        assert!(Instant::now() < until, "메시지 {n} 개를 기다렸다: {ms}");
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn conversation_api_roundtrip() {
    let (a, b) = (side("conv-a").await, side("conv-b").await);
    let code = b.node.open_window(PAIRING_WINDOW);
    let pb = a.node.add(b.addr, Some(code)).await.unwrap();
    let pa = b.node.peers().unwrap().remove(0);
    a.node.approve(pb.id).await.unwrap();
    b.node.approve(pa.id).await.unwrap();
    agents::add(&db::open(&b.paths.db_file()).unwrap(), "review", ".", "").unwrap();
    // B 의 알림(S9) — 받은 요청이 `message` 알림으로 나간다.
    let notifier = Notifier::new(db::open(&b.paths.db_file()).unwrap(), |_| {}).unwrap();
    notifier.set_enabled(true).unwrap();
    let mut notices = notifier.subscribe();
    let n = notifier.clone();
    b.node.on_event(move |e| n.on_event(&e));

    let (port, h) = web(a.chat.clone()).await;
    let (code, r) = call(port, &h, "GET", &format!("/api/peers/{}/agents", pb.id), "").await;
    assert_eq!(code, 200, "{r}");
    assert_eq!(r["names"], serde_json::json!(["review"]));

    let body = serde_json::json!({ "peer_id": pb.id, "agent": "review", "text": "안녕" });
    let (code, r) = call(port, &h, "POST", "/api/conversations", &body.to_string()).await;
    assert_eq!(code, 201, "{r}");
    let conv = r["conversation"]["id"].as_i64().unwrap();
    assert_eq!(r["message"]["state"], "sent");
    let ms = messages(port, &h, conv, 2).await;
    assert_eq!(ms[1]["author"], "agent");
    assert_eq!(ms[1]["text"], "답: 안녕");
    assert_eq!(ms[1]["state"], "done");

    // 이어 쓰기 — 같은 대화.
    let path = format!("/api/conversations/{conv}/messages");
    let (code, r) = call(port, &h, "POST", &path, r#"{"text":"이어서"}"#).await;
    assert_eq!(code, 201, "{r}");
    assert_eq!(r["conversation"]["id"], conv);
    let ms = messages(port, &h, conv, 4).await;
    assert_eq!(ms[3]["text"], "답: 이어서");

    let (_, list) = call(port, &h, "GET", "/api/conversations", "").await;
    assert_eq!(list.as_array().unwrap().len(), 1);
    assert_eq!(
        (list[0]["dir"].as_str(), list[0]["peer_id"].as_i64()),
        (Some("out"), Some(pb.id))
    );
    let notice = notices.try_recv().unwrap();
    assert_eq!(notice.kind, "message");

    // 너무 긴 본문 400 · 없는 대화 404 · 없는 에이전트는 상대 거절.
    let long = serde_json::json!({ "text": "가".repeat(6000) }).to_string();
    assert_eq!(call(port, &h, "POST", &path, &long).await.0, 400);
    assert_eq!(
        call(
            port,
            &h,
            "POST",
            "/api/conversations/999/messages",
            r#"{"text":"x"}"#
        )
        .await
        .0,
        404
    );
    let body = serde_json::json!({ "peer_id": pb.id, "agent": "nope", "text": "x" });
    let (code, r) = call(port, &h, "POST", "/api/conversations", &body.to_string()).await;
    assert_ne!(code, 201);
    assert!(
        r["error"].as_str().unwrap().contains("unknown_agent"),
        "{r}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn conversation_summaries_and_message_meta() {
    let (a, b) = (side("sum-a").await, side("sum-b").await);
    let code = b.node.open_window(PAIRING_WINDOW);
    let pb = a.node.add(b.addr, Some(code)).await.unwrap();
    let pa = b.node.peers().unwrap().remove(0);
    a.node.approve(pb.id).await.unwrap();
    b.node.approve(pa.id).await.unwrap();
    agents::add(&db::open(&b.paths.db_file()).unwrap(), "review", ".", "").unwrap();
    let (port, h) = web(a.chat.clone()).await;
    let body =
        serde_json::json!({ "peer_id": pb.id, "agent": "review", "text": "스키마 초안을 잡아 줘" });
    let (_, r) = call(port, &h, "POST", "/api/conversations", &body.to_string()).await;
    let conv = r["conversation"]["id"].as_i64().unwrap();
    let ms = messages(port, &h, conv, 2).await;
    // 메시지: 시각 · 원 요청 대응 · 실행 정보(대역 2턴 · 1.5초).
    assert!(ms[0]["created_at"].as_i64().unwrap() > 0);
    assert_eq!(ms[0]["reply_to"], serde_json::Value::Null);
    assert_eq!(ms[1]["reply_to"], ms[0]["id"]);
    assert_eq!(
        (ms[1]["turns"].as_i64(), ms[1]["duration_ms"].as_i64()),
        (Some(2), Some(1500))
    );
    // 목록: 제목 = 첫 메시지 앞부분 · 마지막 상태 · 시각.
    let (code, list) = call(port, &h, "GET", "/api/conversations", "").await;
    assert_eq!(code, 200, "{list}");
    assert_eq!(list[0]["title"], "스키마 초안을 잡아 줘");
    assert_eq!(list[0]["last_state"], "done");
    assert!(list[0]["last_at"].as_i64().unwrap() > 0);
    // 대화방 미리보기(#1j) — 마지막 글 = 상대 에이전트 답.
    let last = &list[0]["last"];
    assert_eq!(
        (&last["text"], &last["author"], &last["mine"]),
        (
            &ms[1]["text"],
            &serde_json::json!("agent"),
            &serde_json::json!(false)
        ),
        "{last}"
    );
    assert_eq!(
        last["via"],
        serde_json::Value::Null,
        "웹UI 글 — CLI 출처 아님(라운드 2 `via`)"
    );
    // 화면 로직 모듈도 같은 guard 뒤에서 나간다.
    let (code, raw) = common::send(port, "GET", "/view.js", &h[..3]).await;
    assert_eq!(code, 200);
    assert!(raw.contains("export function timeline"), "view.js 본문");
}

#[tokio::test(flavor = "multi_thread")]
async fn conversation_post_targets() {
    let (a, b) = (side("pt-a").await, side("pt-b").await);
    let code = b.node.open_window(PAIRING_WINDOW);
    let pb = a.node.add(b.addr, Some(code)).await.unwrap();
    let pa = b.node.peers().unwrap().remove(0);
    a.node.approve(pb.id).await.unwrap();
    b.node.approve(pa.id).await.unwrap();
    agents::add(&db::open(&b.paths.db_file()).unwrap(), "review", ".", "").unwrap();
    let (port_a, ha) = web(a.chat.clone()).await;
    let (port_b, hb) = web(b.chat.clone()).await;
    let body = serde_json::json!({ "peer_id": pb.id, "agent": "review", "text": "질문" });
    let (_, r) = call(port_a, &ha, "POST", "/api/conversations", &body.to_string()).await;
    let conv_a = r["conversation"]["id"].as_i64().unwrap();
    messages(port_a, &ha, conv_a, 2).await;
    // A: 사람에게 보내기는 없다(D23 개정) — 400, 이력에 남지 않는다.
    let path_a = format!("/api/conversations/{conv_a}/messages");
    let (code, r) = call(
        port_a,
        &ha,
        "POST",
        &path_a,
        r#"{"text":"A 메모","to":"human"}"#,
    )
    .await;
    assert_eq!(code, 400, "{r}");
    // B: 받은 대화에 자기 에이전트에게.
    let (_, list) = call(port_b, &hb, "GET", "/api/conversations", "").await;
    let conv_b = list[0]["id"].as_i64().unwrap();
    let path_b = format!("/api/conversations/{conv_b}/messages");
    let (code, r) = call(
        port_b,
        &hb,
        "POST",
        &path_b,
        r#"{"text":"B 질문","to":"agent"}"#,
    )
    .await;
    assert_eq!(code, 201, "{r}");
    // A 이력: 질문 · 답 · B 질문 · 답 — 발화자 표시용 mine · exec.
    let ms = messages(port_a, &ha, conv_a, 4).await;
    let flags: Vec<_> = ms
        .as_array()
        .unwrap()
        .iter()
        .map(|m| {
            (
                m["author"].as_str().unwrap().to_string(),
                m["mine"].as_bool().unwrap(),
                m["exec"].as_bool().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        flags,
        [
            ("human".to_string(), true, true),
            ("agent".to_string(), false, false),
            ("human".to_string(), false, true),
            ("agent".to_string(), false, false),
        ]
    );
    // 이상한 대상은 400.
    let (code, _) = call(port_a, &ha, "POST", &path_a, r#"{"text":"x","to":"robot"}"#).await;
    assert_eq!(code, 400);
}

#[tokio::test(flavor = "multi_thread")]
async fn relay_api() {
    let (a, b) = (side("ra-a").await, side("ra-b").await);
    let code = b.node.open_window(PAIRING_WINDOW);
    let pb = a.node.add(b.addr, Some(code)).await.unwrap();
    let pa = b.node.peers().unwrap().remove(0);
    a.node.approve(pb.id).await.unwrap();
    b.node.approve(pa.id).await.unwrap();
    agents::add(&db::open(&a.paths.db_file()).unwrap(), "mine", ".", "").unwrap();
    agents::add(&db::open(&b.paths.db_file()).unwrap(), "review", ".", "").unwrap();
    let (port_a, ha) = web(a.chat.clone()).await;
    let (port_b, hb) = web(b.chat.clone()).await;
    // 없는 내 에이전트 · 범위 밖 상한은 400.
    for bad in [
        serde_json::json!({ "peer_id": pb.id, "agent": "review", "text": "x", "relay_agent": "없음" }),
        serde_json::json!({ "peer_id": pb.id, "agent": "review", "text": "x", "relay_agent": "mine", "relay_cap": 0 }),
        serde_json::json!({ "peer_id": pb.id, "agent": "review", "text": "x", "relay_agent": "mine", "relay_cap": 51 }),
    ] {
        let (code, r) = call(port_a, &ha, "POST", "/api/conversations", &bad.to_string()).await;
        assert_eq!(code, 400, "{bad} → {r}");
    }
    let body = serde_json::json!({ "peer_id": pb.id, "agent": "review", "text": "시작", "relay_agent": "mine", "relay_cap": 1 });
    let (code, r) = call(port_a, &ha, "POST", "/api/conversations", &body.to_string()).await;
    assert_eq!(code, 201, "{r}");
    assert_eq!(r["conversation"]["relay_agent"], "mine");
    assert_eq!(r["conversation"]["relay_cap"], 1);
    let conv_a = r["conversation"]["id"].as_i64().unwrap();
    // 상한 1: 사람 → B 답 → 내 답(B 에 기록만) → 멈춤.
    messages(port_a, &ha, conv_a, 3).await;
    let until = Instant::now() + Duration::from_secs(5);
    loop {
        let (_, list) = call(port_a, &ha, "GET", "/api/conversations", "").await;
        if list[0]["relay_on"] == false {
            assert_eq!(list[0]["relay_count"], 1);
            break;
        }
        assert!(Instant::now() < until, "{list}");
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    // 받은 쪽: 멈춤은 되고 재개는 400.
    let (_, list) = call(port_b, &hb, "GET", "/api/conversations", "").await;
    let path_b = format!("/api/conversations/{}/relay", list[0]["id"]);
    let (code, r) = call(port_b, &hb, "POST", &path_b, r#"{"on":false}"#).await;
    assert_eq!(
        (code, r["relay_blocked"].as_bool()),
        (200, Some(true)),
        "{r}"
    );
    assert_eq!(
        call(port_b, &hb, "POST", &path_b, r#"{"on":true}"#).await.0,
        400
    );
    // 연 쪽 재개 — 카운트 0, 받은 쪽 거절 해제.
    let path_a = format!("/api/conversations/{conv_a}/relay");
    let (code, r) = call(port_a, &ha, "POST", &path_a, r#"{"on":true}"#).await;
    assert_eq!(code, 200, "{r}");
    assert_eq!(
        (r["relay_on"].as_bool(), r["relay_count"].as_i64()),
        (Some(true), Some(0))
    );
    let (_, list) = call(port_b, &hb, "GET", "/api/conversations", "").await;
    assert_eq!(list[0]["relay_blocked"], false);
    messages(port_a, &ha, conv_a, 5).await;
    // 보통 대화는 자동 왕복이 아니다 — 재개 400.
    let body = serde_json::json!({ "peer_id": pb.id, "agent": "review", "text": "보통" });
    let (_, r) = call(port_a, &ha, "POST", "/api/conversations", &body.to_string()).await;
    assert_eq!(r["conversation"]["relay_agent"], serde_json::Value::Null);
    let path = format!("/api/conversations/{}/relay", r["conversation"]["id"]);
    assert_eq!(
        call(port_a, &ha, "POST", &path, r#"{"on":true}"#).await.0,
        400
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn relay_cap_default_from_settings() {
    let (a, b) = (side("rd-a").await, side("rd-b").await);
    let code = b.node.open_window(PAIRING_WINDOW);
    let pb = a.node.add(b.addr, Some(code)).await.unwrap();
    let pa = b.node.peers().unwrap().remove(0);
    a.node.approve(pb.id).await.unwrap();
    b.node.approve(pa.id).await.unwrap();
    let a_db = db::open(&a.paths.db_file()).unwrap();
    agents::add(&a_db, "mine", ".", "").unwrap();
    agents::add(&db::open(&b.paths.db_file()).unwrap(), "review", ".", "").unwrap();
    let (port_a, ha) = web(a.chat.clone()).await;
    let start = serde_json::json!({ "peer_id": pb.id, "agent": "review", "text": "x", "relay_agent": "mine" })
        .to_string();
    // 설정이 없으면 10, 있으면 그 값.
    let (_, r) = call(port_a, &ha, "POST", "/api/conversations", &start).await;
    assert_eq!(r["conversation"]["relay_cap"], 10, "{r}");
    an_core::settings::set(&a_db, an_agent::chat::RELAY_CAP_KEY, "4").unwrap();
    let (_, r) = call(port_a, &ha, "POST", "/api/conversations", &start).await;
    assert_eq!(r["conversation"]["relay_cap"], 4, "{r}");
}

#[tokio::test(flavor = "multi_thread")]
async fn via_api() {
    let (a, b) = (side("via-a").await, side("via-b").await);
    let code = b.node.open_window(PAIRING_WINDOW);
    let pb = a.node.add(b.addr, Some(code)).await.unwrap();
    let pa = b.node.peers().unwrap().remove(0);
    a.node.approve(pb.id).await.unwrap();
    b.node.approve(pa.id).await.unwrap();
    agents::add(&db::open(&b.paths.db_file()).unwrap(), "review", ".", "").unwrap();
    let (port, h) = web(a.chat.clone()).await;
    // CLI 출처 — 새 대화 · 이어 쓰기 모두 via 가 남고, 웹UI 글은 null.
    let body =
        serde_json::json!({ "peer_id": pb.id, "agent": "review", "text": "질문", "via": "cli" });
    let (code, r) = call(port, &h, "POST", "/api/conversations", &body.to_string()).await;
    assert_eq!(code, 201, "{r}");
    assert_eq!(r["message"]["via"], "cli");
    let conv = r["conversation"]["id"].as_i64().unwrap();
    let ms = messages(port, &h, conv, 2).await;
    assert_eq!(
        (ms[0]["via"].as_str(), ms[1]["via"].as_str()),
        (Some("cli"), None)
    );
    assert_eq!(
        ms[1]["reply_to"], ms[0]["id"],
        "답은 질문의 로컬 id 를 가리킨다"
    );
    let path = format!("/api/conversations/{conv}/messages");
    let (code, r) = call(port, &h, "POST", &path, r#"{"text":"메모","via":"cli"}"#).await;
    assert_eq!(code, 201, "{r}");
    assert_eq!(r["message"]["via"], "cli");
    let (code, r) = call(port, &h, "POST", &path, r#"{"text":"웹"}"#).await;
    assert_eq!(code, 201, "{r}");
    assert!(r["message"]["via"].is_null());
    // 모르는 출처는 거부.
    let (code, _) = call(port, &h, "POST", &path, r#"{"text":"x","via":"web"}"#).await;
    assert_eq!(code, 400);
    let body = serde_json::json!({ "peer_id": pb.id, "agent": "review", "text": "q", "via": "cli", "relay_agent": "dev" });
    assert_eq!(
        call(port, &h, "POST", "/api/conversations", &body.to_string())
            .await
            .0,
        400,
        "자동 왕복 시작의 via 는 버리지 않고 거절"
    );
    let body = serde_json::json!({ "peer_id": pb.id, "agent": "review", "text": "q", "via": "x" });
    assert_eq!(
        call(port, &h, "POST", "/api/conversations", &body.to_string())
            .await
            .0,
        400
    );
}

/// 두 쪽을 페어링 — `(a 가 본 b, b 가 본 a)`.
async fn pair_sides(a: &Side, b: &Side) -> (an_core::peers::Peer, an_core::peers::Peer) {
    let code = b.node.open_window(PAIRING_WINDOW);
    let pb = a.node.add(b.addr, Some(code)).await.unwrap();
    let pa = b
        .node
        .peers()
        .unwrap()
        .into_iter()
        .find(|p| p.name == a.node.me().name)
        .unwrap();
    a.node.approve(pb.id).await.unwrap();
    b.node.approve(pa.id).await.unwrap();
    (pb, pa)
}

/// 관람자 API(#1o) — A 가 B 와의 대화에 C 를 넣으면 대화 JSON `observers`, C 목록에 `dir: shared` 사본(`share.other` · 글 `owner`), B 에 `watchers`.
/// 사본 · 원 상대 · 없는 상대 거절, 빼기 · 나가기 204 · 다시 빼기 404.
#[tokio::test(flavor = "multi_thread")]
async fn share_observer_api() {
    let (a, b, c) = (
        side("share-a").await,
        side("share-b").await,
        side("share-c").await,
    );
    let (pb, _) = pair_sides(&a, &b).await;
    let (pc, _) = pair_sides(&a, &c).await;
    agents::add(&db::open(&b.paths.db_file()).unwrap(), "review", ".", "").unwrap();
    let (port, h) = web(a.chat.clone()).await;
    let body = serde_json::json!({ "peer_id": pb.id, "agent": "review", "text": "빌드 명령은?" });
    let (code, r) = call(port, &h, "POST", "/api/conversations", &body.to_string()).await;
    assert_eq!(code, 201, "{r}");
    let conv = r["conversation"]["id"].as_i64().unwrap();
    messages(port, &h, conv, 2).await;

    let path = format!("/api/conversations/{conv}/observers");
    // 원 상대 · 없는 상대는 거절.
    let (code, r) = call(
        port,
        &h,
        "POST",
        &path,
        &serde_json::json!({ "peer_id": pb.id }).to_string(),
    )
    .await;
    assert_eq!(code, 400, "{r}");
    let (code, _) = call(port, &h, "POST", &path, r#"{"peer_id":999}"#).await;
    assert!(matches!(code, 400 | 404), "{code}");
    let add = serde_json::json!({ "peer_id": pc.id }).to_string();
    let (code, r) = call(port, &h, "POST", &path, &add).await;
    assert_eq!(code, 204, "{r}");
    assert_eq!(
        call(port, &h, "POST", &path, &add).await.0,
        409,
        "이미 관람자"
    );
    let (_, list) = call(port, &h, "GET", "/api/conversations", "").await;
    assert_eq!(list[0]["observers"], serde_json::json!([pc.id]));

    // C — 사본이 이력과 함께 온다.
    let (cport, ch) = web(c.chat.clone()).await;
    let until = Instant::now() + Duration::from_secs(10);
    let copy = loop {
        let (_, list) = call(cport, &ch, "GET", "/api/conversations", "").await;
        if let Some(x) = list
            .as_array()
            .unwrap()
            .iter()
            .find(|x| x["dir"] == "shared")
        {
            let id = x["id"].as_i64().unwrap();
            let (_, ms) = call(
                cport,
                &ch,
                "GET",
                &format!("/api/conversations/{id}/messages"),
                "",
            )
            .await;
            if ms.as_array().unwrap().len() >= 2 {
                break (x.clone(), ms);
            }
        }
        assert!(Instant::now() < until, "사본을 기다렸다");
        tokio::time::sleep(Duration::from_millis(100)).await;
    };
    let (cx, ms) = copy;
    assert_eq!(cx["share"]["other"], "user-share-b");
    assert_eq!(cx["share"]["owner_dir"], "out");
    assert!(cx["share"]["ended"].is_null());
    assert!(cx.get("observers").is_none(), "사본에는 관람자 칸 없음");
    assert_eq!(
        (ms[0]["owner"].as_bool(), ms[1]["owner"].as_bool()),
        (Some(true), Some(false))
    );
    let cid = cx["id"].as_i64().unwrap();
    // 사본은 다시 공유 · 글쓰기 불가.
    let (code, _) = call(
        cport,
        &ch,
        "POST",
        &format!("/api/conversations/{cid}/observers"),
        r#"{"peer_id":1}"#,
    )
    .await;
    assert_eq!(code, 400);
    let (code, _) = call(
        cport,
        &ch,
        "POST",
        &format!("/api/conversations/{cid}/messages"),
        r#"{"text":"x"}"#,
    )
    .await;
    assert_eq!(code, 400);

    // B — 관람자 수.
    let (bport, bh) = web(b.chat.clone()).await;
    let until = Instant::now() + Duration::from_secs(10);
    loop {
        let (_, list) = call(bport, &bh, "GET", "/api/conversations", "").await;
        if list[0]["watchers"] == 1 {
            break;
        }
        assert!(Instant::now() < until, "관람자 수를 기다렸다: {list}");
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    // 나가기(C) → A 관람자 비고 사본은 '공유 해제됨'. 나간 뒤 A 의 빼기는 404.
    assert_eq!(
        call(
            cport,
            &ch,
            "POST",
            &format!("/api/conversations/{cid}/leave"),
            ""
        )
        .await
        .0,
        204
    );
    let until = Instant::now() + Duration::from_secs(10);
    loop {
        let (_, list) = call(port, &h, "GET", "/api/conversations", "").await;
        if list[0]["observers"] == serde_json::json!([]) {
            break;
        }
        assert!(Instant::now() < until, "나가기를 기다렸다: {list}");
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    let (_, list) = call(cport, &ch, "GET", "/api/conversations", "").await;
    assert_eq!(list[0]["share"]["ended"], true);
    assert_eq!(
        call(port, &h, "DELETE", &format!("{path}/{}", pc.id), "")
            .await
            .0,
        404
    );
    // 다시 넣고 빼기 204.
    assert_eq!(call(port, &h, "POST", &path, &add).await.0, 204);
    assert_eq!(
        call(port, &h, "DELETE", &format!("{path}/{}", pc.id), "")
            .await
            .0,
        204
    );
}

/// 대화 나가기 · 삭제 경로(#1w) — 모든 대화 나가기 204(목록 `left_at`), 도는 실행이 있으면 409(나가기 · 삭제), 삭제 204 뒤 목록에서 사라짐 · 다시 404.
#[tokio::test(flavor = "multi_thread")]
async fn leave_and_delete_routes() {
    let (a, b) = (side("leave-a").await, side("leave-b").await);
    let (pb, _) = pair_sides(&a, &b).await;
    agents::add(&db::open(&b.paths.db_file()).unwrap(), "review", ".", "").unwrap();
    let (port, h) = web(a.chat.clone()).await;
    let body = serde_json::json!({ "peer_id": pb.id, "agent": "review", "text": "질문" });
    let (code, r) = call(port, &h, "POST", "/api/conversations", &body.to_string()).await;
    assert_eq!(code, 201, "{r}");
    let conv = r["conversation"]["id"].as_i64().unwrap();
    messages(port, &h, conv, 2).await;
    let (_, list) = call(port, &h, "GET", "/api/conversations", "").await;
    assert!(list[0]["left_at"].is_null() && list[0]["peer_left_at"].is_null(), "{list}");
    // 도는 실행(이 PC) — 나가기 · 삭제 409.
    let db = db::open(&a.paths.db_file()).unwrap();
    db.execute(
        "UPDATE messages SET state = 'running' WHERE conversation_id = ?1 AND author = 'agent'",
        [conv],
    )
    .unwrap();
    let leave = format!("/api/conversations/{conv}/leave");
    let one = format!("/api/conversations/{conv}");
    let (code, r) = call(port, &h, "POST", &leave, "").await;
    assert_eq!(code, 409, "{r}");
    assert!(r["error"].as_str().unwrap().contains("실행이 끝난 뒤"), "{r}");
    assert_eq!(call(port, &h, "DELETE", &one, "").await.0, 409);
    db.execute(
        "UPDATE messages SET state = 'done' WHERE conversation_id = ?1 AND author = 'agent'",
        [conv],
    )
    .unwrap();
    assert_eq!(call(port, &h, "POST", &leave, "").await.0, 204);
    let (_, list) = call(port, &h, "GET", "/api/conversations", "").await;
    assert!(list[0]["left_at"].is_i64(), "{list}");
    // 나간 대화는 읽기 전용.
    let (code, _) = call(
        port,
        &h,
        "POST",
        &format!("/api/conversations/{conv}/messages"),
        r#"{"text":"더"}"#,
    )
    .await;
    assert_eq!(code, 400);
    assert_eq!(call(port, &h, "DELETE", &one, "").await.0, 204);
    let (_, list) = call(port, &h, "GET", "/api/conversations", "").await;
    assert!(list.as_array().unwrap().is_empty(), "{list}");
    assert_eq!(call(port, &h, "DELETE", &one, "").await.0, 404);
    assert_eq!(call(port, &h, "POST", &leave, "").await.0, 404);
}
