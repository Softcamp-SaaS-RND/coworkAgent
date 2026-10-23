//! 대화의 마지막 실행 명령(#1z-10 결정 2) — 이 PC 가 실제로 띄운 실행의 실행 파일 + 인자(`runner::launch_line` — 띄울 때와 같은 함수)를 실행 직전에 남긴다.
//! 세션 다시 실행이면 그 다시 실행 것, 실패한 실행도 남는다. 원격 본문은 없다. 보낸 쪽(상대 PC 가 실행)에는 남지 않는다.
mod common;
use an_agent::runner::{RunInput, RunOutput, Session, SessionMissing, launch_line};
use an_core::agents;
use an_core::chat::{self, Author};
use std::io;
use std::sync::{Arc, Mutex};
use std::time::Duration;

fn replies(conn: &an_core::db::Connection, conv: i64) -> usize {
    chat::messages(conn, conv).unwrap().into_iter().filter(|m| m.author == Author::Agent).count()
}

fn sid(s: &Session) -> String {
    match s {
        Session::New(id) | Session::Resume(id) => id.clone(),
    }
}

/// 실행기 대역 — `gone` 세션을 재개하면 [`SessionMissing`], `fail` 에 든 본문이면 다른 실패, 그 밖은 답.
fn run(runs: Arc<Mutex<Vec<RunInput>>>, gone: Arc<Mutex<Vec<String>>>) -> an_agent::chat::Run {
    Arc::new(move |input: RunInput| {
        runs.lock().unwrap().push(input.clone());
        if let Session::Resume(s) = &input.session
            && gone.lock().unwrap().contains(s)
        {
            return Err(io::Error::other(SessionMissing(format!("No conversation found with session ID: {s}"))));
        }
        if common::body(&input.text) == "터져라" {
            return Err(io::Error::other("실행 파일 없음"));
        }
        Ok(RunOutput {
            text: format!("답: {}", common::body(&input.text)),
            session: sid(&input.session),
            is_error: false,
            turns: Some(1),
            duration_ms: Some(10),
            ctx_tokens: None,
        })
    })
}

#[tokio::test(flavor = "multi_thread")]
async fn run_records_launch_line() {
    let gone: Arc<Mutex<Vec<String>>> = Arc::default();
    let a = common::side("cmd-a", Duration::ZERO).await;
    let g = gone.clone();
    let b = common::side_with("cmd-b", move |runs| run(runs, g)).await;
    let (b_id, _) = common::pair(&a, &b).await;
    agents::add(&common::db(&b), "dev", ".", "추가 지시").unwrap();
    let (conv, _) = a.chat.send(b_id, "dev", None, "첫 질문 SECRET-1").await.unwrap();
    let a_db = common::db(&a);
    common::eventually(|| replies(&a_db, conv.id) == 1).await;
    let b_db = common::db(&b);
    let theirs = chat::list(&b_db).unwrap()[0].id;
    let cmd = |db: &an_core::db::Connection, id: i64| chat::info(db, id).unwrap().unwrap().cmd;
    let first = b.runs.lock().unwrap()[0].clone();
    let line = cmd(&b_db, theirs).expect("첫 실행 명령");
    assert_eq!(line, launch_line(&first), "띄운 입력의 실행 명령 그대로");
    assert!(line.contains("--session-id") && !line.contains("SECRET-1"), "새 세션 · 원격 본문 없음: {line}");
    assert!(chat::info(&b_db, theirs).unwrap().unwrap().cmd_at.is_some());
    assert_eq!(cmd(&a_db, conv.id), None, "보낸 쪽은 실행하지 않아 없다");

    // 세션이 사라졌다 → 새 세션으로 다시 실행 = 그 다시 실행의 명령.
    gone.lock().unwrap().push(sid(&first.session));
    a.chat.send(b_id, "dev", Some(conv.id), "이어서").await.unwrap();
    common::eventually(|| replies(&a_db, conv.id) == 2).await;
    let rerun = b.runs.lock().unwrap()[2].clone();
    assert!(matches!(rerun.session, Session::New(_)));
    assert_eq!(cmd(&b_db, theirs).unwrap(), launch_line(&rerun), "다시 실행한 명령");

    // 실패한 실행도 띄웠으면 남는다.
    a.chat.send(b_id, "dev", Some(conv.id), "터져라").await.unwrap();
    common::eventually(|| replies(&a_db, conv.id) == 3).await;
    let failed = b.runs.lock().unwrap()[3].clone();
    let line = cmd(&b_db, theirs).unwrap();
    assert_eq!(line, launch_line(&failed));
    assert!(line.contains("--resume"), "이어 쓰기 세션 인자 그대로: {line}");
}
