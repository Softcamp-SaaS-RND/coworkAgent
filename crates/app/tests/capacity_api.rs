//! 실행 문 로컬 API · CLI(1b A3 · A4 · A5) — `/api/runs` · 메시지 JSON `progress` · CLI 진행 줄 · 대기 중 기다림 멈춤.
mod common;
use an_agent::chat::{Chat, Run};
use an_agent::runner::{RunInput, RunOutput, Session};
use an_app::{chat_api, cli, peers_api, server};
use an_core::instance::InstanceInfo;
use an_core::{agents, db, paths::Paths, settings};
use an_net::identity::Identity;
use an_net::node::{Me, Node, PAIRING_WINDOW};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

/// 실행기 대역 — "답: {본문}". 본문이 "느림" 으로 시작하면 3초 걸린다.
fn echo() -> Run {
    Arc::new(|input: RunInput| {
        let session = match &input.session {
            Session::New(id) | Session::Resume(id) => id.clone(),
        };
        let body = input
            .text
            .rsplit_once("<원격 본문>\n")
            .and_then(|(_, r)| r.strip_suffix("\n</원격 본문>"))
            .unwrap_or(&input.text)
            .to_string();
        if body.starts_with("느림") {
            std::thread::sleep(Duration::from_secs(3));
        }
        Ok(RunOutput {
            text: format!("답: {body}"),
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
}

async fn side(tag: &str, name: &str) -> Side {
    let root = std::env::temp_dir().join(format!("an-app-cap-{tag}-{}", std::process::id()));
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
            name: name.into(),
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

/// a(user-a) · b(user-b, 에이전트 review · 동시 실행 상한 1) — a 쪽 b 의 id.
async fn two(tag: &str) -> (Side, Side, i64) {
    let (a, b) = (
        side(&format!("{tag}-a"), "user-a").await,
        side(&format!("{tag}-b"), "user-b").await,
    );
    let code = b.node.open_window(PAIRING_WINDOW);
    let pb = a.node.add(b.addr, Some(code)).await.unwrap();
    let pa = b.node.peers().unwrap().remove(0);
    a.node.approve(pb.id).await.unwrap();
    b.node.approve(pa.id).await.unwrap();
    agents::add(&db::open(&b.paths.db_file()).unwrap(), "review", ".", None, "").unwrap();
    b.chat.set_max_runs(1).unwrap();
    (a, b, pb.id)
}

/// 운영과 같은 라우터(guard + peers + chat) — (포트, 요청 헤더, `instance.json` 경로).
async fn web(s: &Side) -> (u16, Vec<(&'static str, String)>, PathBuf) {
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
        peers_api::routes(s.node.clone()).merge(chat_api::routes(s.chat.clone())),
    );
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let file = s.paths.instance_file();
    InstanceInfo {
        port,
        token: token.clone(),
    }
    .write(&file)
    .unwrap();
    let headers = vec![
        common::host(port),
        common::origin(port),
        ("Authorization", format!("Bearer {token}")),
        ("Content-Type", "application/json".to_string()),
    ];
    (port, headers, file)
}

async fn call(port: u16, h: &[(&str, String)], method: &str, path: &str, body: &str) -> (u16, Value) {
    let (code, raw) = common::send_body(port, method, path, h, body).await;
    let text = raw.split("\r\n\r\n").nth(1).unwrap_or("");
    (code, serde_json::from_str(text).unwrap_or(Value::Null))
}

/// 조건이 참이 될 때까지(최대 5초).
async fn eventually<F: std::future::Future<Output = bool>>(mut f: impl FnMut() -> F) {
    for _ in 0..250 {
        if f().await {
            return;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    panic!("5초 안에 조건이 참이 되지 않았다");
}

struct Out {
    code: i32,
    out: String,
    err: String,
}

/// CLI 한 번 — 블로킹 HTTP 라 tokio 작업자 밖에서.
async fn run(file: &Path, args: &[&str], wait: Duration) -> Out {
    let file = file.to_path_buf();
    let args: Vec<String> = args.iter().map(|a| a.to_string()).collect();
    tokio::task::spawn_blocking(move || {
        let (mut out, mut err) = (Vec::new(), Vec::new());
        let code = cli::run(&args, &file, &mut out, &mut err, wait);
        Out {
            code,
            out: String::from_utf8(out).unwrap(),
            err: String::from_utf8(err).unwrap(),
        }
    })
    .await
    .unwrap()
}

#[tokio::test(flavor = "multi_thread")]
async fn runs_lists_running_and_waiting() {
    let (a, b, b_id) = two("list").await;
    let (port, h, _) = web(&b).await;
    for q in ["느림 하나", "느림 둘", "느림 셋"] {
        a.chat.send(b_id, "review", None, q).await.unwrap();
    }
    eventually(|| async {
        let (_, v) = call(port, &h, "GET", "/api/runs", "").await;
        v["running"].as_array().map(Vec::len) == Some(1) && v["waiting"].as_array().map(Vec::len) == Some(2)
    })
    .await;
    let (code, v) = call(port, &h, "GET", "/api/runs", "").await;
    assert_eq!(code, 200);
    assert_eq!(v["cap"], 1);
    let first = &v["running"][0];
    assert_eq!((first["peer"].as_str(), first["agent"].as_str()), (Some("user-a"), Some("review")));
    assert!(first["conversation_id"].as_i64().is_some() && first["peer_id"].as_i64().is_some());
    assert_eq!(first["text"], "느림 하나");
    assert_eq!(v["waiting"][0]["text"], "느림 둘", "닿은 순서");
    assert_eq!(v["waiting"][1]["text"], "느림 셋");
}

#[tokio::test(flavor = "multi_thread")]
async fn put_runs_bounds_and_saved() {
    let (_a, b, _) = two("put").await;
    let (port, h, _) = web(&b).await;
    for bad in ["0", "9", "-1", "2.5", r#""3""#, "null"] {
        let (code, _) = call(port, &h, "PUT", "/api/runs", &format!(r#"{{"cap":{bad}}}"#)).await;
        assert_eq!(code, 400, "cap {bad}");
    }
    let (code, v) = call(port, &h, "PUT", "/api/runs", r#"{"cap":3}"#).await;
    assert_eq!((code, v["cap"].as_u64()), (200, Some(3)));
    assert_eq!(b.chat.run_queue().0, 3);
    let db = db::open(&b.paths.db_file()).unwrap();
    assert_eq!(settings::get(&db, "max_runs").unwrap().as_deref(), Some("3"));
}

#[tokio::test(flavor = "multi_thread")]
async fn message_json_carries_progress() {
    let (a, _b, b_id) = two("prog").await;
    let (port, h, _) = web(&a).await;
    let (c1, _) = a.chat.send(b_id, "review", None, "느림 먼저").await.unwrap();
    let (c2, _) = a.chat.send(b_id, "review", None, "다음").await.unwrap();
    let progress = |cid: i64| {
        let h = h.clone();
        async move {
            let (_, v) = call(port, &h, "GET", &format!("/api/conversations/{cid}/messages"), "").await;
            v[0]["progress"].clone()
        }
    };
    eventually(|| async {
        let p = progress(c2.id).await;
        p["state"] == "queued" && p["ahead"] == 0
    })
    .await;
    assert_eq!(progress(c1.id).await["state"], "running");
    // 답이 오면 지운다.
    eventually(|| async { progress(c2.id).await.is_null() && progress(c1.id).await.is_null() }).await;
    let (_, v) = call(port, &h, "GET", &format!("/api/conversations/{}/messages", c2.id), "").await;
    assert_eq!(v[1]["text"], "답: 다음");
}

#[tokio::test(flavor = "multi_thread")]
async fn cli_prints_progress_plain_and_json() {
    let (a, _b, b_id) = two("cli").await;
    let (_, _, file) = web(&a).await;
    // 앞 실행(3초) 뒤에서 기다리고 · 자기도 3초 걸려 처리 중을 본다(확인 간격 1초).
    a.chat.send(b_id, "review", None, "느림 자리").await.unwrap();
    let r = run(&file, &["ask", "user-b", "느림 평문"], Duration::from_secs(20)).await;
    assert_eq!(r.code, 0, "{}", r.err);
    assert_eq!(r.out.trim(), "답: 느림 평문");
    assert_eq!(
        r.err.lines().collect::<Vec<_>>(),
        ["대기 중 — 상대 PC 에 처리 중인 작업이 많다(앞에 0건)", "처리 중"]
    );
    a.chat.send(b_id, "review", None, "느림 자리").await.unwrap();
    let r = run(&file, &["ask", "user-b", "느림 제이슨", "--json"], Duration::from_secs(20)).await;
    assert_eq!(r.code, 0, "{}", r.err);
    let lines: Vec<Value> = r.err.lines().map(|l| serde_json::from_str(l).unwrap()).collect();
    assert_eq!(
        lines,
        [
            serde_json::json!({ "progress": "queued", "ahead": 0 }),
            serde_json::json!({ "progress": "running" })
        ]
    );
    // 표준 출력은 끝 결과 JSON 하나.
    let out: Value = serde_json::from_str(r.out.trim()).unwrap();
    assert_eq!(out["reply"]["text"], "답: 느림 제이슨");
}

#[tokio::test(flavor = "multi_thread")]
async fn cli_waits_while_queued() {
    let (a, _b, b_id) = two("pause").await;
    let (_, _, file) = web(&a).await;
    // 앞 실행 3초 — 대기가 확인되는 동안 CLI 는 기다린다(#1c — 1b 의 "대기 중엔 시계를 멈춤" 은 앱의 트랜잭션 판정으로 대체, 끝은 앱이 정한다).
    a.chat.send(b_id, "review", None, "느림 자리").await.unwrap();
    let r = run(&file, &["ask", "user-b", "빠름"], Duration::from_secs(20)).await;
    assert_eq!(r.code, 0, "{}", r.err);
    assert_eq!(r.out.trim(), "답: 빠름");
    assert!(r.err.contains("대기 중"), "{}", r.err);
}
