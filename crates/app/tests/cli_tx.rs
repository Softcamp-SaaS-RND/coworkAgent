//! CLI 트랜잭션(#1c A6 · A7) — `ask --id` 중복 없음(동시 포함) · `wait` 이어 받기 · `status` · 대기 · 실행 중 기다림 · 닿지 않으면 종료 코드 7.
mod common;
use an_agent::chat::{Chat, Run};
use an_agent::runner::{RunInput, RunOutput, Session};
use an_app::{chat_api, cli, peers_api, server};
use an_core::instance::InstanceInfo;
use an_core::{agents, db, paths::Paths};
use an_net::identity::Identity;
use an_net::node::{Me, Node, PAIRING_WINDOW};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

/// 실행기 대역 — "답: {본문}", 실행 수를 센다. 본문이 "느림" 이면 3초, "아주 느림" 이면 8초.
fn echo(count: Arc<AtomicUsize>) -> Run {
    Arc::new(move |input: RunInput| {
        count.fetch_add(1, Ordering::SeqCst);
        let session = match &input.session {
            Session::New(id) | Session::Resume(id) => id.clone(),
        };
        let body = input
            .text
            .rsplit_once("<원격 본문>\n")
            .and_then(|(_, r)| r.strip_suffix("\n</원격 본문>"))
            .unwrap_or(&input.text)
            .to_string();
        if body.starts_with("아주 느림") {
            std::thread::sleep(Duration::from_secs(8));
        } else if body.starts_with("느림") {
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
    runs: Arc<AtomicUsize>,
}

async fn side(tag: &str, name: &str) -> Side {
    let root = std::env::temp_dir().join(format!("an-app-tx-{tag}-{}", std::process::id()));
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
    let runs = Arc::new(AtomicUsize::new(0));
    let chat = Chat::new(db::open(&paths.db_file()).unwrap(), node.clone(), echo(runs.clone()));
    chat.attach();
    tokio::spawn(node.clone().serve(listener));
    Side {
        node,
        chat,
        addr,
        paths,
        runs,
    }
}

/// a(user-a) · b(user-b, 에이전트 review) — (a, b, a 쪽 b id, a 의 instance.json).
async fn two(tag: &str) -> (Side, Side, i64, PathBuf) {
    let (a, b) = (
        side(&format!("{tag}-a"), "user-a").await,
        side(&format!("{tag}-b"), "user-b").await,
    );
    let code = b.node.open_window(PAIRING_WINDOW);
    let pb = a.node.add(b.addr, Some(code)).await.unwrap();
    let pa = b.node.peers().unwrap().remove(0);
    a.node.approve(pb.id).await.unwrap();
    b.node.approve(pa.id).await.unwrap();
    agents::add(&db::open(&b.paths.db_file()).unwrap(), "review", ".", "").unwrap();
    let file = web(&a).await;
    (a, b, pb.id, file)
}

/// 운영과 같은 라우터 — `instance.json` 경로.
async fn web(s: &Side) -> PathBuf {
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
    InstanceInfo { port, token }.write(&file).unwrap();
    file
}

struct Out {
    code: i32,
    out: String,
    err: String,
}

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

const WAIT: Duration = Duration::from_secs(30);

/// 상대 주소를 닫힌 포트로(꺼진 상대) — 원래 주소를 돌려준다.
fn unplug(s: &Side, peer: i64) -> String {
    let closed = std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap();
    let conn = db::open(&s.paths.db_file()).unwrap();
    let was: String = conn
        .query_row("SELECT addr FROM peers WHERE id = ?1", [peer], |r| r.get(0))
        .unwrap();
    conn.execute(
        "UPDATE peers SET addr = ?1 WHERE id = ?2",
        (closed.to_string(), peer),
    )
    .unwrap();
    was
}

#[tokio::test(flavor = "multi_thread")]
async fn same_id_rerun_returns_same_answer_without_new_run() {
    let (_a, b, _, file) = two("id").await;
    let r = run(&file, &["ask", "user-b", "질문", "--id", "build-q1", "--json"], WAIT).await;
    assert_eq!(r.code, 0, "{}", r.err);
    let first: Value = serde_json::from_str(r.out.trim()).unwrap();
    assert_eq!(first["reply"]["text"], "답: 질문");
    let again = run(&file, &["ask", "user-b", "질문", "--id", "build-q1", "--json"], WAIT).await;
    assert_eq!(again.code, 0, "{}", again.err);
    let second: Value = serde_json::from_str(again.out.trim()).unwrap();
    assert_eq!(second["tx"], first["tx"]);
    assert_eq!(second["reply"]["id"], first["reply"]["id"]);
    assert_eq!(b.runs.load(Ordering::SeqCst), 1, "새로 실행하지 않는다");
    // 키 규칙 밖은 사용법 · API 오류.
    let bad = run(&file, &["ask", "user-b", "q", "--id", "공백 있는 키"], WAIT).await;
    assert_ne!(bad.code, 0);
}

/// 같은 `--id` 를 동시에 두 번 — 보내기는 한 번, 둘 다 같은 트랜잭션(적대적 평가 라운드 1).
#[tokio::test(flavor = "multi_thread")]
async fn same_id_concurrent_sends_once() {
    let (a, b, _, file) = two("idc").await;
    let args = ["ask", "user-b", "동시 질문", "--id", "race-1", "--no-wait", "--json"];
    let (r1, r2) = tokio::join!(run(&file, &args, WAIT), run(&file, &args, WAIT));
    assert_eq!((r1.code, r2.code), (0, 0), "{} / {}", r1.err, r2.err);
    let (v1, v2): (Value, Value) = (
        serde_json::from_str(r1.out.trim()).unwrap(),
        serde_json::from_str(r2.out.trim()).unwrap(),
    );
    assert!(v1["tx"].is_i64() && v1["tx"] == v2["tx"], "{v1} / {v2}");
    let conn = db::open(&a.paths.db_file()).unwrap();
    let sent: i64 = conn
        .query_row("SELECT COUNT(*) FROM messages WHERE author = 'human'", [], |r| r.get(0))
        .unwrap();
    assert_eq!(sent, 1, "글 하나");
    tokio::time::sleep(Duration::from_millis(500)).await;
    assert_eq!(b.runs.load(Ordering::SeqCst), 1);
}

#[tokio::test(flavor = "multi_thread")]
async fn wait_resumes_after_cli_left() {
    let (_a, _b, _, file) = two("wait").await;
    let r = run(&file, &["ask", "user-b", "느림 이어 받기", "--no-wait", "--json"], WAIT).await;
    assert_eq!(r.code, 0, "{}", r.err);
    let sent: Value = serde_json::from_str(r.out.trim()).unwrap();
    let tx = sent["tx"].as_i64().expect("tx").to_string();
    let w = run(&file, &["wait", &tx], WAIT).await;
    assert_eq!(w.code, 0, "{}", w.err);
    assert_eq!(w.out.trim(), "답: 느림 이어 받기");
}

#[tokio::test(flavor = "multi_thread")]
async fn status_prints_once() {
    let (_a, _b, _, file) = two("stat").await;
    let r = run(&file, &["ask", "user-b", "상태", "--json"], WAIT).await;
    let tx = serde_json::from_str::<Value>(r.out.trim()).unwrap()["tx"]
        .as_i64()
        .unwrap()
        .to_string();
    let s = run(&file, &["status", &tx], WAIT).await;
    assert_eq!((s.code, s.out.trim()), (0, format!("{tx}\tanswered").as_str()));
    let j = run(&file, &["status", &tx, "--json"], WAIT).await;
    let v: Value = serde_json::from_str(j.out.trim()).unwrap();
    assert_eq!((v["state"].as_str(), v["reply"]["text"].as_str()), (Some("answered"), Some("답: 상태")));
    let none = run(&file, &["status", "9999"], WAIT).await;
    assert_eq!(none.code, cli::EXIT_API);
}

#[tokio::test(flavor = "multi_thread")]
async fn waits_while_peer_confirms_running() {
    let (a, _b, _, file) = two("run").await;
    // 1초마다 상태를 묻고 2초 닿지 않으면 멈춘다 — 실행 중(3초)이 확인되는 동안은 멈추지 않는다.
    a.chat.set_watch(1, 2);
    let r = run(&file, &["ask", "user-b", "느림 확인"], Duration::from_secs(20)).await;
    assert_eq!(r.code, 0, "{}", r.err);
    assert_eq!(r.out.trim(), "답: 느림 확인");
    assert!(r.err.contains("처리 중"), "{}", r.err);
}

#[tokio::test(flavor = "multi_thread")]
async fn unreachable_peer_stalls_with_code_7_then_wait_resumes() {
    let (a, _b, b_id, file) = two("stall").await;
    a.chat.set_watch(1, 2);
    let r = run(&file, &["ask", "user-b", "아주 느림 질문", "--no-wait", "--json"], WAIT).await;
    let tx = serde_json::from_str::<Value>(r.out.trim()).unwrap()["tx"]
        .as_i64()
        .unwrap()
        .to_string();
    // 상대가 실행하는 동안 이 PC 에서 상대에 닿지 않는다.
    let was = unplug(&a, b_id);
    let w = run(&file, &["wait", &tx], Duration::from_secs(20)).await;
    assert_eq!(w.code, cli::EXIT_STALLED, "{}", w.err);
    assert!(w.err.contains(&format!("coworkAgent wait {tx}")), "{}", w.err);
    // 다시 닿으면 이어 받는다(답은 그새 왔거나 곧 온다).
    db::open(&a.paths.db_file())
        .unwrap()
        .execute("UPDATE peers SET addr = ?1 WHERE id = ?2", (was, b_id))
        .unwrap();
    let again = run(&file, &["wait", &tx], Duration::from_secs(20)).await;
    assert_eq!(again.code, 0, "{}", again.err);
    assert_eq!(again.out.trim(), "답: 아주 느림 질문");
}
