//! 사용자 AI 에이전트용 CLI — 운영과 같은 로컬 API(guard · peers · chat)를 띄우고 `cli::run` 을 실제 TCP 로 부른다.
use an_agent::chat::{Chat, Run};
use an_agent::runner::{RunInput, RunOutput, Session};
use an_app::{chat_api, cli, peers_api, server};
use an_core::instance::InstanceInfo;
use an_core::{agents, chat, db, paths::Paths};
use an_net::identity::Identity;
use an_net::node::{Me, Node, PAIRING_WINDOW};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

/// 실행기 대역 — "답: {입력}". "실패" 로 시작하면 실행 오류, "느림" 이면 2초 걸린다.
fn echo() -> Run {
    Arc::new(|input: RunInput| {
        let session = match &input.session {
            Session::New(id) | Session::Resume(id) => id.clone(),
        };
        // 실행 입력의 원격 본문(`<원격 본문>` 구분자 안)만 본다.
        let body = input
            .text
            .rsplit_once("<원격 본문>\n")
            .and_then(|(_, r)| r.strip_suffix("\n</원격 본문>"))
            .unwrap_or(&input.text)
            .to_string();
        if body.starts_with("느림") {
            std::thread::sleep(Duration::from_secs(2));
        }
        Ok(RunOutput {
            text: format!("답: {body}"),
            session,
            is_error: body.starts_with("실패"),
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
}

async fn side(tag: &str, name: &str) -> Side {
    let root = std::env::temp_dir().join(format!("an-app-cli-{tag}-{}", std::process::id()));
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

/// `a` 에서 `b` 를 페어링하고 `b` 에 에이전트 `names` 를 등록한다. `a` 쪽 피어 id.
async fn pair(a: &Side, b: &Side, names: &[&str]) -> i64 {
    let code = b.node.open_window(PAIRING_WINDOW);
    let pb = a.node.add(b.addr, Some(code)).await.unwrap();
    // b 의 피어는 a 하나(테스트마다 새 PC).
    let pa = b.node.peers().unwrap().remove(0);
    a.node.approve(pb.id).await.unwrap();
    b.node.approve(pa.id).await.unwrap();
    let db = db::open(&b.paths.db_file()).unwrap();
    for n in names {
        let _ = agents::add(&db, n, ".", "");
    }
    pb.id
}

/// 운영과 같은 라우터(guard + peers + chat)를 띄우고 그 `instance.json` 경로를 돌려준다.
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

const WAIT: Duration = Duration::from_secs(10);

#[tokio::test(flavor = "multi_thread")]
async fn peers_lists_paired_with_json() {
    let (a, b) = (side("pl-a", "user-a").await, side("pl-b", "user-b").await);
    pair(&a, &b, &["review"]).await;
    let file = web(&a).await;
    let r = run(&file, &["peers"], WAIT).await;
    assert_eq!(r.code, 0, "{}", r.err);
    let line = r.out.lines().next().unwrap();
    assert!(line.starts_with("user-b\tPC-pl-b\t"), "{line}");
    assert!(line.ends_with("\t페어링됨"), "{line}");
    let r = run(&file, &["peers", "--json"], WAIT).await;
    assert_eq!(r.code, 0, "{}", r.err);
    let v: serde_json::Value = serde_json::from_str(&r.out).unwrap();
    assert_eq!(v.as_array().unwrap().len(), 1);
    assert_eq!(
        (v[0]["name"].as_str(), v[0]["paired"].as_bool()),
        (Some("user-b"), Some(true))
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn peer_name_ambiguous_errors() {
    let (a, b, c) = (
        side("pa-a", "user-a").await,
        side("pa-b", "same").await,
        side("pa-c", "same").await,
    );
    pair(&a, &b, &["review"]).await;
    pair(&a, &c, &["review"]).await;
    let file = web(&a).await;
    let r = run(&file, &["ask", "same", "q"], WAIT).await;
    assert_eq!(r.code, cli::EXIT_USAGE, "{}", r.out);
    assert!(r.err.contains("여럿"), "{}", r.err);
    let r = run(&file, &["ask", "없는사람", "q"], WAIT).await;
    assert_eq!(r.code, cli::EXIT_USAGE);
    // 지문 앞자리로는 하나를 고른다.
    let b_fp = a
        .node
        .peers()
        .unwrap()
        .into_iter()
        .find(|p| p.addr.ends_with(&format!(":{}", b.addr.port())))
        .map(|p| an_net::identity::fingerprint(&p.spki))
        .unwrap();
    let prefix: String = b_fp
        .chars()
        .filter(|c| !c.is_whitespace())
        .take(8)
        .collect();
    let r = run(&file, &["ask", &prefix.to_lowercase(), "q"], WAIT).await;
    assert_eq!(r.code, 0, "{}", r.err);
    assert_eq!(r.out, "답: q\n");
}

#[tokio::test(flavor = "multi_thread")]
async fn ask_new_conversation_waits_for_reply() {
    let (a, b) = (side("an-a", "user-a").await, side("an-b", "user-b").await);
    pair(&a, &b, &["review"]).await;
    let file = web(&a).await;
    let r = run(&file, &["ask", "user-b", "안녕"], WAIT).await;
    assert_eq!(r.code, 0, "{}", r.err);
    assert_eq!(r.out, "답: 안녕\n");
    let db = db::open(&a.paths.db_file()).unwrap();
    let conv = chat::list(&db).unwrap()[0].id;
    let ms = chat::messages(&db, conv).unwrap();
    assert_eq!(ms[0].via.as_deref(), Some("cli"), "CLI 글로 남는다");
    // --json — 대화 · 질문 · 답.
    let r = run(&file, &["ask", "user-b", "둘", "--json"], WAIT).await;
    assert_eq!(r.code, 0, "{}", r.err);
    let v: serde_json::Value = serde_json::from_str(&r.out).unwrap();
    assert_eq!(v["asked"]["via"], "cli");
    assert_eq!(v["reply"]["text"], "답: 둘");
    assert!(v["conversation"]["id"].is_i64());
    // -- 뒤는 전부 본문 — 옵션처럼 보이는 질문.
    let r = run(&file, &["ask", "user-b", "--", "--json 은 무슨 뜻?"], WAIT).await;
    assert_eq!(r.code, 0, "{}", r.err);
    assert_eq!(r.out, "답: --json 은 무슨 뜻?\n");
}

#[tokio::test(flavor = "multi_thread")]
async fn ask_conv_continues_same_conversation() {
    let (a, b) = (side("ac-a", "user-a").await, side("ac-b", "user-b").await);
    pair(&a, &b, &["review"]).await;
    let file = web(&a).await;
    let r = run(&file, &["ask", "user-b", "하나", "--no-wait"], WAIT).await;
    let conv = r.out.trim().to_string();
    let r = run(
        &file,
        &["ask", "user-b", "둘", "--conv", &conv, "--json"],
        WAIT,
    )
    .await;
    assert_eq!(r.code, 0, "{}", r.err);
    let v: serde_json::Value = serde_json::from_str(&r.out).unwrap();
    assert_eq!(v["conversation"]["id"].to_string(), conv);
    assert_eq!(
        v["reply"]["text"], "답: 둘",
        "이 질문의 답(reply_to) — 앞선 답이 아니다"
    );
    let db = db::open(&a.paths.db_file()).unwrap();
    assert_eq!(chat::list(&db).unwrap().len(), 1, "새 대화를 열지 않는다");
    // 다른 에이전트를 고르면 오류 · 없는 대화도 오류.
    let r = run(
        &file,
        &["ask", "user-b", "셋", "--conv", &conv, "--agent", "dev"],
        WAIT,
    )
    .await;
    assert_eq!(r.code, cli::EXIT_USAGE);
    let r = run(&file, &["ask", "user-b", "셋", "--conv", "9999"], WAIT).await;
    assert_eq!(r.code, cli::EXIT_USAGE);
}

#[tokio::test(flavor = "multi_thread")]
async fn ask_no_wait_prints_conversation_id() {
    let (a, b) = (side("nw-a", "user-a").await, side("nw-b", "user-b").await);
    pair(&a, &b, &["review"]).await;
    let file = web(&a).await;
    let r = run(&file, &["ask", "user-b", "느림", "--no-wait"], WAIT).await;
    assert_eq!(r.code, 0, "{}", r.err);
    let id: i64 = r.out.trim().parse().unwrap();
    let db = db::open(&a.paths.db_file()).unwrap();
    assert_eq!(chat::list(&db).unwrap()[0].id, id);
    assert_eq!(
        chat::messages(&db, id).unwrap().len(),
        1,
        "답을 기다리지 않았다"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn ask_agent_ambiguous_errors() {
    let (a, b) = (side("ag-a", "user-a").await, side("ag-b", "user-b").await);
    pair(&a, &b, &["review", "dev"]).await;
    let file = web(&a).await;
    let r = run(&file, &["ask", "user-b", "q"], WAIT).await;
    assert_eq!(r.code, cli::EXIT_USAGE);
    assert!(
        r.err.contains("review") && r.err.contains("dev"),
        "{}",
        r.err
    );
    let r = run(&file, &["ask", "user-b", "q", "--agent", "dev"], WAIT).await;
    assert_eq!(r.code, 0, "{}", r.err);
}

#[tokio::test(flavor = "multi_thread")]
async fn ask_remote_failure_exit_code() {
    let (a, b) = (side("rf-a", "user-a").await, side("rf-b", "user-b").await);
    pair(&a, &b, &["review"]).await;
    let file = web(&a).await;
    let r = run(&file, &["ask", "user-b", "실패해"], WAIT).await;
    assert_eq!(r.code, cli::EXIT_REPLY_FAILED, "{}", r.out);
    // 상대 실행 오류의 원문은 상대 PC 에만 — 이쪽에는 정해진 문구(1a A4).
    assert!(
        r.err.contains("받는 PC 의 에이전트가 오류로 끝났다"),
        "{}",
        r.err
    );
    // 없는 에이전트 — 상대가 거절(로컬 API 4xx · 5xx).
    let r = run(&file, &["ask", "user-b", "q", "--agent", "없음"], WAIT).await;
    assert_eq!(r.code, cli::EXIT_API, "{}", r.err);
}

#[tokio::test(flavor = "multi_thread")]
async fn ask_timeout_exit_code() {
    let (a, b) = (side("to-a", "user-a").await, side("to-b", "user-b").await);
    pair(&a, &b, &["review"]).await;
    let file = web(&a).await;
    let r = run(
        &file,
        &["ask", "user-b", "느림"],
        Duration::from_millis(300),
    )
    .await;
    assert_eq!(r.code, cli::EXIT_TIMEOUT);
    assert!(r.err.contains("history --conv"), "{}", r.err);
}

/// 사람에게 보내기 제거(D23 개정) — `say` 는 모르는 명령, 아무것도 보내지 않는다.
#[tokio::test(flavor = "multi_thread")]
async fn say_is_unknown_command() {
    let (a, b) = (side("sx-a", "user-a").await, side("sx-b", "user-b").await);
    pair(&a, &b, &["review"]).await;
    let file = web(&a).await;
    let r = run(&file, &["say", "user-b", "메모"], WAIT).await;
    assert_eq!(r.code, cli::EXIT_USAGE);
    assert!(r.err.contains("모르는 명령"), "{}", r.err);
    assert!(!cli::USAGE.contains("say"));
    let db = db::open(&a.paths.db_file()).unwrap();
    assert!(chat::list(&db).unwrap().is_empty(), "대화가 생기지 않는다");
}

#[tokio::test(flavor = "multi_thread")]
async fn history_lists_and_shows_timeline() {
    let (a, b) = (side("hi-a", "user-a").await, side("hi-b", "user-b").await);
    pair(&a, &b, &["review"]).await;
    let file = web(&a).await;
    run(&file, &["ask", "user-b", "안녕"], WAIT).await;
    let db = db::open(&a.paths.db_file()).unwrap();
    let conv = chat::list(&db).unwrap()[0].id.to_string();
    let r = run(&file, &["history"], WAIT).await;
    assert_eq!(r.code, 0, "{}", r.err);
    assert!(
        r.out
            .starts_with(&format!("{conv}\tuser-b\treview\t안녕\t")),
        "{}",
        r.out
    );
    let r = run(&file, &["history", "user-b", "--conv", &conv], WAIT).await;
    assert_eq!(r.code, 0, "{}", r.err);
    assert!(r.out.contains("] 내 AI 에이전트(CLI): 안녕\n"), "{}", r.out);
    assert!(r.out.contains("] user-b/review: 답: 안녕\n"), "{}", r.out);
    let r = run(&file, &["history", "--conv", &conv, "--json"], WAIT).await;
    let v: serde_json::Value = serde_json::from_str(&r.out).unwrap();
    assert_eq!(v.as_array().unwrap().len(), 2);
}

#[tokio::test(flavor = "multi_thread")]
async fn not_running_exits_with_code() {
    let dir = std::env::temp_dir().join(format!("an-app-cli-nr-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let missing = dir.join("instance.json");
    let _ = std::fs::remove_file(&missing);
    let r = run(&missing, &["peers"], WAIT).await;
    assert_eq!(r.code, cli::EXIT_NOT_RUNNING);
    assert!(
        r.err.contains("coworkAgent 가 실행 중이 아니다"),
        "{}",
        r.err
    );
    // 죽은 프로세스가 남긴 instance.json — 닫힌 포트.
    let closed = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = closed.local_addr().unwrap().port();
    drop(closed);
    InstanceInfo {
        port,
        token: "ab".repeat(32),
    }
    .write(&missing)
    .unwrap();
    let r = run(&missing, &["peers"], WAIT).await;
    assert_eq!(r.code, cli::EXIT_NOT_RUNNING, "{}", r.err);
    assert!(
        r.err.contains("coworkAgent 가 실행 중이 아니다"),
        "{}",
        r.err
    );
    // 남은 포트를 다른 프로그램이 쓰는 경우 — HTTP 가 아닌 응답.
    let other = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = other.local_addr().unwrap().port();
    std::thread::spawn(move || {
        use std::io::{Read, Write};
        let (mut s, _) = other.accept().unwrap();
        let mut buf = [0u8; 1024];
        let _ = s.read(&mut buf);
        let _ = s.write_all(b"SSH-2.0-other\r\n");
        // FIN 으로 닫고 상대가 닫을 때까지 기다린다 — 읽지 않은 데이터로 RST 가 나면 다른 분기(연결 끊김)를 탄다.
        let _ = s.shutdown(std::net::Shutdown::Write);
        let _ = s.read_to_end(&mut Vec::new());
    });
    InstanceInfo {
        port,
        token: "ab".repeat(32),
    }
    .write(&missing)
    .unwrap();
    let r = run(&missing, &["peers"], WAIT).await;
    assert_eq!(r.code, cli::EXIT_NOT_RUNNING, "{}", r.err);
}

#[tokio::test(flavor = "multi_thread")]
async fn bad_token_rejected() {
    let a = side("bt-a", "user-a").await;
    let file = web(&a).await;
    let info = InstanceInfo::read(&file).unwrap().unwrap();
    InstanceInfo {
        port: info.port,
        token: "cd".repeat(32),
    }
    .write(&file)
    .unwrap();
    let r = run(&file, &["peers"], WAIT).await;
    assert_eq!(r.code, cli::EXIT_API);
    assert!(r.err.contains("401"), "{}", r.err);
    // 인자가 모자라면 사용법 오류.
    let r = run(&file, &["ask"], WAIT).await;
    assert_eq!(r.code, cli::EXIT_USAGE);
    assert!(r.err.contains("coworkAgent ask"), "{}", r.err);
}

/// 실제 실행 파일 — 모르는 명령은 트레이로 새지 않고 사용법 오류(인스턴스 유무와 무관, 인자 해석이 먼저다).
#[test]
fn unknown_command_exe_exits_usage() {
    for args in [vec!["peerz"], vec!["--json"], vec!["chat", "--peer", "x"]] {
        let r = std::process::Command::new(env!("CARGO_BIN_EXE_coworkAgent"))
            .args(&args)
            .output()
            .unwrap();
        assert_eq!(r.status.code(), Some(cli::EXIT_USAGE), "{args:?}");
        let err = String::from_utf8_lossy(&r.stderr);
        assert!(
            err.contains("모르는 명령") && err.contains("coworkAgent peers"),
            "{err}"
        );
    }
}

/// 실제 실행 파일 — `/?` · `-h` · `--help`(첫 인자)는 앱이 꺼져 있어도 표준 출력에 명령 · 옵션 · 종료 코드 · `--json` 형식을 쓰고 0(#1q — 다른 AI
/// 도구가 파이프로 읽고 연동). 표준 오류는 비어 있다.
#[test]
fn help_exe_prints_commands() {
    for arg in ["/?", "-h", "--help"] {
        let r = std::process::Command::new(env!("CARGO_BIN_EXE_coworkAgent"))
            .arg(arg)
            .output()
            .unwrap();
        assert_eq!(r.status.code(), Some(cli::EXIT_OK), "{arg}");
        let out = String::from_utf8_lossy(&r.stdout);
        for want in [
            "coworkAgent /?",
            "coworkAgent ask",
            "coworkAgent start",
            "coworkAgent history",
            "종료 코드",
            "7 = ",
            "--json",
            "\"progress\"",
        ] {
            assert!(out.contains(want), "{arg}: {want} 없음
{out}");
        }
        assert!(r.stderr.is_empty(), "{arg}: {}", String::from_utf8_lossy(&r.stderr));
    }
    // 첫 인자만 — 다른 자리의 `/?` 는 도움말이 아니다(묻는 본문일 수 있다).
    let r = std::process::Command::new(env!("CARGO_BIN_EXE_coworkAgent"))
        .args(["peerz", "/?"])
        .output()
        .unwrap();
    assert_eq!(r.status.code(), Some(cli::EXIT_USAGE));
}

/// 이전 버전 주 인스턴스(`/api/status` 에 `cli_api` 없음) — 아무것도 보내지 않고 4.
#[test]
fn old_primary_refused_before_send() {
    use std::io::{Read, Write};
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let seen = Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
    let log = seen.clone();
    std::thread::spawn(move || {
        for s in listener.incoming() {
            let mut s = s.unwrap();
            // 요청 머리가 여러 조각으로 올 수 있다 — 빈 줄까지 읽는다.
            let (mut req, mut buf) = (Vec::new(), [0u8; 4096]);
            while !req.windows(4).any(|w| w == b"\r\n\r\n") {
                match s.read(&mut buf) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => req.extend_from_slice(&buf[..n]),
                }
            }
            let first = String::from_utf8_lossy(&req)
                .lines()
                .next()
                .unwrap_or("")
                .to_string();
            log.lock().unwrap().push(first);
            let body = r#"{"version":"0.1.0","data_dir":"d","db":"ok (스키마 7)"}"#;
            let _ = write!(
                s,
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            let _ = s.shutdown(std::net::Shutdown::Write);
            let _ = s.read_to_end(&mut Vec::new());
        }
    });
    let dir = std::env::temp_dir().join(format!("an-app-cli-old-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("instance.json");
    InstanceInfo {
        port,
        token: "ab".repeat(32),
    }
    .write(&file)
    .unwrap();
    let args: Vec<String> = ["ask", "user-b", "q"]
        .iter()
        .map(|a| a.to_string())
        .collect();
    let (mut out, mut err) = (Vec::new(), Vec::new());
    let code = cli::run(&args, &file, &mut out, &mut err, WAIT);
    let err = String::from_utf8(err).unwrap();
    assert_eq!(code, cli::EXIT_API, "{err}");
    assert!(err.contains("지원하지 않는다"), "{err}");
    assert_eq!(
        *seen.lock().unwrap(),
        ["GET /api/status HTTP/1.1"],
        "상태 확인 말고는 보내지 않았다"
    );
}

/// 앱이 POST 를 받은 뒤 응답 없이 끊긴 경우 — "실행 중 아님"(3)이 아니라 전달 여부 불명(4) + history 안내.
#[test]
fn post_lost_response_is_not_not_running() {
    use std::io::{Read, Write};
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    std::thread::spawn(move || {
        for s in listener.incoming() {
            let mut s = s.unwrap();
            let (mut req, mut buf) = (Vec::new(), [0u8; 4096]);
            while !req.windows(4).any(|w| w == b"\r\n\r\n") {
                match s.read(&mut buf) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => req.extend_from_slice(&buf[..n]),
                }
            }
            let first = String::from_utf8_lossy(&req)
                .lines()
                .next()
                .unwrap_or("")
                .to_string();
            let body = match first.split_whitespace().nth(1).unwrap_or("") {
                "/api/status" => r#"{"version":"t","data_dir":"d","db":"ok","cli_api":1}"#,
                "/api/peers" => {
                    r#"[{"id":1,"name":"user-b","host":"PC-b","fingerprint":"AB12 CD34","paired":true}]"#
                }
                "/api/peers/1/agents" => r#"{"names":["review"]}"#,
                // POST /api/conversations — 받고 응답 없이 닫는다(앱이 그 사이 종료).
                _ => "",
            };
            if !body.is_empty() {
                let _ = write!(
                    s,
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
            }
            let _ = s.shutdown(std::net::Shutdown::Write);
            let _ = s.read_to_end(&mut Vec::new());
        }
    });
    let dir = std::env::temp_dir().join(format!("an-app-cli-lost-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("instance.json");
    InstanceInfo {
        port,
        token: "ab".repeat(32),
    }
    .write(&file)
    .unwrap();
    let args: Vec<String> = ["ask", "user-b", "q"]
        .iter()
        .map(|a| a.to_string())
        .collect();
    let (mut out, mut err) = (Vec::new(), Vec::new());
    let code = cli::run(&args, &file, &mut out, &mut err, WAIT);
    let err = String::from_utf8(err).unwrap();
    assert_eq!(code, cli::EXIT_API, "{err}");
    assert!(
        err.contains("전달 여부를 모른다") && err.contains("history"),
        "{err}"
    );
}

/// `ask --conv N --agent X` — X 가 그 대화 에이전트가 아니면 멘션(상대 PC 에 있는 이름만, 없으면 사용법 오류). 답은 부른 에이전트가 하고 이력
/// 발화자도 그 이름.
#[tokio::test(flavor = "multi_thread")]
async fn ask_mention_in_conversation() {
    let (a, b) = (side("am-a", "user-a").await, side("am-b", "user-b").await);
    pair(&a, &b, &["review", "guide"]).await;
    let file = web(&a).await;
    let r = run(
        &file,
        &["ask", "user-b", "하나", "--agent", "review", "--no-wait"],
        WAIT,
    )
    .await;
    let conv = r.out.trim().to_string();
    let r = run(
        &file,
        &[
            "ask",
            "user-b",
            "@guide 둘",
            "--conv",
            &conv,
            "--agent",
            "guide",
            "--json",
        ],
        WAIT,
    )
    .await;
    assert_eq!(r.code, 0, "{}", r.err);
    let v: serde_json::Value = serde_json::from_str(&r.out).unwrap();
    assert_eq!(
        (
            &v["asked"]["agent"],
            &v["reply"]["agent"],
            &v["reply"]["text"]
        ),
        (
            &serde_json::json!("guide"),
            &serde_json::json!("guide"),
            &serde_json::json!("답: @guide 둘")
        ),
        "{v}"
    );
    let r = run(&file, &["history", "--conv", &conv], WAIT).await;
    assert!(r.out.contains("user-b/guide"), "{}", r.out);
    let r = run(
        &file,
        &["ask", "user-b", "셋", "--conv", &conv, "--agent", "nobody"],
        WAIT,
    )
    .await;
    assert_eq!(r.code, cli::EXIT_USAGE, "{}", r.err);
}
