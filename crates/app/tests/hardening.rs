//! 데이터 루트를 품는 에이전트 폴더 거부 · CLI 출력 제어 문자 제거(1a A7 · A14).
mod common;
use an_agent::chat::{Chat, Run};
use an_agent::runner::{RunInput, RunOutput, Session};
use an_app::{chat_api, cli, server};
use an_core::{db, paths::Paths};
use an_net::identity::Identity;
use an_net::node::{Me, Node};
use std::path::PathBuf;
use std::sync::Arc;

fn temp(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("an-app-hard-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn folder_blocks_data_root_inside_and_ancestor() {
    let base = temp("block");
    let root = base.join("coworkAgent");
    std::fs::create_dir_all(root.join("logs")).unwrap();
    let protected = [root.clone()];
    for f in [root.clone(), root.join("logs"), base.clone()] {
        assert!(an_core::paths::folder_blocked(&f, &protected), "{f:?}");
    }
    // 대소문자 · 아직 없는 루트(옛 루트)도 같은 규칙.
    let upper = PathBuf::from(base.display().to_string().to_uppercase());
    assert!(an_core::paths::folder_blocked(&upper, &protected));
    assert!(an_core::paths::folder_blocked(
        &base,
        &[base.join("agentNetwork")]
    ));
    // 네트워크 경로(관리 공유로 로컬을 가리켜도)는 정규화가 로컬과 맞지 않아 통째로 거부.
    for unc in [r"\\localhost\C$\Users", r"\\?\UNC\server\share\x"] {
        assert_eq!(
            an_core::paths::agent_folder_problem(std::path::Path::new(unc)),
            Some("네트워크 경로는 에이전트 폴더로 쓸 수 없다"),
            "{unc}"
        );
    }
}

#[test]
fn folder_allows_siblings_and_prefix_names() {
    let base = temp("allow");
    let root = base.join("coworkAgent");
    let sibling = base.join("coworkAgent2");
    let other = base.join("work").join("docs");
    for d in [&root, &sibling, &other] {
        std::fs::create_dir_all(d).unwrap();
    }
    for f in [&sibling, &other] {
        assert!(
            !an_core::paths::folder_blocked(f, std::slice::from_ref(&root)),
            "{f:?}"
        );
    }
}

fn echo() -> Run {
    Arc::new(|input: RunInput| {
        let session = match &input.session {
            Session::New(id) | Session::Resume(id) => id.clone(),
        };
        Ok(RunOutput {
            text: input.text,
            session,
            is_error: false,
            turns: None,
            duration_ms: None,
            ctx_tokens: None,
        })
    })
}

/// 운영과 같은 `router_with` + `chat_api::routes` — 돌려주는 값: 포트 · 헤더.
async fn web(tag: &str) -> (u16, Vec<(&'static str, String)>) {
    let root = temp(tag);
    let paths = Paths::under(&root);
    paths.ensure().unwrap();
    let node = Node::new(
        Identity::load_or_create(&paths.identity_file()).unwrap(),
        db::open(&paths.db_file()).unwrap(),
        Me {
            name: "user-h".into(),
            host: "PC-h".into(),
            port: 1,
        },
    );
    let chat = Chat::new(db::open(&paths.db_file()).unwrap(), node, echo());
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
        ("Authorization", format!("Bearer {token}")),
        ("Content-Type", "application/json".to_string()),
    ];
    (port, headers)
}

/// 운영 데이터 루트(`%LOCALAPPDATA%\coworkAgent`)의 부모 = 늘 있는 폴더.
fn data_parent() -> String {
    Paths::default_root()
        .unwrap()
        .parent()
        .unwrap()
        .display()
        .to_string()
}

#[tokio::test(flavor = "multi_thread")]
async fn add_agent_rejects_folder_with_data_root() {
    let (port, h) = web("add").await;
    let body = serde_json::json!({ "name": "home", "folder": data_parent() }).to_string();
    let (code, raw) = common::send_body(port, "POST", "/api/agents", &h, &body).await;
    assert_eq!(code, 400, "{raw}");
    assert!(raw.contains("데이터 폴더"), "{raw}");
}

#[tokio::test(flavor = "multi_thread")]
async fn update_agent_rejects_folder_with_data_root() {
    let (port, h) = web("upd").await;
    let ok = temp("upd-ok").display().to_string();
    let body = serde_json::json!({ "name": "docs", "folder": ok }).to_string();
    let (code, raw) = common::send_body(port, "POST", "/api/agents", &h, &body).await;
    assert_eq!(code, 201, "{raw}");
    let made: serde_json::Value =
        serde_json::from_str(raw.split("\r\n\r\n").nth(1).unwrap()).unwrap();
    let path = format!("/api/agents/{}", made["id"]);
    let body = serde_json::json!({ "folder": data_parent() }).to_string();
    let (code, raw) = common::send_body(port, "PATCH", &path, &h, &body).await;
    assert_eq!(code, 400, "{raw}");
    assert!(raw.contains("데이터 폴더"), "{raw}");
}

#[test]
fn printable_strips_control_chars() {
    let s = "답\x1b[31m빨강\x1b]0;제목\x07\r\n줄\t탭\u{9b}2J\u{7f}끝";
    assert_eq!(cli::printable(s), "답[31m빨강]0;제목\n줄\t탭2J끝");
    assert_eq!(cli::printable("보통 글"), "보통 글");
}
