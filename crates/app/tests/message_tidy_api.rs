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
            turns: Some(1),
            duration_ms: Some(10),
            ctx_tokens: None,
        })
    })
}

struct Side {
    node: Arc<Node>,
    addr: std::net::SocketAddr,
    paths: Paths,
    port: u16,
    h: Vec<(&'static str, String)>,
}

/// 노드 · 대화 · 운영과 같은 로컬 API(피어 · 설정 · 대화) 한 벌.
async fn side(tag: &str) -> Side {
    let root = std::env::temp_dir().join(format!("an-app-tidy-{tag}-{}", std::process::id()));
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
        ("Authorization", format!("Bearer {token}")),
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

/// 대화 `conv` 의 글(API) — (내 사람 글, 답).
async fn mine_and_answer(s: &Side, conv: i64) -> (Value, Value) {
    let (code, ms) = s
        .call(
            "GET",
            &format!("/api/conversations/{conv}/messages"),
            Value::Null,
        )
        .await;
    assert_eq!(code, 200, "{ms}");
    let ms = ms.as_array().unwrap().clone();
    let pick = |author: &str| ms.iter().find(|m| m["author"] == author).unwrap().clone();
    (pick("human"), pick("agent"))
}

#[tokio::test(flavor = "multi_thread")]
async fn patch_text_rules_and_mirror() {
    let (a, b) = (side("patch-a").await, side("patch-b").await);
    let (pb, _) = pair(&a, &b).await;
    agents::add(&b.db(), "review", ".", "").unwrap();
    let conv = ask(&a, pb, "질의").await;
    let (mine, answer) = mine_and_answer(&a, conv).await;
    let path = |m: &Value| format!("/api/messages/{}", m["id"]);
    for body in [
        json!({ "text": "고침", "saved": true }),
        json!({ "text": "고침", "req_state": "accepted" }),
        json!({ "mirror": true }),
        json!({ "text": "  " }),
    ] {
        let (code, r) = a.call("PATCH", &path(&mine), body.clone()).await;
        assert_eq!(code, 400, "{body} → {r}");
    }
    let (code, r) = a
        .call("PATCH", &path(&answer), json!({ "text": "고침" }))
        .await;
    assert_eq!(code, 400, "답은 편집하지 않는다: {r}");
    let (code, r) = a
        .call(
            "PATCH",
            &path(&mine),
            json!({ "text": "고친 질의", "mirror": true }),
        )
        .await;
    assert_eq!(code, 200, "{r}");
    assert_eq!(r["text"], "고친 질의");
    assert!(r["edited_at"].is_i64() && r.get("warning").is_none(), "{r}");
    let uuid: String = a
        .db()
        .query_row(
            "SELECT uuid FROM messages WHERE id = ?1",
            [mine["id"].as_i64().unwrap()],
            |r| r.get(0),
        )
        .unwrap();
    let until = Instant::now() + Duration::from_secs(5);
    while chat::message_by_uuid(&b.db(), &uuid).unwrap().unwrap().text != "고친 질의" {
        assert!(Instant::now() < until, "상대 창에 반영");
        tokio::time::sleep(Duration::from_millis(30)).await;
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn delete_all_or_nothing() {
    let (a, b) = (side("del-a").await, side("del-b").await);
    let (pb, _) = pair(&a, &b).await;
    agents::add(&b.db(), "review", ".", "").unwrap();
    let (c1, c2) = (ask(&a, pb, "첫 질의").await, ask(&a, pb, "둘째 질의").await);
    let ((m1, ans1), (m2, _)) = (mine_and_answer(&a, c1).await, mine_and_answer(&a, c2).await);
    for ids in [
        json!([]),
        json!([m1["id"], ans1["id"]]),
        json!([m1["id"], m2["id"]]),
        json!([m1["id"], 999_999]),
    ] {
        let (code, r) = a
            .call("POST", "/api/messages/delete", json!({ "ids": ids }))
            .await;
        assert!(code == 400 || code == 404, "{ids} → {code} {r}");
    }
    assert_eq!(
        chat::messages(&a.db(), c1).unwrap().len(),
        2,
        "아무것도 지우지 않았다"
    );
    let (code, r) = a
        .call("POST", "/api/messages/delete", json!({ "ids": [m1["id"]] }))
        .await;
    assert_eq!((code, &r["deleted"]), (200, &json!(1)), "{r}");
    let left = chat::messages(&a.db(), c1).unwrap();
    assert_eq!(left.len(), 1);
    assert_eq!(left[0].author, chat::Author::Agent, "답은 남는다");
}

#[tokio::test(flavor = "multi_thread")]
async fn messages_carry_editable_and_edited_at() {
    let (a, b) = (side("json-a").await, side("json-b").await);
    let (pb, _) = pair(&a, &b).await;
    agents::add(&b.db(), "review", ".", "").unwrap();
    let conv = ask(&a, pb, "질의").await;
    let (mine, answer) = mine_and_answer(&a, conv).await;
    assert_eq!(
        (&mine["editable"], &answer["editable"]),
        (&json!(true), &json!(false))
    );
    assert!(mine["edited_at"].is_null());
    let (code, _) = a
        .call(
            "PATCH",
            &format!("/api/messages/{}", mine["id"]),
            json!({ "text": "고침" }),
        )
        .await;
    assert_eq!(code, 200);
    let (mine, _) = mine_and_answer(&a, conv).await;
    assert!(mine["edited_at"].is_i64(), "{mine}");
    assert_eq!(mine["text"], "고침");
}

#[tokio::test(flavor = "multi_thread")]
async fn activity_lists_sync_failed() {
    let (a, b) = (side("act-a").await, side("act-b").await);
    let (pb, _) = pair(&a, &b).await;
    agents::add(&b.db(), "review", ".", "").unwrap();
    let conv = ask(&a, pb, "질의").await;
    chat::add_relay_event(&a.db(), conv, chat::RelayMark::SyncFailed(None), "삭제").unwrap();
    let kinds = |v: &Value| -> Vec<String> {
        v["items"]
            .as_array()
            .unwrap()
            .iter()
            .map(|i| i["kind"].as_str().unwrap().to_string())
            .collect()
    };
    for (kind, want) in [("all", true), ("failed", true), ("relay", false)] {
        let (code, r) = a
            .call("GET", &format!("/api/activity?kind={kind}"), Value::Null)
            .await;
        assert_eq!(code, 200, "{r}");
        assert_eq!(
            kinds(&r).contains(&"sync_failed".to_string()),
            want,
            "{kind}: {r}"
        );
    }
    let (_, r) = a
        .call("GET", "/api/activity?kind=failed", Value::Null)
        .await;
    let item = &r["items"][0];
    assert_eq!(
        (&item["detail"], &item["conversation"]["id"]),
        (&json!("삭제"), &json!(conv))
    );
    assert!(r["unseen"].as_i64().unwrap() >= 1, "{r}");
}
