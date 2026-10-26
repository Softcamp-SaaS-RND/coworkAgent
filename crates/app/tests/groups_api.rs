//! #1i — 레일 그룹 · 손으로 놓는 순서 API(이 PC 에만).
mod common;
use an_agent::chat::{Chat, Compose, Run};
use an_agent::runner::{RunInput, RunOutput, Session};
use an_app::notify::{self, Notifier};
use an_app::{chat_api, peers_api, server};
use an_core::{agents, chat, db, groups, paths::Paths};
use an_net::identity::Identity;
use an_net::node::{Me, Node, PAIRING_WINDOW};
use serde_json::{Value, json};
use std::sync::Arc;
use std::time::{Duration, Instant};

fn echo() -> Run {
    Arc::new(|input: RunInput| {
        let session = match &input.session {
            Session::New(id) | Session::Resume(id) => id.clone(),
        };
        Ok(RunOutput {
            text: "답".to_string(),
            session,
            is_error: false,
            turns: Some(1),
            duration_ms: Some(10),
            ctx_tokens: None,
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

/// 노드 · 대화 · 운영과 같은 로컬 API(피어 · 설정 · 대화 · 그룹) 한 벌.
async fn side(tag: &str) -> Side {
    let root = std::env::temp_dir().join(format!("an-app-groups-{tag}-{}", std::process::id()));
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
    let web = server::bind().await.unwrap();
    let port = web.local_addr().unwrap().port();
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
        peers_api::routes(node.clone())
            .merge(notify::routes(notifier))
            .merge(chat_api::routes(chat.clone())),
    );
    tokio::spawn(async move { axum::serve(web, app).await.unwrap() });
    let h = vec![
        common::host(port),
        common::origin(port),
        ("Authorization", format!("Bearer {token}")),
        ("Content-Type", "application/json".to_string()),
    ];
    Side {
        node,
        chat,
        addr,
        paths,
        port,
        h,
    }
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

    async fn group(&self, name: &str) -> i64 {
        let (code, g) = self
            .call("POST", "/api/groups", json!({ "name": name }))
            .await;
        assert_eq!(code, 201, "{g}");
        g["id"].as_i64().unwrap()
    }

    async fn groups(&self) -> Vec<Value> {
        let (code, gs) = self.call("GET", "/api/groups", Value::Null).await;
        assert_eq!(code, 200, "{gs}");
        gs.as_array().unwrap().clone()
    }

    /// 대화 id — 자리(`pos`, 같으면 id) 순.
    async fn conv_order(&self) -> Vec<i64> {
        let (code, cs) = self.call("GET", "/api/conversations", Value::Null).await;
        assert_eq!(code, 200, "{cs}");
        let mut cs: Vec<(i64, i64)> = cs
            .as_array()
            .unwrap()
            .iter()
            .map(|c| (c["pos"].as_i64().unwrap(), c["id"].as_i64().unwrap()))
            .collect();
        cs.sort();
        cs.into_iter().map(|(_, id)| id).collect()
    }

    async fn peer_ids(&self) -> Vec<i64> {
        let (_, ps) = self.call("GET", "/api/peers", Value::Null).await;
        ps.as_array()
            .unwrap()
            .iter()
            .map(|p| p["id"].as_i64().unwrap())
            .collect()
    }

    fn outbox(&self) -> i64 {
        self.db()
            .query_row("SELECT count(*) FROM outbox", [], |r| r.get(0))
            .unwrap()
    }
}

fn convs_of(g: &Value) -> Vec<i64> {
    g["convs"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c.as_i64().unwrap())
        .collect()
}

/// a 와 b 를 페어링 — a 쪽 b 의 id. b 에는 에이전트 `review`.
async fn pair(a: &Side, b: &Side) -> i64 {
    let code = b.node.open_window(PAIRING_WINDOW);
    let pb = a.node.add(b.addr, Some(code)).await.unwrap();
    a.node.approve(pb.id).await.unwrap();
    agents::add(&b.db(), "review", ".", "").unwrap();
    pb.id
}

/// 대화 `conv` 의 글이 `n` 개가 될 때까지(최대 5초).
async fn wait_messages(s: &Side, conv: i64, n: usize) {
    let until = Instant::now() + Duration::from_secs(5);
    while chat::messages(&s.db(), conv).unwrap().len() < n {
        assert!(Instant::now() < until, "글 {n} 개를 기다렸다");
        tokio::time::sleep(Duration::from_millis(30)).await;
    }
}

/// 새 대화로 묻고 답까지 기다린다 — 대화 id. `group` 이 있으면 그 그룹으로.
async fn ask(from: &Side, peer: i64, text: &str, group: Option<i64>) -> i64 {
    let mut body = json!({ "peer_id": peer, "agent": "review", "text": text });
    if let Some(g) = group {
        body["group"] = json!(g);
    }
    let (code, r) = from.call("POST", "/api/conversations", body).await;
    assert_eq!(code, 201, "{r}");
    let conv = r["conversation"]["id"].as_i64().unwrap();
    wait_messages(from, conv, 2).await;
    conv
}

/// 두 PC · 대화 셋(c1 이 가장 먼저).
async fn three(tag: &str) -> (Side, Side, i64, [i64; 3]) {
    let (a, b) = (
        side(&format!("{tag}-a")).await,
        side(&format!("{tag}-b")).await,
    );
    let pb = pair(&a, &b).await;
    let c1 = ask(&a, pb, "하나", None).await;
    let c2 = ask(&a, pb, "둘", None).await;
    let c3 = ask(&a, pb, "셋", None).await;
    (a, b, pb, [c1, c2, c3])
}

#[tokio::test(flavor = "multi_thread")]
async fn create_name_rules() {
    let a = side("name").await;
    for bad in ["", "   ", &"가".repeat(41), "줄\n바꿈"] {
        let (code, r) = a.call("POST", "/api/groups", json!({ "name": bad })).await;
        assert_eq!(code, 400, "{bad:?} → {r}");
    }
    let g = a.group(&format!(" {} ", "가".repeat(40))).await;
    let (code, r) = a
        .call("POST", "/api/groups", json!({ "name": "가".repeat(40) }))
        .await;
    assert_eq!(code, 409, "같은 이름: {r}");
    assert_eq!(r["error"], "같은 이름의 그룹이 있습니다");
    let h = a.group("결제").await;
    let gs = a.groups().await;
    assert_eq!(
        gs.iter()
            .map(|g| g["id"].as_i64().unwrap())
            .collect::<Vec<_>>(),
        [g, h],
        "만든 순서(끝에 붙는다)"
    );
    assert_eq!(gs[0]["name"], "가".repeat(40), "앞뒤 공백은 뺀다");
    let (code, _) = a
        .call("POST", "/api/groups", json!({ "name": "x", "pos": 0 }))
        .await;
    assert!((400..500).contains(&code), "모르는 필드 거절");
}

#[tokio::test(flavor = "multi_thread")]
async fn rename_and_icon() {
    let a = side("rename").await;
    let g = a.group("결제").await;
    a.group("배포").await;
    let path = format!("/api/groups/{g}");
    let (code, r) = a.call("PATCH", &path, json!({ "name": "배포" })).await;
    assert_eq!(
        (code, r["error"].clone()),
        (409, json!("같은 이름의 그룹이 있습니다"))
    );
    let (code, _) = a.call("PATCH", &path, json!({})).await;
    assert_eq!(code, 400, "바꿀 것 없음");
    let (code, _) = a
        .call(
            "PATCH",
            &path,
            json!({ "name": "결제2", "icon": "data:image/gif;base64,AA" }),
        )
        .await;
    assert_eq!(code, 400, "틀린 그림");
    assert_eq!(
        a.groups().await[0]["name"],
        "결제",
        "하나라도 틀리면 아무것도 안 바뀜"
    );
    let icon = format!("{}AAAA", an_core::run_opts::ICON_PREFIX);
    let (code, r) = a
        .call(
            "PATCH",
            &path,
            json!({ "name": " 결제 모듈 ", "icon": icon }),
        )
        .await;
    assert_eq!(code, 200, "{r}");
    assert_eq!(
        (r["name"].clone(), r["icon"].clone()),
        (json!("결제 모듈"), json!(icon))
    );
    let (code, _) = a.call("PATCH", &path, json!({ "name": "결제 모듈" })).await;
    assert_eq!(code, 200, "자기 이름 그대로는 겹침 아님");
    let (code, r) = a.call("PATCH", &path, json!({ "icon": "" })).await;
    assert_eq!((code, r["icon"].clone()), (200, Value::Null), "그림 지우기");
    // 타일 색 — 처음 무작위(여섯 중), 여섯만, 하나라도 틀리면 아무것도 안 바뀜.
    assert!(an_core::agents::ICON_COLORS.contains(&a.groups().await[0]["color"].as_str().unwrap()));
    let (code, r) = a.call("PATCH", &path, json!({ "color": "peer-5" })).await;
    assert_eq!((code, r["color"].clone(), r["name"].clone()), (200, json!("peer-5"), json!("결제 모듈")));
    let (code, _) = a.call("PATCH", &path, json!({ "name": "결제 새", "color": "pink" })).await;
    assert_eq!(code, 400, "틀린 색");
    assert_eq!((a.groups().await[0]["name"].clone(), a.groups().await[0]["color"].clone()), (json!("결제 모듈"), json!("peer-5")));
    let (code, _) = a
        .call("PATCH", "/api/groups/999", json!({ "name": "x" }))
        .await;
    assert_eq!(code, 404);
}

#[tokio::test(flavor = "multi_thread")]
async fn membership_many_groups() {
    let (a, _b, _, [c1, c2, c3]) = three("member").await;
    let (g1, g2) = (a.group("결제").await, a.group("배포").await);
    let put = |g: i64, body: Value| {
        let a = &a;
        async move {
            a.call("PUT", &format!("/api/groups/{g}/conversations"), body)
                .await
        }
    };
    let (code, r) = put(g1, json!({ "add": [c1, c2, c2] })).await;
    assert_eq!((code, convs_of(&r)), (200, vec![c1, c2]));
    let (code, r) = put(g2, json!({ "add": [c2, c3] })).await;
    assert_eq!((code, convs_of(&r)), (200, vec![c2, c3]), "c2 는 두 그룹에");
    let (code, r) = put(g1, json!({ "add": [c3], "remove": [c1] })).await;
    assert_eq!(
        (code, convs_of(&r)),
        (200, vec![c2, c3]),
        "넣고 빼기 한 번에"
    );
    let (code, r) = put(g1, json!({ "remove": [c1] })).await;
    assert_eq!(
        (code, convs_of(&r)),
        (200, vec![c2, c3]),
        "있는 대화가 그룹에 없으면 빼기는 그대로"
    );
    let (code, r) = put(g1, json!({ "remove": [c2, 999] })).await;
    assert_eq!(code, 400, "없는 대화 빼기도 거절(라운드 2): {r}");
    let (code, r) = put(g1, json!({ "add": [999] })).await;
    assert_eq!(code, 400, "없는 대화: {r}");
    assert_eq!(convs_of(&a.groups().await[0]), [c2, c3], "거절이면 그대로");
    let (code, _) = put(999, json!({ "add": [c1] })).await;
    assert_eq!(code, 404);
}

#[tokio::test(flavor = "multi_thread")]
async fn delete_keeps_conversations() {
    let (a, _b, _, [c1, c2, _]) = three("delete").await;
    let g = a.group("결제").await;
    a.call(
        "PUT",
        &format!("/api/groups/{g}/conversations"),
        json!({ "add": [c1, c2] }),
    )
    .await;
    let (code, _) = a
        .call("DELETE", &format!("/api/groups/{g}"), Value::Null)
        .await;
    assert_eq!(code, 204);
    assert!(a.groups().await.is_empty());
    assert_eq!(a.conv_order().await.len(), 3, "대화는 남는다");
    let left: i64 = a
        .db()
        .query_row("SELECT count(*) FROM group_convs", [], |r| r.get(0))
        .unwrap();
    assert_eq!(left, 0, "소속도 지운다");
    let (code, _) = a
        .call("DELETE", &format!("/api/groups/{g}"), Value::Null)
        .await;
    assert_eq!(code, 404);
}

#[tokio::test(flavor = "multi_thread")]
async fn new_conversation_top_and_stays() {
    let (a, _b, pb, [c1, c2, c3]) = three("top").await;
    assert_eq!(a.conv_order().await, [c3, c2, c1], "새 대화는 맨 위");
    // 이어 써도(새 글 · 답) 자리는 그대로.
    let (code, r) = a
        .call(
            "POST",
            &format!("/api/conversations/{c1}/messages"),
            json!({ "text": "더" }),
        )
        .await;
    assert_eq!(code, 201, "{r}");
    wait_messages(&a, c1, 4).await;
    assert_eq!(a.conv_order().await, [c3, c2, c1]);
    let g = a.group("결제").await;
    let c4 = ask(&a, pb, "넷", Some(g)).await;
    assert_eq!(a.conv_order().await, [c4, c3, c2, c1]);
    assert_eq!(
        convs_of(&a.groups().await[0]),
        [c4],
        "그룹 화면에서 만든 대화는 그 그룹에"
    );
    // 없는 그룹이면 보내지 않는다.
    let (code, _) = a
        .call(
            "POST",
            "/api/conversations",
            json!({ "peer_id": pb, "agent": "review", "text": "x", "group": 999 }),
        )
        .await;
    assert_eq!(code, 404);
    assert_eq!(a.conv_order().await.len(), 4, "대화가 생기지 않는다");
    // 자동 왕복 시작도 같은 트랜잭션으로 그룹에.
    agents::add(&a.db(), "mine", ".", "").unwrap();
    let (code, r) = a
        .call(
            "POST",
            "/api/conversations",
            json!({ "peer_id": pb, "agent": "review", "text": "왕복", "relay_agent": "mine", "relay_cap": 1, "group": g }),
        )
        .await;
    assert_eq!(code, 201, "{r}");
    let c5 = r["conversation"]["id"].as_i64().unwrap();
    assert_eq!(
        convs_of(&a.groups().await[0]),
        [c4, c5],
        "자동 왕복 대화도 그 그룹에"
    );
    assert_eq!(a.conv_order().await[0], c5, "맨 위");
}

#[tokio::test(flavor = "multi_thread")]
async fn order_conversations() {
    let (a, _b, _, [c1, c2, c3]) = three("orderc").await;
    let put = |ids: Value| {
        let a = &a;
        async move {
            a.call(
                "PUT",
                "/api/order",
                json!({ "kind": "conversations", "ids": ids }),
            )
            .await
        }
    };
    let (code, _) = put(json!([c1, c3, c2])).await;
    assert_eq!(code, 204);
    assert_eq!(a.conv_order().await, [c1, c3, c2]);
    for bad in [json!([c1, c1]), json!([c1, 999])] {
        let (code, r) = put(bad.clone()).await;
        assert_eq!(code, 400, "{bad} → {r}");
    }
    assert_eq!(a.conv_order().await, [c1, c3, c2], "거절이면 그대로");
    let (code, _) = a
        .call(
            "PUT",
            "/api/order",
            json!({ "kind": "messages", "ids": [] }),
        )
        .await;
    assert_eq!(code, 400, "모르는 kind");
}

#[tokio::test(flavor = "multi_thread")]
async fn order_partial_ties_by_id() {
    let (a, _b, _, [c1, c2, c3]) = three("tie").await;
    let put = |ids: Value| {
        let a = &a;
        async move {
            a.call(
                "PUT",
                "/api/order",
                json!({ "kind": "conversations", "ids": ids }),
            )
            .await
        }
    };
    assert_eq!(put(json!([c1, c2, c3])).await.0, 204);
    // c3 만 보내면 c3 = 0 — 빠진 c1(0) · c2(1)는 제 자리. 같은 자리(0)는 id 순.
    assert_eq!(put(json!([c3])).await.0, 204);
    assert_eq!(a.conv_order().await, [c1, c3, c2]);
}

#[tokio::test(flavor = "multi_thread")]
async fn order_peers_and_groups() {
    let (a, b, c) = (
        side("orderp-a").await,
        side("orderp-b").await,
        side("orderp-c").await,
    );
    let pb = pair(&a, &b).await;
    let pc = pair(&a, &c).await;
    assert_eq!(a.peer_ids().await, [pb, pc], "새 상대는 맨 뒤");
    let (code, _) = a
        .call(
            "PUT",
            "/api/order",
            json!({ "kind": "peers", "ids": [pc, pb] }),
        )
        .await;
    assert_eq!(code, 204);
    assert_eq!(a.peer_ids().await, [pc, pb]);
    let (g1, g2, g3) = (
        a.group("하나").await,
        a.group("둘").await,
        a.group("셋").await,
    );
    let (code, _) = a
        .call(
            "PUT",
            "/api/order",
            json!({ "kind": "groups", "ids": [g3, g1, g2] }),
        )
        .await;
    assert_eq!(code, 204);
    let ids: Vec<i64> = a
        .groups()
        .await
        .iter()
        .map(|g| g["id"].as_i64().unwrap())
        .collect();
    assert_eq!(ids, [g3, g1, g2]);
}

#[tokio::test(flavor = "multi_thread")]
async fn lists_by_group() {
    let (a, _b, pb, [c1, c2, _c3]) = three("filter").await;
    let g = a.group("결제").await;
    a.call(
        "PUT",
        &format!("/api/groups/{g}/conversations"),
        json!({ "add": [c2] }),
    )
    .await;
    let convs_in = |v: &Value| {
        let mut ids: Vec<i64> = v
            .as_array()
            .unwrap()
            .iter()
            .map(|m| m["conversation"]["id"].as_i64().unwrap())
            .collect();
        ids.dedup();
        ids
    };
    let (code, ms) = a
        .call(
            "GET",
            &format!("/api/messages?side=mine&group={g}"),
            Value::Null,
        )
        .await;
    assert_eq!((code, convs_in(&ms)), (200, vec![c2]));
    let (code, ms) = a
        .call(
            "GET",
            &format!("/api/messages?side=theirs&group={g}"),
            Value::Null,
        )
        .await;
    assert_eq!((code, convs_in(&ms)), (200, vec![c2]));
    let (code, _) = a
        .call(
            "GET",
            &format!("/api/messages?side=mine&group={g}&peer={pb}"),
            Value::Null,
        )
        .await;
    assert_eq!(code, 400, "상대와 그룹은 함께 주지 않는다");
    // 저장된 항목 — c1 · c2 의 답을 저장, 그룹은 c2 것만.
    for c in [c1, c2] {
        let answer = chat::messages(&a.db(), c)
            .unwrap()
            .into_iter()
            .find(|m| m.author == chat::Author::Agent)
            .unwrap();
        let (code, r) = a
            .call(
                "PATCH",
                &format!("/api/messages/{}", answer.id),
                json!({ "saved": true }),
            )
            .await;
        assert_eq!(code, 200, "{r}");
    }
    let (code, ms) = a
        .call("GET", &format!("/api/saved?group={g}"), Value::Null)
        .await;
    assert_eq!((code, convs_in(&ms)), (200, vec![c2]));
    let (code, ms) = a.call("GET", "/api/saved", Value::Null).await;
    assert_eq!(
        (code, ms.as_array().unwrap().len()),
        (200, 2),
        "그룹이 없으면 전부"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn removed_peer_conversation_stays_in_group() {
    let (a, _b, pb, [c1, _, _]) = three("gone").await;
    let g = a.group("결제").await;
    a.call(
        "PUT",
        &format!("/api/groups/{g}/conversations"),
        json!({ "add": [c1] }),
    )
    .await;
    let (code, r) = a
        .call("DELETE", &format!("/api/peers/{pb}"), Value::Null)
        .await;
    assert_eq!(code, 200, "{r}");
    assert_eq!(convs_of(&a.groups().await[0]), [c1], "그룹에서 빼지 않는다");
    assert!(a.conv_order().await.contains(&c1), "대화도 남는다");
}

#[tokio::test(flavor = "multi_thread")]
async fn local_only_and_survives_reopen() {
    let (a, _b, _, [c1, c2, c3]) = three("local").await;
    // 보낸 대화의 보관 줄이 빌 때까지 — 그 뒤 그룹 · 순서는 보관 줄에 아무것도 남기지 않는다.
    let until = Instant::now() + Duration::from_secs(5);
    while a
        .db()
        .query_row(
            "SELECT count(*) FROM outbox WHERE state IN ('pending', 'inflight')",
            [],
            |r| r.get::<_, i64>(0),
        )
        .unwrap()
        > 0
    {
        assert!(Instant::now() < until, "보관 줄이 비기를 기다렸다");
        tokio::time::sleep(Duration::from_millis(30)).await;
    }
    let before = a.outbox();
    let g = a.group("결제").await;
    a.call(
        "PUT",
        &format!("/api/groups/{g}/conversations"),
        json!({ "add": [c1, c3] }),
    )
    .await;
    a.call(
        "PATCH",
        &format!("/api/groups/{g}"),
        json!({ "name": "결제 모듈" }),
    )
    .await;
    a.call(
        "PUT",
        "/api/order",
        json!({ "kind": "conversations", "ids": [c1, c2, c3] }),
    )
    .await;
    assert_eq!(a.outbox(), before, "상대로 가는 프레임 없음");
    // 다시 열어도 남는다(DB).
    let db = a.db();
    let gs = groups::list(&db).unwrap();
    assert_eq!(
        (gs[0].name.as_str(), gs[0].convs.clone()),
        ("결제 모듈", vec![c1, c3])
    );
    let mut s = chat::summaries(&db).unwrap();
    s.sort_by_key(|s| (s.pos, s.conversation.id));
    assert_eq!(
        s.iter().map(|s| s.conversation.id).collect::<Vec<_>>(),
        [c1, c2, c3]
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn requests_by_group() {
    let (a, _b, _, [c1, c2, _]) = three("req").await;
    let g = a.group("결제").await;
    a.call(
        "PUT",
        &format!("/api/groups/{g}/conversations"),
        json!({ "add": [c2] }),
    )
    .await;
    for c in [c1, c2] {
        let (code, r) = a
            .call(
                "POST",
                &format!("/api/conversations/{c}/messages"),
                json!({ "text": "고쳐 주세요", "request": true }),
            )
            .await;
        assert_eq!(code, 201, "{r}");
    }
    let (code, ms) = a
        .call(
            "GET",
            &format!("/api/messages?side=mine&ask=request&group={g}"),
            Value::Null,
        )
        .await;
    assert_eq!(code, 200, "{ms}");
    let ms = ms.as_array().unwrap();
    assert_eq!(ms.len(), 1, "그룹 대화의 요청만");
    assert_eq!(ms[0]["conversation"]["id"], c2);
}

#[tokio::test(flavor = "multi_thread")]
async fn group_deleted_after_check_sends_nothing() {
    // API 선검사 뒤 다른 창에서 그룹을 지운 경쟁 — 대화 트랜잭션 안의 소속이 실패하면 대화도 글도 남기지 않는다(라운드 1 high).
    let (a, b) = (side("race-a").await, side("race-b").await);
    let pb = pair(&a, &b).await;
    agents::add(&a.db(), "mine", ".", "").unwrap();
    let g = a.group("결제").await;
    let (code, _) = a
        .call("DELETE", &format!("/api/groups/{g}"), Value::Null)
        .await;
    assert_eq!(code, 204);
    let before = (a.conv_order().await.len(), a.outbox());
    let opts = Compose {
        group: Some(g),
        ..Compose::default()
    };
    let sent = a.chat.send_with(pb, "review", None, "x", opts).await;
    assert_eq!(
        sent.map(|_| ()).unwrap_err().kind(),
        std::io::ErrorKind::NotFound
    );
    let relay = a
        .chat
        .start_relay_with(pb, "review", "mine", 1, "x", opts)
        .await;
    assert_eq!(
        relay.map(|_| ()).unwrap_err().kind(),
        std::io::ErrorKind::NotFound
    );
    assert_eq!(
        (a.conv_order().await.len(), a.outbox()),
        before,
        "대화 · 보관 줄 그대로"
    );
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert!(
        chat::summaries(&b.db()).unwrap().is_empty(),
        "상대에 간 것 없음"
    );
}

/// 그룹 = 대화만(#1w — D37 개정) — 상대 소속 경로는 없다.
#[tokio::test(flavor = "multi_thread")]
async fn peer_routes_gone() {
    let (a, b) = (side("pgone-a").await, side("pgone-b").await);
    let pb = pair(&a, &b).await;
    let g = a.group("결제").await;
    for path in [format!("/api/groups/{g}/peers"), format!("/api/peers/{pb}/groups")] {
        let (code, r) = a.call("PUT", &path, json!({ "add": [pb] })).await;
        assert!(matches!(code, 404 | 405), "{path} {code} {r}");
    }
}

/// 그룹 JSON 에는 대화 소속만(`peers` 없음).
#[tokio::test(flavor = "multi_thread")]
async fn group_json_has_no_peers() {
    let (a, _b, _pb, [c1, _, _]) = three("pnone").await;
    let g = a.group("결제").await;
    let (code, r) = a
        .call(
            "PUT",
            &format!("/api/groups/{g}/conversations"),
            json!({ "add": [c1] }),
        )
        .await;
    assert_eq!((code, convs_of(&r)), (200, vec![c1]), "{r}");
    assert!(r.get("peers").is_none(), "{r}");
    assert!(a.groups().await[0].get("peers").is_none());
}
