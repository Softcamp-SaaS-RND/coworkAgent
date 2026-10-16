//! 이어 쓸 Claude 세션이 사라졌을 때(#1h — 원장 #15 R2-F2r): 판별(claude 2.1.299 결과 줄 · 2.1.296 stderr) · 새 세션으로 한 번 다시 실행 · 다음 요청은
//! 새 세션을 잇는다 · 사라짐이 아닌 실패는 다시 실행하지 않는다.
mod common;
use an_agent::runner::{self, RunInput, RunOutput, Session, SessionMissing};
use an_core::agents;
use an_core::chat::{self, Author, State};
use std::io;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// claude 2.1.299 실측(없는 세션 · `stream-json`) — 값은 가상.
const RESULT_LINE: &str = r#"{"type":"result","subtype":"error_during_execution","duration_ms":0,"duration_api_ms":0,"is_error":true,"num_turns":0,"stop_reason":null,"session_id":"3f1c2a9e-0000-4000-8000-000000000001","total_cost_usd":0,"permission_denials":[],"errors":["No conversation found with session ID: 3f1c2a9e-0000-4000-8000-000000000001"]}"#;

fn agent_messages(conn: &an_core::db::Connection, conv: i64) -> Vec<chat::Message> {
    chat::messages(conn, conv)
        .unwrap()
        .into_iter()
        .filter(|m| m.author == Author::Agent)
        .collect()
}

fn id(s: &Session) -> String {
    match s {
        Session::New(id) | Session::Resume(id) => id.clone(),
    }
}

#[test]
fn session_gone_result_line() {
    let out = format!("{{\"type\":\"system\",\"subtype\":\"init\"}}\n{RESULT_LINE}\n");
    assert_eq!(
        runner::session_gone(out.as_bytes(), b"No conversation found with session ID: x")
            .as_deref(),
        Some("No conversation found with session ID: 3f1c2a9e-0000-4000-8000-000000000001")
    );
}

#[test]
fn session_gone_stderr_only() {
    // 2.1.296 — JSON 없음 · 종료 1 · stderr 한 줄.
    assert_eq!(
        runner::session_gone(b"", b"\nNo conversation found with session ID: abc\r\n").as_deref(),
        Some("No conversation found with session ID: abc")
    );
}

#[test]
fn session_gone_ignores_success_and_other_failures() {
    let ok = r#"{"type":"result","is_error":false,"result":"No conversation found with session ID 라는 문장을 찾았다","session_id":"s"}"#;
    assert_eq!(
        runner::session_gone(ok.as_bytes(), b""),
        None,
        "답 본문의 같은 문장"
    );
    let other = r#"{"type":"result","is_error":true,"errors":["rate limited"],"session_id":"s"}"#;
    assert_eq!(
        runner::session_gone(
            other.as_bytes(),
            b"No conversation found with session ID: s"
        ),
        None,
        "결과 줄이 있으면 stderr 는 보지 않는다"
    );
    assert_eq!(runner::session_gone(b"", b"some other failure"), None);
    assert!(!runner::session_missing(&io::Error::other("boom")));
    assert!(runner::session_missing(&io::Error::other(SessionMissing(
        "x".into()
    ))));
}

/// 실행기 대역 — `gone` 에 든 세션을 재개하면 [`SessionMissing`], `fail_resume` 이면 다른 실패, 그 밖은 답.
fn flaky_run(
    runs: Arc<Mutex<Vec<RunInput>>>,
    gone: Arc<Mutex<Vec<String>>>,
    fail_resume: bool,
) -> an_agent::chat::Run {
    Arc::new(move |input: RunInput| {
        runs.lock().unwrap().push(input.clone());
        if let Session::Resume(s) = &input.session {
            if gone.lock().unwrap().contains(s) {
                return Err(io::Error::other(SessionMissing(format!(
                    "No conversation found with session ID: {s}"
                ))));
            }
            if fail_resume {
                return Err(io::Error::other("다른 실패"));
            }
        }
        Ok(RunOutput {
            text: format!("답: {}", common::body(&input.text)),
            session: id(&input.session),
            is_error: false,
            turns: Some(1),
            duration_ms: Some(10),
        })
    })
}

#[tokio::test(flavor = "multi_thread")]
async fn missing_session_reruns_with_new_session() {
    let gone: Arc<Mutex<Vec<String>>> = Arc::default();
    let a = common::side("rm-a", Duration::ZERO).await;
    let g = gone.clone();
    let b = common::side_with("rm-b", move |runs| flaky_run(runs, g, false)).await;
    let (b_id, _) = common::pair(&a, &b).await;
    agents::add(&common::db(&b), "dev", ".", "").unwrap();
    let (conv, _) = a.chat.send(b_id, "dev", None, "첫 질문").await.unwrap();
    let a_db = common::db(&a);
    common::eventually(|| agent_messages(&a_db, conv.id).len() == 1).await;
    let first = id(&b.runs.lock().unwrap()[0].session);
    // 세션 파일이 사라졌다.
    gone.lock().unwrap().push(first.clone());
    a.chat
        .send(b_id, "dev", Some(conv.id), "이어서")
        .await
        .unwrap();
    common::eventually(|| agent_messages(&a_db, conv.id).len() == 2).await;
    let reply = agent_messages(&a_db, conv.id).remove(1);
    assert_eq!(
        (reply.text.as_str(), reply.state),
        ("답: 이어서", State::Done),
        "새 세션으로 답이 왔다"
    );
    let second = {
        let runs = b.runs.lock().unwrap();
        assert_eq!(runs.len(), 3, "재개 1번 · 새 세션 1번");
        assert_eq!(runs[1].session, Session::Resume(first.clone()));
        assert!(matches!(runs[2].session, Session::New(_)));
        assert_ne!(id(&runs[2].session), first);
        assert_eq!(runs[2].text, runs[1].text, "같은 요청을 다시");
        id(&runs[2].session)
    };
    let b_db = common::db(&b);
    assert_eq!(
        chat::list(&b_db).unwrap()[0].session.as_deref(),
        Some(second.as_str()),
        "새 세션을 기록"
    );
    // 다음 요청은 새 세션을 잇는다 — 실패하지 않는다.
    a.chat
        .send(b_id, "dev", Some(conv.id), "그다음")
        .await
        .unwrap();
    common::eventually(|| agent_messages(&a_db, conv.id).len() == 3).await;
    assert_eq!(agent_messages(&a_db, conv.id)[2].state, State::Done);
    assert_eq!(b.runs.lock().unwrap()[3].session, Session::Resume(second));
}

#[tokio::test(flavor = "multi_thread")]
async fn other_failure_not_rerun() {
    let a = common::side("rf-a", Duration::ZERO).await;
    let b = common::side_with("rf-b", |runs| flaky_run(runs, Arc::default(), true)).await;
    let (b_id, _) = common::pair(&a, &b).await;
    agents::add(&common::db(&b), "dev", ".", "").unwrap();
    let (conv, _) = a.chat.send(b_id, "dev", None, "첫 질문").await.unwrap();
    let a_db = common::db(&a);
    common::eventually(|| agent_messages(&a_db, conv.id).len() == 1).await;
    a.chat
        .send(b_id, "dev", Some(conv.id), "이어서")
        .await
        .unwrap();
    common::eventually(|| agent_messages(&a_db, conv.id).len() == 2).await;
    assert_eq!(agent_messages(&a_db, conv.id)[1].state, State::Error);
    assert_eq!(
        b.runs.lock().unwrap().len(),
        2,
        "다른 실패는 다시 실행하지 않는다"
    );
}
