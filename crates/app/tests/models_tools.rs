//! #1d — 모델 드롭다운 목록 · 도구 칸 없음 · 요청 거르기(로컬 API 한 벌, 상대 없이).
mod common;
use an_agent::chat::{Chat, Run};
use an_agent::runner::RunOutput;
use an_app::notify::{self, Notifier};
use an_app::{chat_api, peers_api, server};
use an_core::{db, paths::Paths};
use an_net::identity::Identity;
use an_net::node::{Me, Node};
use serde_json::{Value, json};
use std::sync::Arc;

struct Side {
    paths: Paths,
    port: u16,
    h: Vec<(&'static str, String)>,
}

/// 노드 · 대화 · 운영과 같은 로컬 API(피어 · 설정 · 대화) 한 벌 — 실행은 하지 않는다.
async fn side(tag: &str) -> Side {
    let root = std::env::temp_dir().join(format!("an-app-mt-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let paths = Paths::under(&root);
    paths.ensure().unwrap();
    let node = Node::new(
        Identity::load_or_create(&paths.identity_file()).unwrap(),
        db::open(&paths.db_file()).unwrap(),
        Me {
            name: format!("user-{tag}"),
            host: format!("PC-{tag}"),
            port: 1,
        },
    );
    let run: Run =
        Arc::new(|_| Err(std::io::Error::other("실행 없음")) as std::io::Result<RunOutput>);
    let chat = Chat::new(db::open(&paths.db_file()).unwrap(), node.clone(), run);
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
    Side { paths, port, h }
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

#[tokio::test(flavor = "multi_thread")]
async fn models_tools_model_list_only() {
    let a = side("model").await;
    for m in ["opus", "sonnet", "haiku"] {
        let (code, s) = a
            .call("PUT", "/api/settings", json!({ "claude_model": m }))
            .await;
        assert_eq!((code, &s["claude_model"]), (200, &json!(m)), "{s}");
    }
    for bad in ["claude-sonnet-5-5", "sonnet[1m]", "Opus", ""] {
        let (code, r) = a
            .call("PUT", "/api/settings", json!({ "claude_model": bad }))
            .await;
        assert_eq!(code, 400, "{bad} → {r}");
    }
    // 상대별 — 목록 안 · 빈 값(기본값)만.
    a.db()
        .execute_batch(
            "INSERT INTO peers (id, spki, name, host, addr, round, created_at) VALUES (1, x'01', 'p', 'h', '1.2.3.4:1', 'r', 0);",
        )
        .unwrap();
    let (code, r) = a
        .call("PATCH", "/api/peers/1", json!({ "model": "haiku" }))
        .await;
    assert_eq!(code, 200, "{r}");
    let (code, r) = a
        .call(
            "PATCH",
            "/api/peers/1",
            json!({ "model": "claude-opus-5-5" }),
        )
        .await;
    assert_eq!(code, 400, "{r}");
    let (code, r) = a
        .call("PATCH", "/api/peers/1", json!({ "model": "" }))
        .await;
    assert_eq!(code, 200, "{r}");
    let p = an_core::peers::by_id(&a.db(), 1).unwrap().unwrap();
    assert_eq!(p.model, None, "빈 값 = 기본값");
}

#[tokio::test(flavor = "multi_thread")]
async fn models_tools_no_tools_field() {
    let a = side("tools").await;
    let folder = a.paths.root().display().to_string();
    let (code, made) = a
        .call(
            "POST",
            "/api/agents",
            json!({ "name": "review", "folder": folder }),
        )
        .await;
    assert_eq!(code, 201, "{made}");
    assert!(made.get("tools").is_none(), "{made}");
    let (code, r) = a
        .call(
            "POST",
            "/api/agents",
            json!({ "name": "w", "folder": folder, "tools": "Read" }),
        )
        .await;
    assert_eq!(code, 422, "등록에 도구(모르는 필드) → {r}");
    let path = format!("/api/agents/{}", made["id"]);
    let (code, r) = a
        .call("PATCH", &path, json!({ "tools": "Read,Grep" }))
        .await;
    assert_eq!(code, 422, "수정에 도구(모르는 필드) → {r}");
    let (_, list) = a.call("GET", "/api/agents", Value::Null).await;
    assert!(list[0].get("tools").is_none(), "{list}");
}

#[tokio::test(flavor = "multi_thread")]
async fn models_tools_request_filter() {
    let a = side("req").await;
    a.db()
        .execute_batch(
            "INSERT INTO peers (id, spki, name, host, addr, round, created_at) VALUES (1, x'01', 'p', 'h', '1.2.3.4:1', 'r', 0);
             INSERT INTO conversations (id, uuid, peer_id, agent, dir, created_at) VALUES (1, 'c', 1, 'a', 'in', 0);
             INSERT INTO messages (id, uuid, conversation_id, author, text, state, created_at, mine, exec, request, req_state)
               VALUES (1, 'm1', 1, 'human', '대기 요청', 'done', 1, 0, 0, 1, 'wait'),
                      (2, 'm2', 1, 'human', '수용 요청', 'done', 2, 0, 0, 1, 'accepted'),
                      (3, 'm3', 1, 'human', '질의', 'done', 3, 0, 1, 0, NULL);",
        )
        .unwrap();
    let texts = |v: &Value| -> Vec<String> {
        v.as_array()
            .unwrap()
            .iter()
            .map(|m| m["text"].as_str().unwrap().to_string())
            .collect()
    };
    for (q, want) in [
        ("ask=request&req=wait", vec!["대기 요청"]),
        ("ask=request&req=accepted", vec!["수용 요청"]),
        ("ask=request", vec!["수용 요청", "대기 요청"]),
        ("ask=query", vec!["질의"]),
    ] {
        let (code, l) = a
            .call("GET", &format!("/api/messages?side=theirs&{q}"), Value::Null)
            .await;
        assert_eq!((code, texts(&l)), (200, want.iter().map(|t| t.to_string()).collect()), "{q}");
    }
    let (_, l) = a
        .call("GET", "/api/messages?side=theirs&ask=request&req=wait", Value::Null)
        .await;
    assert_eq!(
        (&l[0]["req_state"], &l[0]["req_note"]),
        (&json!("wait"), &Value::Null)
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn models_tools_activity_has_no_ask_kinds() {
    let a = side("act").await;
    // 질의 거르기는 없다. '받은 요청'(`request`)은 #1q 부터 내 활동 거르기다.
    let (code, r) = a.call("GET", "/api/activity?kind=query", Value::Null).await;
    assert_eq!(code, 400, "{r}");
    let (code, r) = a.call("GET", "/api/activity?kind=request", Value::Null).await;
    assert_eq!(code, 200, "{r}");
    let (code, _) = a.call("GET", "/api/activity?kind=all", Value::Null).await;
    assert_eq!(code, 200);
}
