//! 컨텍스트 압축 실물(#1m E9) — 실제 `claude` 로 답 → 창 사용량 → 요약 실행 → 새 세션 첫 실행에 요약이 들어가 앞 대화 내용을 답하는가.
//! 개발 PC 전용 · 사용량을 쓴다: `cargo test -p an-agent --test context_live -- --ignored --nocapture`.
mod common;
use an_agent::runner::{Session, find_claude, run_claude};
use an_core::chat::{self, Author, State};
use std::sync::Arc;

#[tokio::test(flavor = "multi_thread")]
#[ignore = "실제 claude · 사용량 — 개발 PC 에서 --ignored"]
async fn live_compact_carries_summary_into_new_session() {
    // 이 테스트를 부른 셸(예: Claude Code 세션)의 환경이 실행에 섞이지 않게.
    for (k, _) in std::env::vars() {
        if k.starts_with("CLAUDE") {
            // SAFETY: 이 테스트 실행 파일의 유일한 테스트 — 실행기 스레드가 뜨기 전에 지운다.
            unsafe { std::env::remove_var(k) };
        }
    }
    let real =
        |runs: Arc<std::sync::Mutex<Vec<an_agent::runner::RunInput>>>| -> an_agent::chat::Run {
            Arc::new(move |input| {
                runs.lock().unwrap().push(input.clone());
                run_claude(&find_claude(), &input)
            })
        };
    let a = common::side_with("cx-live-a", real).await;
    let b = common::side_with("cx-live-b", real).await;
    let (b_id, _) = common::pair(&a, &b).await;
    let work = std::env::temp_dir().join(format!("an-cx-live-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&work);
    std::fs::create_dir_all(&work).unwrap();
    std::fs::write(work.join("note.txt"), "코드명: 노을-42\n").unwrap();
    an_core::agents::add(&common::db(&b), "live", &work.display().to_string(), "").unwrap();
    an_core::settings::set(&common::db(&b), an_core::run_opts::MODEL_KEY, "haiku").unwrap();
    an_core::settings::set(&common::db(&b), an_core::run_opts::EFFORT_KEY, "low").unwrap();

    let done = |cid: i64, n: usize| {
        chat::messages(&common::db(&a), cid)
            .unwrap()
            .iter()
            .filter(|m| m.author == Author::Agent && m.state != State::Sent)
            .count()
            >= n
    };
    let wait = |f: &dyn Fn() -> bool| {
        let until = std::time::Instant::now() + std::time::Duration::from_secs(300);
        while !f() {
            assert!(std::time::Instant::now() < until, "5분 안에 끝나지 않았다");
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
    };
    let (conv, _) = a
        .chat
        .send(
            b_id,
            "live",
            None,
            "note.txt 를 Read 도구로 읽고 코드명만 답하라.",
        )
        .await
        .unwrap();
    tokio::task::block_in_place(|| wait(&|| done(conv.id, 1)));
    let reply = chat::messages(&common::db(&a), conv.id)
        .unwrap()
        .pop()
        .unwrap();
    eprintln!("답: {:?} · 창 {:?}", reply.text, reply.ctx_tokens);
    assert!(reply.text.contains("노을-42"), "{}", reply.text);
    assert!(
        reply.ctx_tokens.is_some_and(|n| n > 1000),
        "{:?}",
        reply.ctx_tokens
    );

    a.chat.context(conv.id, "compact").unwrap();
    tokio::task::block_in_place(|| {
        wait(&|| {
            chat::messages(&common::db(&a), conv.id)
                .unwrap()
                .iter()
                .any(|m| m.author == Author::Context && m.state != State::Sent)
        })
    });
    let marker = chat::messages(&common::db(&a), conv.id)
        .unwrap()
        .into_iter()
        .find(|m| m.author == Author::Context)
        .unwrap();
    eprintln!("요약({:?}): {}", marker.state, marker.text);
    assert_eq!(marker.state, State::Done, "{:?}", marker.refused);

    a.chat
        .send(
            b_id,
            "live",
            Some(conv.id),
            "앞에서 확인한 코드명이 뭐였지? 파일을 다시 읽지 말고 기억으로만 답하라.",
        )
        .await
        .unwrap();
    tokio::task::block_in_place(|| wait(&|| done(conv.id, 2)));
    let last = chat::messages(&common::db(&a), conv.id)
        .unwrap()
        .pop()
        .unwrap();
    let run = b.runs.lock().unwrap().last().unwrap().clone();
    eprintln!("새 세션 답: {:?} · 창 {:?}", last.text, last.ctx_tokens);
    assert!(matches!(run.session, Session::New(_)), "{:?}", run.session);
    assert!(run.text.contains("<대화 요약>"));
    assert!(last.text.contains("노을-42"), "{}", last.text);
}
