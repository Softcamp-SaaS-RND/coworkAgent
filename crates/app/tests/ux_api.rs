mod common;
use an_agent::chat::{Chat, Run};
use an_agent::runner::{RunInput, RunOutput, Session};
use an_app::notify::{self, Notifier};
use an_app::{chat_api, peers_api, server};
use an_core::{agents, chat, db, paths::Paths};
use an_net::identity::Identity;
use an_net::node::{Me, Node, PAIRING_WINDOW};
use serde_json::{Value, json};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

fn echo() -> Run {
    Arc::new(|input: RunInput| {
        let session = match &input.session {
            Session::New(id) | Session::Resume(id) => id.clone(),
        };
        Ok(RunOutput {
            text: format!("답: {}", input.text),
            session,
            is_error: false,
            turns: Some(1),
            duration_ms: Some(10),
        })
    })
}

struct Side {
    node: Arc<Node>,
    chat: Arc<Chat>,
    addr: std::net::SocketAddr,
    paths: Paths,
    port: u16,
    h: Vec<(&'static str, String)>,
}

/// 노드 · 대화 · 운영과 같은 로컬 API(피어 · 설정 · 대화) 한 벌.
async fn side(tag: &str) -> Side {
    let root = std::env::temp_dir().join(format!("an-app-ux-{tag}-{}", std::process::id()));
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
    let (port, h) = web(&paths, node.clone(), chat.clone()).await;
    Side {
        node,
        chat,
        addr,
        paths,
        port,
        h,
    }
}

async fn web(
    paths: &Paths,
    node: Arc<Node>,
    chat: Arc<Chat>,
) -> (u16, Vec<(&'static str, String)>) {
    let listener = server::bind().await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let token = server::new_token();
    let status = server::Status {
        version: "t".into(),
        data_dir: "d".into(),
        db: "ok".into(),
    };
    let notifier = Notifier::new(db::open(&paths.db_file()).unwrap(), |_| {}).unwrap();
    let app = server::router_with(
        port,
        token.clone(),
        status,
        || Ok(()),
        peers_api::routes(node)
            .merge(notify::routes(notifier))
            .merge(chat_api::routes(chat)),
    );
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let h = vec![
        common::host(port),
        common::origin(port),
        ("Cookie", format!("{}={token}", server::COOKIE)),
        ("Content-Type", "application/json".to_string()),
    ];
    (port, h)
}

impl Side {
    async fn call(&self, method: &str, path: &str, body: Value) -> (u16, Value) {
        let body = if body.is_null() {
            String::new()
        } else {
            body.to_string()
        };
        let (code, raw) = common::send_body(self.port, method, path, &self.h, &body).await;
        let text = raw.split("\r\n\r\n").nth(1).unwrap_or("");
        (code, serde_json::from_str(text).unwrap_or(Value::Null))
    }

    fn db(&self) -> db::Connection {
        db::open(&self.paths.db_file()).unwrap()
    }
}

/// a 와 b 를 페어링 — (a 쪽 b 의 id, b 쪽 a 의 id).
async fn pair(a: &Side, b: &Side) -> (i64, i64) {
    let code = b.node.open_window(PAIRING_WINDOW);
    let pb = a.node.add(b.addr, Some(code)).await.unwrap();
    a.node.approve(pb.id).await.unwrap();
    let pa = b.node.peers().unwrap().remove(0);
    assert!(pa.paired());
    (pb.id, pa.id)
}

/// 대화 `conv` 의 글이 `n` 개가 될 때까지(최대 5초).
async fn wait_messages(s: &Side, conv: i64, n: usize) {
    let until = Instant::now() + Duration::from_secs(5);
    while chat::messages(&s.db(), conv).unwrap().len() < n {
        assert!(Instant::now() < until, "글 {n} 개를 기다렸다");
        tokio::time::sleep(Duration::from_millis(30)).await;
    }
}

/// 새 대화로 묻고 답까지 기다린다 — 대화 id.
async fn ask(from: &Side, peer: i64, text: &str) -> i64 {
    let (code, r) = from
        .call(
            "POST",
            "/api/conversations",
            json!({ "peer_id": peer, "agent": "review", "text": text }),
        )
        .await;
    assert_eq!(code, 201, "{r}");
    let conv = r["conversation"]["id"].as_i64().unwrap();
    wait_messages(from, conv, 2).await;
    conv
}

fn texts(list: &Value) -> Vec<&str> {
    list.as_array()
        .unwrap()
        .iter()
        .map(|m| m["text"].as_str().unwrap())
        .collect()
}

#[tokio::test(flavor = "multi_thread")]
async fn favicon_served() {
    let a = side("fav").await;
    let mut req = "GET /favicon.ico HTTP/1.1\r\nConnection: close\r\n".to_string();
    for (k, v) in &a.h {
        req.push_str(&format!("{k}: {v}\r\n"));
    }
    req.push_str("\r\n");
    let mut s = tokio::net::TcpStream::connect(("127.0.0.1", a.port))
        .await
        .unwrap();
    s.write_all(req.as_bytes()).await.unwrap();
    let mut raw = Vec::new();
    s.read_to_end(&mut raw).await.unwrap();
    let split = raw.windows(4).position(|w| w == b"\r\n\r\n").unwrap();
    let head = String::from_utf8_lossy(&raw[..split]).to_lowercase();
    assert!(head.starts_with("http/1.1 200"), "{head}");
    assert!(head.contains("content-type: image/x-icon"), "{head}");
    let body = &raw[split + 4..];
    let ico = include_bytes!("../assets/coworkAgent.ico");
    assert_eq!(body, ico.as_slice(), "실행 파일과 같은 아이콘");
    assert_eq!(&body[..4], &[0, 0, 1, 0], "ICO 머리");
}

#[tokio::test(flavor = "multi_thread")]
async fn peer_patch_group_model_effort() {
    let (a, b) = (side("pp-a").await, side("pp-b").await);
    let (pb, _) = pair(&a, &b).await;
    let path = format!("/api/peers/{pb}");
    let (code, r) = a
        .call(
            "PATCH",
            &path,
            json!({ "group": " 개발팀 ", "model": "opus", "effort": "max" }),
        )
        .await;
    assert_eq!(code, 200, "{r}");
    assert_eq!(
        (&r["group"], &r["model"], &r["effort"]),
        (&json!("개발팀"), &json!("opus"), &json!("max"))
    );
    // 준 항목만 — 빈 문자열은 지움(전역 기본값으로).
    let (code, r) = a.call("PATCH", &path, json!({ "model": "" })).await;
    assert_eq!(code, 200, "{r}");
    assert_eq!(
        (&r["group"], &r["model"], &r["effort"]),
        (&json!("개발팀"), &Value::Null, &json!("max"))
    );
    let (_, list) = a.call("GET", "/api/peers", Value::Null).await;
    assert_eq!(list[0]["group"], "개발팀");
    for bad in [
        json!({ "model": "-x" }),
        json!({ "model": "a b" }),
        json!({ "effort": "HIGH" }),
        json!({ "group": "가".repeat(33) }),
    ] {
        let (code, r) = a.call("PATCH", &path, bad.clone()).await;
        assert_eq!(code, 400, "{bad} → {r}");
    }
    let (code, _) = a
        .call("PATCH", "/api/peers/9999", json!({ "group": "x" }))
        .await;
    assert_eq!(code, 404);
    // 틀린 값은 저장되지 않았다.
    let p = an_core::peers::by_id(&a.db(), pb).unwrap().unwrap();
    assert_eq!(
        (p.group.as_deref(), p.model, p.effort.as_deref()),
        (Some("개발팀"), None, Some("max"))
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn agent_patch_edits_but_not_name() {
    let a = side("ap").await;
    let folder = a.paths.root().display().to_string();
    let (code, made) = a
        .call(
            "POST",
            "/api/agents",
            json!({ "name": "review", "folder": folder }),
        )
        .await;
    assert_eq!(code, 201, "{made}");
    let path = format!("/api/agents/{}", made["id"]);
    let other = a.paths.root().join("sub");
    std::fs::create_dir_all(&other).unwrap();
    let (code, r) = a
        .call(
            "PATCH",
            &path,
            json!({ "folder": other.display().to_string(), "tools": "Read,Grep", "prompt": " 짧게 ", "group": "문서" }),
        )
        .await;
    assert_eq!(code, 200, "{r}");
    assert_eq!(r["name"], "review");
    assert_eq!(r["folder"], other.display().to_string());
    assert_eq!(
        (&r["tools"], &r["prompt"], &r["group"]),
        (&json!("Read,Grep"), &json!("짧게"), &json!("문서"))
    );
    // 준 항목만 · 빈 그룹은 지움.
    let (code, r) = a.call("PATCH", &path, json!({ "group": "" })).await;
    assert_eq!(code, 200, "{r}");
    assert_eq!(
        (&r["tools"], &r["group"]),
        (&json!("Read,Grep"), &Value::Null)
    );
    for bad in [
        json!({ "name": "renamed" }),
        json!({ "tools": "Read,Edit" }),
        json!({ "folder": "." }),
        json!({ "group": "가".repeat(33) }),
    ] {
        let (code, r) = a.call("PATCH", &path, bad.clone()).await;
        assert_eq!(code, 400, "{bad} → {r}");
    }
    assert!(
        agents::by_name(&a.db(), "review").unwrap().is_some(),
        "이름 그대로"
    );
    let (code, _) = a
        .call("PATCH", "/api/agents/9999", json!({ "prompt": "x" }))
        .await;
    assert_eq!(code, 404);
}

#[tokio::test(flavor = "multi_thread")]
async fn settings_claude_defaults() {
    let a = side("cd").await;
    let (code, s) = a.call("GET", "/api/settings", Value::Null).await;
    assert_eq!(code, 200);
    assert_eq!(
        (&s["claude_model"], &s["claude_effort"]),
        (&json!("sonnet"), &json!("high"))
    );
    let (code, s) = a
        .call(
            "PUT",
            "/api/settings",
            json!({ "claude_model": "opus", "claude_effort": "max" }),
        )
        .await;
    assert_eq!(code, 200, "{s}");
    assert_eq!(
        (&s["claude_model"], &s["claude_effort"]),
        (&json!("opus"), &json!("max"))
    );
    for bad in [
        json!({ "claude_model": "" }),
        json!({ "claude_model": "--dangerously-skip-permissions" }),
        json!({ "claude_effort": "turbo" }),
    ] {
        let (code, r) = a.call("PUT", "/api/settings", bad.clone()).await;
        assert_eq!(code, 400, "{bad} → {r}");
    }
    let (_, s) = a.call("GET", "/api/settings", Value::Null).await;
    assert_eq!(
        (&s["claude_model"], &s["claude_effort"]),
        (&json!("opus"), &json!("max"))
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn inbox_lists_their_messages_with_search() {
    let (a, b) = (side("in-a").await, side("in-b").await);
    let (pb, pa) = pair(&a, &b).await;
    agents::add(&b.db(), "review", ".", None, "").unwrap();
    ask(&a, pb, "할인 100% 맞나").await;
    tokio::time::sleep(Duration::from_millis(1100)).await; // 최신순은 초 단위 시각
    ask(&a, pb, "다른 질문 1000").await;
    // B 의 '나' 목록 = 상대(A) 쪽이 쓴 글, 최신부터.
    let (code, list) = b
        .call("GET", "/api/messages?side=theirs", Value::Null)
        .await;
    assert_eq!(code, 200, "{list}");
    assert_eq!(texts(&list), ["다른 질문 1000", "할인 100% 맞나"]);
    let first = &list[0];
    assert_eq!(first["mine"], false);
    assert_eq!(
        (
            &first["conversation"]["peer_id"],
            &first["conversation"]["agent"],
            &first["conversation"]["dir"]
        ),
        (&json!(pa), &json!("review"), &json!("in"))
    );
    // 검색 — % · _ 는 글자 그대로.
    let (_, list) = b
        .call("GET", "/api/messages?side=theirs&q=100%25", Value::Null)
        .await;
    assert_eq!(texts(&list), ["할인 100% 맞나"]);
    for q in ["_", "%5C1", "100%5C%25"] {
        // `_` · `\1` · `\%` 도 글자 그대로 — 어느 글에도 없다(`\` 를 안 막으면 `\1` 이 `1` 로 읽힌다).
        let (_, list) = b
            .call(
                "GET",
                &format!("/api/messages?side=theirs&q={q}"),
                Value::Null,
            )
            .await;
        assert!(texts(&list).is_empty(), "{q} → {list}");
    }
    let (_, list) = b
        .call("GET", "/api/messages?side=theirs&limit=1", Value::Null)
        .await;
    assert_eq!(texts(&list), ["다른 질문 1000"]);
    for bad in ["side=all", "side=theirs&limit=0", "side=theirs&limit=501"] {
        let (code, r) = b
            .call("GET", &format!("/api/messages?{bad}"), Value::Null)
            .await;
        assert_eq!(code, 400, "{bad} → {r}");
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn outbox_lists_my_messages_to_peer() {
    let (a, b, c) = (
        side("out-a").await,
        side("out-b").await,
        side("out-c").await,
    );
    let (pb, _) = pair(&a, &b).await;
    let (pc, _) = pair(&a, &c).await;
    agents::add(&b.db(), "review", ".", None, "").unwrap();
    agents::add(&c.db(), "review", ".", None, "").unwrap();
    ask(&a, pb, "B 에게").await;
    ask(&a, pc, "C 에게").await;
    let (code, list) = a
        .call(
            "GET",
            &format!("/api/messages?side=mine&peer={pb}"),
            Value::Null,
        )
        .await;
    assert_eq!(code, 200, "{list}");
    assert_eq!(
        texts(&list),
        ["B 에게"],
        "그 상대에게 내가 쓴 글만(답은 상대 쪽 글)"
    );
    assert_eq!(list[0]["mine"], true);
    assert_eq!(list[0]["conversation"]["dir"], "out");
    let (_, list) = a.call("GET", "/api/messages?side=mine", Value::Null).await;
    assert_eq!(list.as_array().unwrap().len(), 2);
}

#[tokio::test(flavor = "multi_thread")]
async fn theme_setting_persists() {
    let a = side("th").await;
    let (_, s) = a.call("GET", "/api/settings", Value::Null).await;
    assert_eq!(s["ui_theme"], "dark", "기본은 다크");
    let (code, s) = a
        .call("PUT", "/api/settings", json!({ "ui_theme": "light" }))
        .await;
    assert_eq!(code, 200, "{s}");
    assert_eq!(s["ui_theme"], "light");
    let (code, _) = a
        .call("PUT", "/api/settings", json!({ "ui_theme": "blue" }))
        .await;
    assert_eq!(code, 400);
    // 다시 열어도(새 서버 · 같은 DB) 유지.
    let (port, h) = web(&a.paths, a.node.clone(), a.chat.clone()).await;
    let (_, raw) = common::send_body(port, "GET", "/api/settings", &h, "").await;
    let s: Value = serde_json::from_str(raw.split("\r\n\r\n").nth(1).unwrap()).unwrap();
    assert_eq!(s["ui_theme"], "light");
}

#[tokio::test(flavor = "multi_thread")]
async fn to_human_rejected() {
    let (a, b) = (side("th-a").await, side("th-b").await);
    let (pb, _) = pair(&a, &b).await;
    agents::add(&b.db(), "review", ".", None, "").unwrap();
    let conv = ask(&a, pb, "질문").await;
    let path = format!("/api/conversations/{conv}/messages");
    let (code, r) = a
        .call("POST", &path, json!({ "text": "메모", "to": "human" }))
        .await;
    assert_eq!(code, 400, "{r}");
    assert_eq!(
        chat::messages(&a.db(), conv).unwrap().len(),
        2,
        "남지 않는다"
    );
    let (code, r) = a
        .call("POST", &path, json!({ "text": "다시", "to": "agent" }))
        .await;
    assert_eq!(code, 201, "{r}");
    assert_eq!(r["message"]["exec"], true);
}

/// 새 대화도 `to` 는 agent 만 — `human` · 모르는 값은 400 이고 양쪽 다 대화가 남지 않는다(상대 실행 없음).
#[tokio::test(flavor = "multi_thread")]
async fn new_conversation_to_human_rejected() {
    let (a, b) = (side("nh-a").await, side("nh-b").await);
    let (pb, _) = pair(&a, &b).await;
    agents::add(&b.db(), "review", ".", None, "").unwrap();
    let start = |to: &str| json!({ "peer_id": pb, "agent": "review", "text": "메모", "to": to });
    for to in ["human", "peer"] {
        let (code, r) = a.call("POST", "/api/conversations", start(to)).await;
        assert_eq!(code, 400, "{to} → {r}");
    }
    for s in [&a, &b] {
        assert!(chat::list(&s.db()).unwrap().is_empty(), "남지 않는다");
    }
    let (code, r) = a.call("POST", "/api/conversations", start("agent")).await;
    assert_eq!(code, 201, "{r}");
    assert_eq!(r["message"]["exec"], true);
}

/// 받은 글 종류 거르기는 건수 제한보다 먼저 — 최신 글이 다른 종류여도 더 오래된 맞는 글이 나온다.
#[tokio::test(flavor = "multi_thread")]
async fn inbox_kind_filter_applies_before_limit() {
    let (a, b) = (side("ik-a").await, side("ik-b").await);
    let (pb, pa) = pair(&a, &b).await;
    agents::add(&a.db(), "review", ".", None, "").unwrap();
    agents::add(&b.db(), "review", ".", None, "").unwrap();
    ask(&b, pa, "B 가 묻는다").await; // A 가 받은 사람 글 — 오래된 것
    ask(&a, pb, "A 가 묻는다").await; // A 가 받은 에이전트 답 — 최신
    for (kind, author) in [
        ("", "agent"),
        ("&kind=human", "human"),
        ("&kind=agent", "agent"),
    ] {
        let path = format!("/api/messages?side=theirs&limit=1{kind}");
        let (code, list) = a.call("GET", &path, Value::Null).await;
        assert_eq!(code, 200, "{list}");
        assert_eq!(list[0]["author"], author, "{kind} → {list}");
    }
    let (code, r) = a
        .call("GET", "/api/messages?side=theirs&kind=bot", Value::Null)
        .await;
    assert_eq!(code, 400, "{r}");
}

/// 설정을 못 읽으면(DB 오류) 기본값을 성공처럼 내지 않고 500.
#[tokio::test(flavor = "multi_thread")]
async fn settings_read_error_is_500() {
    let a = side("se").await;
    a.db().execute_batch("DROP TABLE settings").unwrap();
    let (code, r) = a.call("GET", "/api/settings", Value::Null).await;
    assert_eq!(code, 500, "{r}");
}

/// 대화 목록 첫 줄(가장 최근 대화)의 제목.
async fn first_title(s: &Side) -> Value {
    let (_, list) = s.call("GET", "/api/conversations", Value::Null).await;
    list[0]["title"].clone()
}

/// 1×1 PNG 앞부분 — 서버는 PNG data URL 모양(서명 · base64 글자 · 길이)만 본다.
fn png() -> String {
    format!(
        "{}AAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR4nGNgYGD4DwABBAEAwS2OUAAAAABJRU5ErkJggg==",
        an_core::run_opts::ICON_PREFIX
    )
}

#[tokio::test(flavor = "multi_thread")]
async fn conversation_title_set_and_edit() {
    let (a, b) = (side("ti-a").await, side("ti-b").await);
    let (pb, _) = pair(&a, &b).await;
    agents::add(&b.db(), "review", ".", None, "").unwrap();
    // 틀린 제목이면 보내지 않는다.
    let (code, r) = a
        .call(
            "POST",
            "/api/conversations",
            json!({ "peer_id": pb, "agent": "review", "text": "질문", "title": "제".repeat(101) }),
        )
        .await;
    assert_eq!(code, 400, "{r}");
    assert!(
        chat::summaries(&a.db()).unwrap().is_empty(),
        "대화가 생기지 않았다"
    );
    let (code, r) = a
        .call(
            "POST",
            "/api/conversations",
            json!({ "peer_id": pb, "agent": "review", "text": "배포 순서 알려줘", "title": " 배포 문의 " }),
        )
        .await;
    assert_eq!(code, 201, "{r}");
    assert!(r.get("warning").is_none(), "{r}");
    let conv = r["conversation"]["id"].as_i64().unwrap();
    wait_messages(&a, conv, 2).await;
    assert_eq!(first_title(&a).await, "배포 문의");
    assert_eq!(
        first_title(&b).await,
        "배포 순서 알려줘",
        "상대에는 보내지 않는다(첫 글 앞부분)"
    );
    let path = format!("/api/conversations/{conv}");
    let (code, r) = a.call("PATCH", &path, json!({ "title": "새 제목" })).await;
    assert_eq!((code, &r["title"]), (200, &json!("새 제목")), "{r}");
    assert_eq!(first_title(&a).await, "새 제목");
    let (code, r) = a.call("PATCH", &path, json!({ "title": "  " })).await;
    assert_eq!((code, &r["title"]), (200, &Value::Null), "{r}");
    assert_eq!(
        first_title(&a).await,
        "배포 순서 알려줘",
        "지우면 첫 글 앞부분"
    );
    for bad in [
        json!({ "title": "제".repeat(101) }),
        json!({ "title": "a\nb" }),
    ] {
        let (code, r) = a.call("PATCH", &path, bad.clone()).await;
        assert_eq!(code, 400, "{bad} → {r}");
    }
    let (code, _) = a
        .call("PATCH", &path, json!({ "title": "x", "peer_id": 1 }))
        .await;
    assert!((400..500).contains(&code), "모르는 항목 거절 — {code}");
    let (code, _) = a
        .call("PATCH", "/api/conversations/9999", json!({ "title": "x" }))
        .await;
    assert_eq!(code, 404);
    assert_eq!(
        first_title(&a).await,
        "배포 순서 알려줘",
        "거절된 요청은 바꾸지 않았다"
    );
    // 상대가 거절해도(없는 에이전트) 대화는 남는다 — 제목도 함께.
    let (code, r) = a
        .call(
            "POST",
            "/api/conversations",
            json!({ "peer_id": pb, "agent": "없음", "text": "질문", "title": "거절된 질문" }),
        )
        .await;
    assert_eq!(code, 403, "{r}");
    assert_eq!(first_title(&a).await, "거절된 질문");
    // 자동 왕복 대화도 같은 제목 규칙.
    agents::add(&a.db(), "mine", ".", None, "").unwrap();
    let (code, r) = a
        .call(
            "POST",
            "/api/conversations",
            json!({ "peer_id": pb, "agent": "review", "text": "왕복", "relay_agent": "mine", "relay_cap": 1, "title": "왕복 제목" }),
        )
        .await;
    assert_eq!(code, 201, "{r}");
    assert_eq!(first_title(&a).await, "왕복 제목");
}

#[tokio::test(flavor = "multi_thread")]
async fn peer_tile_alias_icon() {
    let (a, b) = (side("tl-a").await, side("tl-b").await);
    let (pb, _) = pair(&a, &b).await;
    let path = format!("/api/peers/{pb}");
    let (code, _) = a.call("PATCH", &path, json!({ "group": "개발팀" })).await;
    assert_eq!(code, 200);
    let (code, r) = a
        .call(
            "PATCH",
            &path,
            json!({ "alias": " 세일즈 ", "icon": png() }),
        )
        .await;
    assert_eq!(code, 200, "{r}");
    assert_eq!(
        (&r["alias"], &r["icon"], &r["name"], &r["group"]),
        (
            &json!("세일즈"),
            &json!(png()),
            &json!("user-tl-b"),
            &json!("개발팀")
        ),
        "원래 이름 · 다른 항목은 그대로"
    );
    let (code, r) = a.call("PATCH", &path, json!({ "alias": "" })).await;
    assert_eq!(code, 200, "{r}");
    assert_eq!(
        (&r["alias"], &r["icon"]),
        (&Value::Null, &json!(png())),
        "준 항목만"
    );
    // 준 쪽만 — 이전 버전이 남긴 틀린 실행 설정이 있어도 타일 · 그룹은 고치고, 저장된 값은 그대로 둔다.
    an_core::peers::set_meta(&a.db(), pb, Some("영업팀"), Some("bad model"), None).unwrap();
    let (code, r) = a.call("PATCH", &path, json!({ "alias": "세일즈" })).await;
    assert_eq!(
        (code, &r["group"], &r["model"]),
        (200, &json!("영업팀"), &json!("bad model")),
        "{r}"
    );
    let (code, r) = a.call("PATCH", &path, json!({ "group": "개발팀" })).await;
    assert_eq!((code, &r["model"]), (200, &json!("bad model")), "{r}");
    let (code, r) = a.call("PATCH", &path, json!({})).await;
    assert_eq!(
        (code, &r["alias"]),
        (200, &json!("세일즈")),
        "빈 요청 = 그대로"
    );
    let (code, _) = a.call("PATCH", &path, json!({ "alias": "" })).await;
    assert_eq!(code, 200);
    for bad in [
        json!({ "icon": "data:image/svg+xml;base64,PHN2Zz4=" }),
        json!({ "icon": "javascript:alert(1)" }),
        json!({ "icon": format!("{}\"><img>", an_core::run_opts::ICON_PREFIX) }),
        json!({ "alias": "가".repeat(33) }),
        json!({ "alias": "새 이름", "icon": "x" }),
    ] {
        let (code, r) = a.call("PATCH", &path, bad.clone()).await;
        assert_eq!(code, 400, "{bad} → {r}");
    }
    let (_, list) = a.call("GET", "/api/peers", Value::Null).await;
    assert_eq!(
        (&list[0]["alias"], &list[0]["icon"]),
        (&Value::Null, &json!(png())),
        "틀린 요청은 하나도 쓰지 않았다"
    );
    let (_, list) = b.call("GET", "/api/peers", Value::Null).await;
    assert_eq!(
        (&list[0]["alias"], &list[0]["icon"]),
        (&Value::Null, &Value::Null),
        "상대에는 알리지 않는다"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn settings_me_tile_side_width() {
    let a = side("mt").await;
    let (_, s) = a.call("GET", "/api/settings", Value::Null).await;
    assert_eq!(
        (&s["ui_side_width"], &s["me_alias"], &s["me_icon"]),
        (&json!(272), &Value::Null, &Value::Null)
    );
    let (code, s) = a
        .call(
            "PUT",
            "/api/settings",
            json!({ "ui_side_width": 400, "me_alias": " 김팀장 ", "me_icon": png() }),
        )
        .await;
    assert_eq!(code, 200, "{s}");
    assert_eq!(
        (&s["ui_side_width"], &s["me_alias"], &s["me_icon"]),
        (&json!(400), &json!("김팀장"), &json!(png()))
    );
    for edge in [200, 560] {
        let (code, s) = a
            .call("PUT", "/api/settings", json!({ "ui_side_width": edge }))
            .await;
        assert_eq!(
            (code, &s["ui_side_width"]),
            (200, &json!(edge)),
            "경계 {edge}"
        );
    }
    let (code, _) = a
        .call("PUT", "/api/settings", json!({ "ui_side_width": 400 }))
        .await;
    assert_eq!(code, 200);
    for bad in [
        json!({ "ui_side_width": 199 }),
        json!({ "ui_side_width": 561 }),
        json!({ "me_icon": "data:image/gif;base64,R0lGODlh" }),
        json!({ "me_alias": "a\nb" }),
        json!({ "ui_side_width": 300, "me_alias": "가".repeat(33) }),
    ] {
        let (code, r) = a.call("PUT", "/api/settings", bad.clone()).await;
        assert_eq!(code, 400, "{bad} → {r}");
    }
    let (_, s) = a.call("GET", "/api/settings", Value::Null).await;
    assert_eq!(
        (&s["ui_side_width"], &s["me_alias"]),
        (&json!(400), &json!("김팀장")),
        "틀린 요청은 하나도 쓰지 않았다"
    );
    let (code, s) = a
        .call(
            "PUT",
            "/api/settings",
            json!({ "me_alias": "", "me_icon": "" }),
        )
        .await;
    assert_eq!(code, 200, "{s}");
    assert_eq!(
        (&s["me_alias"], &s["me_icon"]),
        (&Value::Null, &Value::Null)
    );
}

/// 주소+코드 직접 추가 — 코드가 맞아 상대(초대 생성 측)가 승인하면 API 한 번으로 양쪽이 페어링된다(초대 붙여넣기와 같다).
#[tokio::test]
async fn direct_add_with_code_pairs_both_sides() {
    let (a, b) = (side("dadd-a").await, side("dadd-b").await);
    let code = b.node.open_window(PAIRING_WINDOW);
    let (st, v) = a
        .call(
            "POST",
            "/api/peers",
            json!({ "addr": b.addr.to_string(), "code": code }),
        )
        .await;
    assert_eq!(st, 201, "{v}");
    assert_eq!(
        (&v["my_ok"], &v["their_ok"], &v["paired"]),
        (&json!(true), &json!(true), &json!(true)),
        "{v}"
    );
    assert!(b.node.peers().unwrap().remove(0).paired());
    let id = v["id"].as_i64().unwrap();
    let (st, _) = a
        .call("POST", &format!("/api/peers/{id}/ping"), Value::Null)
        .await;
    assert_eq!(st, 204);
}

/// 아는 키의 다시 추가는 코드가 있든 없든 상대가 자동 승인하지 않으므로 붙인 쪽도 자동 승인하지 않는다 — 승인 대기로 남는다.
#[tokio::test]
async fn direct_readd_known_key_waits_for_approval() {
    let (a, b) = (side("dre-a").await, side("dre-b").await);
    pair(&a, &b).await;
    for code in [None, Some(b.node.open_window(PAIRING_WINDOW))] {
        let (st, v) = a
            .call(
                "POST",
                "/api/peers",
                json!({ "addr": b.addr.to_string(), "code": code }),
            )
            .await;
        assert_eq!(st, 201, "{v}");
        assert_eq!(
            (&v["my_ok"], &v["their_ok"], &v["paired"]),
            (&json!(false), &json!(false), &json!(false)),
            "{v}"
        );
    }
}

/// 쿼리 문자열 값 — 영문 · 숫자 밖의 UTF-8 바이트는 %XX.
fn enc(s: &str) -> String {
    s.bytes()
        .map(|b| {
            if b.is_ascii_alphanumeric() {
                (b as char).to_string()
            } else {
                format!("%{b:02X}")
            }
        })
        .collect()
}

/// 통합 검색 — 메시지 본문(양쪽 글) · 대화 제목 · 사람(이름 · 호스트) · 에이전트(내 것 · 대화에 나온 상대 것).
#[tokio::test(flavor = "multi_thread")]
async fn search_groups_messages_titles_peers_agents() {
    let (a, b) = (side("sg-a").await, side("sg-b").await);
    let (pb, _) = pair(&a, &b).await;
    agents::add(&b.db(), "review", ".", None, "").unwrap();
    agents::add(&a.db(), "billing-bot", ".", None, "").unwrap();
    let (code, r) = a
        .call(
            "POST",
            "/api/conversations",
            json!({ "peer_id": pb, "agent": "review", "text": "결제 모듈 구조 알려줘", "title": "정산 검토" }),
        )
        .await;
    assert_eq!(code, 201, "{r}");
    let conv = r["conversation"]["id"].as_i64().unwrap();
    wait_messages(&a, conv, 2).await;
    let search = |q: String| {
        let a = &a;
        async move {
            let (code, r) = a
                .call("GET", &format!("/api/search?q={q}"), Value::Null)
                .await;
            assert_eq!(code, 200, "{q} → {r}");
            r
        }
    };
    // 메시지 — 내 질문과 상대 에이전트 답 둘 다, 대화 정보 · 제목과 함께. 제목에는 없다.
    let r = search(enc("결제")).await;
    let ms = r["messages"].as_array().unwrap();
    assert_eq!(ms.len(), 2, "{r}");
    assert!(
        ms.iter()
            .all(|m| m["conversation"]["id"] == conv && m["conversation"]["title"] == "정산 검토"),
        "{r}"
    );
    assert!(r["conversations"].as_array().unwrap().is_empty(), "{r}");
    // 대화 제목.
    let r = search(enc("정산")).await;
    assert_eq!(r["conversations"][0]["id"], conv, "{r}");
    assert!(r["messages"].as_array().unwrap().is_empty(), "{r}");
    // 사람 — 이름 · 호스트.
    let r = search("sg-b".into()).await;
    assert_eq!(r["peers"].as_array().unwrap().len(), 1, "{r}");
    assert_eq!(r["peers"][0]["id"], pb);
    // 에이전트 — 내 것은 owner "me", 대화에 나온 상대 것은 그 상대 id.
    let r = search("billing".into()).await;
    assert_eq!(
        r["agents"],
        json!([{ "name": "billing-bot", "owner": "me" }]),
        "{r}"
    );
    let r = search("revi".into()).await;
    assert_eq!(
        r["agents"],
        json!([{ "name": "review", "owner": pb }]),
        "{r}"
    );
    for bad in [
        "/api/search?q=".to_string(),
        "/api/search?q=%20%20".to_string(),
        format!("/api/search?q={}", "x".repeat(101)),
    ] {
        let (code, r) = a.call("GET", &bad, Value::Null).await;
        assert_eq!(code, 400, "{bad} → {r}");
    }
    // 지운 상대의 에이전트는 나오지 않는다(대화 이력은 남는다).
    an_core::peers::remove(&a.db(), pb).unwrap();
    let r = search("revi".into()).await;
    assert_eq!(r["agents"], json!([]), "{r}");
}

/// 검색어의 % · _ · \ 는 글자 그대로(LIKE 와일드카드 아님) — 메시지 · 대화 제목 둘 다.
#[tokio::test(flavor = "multi_thread")]
async fn search_treats_like_wildcards_literally() {
    let (a, b) = (side("sw-a").await, side("sw-b").await);
    let (pb, _) = pair(&a, &b).await;
    agents::add(&b.db(), "review", ".", None, "").unwrap();
    ask(&a, pb, "할인 100% 맞나").await;
    ask(&a, pb, "a_b 변수").await;
    let found = |q: &'static str| {
        let a = &a;
        async move {
            let (_, r) = a
                .call("GET", &format!("/api/search?q={q}"), Value::Null)
                .await;
            (
                r["messages"].as_array().unwrap().len(),
                r["conversations"].as_array().unwrap().len(),
            )
        }
    };
    assert_eq!(found("%25").await, (2, 1), "% — 질문 · 그 답 · 그 대화만");
    assert_eq!(found("_").await, (2, 1), "_ — 질문 · 그 답 · 그 대화만");
    assert_eq!(found("%5C").await, (0, 0), "역슬래시는 어디에도 없다");
}

/// 내 활동 — 상대 쪽 글(사람 · 에이전트) · 내 쪽 실패 · 자동 왕복 시작 · 끝을 최신순으로 합치고 종류 · 본문 · 상대로 거른다.
#[tokio::test(flavor = "multi_thread")]
async fn activity_merges_theirs_failures_relay_events() {
    let (a, b) = (side("ac-a").await, side("ac-b").await);
    let (pb, pa) = pair(&a, &b).await;
    agents::add(&a.db(), "review", ".", None, "").unwrap();
    agents::add(&b.db(), "review", ".", None, "").unwrap();
    let mine = ask(&a, pb, "A 의 질문").await; // A 가 받은 에이전트 답
    ask(&b, pa, "B 의 질문").await; // A 가 받은 사람 글
    // 내 쪽 실패 — 내 질문을 실패로.
    let asked = chat::messages(&a.db(), mine).unwrap().remove(0);
    chat::set_state(&a.db(), asked.id, chat::State::Error).unwrap();
    chat::add_relay_event(&a.db(), mine, chat::RelayMark::Start, "자동 왕복 미리보기").unwrap();
    chat::add_relay_event(&a.db(), mine, chat::RelayMark::End, "상한 도달").unwrap();
    // 시각을 벌린다 — 끝(가장 최근) > 두 글 > 시작 > 실패(가장 오래). 같은 초면 순서를 가릴 수 없다.
    let db = a.db();
    db.execute(
        "UPDATE messages SET created_at = created_at - 100 WHERE id = ?1",
        [asked.id],
    )
    .unwrap();
    for (kind, by) in [("start", -50), ("end", 50)] {
        db.execute(
            "UPDATE relay_events SET created_at = created_at + ?2 WHERE kind = ?1",
            (kind, by),
        )
        .unwrap();
    }
    let get = |q: &'static str| {
        let a = &a;
        async move {
            let (code, r) = a
                .call("GET", &format!("/api/activity{q}"), Value::Null)
                .await;
            assert_eq!(code, 200, "{q} → {r}");
            r
        }
    };
    let kinds = |r: &Value| {
        let mut k: Vec<String> = r["items"]
            .as_array()
            .unwrap()
            .iter()
            .map(|i| {
                let author = i["message"]["author"].as_str().unwrap_or("");
                format!("{}/{author}", i["kind"].as_str().unwrap())
            })
            .collect();
        k.sort();
        k
    };
    let all = get("").await;
    assert_eq!(
        kinds(&all),
        [
            "failed/human",
            "message/agent",
            "message/human",
            "relay_end/",
            "relay_start/"
        ],
        "{all}"
    );
    let at: Vec<i64> = all["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["at"].as_i64().unwrap())
        .collect();
    assert!(at.windows(2).all(|w| w[0] >= w[1]), "최신순 {at:?}");
    let order: Vec<&str> = all["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["kind"].as_str().unwrap())
        .collect();
    assert_eq!(
        order,
        ["relay_end", "message", "message", "relay_start", "failed"],
        "{all}"
    );
    // 합친 뒤 자른다 — 두 목록 각각의 최신이 아니라 합친 최신 2건.
    let two = get("?limit=2").await;
    assert_eq!(
        two["items"]
            .as_array()
            .unwrap()
            .iter()
            .map(|i| i["kind"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["relay_end", "message"],
        "{two}"
    );
    let end = all["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["kind"] == "relay_end")
        .unwrap()
        .clone();
    assert_eq!(
        (
            &end["detail"],
            &end["conversation"]["id"],
            &end["conversation"]["peer_id"]
        ),
        (&json!("상한 도달"), &json!(mine), &json!(pb))
    );
    assert_eq!(kinds(&get("?kind=human").await), ["message/human"]);
    assert_eq!(kinds(&get("?kind=agent").await), ["message/agent"]);
    assert_eq!(kinds(&get("?kind=failed").await), ["failed/human"]);
    assert_eq!(
        kinds(&get("?kind=relay").await),
        ["relay_end/", "relay_start/"]
    );
    let q = get("?q=%EC%83%81%ED%95%9C").await; // "상한"
    assert_eq!(kinds(&q), ["relay_end/"], "{q}");
    let none = get("?peer=999").await;
    assert!(none["items"].as_array().unwrap().is_empty(), "{none}");
    let (code, r) = a.call("GET", "/api/activity?kind=bogus", Value::Null).await;
    assert_eq!(code, 400, "{r}");
}

/// 내 활동 안 본 수 — 상대별 마지막 열람 뒤의 것. 열람은 상대 하나 또는 전부.
#[tokio::test(flavor = "multi_thread")]
async fn activity_unseen_by_peer_and_mark_seen() {
    let (a, b, c) = (side("au-a").await, side("au-b").await, side("au-c").await);
    let (_, pa) = pair(&b, &a).await;
    let (_, pa2) = pair(&c, &a).await;
    agents::add(&a.db(), "review", ".", None, "").unwrap();
    ask(&b, pa, "B 가 묻는다").await;
    ask(&c, pa2, "C 가 묻는다").await;
    // A 쪽에서 본 상대 id.
    let peer_of = |name: &str| {
        a.node
            .peers()
            .unwrap()
            .into_iter()
            .find(|p| p.name == name)
            .unwrap()
            .id
    };
    let (ab, ac) = (peer_of("user-au-b"), peer_of("user-au-c"));
    let unseen = |q: String| {
        let a = &a;
        async move {
            let (_, r) = a
                .call("GET", &format!("/api/activity{q}"), Value::Null)
                .await;
            (
                r["unseen"].as_i64().unwrap(),
                r["unseen_all"].as_i64().unwrap(),
            )
        }
    };
    // 화면이 받은 범위 — GET 이 돌려준 마지막 글 · 자동 왕복 기록 id. 본 표시는 그 범위까지만.
    let shown = |q: String| {
        let a = &a;
        async move {
            let (_, r) = a
                .call("GET", &format!("/api/activity{q}"), Value::Null)
                .await;
            r["upto"].clone()
        }
    };
    let seen = |peer: Option<i64>, upto: Value| {
        let a = &a;
        async move {
            let mut body = json!({ "message": upto["message"], "event": upto["event"] });
            if let Some(p) = peer {
                body["peer"] = json!(p);
            }
            a.call("POST", "/api/activity/seen", body).await.0
        }
    };
    assert_eq!(unseen(String::new()).await, (2, 2));
    assert_eq!(unseen(format!("?peer={ab}")).await, (1, 2));
    assert_eq!(
        seen(Some(ab), shown(format!("?peer={ab}")).await).await,
        204
    );
    assert_eq!(unseen(format!("?peer={ab}")).await, (0, 1));
    assert_eq!(unseen(format!("?peer={ac}")).await, (1, 1));
    let (_, r) = a
        .call("GET", &format!("/api/activity?peer={ab}"), Value::Null)
        .await;
    assert_eq!(r["items"][0]["unseen"], false, "{r}");
    assert_eq!(seen(None, shown(String::new()).await).await, 204);
    assert_eq!(unseen(String::new()).await, (0, 0));
    // 본 뒤에 온 것은 같은 초여도 안 본 것(시각이 아니라 id).
    ask(&b, pa, "B 가 또 묻는다").await;
    assert_eq!(unseen(format!("?peer={ab}")).await, (1, 1));
    // 자동 왕복 기록 · 내 쪽 실패도 센다.
    let latest = chat::activity(&a.db(), Some(ab), chat::ActivityFilter::All, None, 1)
        .unwrap()
        .remove(0);
    chat::add_relay_event(
        &a.db(),
        latest.conversation_id,
        chat::RelayMark::Start,
        "시작",
    )
    .unwrap();
    let answered = chat::messages(&a.db(), latest.conversation_id)
        .unwrap()
        .pop()
        .unwrap();
    assert!(answered.mine, "{answered:?}");
    chat::set_state(&a.db(), answered.id, chat::State::Error).unwrap();
    assert_eq!(
        unseen(format!("?peer={ab}")).await,
        (3, 3),
        "글 · 자동 왕복 · 실패"
    );
    // 받은 뒤 본 표시를 보내기 전에 온 것은 안 본 것으로 남는다.
    let upto = shown(String::new()).await;
    ask(&c, pa2, "C 가 또 묻는다").await;
    assert_eq!(seen(None, upto).await, 204);
    assert_eq!(unseen(String::new()).await, (1, 1), "C 의 새 글");
    // 지금 마지막 항목을 넘는 범위는 거기까지만 — 앞으로 올 것을 미리 본 것으로 하지 않는다.
    assert_eq!(
        seen(None, json!({ "message": 1_000_000, "event": 1_000_000 })).await,
        204
    );
    assert_eq!(unseen(String::new()).await, (0, 0));
    ask(&b, pa, "B 가 세 번째").await;
    assert_eq!(unseen(String::new()).await, (1, 1));
    // 수만(limit=0) — 목록 없이 안 본 수(5초 새로 고침).
    let (code, r) = a.call("GET", "/api/activity?limit=0", Value::Null).await;
    assert_eq!(code, 200, "{r}");
    assert_eq!(
        (r["items"].as_array().map(Vec::len), &r["unseen_all"]),
        (Some(0), &json!(1)),
        "{r}"
    );
    // 범위 없는 본 표시(빈 몸 · Content-Type 없음 · 범위 빠짐)는 거절 — 본 범위를 모른 채 전부를 본 것으로 하지 않는다.
    let bare: Vec<(&str, String)> =
        a.h.iter()
            .filter(|(k, _)| *k != "Content-Type")
            .cloned()
            .collect();
    let (code, _) = common::send_body(a.port, "POST", "/api/activity/seen", &bare, "").await;
    assert_eq!(code, 415);
    let (code, _) = a
        .call("POST", "/api/activity/seen", json!({ "peer": ab }))
        .await;
    assert_eq!(code, 422);
    assert_eq!(unseen(String::new()).await, (1, 1), "그대로");
    // 본 뒤에 실패로 바뀐 내 글(이미 본 id) — 글 id 는 그대로라 실패 기록으로 다시 센다. 기록은 자동 왕복 목록에 나오지 않는다.
    assert_eq!(seen(None, shown(String::new()).await).await, 204);
    let old = chat::list(&a.db())
        .unwrap()
        .into_iter()
        .filter(|c| c.peer_id == ab)
        .flat_map(|c| chat::messages(&a.db(), c.id).unwrap())
        .find(|m| m.mine && m.state == chat::State::Done)
        .unwrap();
    chat::set_state(&a.db(), old.id, chat::State::Error).unwrap();
    assert_eq!(unseen(format!("?peer={ab}")).await, (1, 1), "늦은 실패");
    let (_, r) = a
        .call(
            "GET",
            &format!("/api/activity?peer={ab}&kind=failed"),
            Value::Null,
        )
        .await;
    assert!(
        r["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|i| i["message"]["id"] == old.id && i["unseen"] == true),
        "{r}"
    );
    let (_, r) = a
        .call(
            "GET",
            &format!("/api/activity?peer={ab}&kind=relay"),
            Value::Null,
        )
        .await;
    assert_eq!(r["items"].as_array().map(Vec::len), Some(1), "{r}");
    // 지운 상대의 이력은 남고 본 것으로 센다 — 본 표시를 둘 곳이 없어 영영 안 본 것으로 남지 않게.
    assert_eq!(seen(None, shown(String::new()).await).await, 204);
    a.db()
        .execute(
            "UPDATE peers SET activity_seen_msg = 0, activity_seen_event = 0 WHERE id = ?1",
            [ac],
        )
        .unwrap();
    assert_eq!(unseen(String::new()).await, (2, 2), "C 의 두 글");
    an_core::peers::remove(&a.db(), ac).unwrap();
    assert_eq!(unseen(String::new()).await, (0, 0));
    let (_, r) = a.call("GET", "/api/activity", Value::Null).await;
    assert!(
        r["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|i| i["conversation"]["peer_id"] == ac && i["unseen"] == false),
        "{r}"
    );
    assert_eq!(seen(Some(999), shown(String::new()).await).await, 404);
}

/// ★ 즐겨찾기 — 이 PC 에만, 상대 목록은 그대로. 제목과 함께 바꿀 수 있고, 둘 다 없거나 모르는 항목이면 거절.
#[tokio::test(flavor = "multi_thread")]
async fn star_toggles_local_only() {
    let (a, b) = (side("st-a").await, side("st-b").await);
    let (pb, _) = pair(&a, &b).await;
    agents::add(&b.db(), "review", ".", None, "").unwrap();
    let conv = ask(&a, pb, "질문").await;
    let path = format!("/api/conversations/{conv}");
    let (code, r) = a.call("PATCH", &path, json!({ "starred": true })).await;
    assert_eq!(code, 200, "{r}");
    assert_eq!(r, json!({ "id": conv, "title": null, "starred": true }));
    let (_, list) = a.call("GET", "/api/conversations", Value::Null).await;
    assert_eq!(list[0]["starred"], true);
    let (_, list) = b.call("GET", "/api/conversations", Value::Null).await;
    assert_eq!(list[0]["starred"], false, "상대에 보내지 않는다");
    let (code, r) = a
        .call("PATCH", &path, json!({ "title": "검토", "starred": false }))
        .await;
    assert_eq!(code, 200, "{r}");
    assert_eq!(r, json!({ "id": conv, "title": "검토", "starred": false }));
    for bad in [json!({}), json!({ "pin": true })] {
        let (code, r) = a.call("PATCH", &path, bad.clone()).await;
        assert!(code == 400 || code == 422, "{bad} → {code} {r}");
    }
    let (code, _) = a
        .call(
            "PATCH",
            "/api/conversations/999",
            json!({ "starred": true }),
        )
        .await;
    assert_eq!(code, 404);
}

/// 저장된 항목 — 메시지 저장 · 해제(이 PC 에만), 저장 순 최신부터, 상대 · 대화로 거른다.
#[tokio::test(flavor = "multi_thread")]
async fn saved_toggle_and_list_scoped() {
    let (a, b, c) = (side("sv-a").await, side("sv-b").await, side("sv-c").await);
    let (pb, _) = pair(&a, &b).await;
    let (pc, _) = pair(&a, &c).await;
    agents::add(&b.db(), "review", ".", None, "").unwrap();
    agents::add(&c.db(), "review", ".", None, "").unwrap();
    let cb = ask(&a, pb, "B 에게").await;
    let cc = ask(&a, pc, "C 에게").await;
    let (rb, qc) = (
        chat::messages(&a.db(), cb).unwrap().remove(1),
        chat::messages(&a.db(), cc).unwrap().remove(0),
    );
    for id in [rb.id, qc.id] {
        let (code, r) = a
            .call(
                "PATCH",
                &format!("/api/messages/{id}"),
                json!({ "saved": true }),
            )
            .await;
        assert_eq!(code, 200, "{r}");
        assert_eq!(r, json!({ "id": id, "saved": true }));
    }
    let ids = |r: &Value| -> Vec<i64> {
        r.as_array()
            .unwrap()
            .iter()
            .map(|m| m["id"].as_i64().unwrap())
            .collect()
    };
    let (code, all) = a.call("GET", "/api/saved", Value::Null).await;
    assert_eq!(code, 200, "{all}");
    assert_eq!(
        ids(&all),
        [qc.id, rb.id],
        "저장 시각 최신부터(같은 초면 id 큰 것)"
    );
    assert_eq!(all[0]["saved"], true);
    assert_eq!(all[0]["conversation"]["id"], cc);
    // 저장 시각이 다르면 그것으로 — id 순과 반대가 되게 벌린다.
    a.db()
        .execute(
            "UPDATE messages SET saved_at = saved_at + 10 WHERE id = ?1",
            [rb.id],
        )
        .unwrap();
    let (_, r) = a.call("GET", "/api/saved", Value::Null).await;
    assert_eq!(ids(&r), [rb.id, qc.id], "{r}");
    let (_, r) = a
        .call("GET", &format!("/api/saved?peer={pb}"), Value::Null)
        .await;
    assert_eq!(ids(&r), [rb.id]);
    let (_, r) = a
        .call("GET", &format!("/api/saved?conversation={cc}"), Value::Null)
        .await;
    assert_eq!(ids(&r), [qc.id]);
    let (_, ms) = a
        .call(
            "GET",
            &format!("/api/conversations/{cb}/messages"),
            Value::Null,
        )
        .await;
    assert_eq!(
        (&ms[0]["saved"], &ms[1]["saved"]),
        (&json!(false), &json!(true))
    );
    let (_, r) = a
        .call(
            "PATCH",
            &format!("/api/messages/{}", rb.id),
            json!({ "saved": false }),
        )
        .await;
    assert_eq!(r["saved"], false);
    let (_, r) = a.call("GET", "/api/saved", Value::Null).await;
    assert_eq!(ids(&r), [qc.id]);
    let (code, _) = a
        .call("PATCH", "/api/messages/9999", json!({ "saved": true }))
        .await;
    assert_eq!(code, 404);
}

/// 대화 목록 첫 줄(가장 최근 대화)의 안 읽음 수.
async fn first_unread(s: &Side) -> i64 {
    let (_, list) = s.call("GET", "/api/conversations", Value::Null).await;
    list[0]["unread"].as_i64().unwrap()
}

/// 대화 읽음 — 상대 쪽 글 중 읽은 뒤에 온 것이 안 읽음. 읽음은 화면이 받은 마지막 글(`upto`)까지만 — 받은 뒤 읽음을 보내기 전에 온 글은
/// 안 읽음으로 남는다. 읽은 위치는 뒤로 가지 않고, 상대 PC 는 따로.
#[tokio::test(flavor = "multi_thread")]
async fn conversation_read_clears_unread() {
    let (a, b) = (side("rd-a").await, side("rd-b").await);
    let (pb, _) = pair(&a, &b).await;
    agents::add(&b.db(), "review", ".", None, "").unwrap();
    let conv = ask(&a, pb, "질문").await;
    assert_eq!(first_unread(&a).await, 1, "상대 에이전트 답 하나");
    // 화면이 받은 마지막 글.
    let last_id = |conv: i64| {
        let a = &a;
        async move {
            let (_, ms) = a
                .call(
                    "GET",
                    &format!("/api/conversations/{conv}/messages"),
                    Value::Null,
                )
                .await;
            ms.as_array()
                .unwrap()
                .iter()
                .filter_map(|m| m["id"].as_i64())
                .max()
                .unwrap()
        }
    };
    let read = |conv: i64, body: Value| {
        let a = &a;
        async move {
            a.call("POST", &format!("/api/conversations/{conv}/read"), body)
                .await
                .0
        }
    };
    let shown = last_id(conv).await;
    assert_eq!(read(conv, json!({ "upto": shown })).await, 204);
    assert_eq!(first_unread(&a).await, 0);
    assert_eq!(first_unread(&b).await, 1, "상대 PC 의 읽음은 따로");
    // 다시 묻고 답이 온 뒤 — 앞서 받은 글까지만 읽음으로 보내면 새 답은 안 읽음으로 남는다.
    let (code, r) = a
        .call(
            "POST",
            &format!("/api/conversations/{conv}/messages"),
            json!({ "text": "다시" }),
        )
        .await;
    assert_eq!(code, 201, "{r}");
    wait_messages(&a, conv, 4).await;
    assert_eq!(read(conv, json!({ "upto": shown })).await, 204);
    assert_eq!(
        first_unread(&a).await,
        1,
        "보지 못한 답 하나 — 내 글은 세지 않는다"
    );
    let last = last_id(conv).await;
    assert_eq!(read(conv, json!({ "upto": last })).await, 204);
    assert_eq!(first_unread(&a).await, 0);
    assert_eq!(read(conv, json!({ "upto": shown })).await, 204);
    assert_eq!(first_unread(&a).await, 0, "읽은 위치는 뒤로 가지 않는다");
    // 다른 대화의 글 · 없는 글 · 몸 없음은 거절, 없는 대화는 404.
    let other = ask(&a, pb, "다른 질문").await;
    let foreign = last_id(other).await;
    assert_eq!(read(conv, json!({ "upto": foreign })).await, 400);
    assert_eq!(read(conv, json!({ "upto": foreign + 100 })).await, 400);
    assert_eq!(read(conv, Value::Null).await, 400);
    assert_eq!(read(999, json!({ "upto": last })).await, 404);
}

/// 알림 일시 중지 설정 — 기본 꺼짐, 저장 · 다시 열어도 유지, 전체 켜짐(notify_inbound)과 별개.
#[tokio::test(flavor = "multi_thread")]
async fn settings_notify_paused_roundtrip() {
    let a = side("np").await;
    let (_, s) = a.call("GET", "/api/settings", Value::Null).await;
    assert_eq!(s["notify_paused"], false, "{s}");
    let (code, s) = a
        .call("PUT", "/api/settings", json!({ "notify_paused": true }))
        .await;
    assert_eq!(code, 200, "{s}");
    assert_eq!(
        (&s["notify_paused"], &s["notify_inbound"]),
        (&json!(true), &json!(false))
    );
    let (port, h) = web(&a.paths, a.node.clone(), a.chat.clone()).await;
    let (_, raw) = common::send_body(port, "GET", "/api/settings", &h, "").await;
    let s: Value = serde_json::from_str(raw.split("\r\n\r\n").nth(1).unwrap()).unwrap();
    assert_eq!(s["notify_paused"], true, "다시 열어도 유지");
}

/// 집중 — 이 PC 설정(기본 꺼짐 · 다시 열어도 유지, 알림 일시 중지와 별개). 상대가 연결 확인(Ping)하면 접속 · 집중이 보인다 — 피어 JSON 은
/// `online` · `focus` 두 참거짓만(시각은 내지 않는다).
#[tokio::test(flavor = "multi_thread")]
async fn focus_setting_and_peer_presence() {
    let (a, b) = (side("fc-a").await, side("fc-b").await);
    let (_, pa) = pair(&a, &b).await;
    let (_, s) = a.call("GET", "/api/settings", Value::Null).await;
    assert_eq!(s["focus"], false, "{s}");
    let (code, s) = a
        .call("PUT", "/api/settings", json!({ "focus": true }))
        .await;
    assert_eq!(code, 200, "{s}");
    assert_eq!(
        (&s["focus"], &s["notify_paused"]),
        (&json!(true), &json!(false))
    );
    let (_, list) = b.call("GET", "/api/peers", Value::Null).await;
    assert_eq!(
        (&list[0]["online"], &list[0]["focus"]),
        (&json!(false), &json!(false)),
        "아직 Ping 전 {list}"
    );
    assert!(list[0].get("seen_at").is_none(), "{list}");
    let (code, _) = b
        .call("POST", &format!("/api/peers/{pa}/ping"), Value::Null)
        .await;
    assert_eq!(code, 204);
    let (_, list) = b.call("GET", "/api/peers", Value::Null).await;
    assert_eq!(
        (&list[0]["online"], &list[0]["focus"]),
        (&json!(true), &json!(true)),
        "{list}"
    );
    let (code, _) = a
        .call("PUT", "/api/settings", json!({ "focus": false }))
        .await;
    assert_eq!(code, 200);
    b.call("POST", &format!("/api/peers/{pa}/ping"), Value::Null)
        .await;
    let (_, list) = b.call("GET", "/api/peers", Value::Null).await;
    assert_eq!(
        (&list[0]["online"], &list[0]["focus"]),
        (&json!(true), &json!(false)),
        "끄면 다음 확인부터 {list}"
    );
    a.call("PUT", "/api/settings", json!({ "focus": true }))
        .await;
    // 집중 중인 A 에게 보낸 B 의 요청 — 403 "상대가 집중 중", B 이력의 그 글은 실패 + 거절 코드(화면 사유), A 는 접속 · 집중(거절도 접속 증거).
    agents::add(&a.db(), "review", ".", None, "").unwrap();
    let (code, r) = b
        .call(
            "POST",
            "/api/conversations",
            json!({ "peer_id": pa, "agent": "review", "text": "집중 중에 묻는다" }),
        )
        .await;
    assert_eq!(code, 403, "{r}");
    assert!(
        r["error"].as_str().unwrap().contains("상대가 집중 중"),
        "{r}"
    );
    let (_, list) = b.call("GET", "/api/conversations", Value::Null).await;
    let (_, ms) = b
        .call(
            "GET",
            &format!("/api/conversations/{}/messages", list[0]["id"]),
            Value::Null,
        )
        .await;
    assert_eq!(
        (&ms[0]["state"], &ms[0]["refused"]),
        (&json!("error"), &json!("focus")),
        "{ms}"
    );
    let (_, list) = b.call("GET", "/api/peers", Value::Null).await;
    assert_eq!(list[0]["focus"], true, "{list}");
    let (port, h) = web(&a.paths, a.node.clone(), a.chat.clone()).await;
    let (_, raw) = common::send_body(port, "GET", "/api/settings", &h, "").await;
    let s: Value = serde_json::from_str(raw.split("\r\n\r\n").nth(1).unwrap()).unwrap();
    assert_eq!(s["focus"], true, "다시 열어도 유지");
}

/// 질의 · 요청 — 새 대화 · 이어 쓰기의 `request`(기본 질의)가 양쪽 이력(메시지 JSON `request`)에 남고, 보낸 목록(`GET /api/messages` 의 `ask`) ·
/// 받은 목록(내 활동 `kind`)을 질의 · 요청으로 거른다(제한 전에).
#[tokio::test(flavor = "multi_thread")]
async fn ask_kind_recorded_and_filtered() {
    let (a, b) = (side("aq-a").await, side("aq-b").await);
    let (pb, _) = pair(&a, &b).await;
    agents::add(&b.db(), "review", ".", None, "").unwrap();
    let (code, r) = a
        .call(
            "POST",
            "/api/conversations",
            json!({ "peer_id": pb, "agent": "review", "text": "요청 첫 글", "request": true }),
        )
        .await;
    assert_eq!(code, 201, "{r}");
    assert_eq!(r["message"]["request"], true, "{r}");
    let conv = r["conversation"]["id"].as_i64().unwrap();
    wait_messages(&a, conv, 2).await;
    let (code, r) = a
        .call(
            "POST",
            &format!("/api/conversations/{conv}/messages"),
            json!({ "text": "질의 둘째" }),
        )
        .await;
    assert_eq!(
        (code, &r["message"]["request"]),
        (201, &json!(false)),
        "{r}"
    );
    wait_messages(&a, conv, 4).await;
    let mine = |ask: &'static str| {
        let a = &a;
        async move {
            a.call(
                "GET",
                &format!("/api/messages?side=mine&peer={pb}&ask={ask}&limit=1"),
                Value::Null,
            )
            .await
        }
    };
    let (_, list) = mine("request").await;
    assert_eq!(texts(&list), ["요청 첫 글"], "제한 전에 거른다");
    let (_, list) = mine("query").await;
    assert_eq!(texts(&list), ["질의 둘째"]);
    let (code, _) = mine("x").await;
    assert_eq!(code, 400);
    // 받은 쪽(B) 내 활동 — '내게 요청된 목록'.
    let items = |kind: &'static str| {
        let b = &b;
        async move {
            let (_, r) = b
                .call("GET", &format!("/api/activity?kind={kind}"), Value::Null)
                .await;
            r["items"]
                .as_array()
                .unwrap()
                .iter()
                .map(|i| {
                    (
                        i["message"]["text"].as_str().unwrap().to_string(),
                        i["message"]["request"].as_bool().unwrap(),
                    )
                })
                .collect::<Vec<_>>()
        }
    };
    assert_eq!(items("request").await, [("요청 첫 글".to_string(), true)]);
    assert_eq!(items("query").await, [("질의 둘째".to_string(), false)]);
}

/// 완료 표시 — 받은 요청에만(받은 쪽), 바뀐 상태는 쓴 쪽 이력에도. 쓴 쪽 · 답 · 빈 몸 · 모르는 항목은 거절. `saved` 와 함께 줄 수 있다.
#[tokio::test(flavor = "multi_thread")]
async fn done_only_by_receiver() {
    let (a, b) = (side("dn-a").await, side("dn-b").await);
    let (pb, _) = pair(&a, &b).await;
    agents::add(&b.db(), "review", ".", None, "").unwrap();
    let (_, r) = a
        .call(
            "POST",
            "/api/conversations",
            json!({ "peer_id": pb, "agent": "review", "text": "처리해 주세요", "request": true }),
        )
        .await;
    let conv = r["conversation"]["id"].as_i64().unwrap();
    let a_req = r["message"]["id"].as_i64().unwrap();
    wait_messages(&a, conv, 2).await;
    let (_, list) = b.call("GET", "/api/conversations", Value::Null).await;
    let (_, ms) = b
        .call(
            "GET",
            &format!("/api/conversations/{}/messages", list[0]["id"]),
            Value::Null,
        )
        .await;
    let (b_req, b_reply) = (ms[0]["id"].as_i64().unwrap(), ms[1]["id"].as_i64().unwrap());
    let patch = |s: &'static str, id: i64, body: Value| {
        let side = if s == "a" { &a } else { &b };
        async move {
            side.call("PATCH", &format!("/api/messages/{id}"), body)
                .await
        }
    };
    let (code, r) = patch("b", b_req, json!({ "done": true })).await;
    assert_eq!(code, 200, "{r}");
    assert_eq!(
        (&r["done"], &r["warning"]),
        (&json!(true), &Value::Null),
        "{r}"
    );
    let (_, ms) = a
        .call(
            "GET",
            &format!("/api/conversations/{conv}/messages"),
            Value::Null,
        )
        .await;
    assert_eq!(ms[0]["done"], true, "쓴 쪽은 받은 대로 {ms}");
    for (s, id) in [("a", a_req), ("b", b_reply)] {
        let (code, r) = patch(s, id, json!({ "done": true })).await;
        assert_eq!(code, 400, "{s} {id} {r}");
        // 둘 다 주면 한 트랜잭션 — 완료 자격이 없으면 저장도 쓰지 않는다.
        let (code, r) = patch(s, id, json!({ "done": true, "saved": true })).await;
        assert_eq!(code, 400, "{s} {id} {r}");
    }
    let (_, ms) = a
        .call(
            "GET",
            &format!("/api/conversations/{conv}/messages"),
            Value::Null,
        )
        .await;
    assert_eq!(ms[0]["saved"], false, "부분 적용 없음 {ms}");
    for body in [json!({}), json!({ "done": true, "pin": true })] {
        let (code, r) = patch("b", b_req, body.clone()).await;
        assert!(code == 400 || code == 422, "{body} → {code} {r}");
    }
    let (code, r) = patch("b", b_req, json!({ "done": false, "saved": true })).await;
    assert_eq!(code, 200, "{r}");
    assert_eq!(
        (&r["done"], &r["saved"]),
        (&json!(false), &json!(true)),
        "{r}"
    );
    let (_, ms) = a
        .call(
            "GET",
            &format!("/api/conversations/{conv}/messages"),
            Value::Null,
        )
        .await;
    assert_eq!(ms[0]["done"], false, "취소도 쓴 쪽에 {ms}");
}

/// 이어 쓰기의 `agent`(멘션) — 보낸 대화 · 자동 왕복 아님에서만(받은 대화 · 자동 왕복 · 빈 이름은 400), 대화 에이전트 이름이면 멘션이 아니다
/// (`agent` null). 메시지 JSON 의 `agent` 가 요청 · 답에 남는다.
#[tokio::test(flavor = "multi_thread")]
async fn mention_api_rules() {
    let (a, b) = (side("mn-a").await, side("mn-b").await);
    let (pb, _) = pair(&a, &b).await;
    agents::add(&b.db(), "review", ".", None, "").unwrap();
    agents::add(&b.db(), "guide", ".", None, "").unwrap();
    agents::add(&a.db(), "mine", ".", None, "").unwrap();
    let (_, r) = a
        .call(
            "POST",
            "/api/conversations",
            json!({ "peer_id": pb, "agent": "review", "text": "처음" }),
        )
        .await;
    let conv = r["conversation"]["id"].as_i64().unwrap();
    wait_messages(&a, conv, 2).await;
    let post = |s: &'static str, id: i64, body: Value| {
        let side = if s == "a" { &a } else { &b };
        async move {
            side.call("POST", &format!("/api/conversations/{id}/messages"), body)
                .await
        }
    };
    let (code, r) = post(
        "a",
        conv,
        json!({ "text": "@guide 봐 줘", "agent": "guide" }),
    )
    .await;
    assert_eq!(
        (code, &r["message"]["agent"]),
        (201, &json!("guide")),
        "{r}"
    );
    wait_messages(&a, conv, 4).await;
    let (_, ms) = a
        .call(
            "GET",
            &format!("/api/conversations/{conv}/messages"),
            Value::Null,
        )
        .await;
    assert_eq!(ms[3]["agent"], "guide", "답도 부른 에이전트 {ms}");
    let (code, r) = post(
        "a",
        conv,
        json!({ "text": "@review 다시", "agent": "review" }),
    )
    .await;
    assert_eq!((code, &r["message"]["agent"]), (201, &Value::Null), "{r}");
    let (code, r) = post("a", conv, json!({ "text": "빈 이름", "agent": "" })).await;
    assert_eq!(code, 400, "{r}");
    let (_, list) = b.call("GET", "/api/conversations", Value::Null).await;
    let bconv = list[0]["id"].as_i64().unwrap();
    let (code, r) = post("b", bconv, json!({ "text": "@guide", "agent": "guide" })).await;
    assert_eq!(code, 400, "받은 대화 {r}");
    let (_, r) = a
        .call(
            "POST",
            "/api/conversations",
            json!({ "peer_id": pb, "agent": "review", "text": "왕복", "relay_agent": "mine", "relay_cap": 1 }),
        )
        .await;
    let relay = r["conversation"]["id"].as_i64().unwrap();
    let (code, r) = post("a", relay, json!({ "text": "@guide", "agent": "guide" })).await;
    assert_eq!(code, 400, "자동 왕복 {r}");
}
