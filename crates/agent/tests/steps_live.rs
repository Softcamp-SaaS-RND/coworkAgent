//! 처리경과 실물(#1c A10 · E9) — 실제 `claude` stream-json 실행에서 단계 요약이 실행 중에 오고, 결과 파싱은 그대로인가.
//! 개발 PC 전용 · 사용량을 쓴다: `cargo test -p an-agent --test steps_live -- --ignored --nocapture`.
use an_agent::runner::{RunInput, Session, StepSink, find_claude, run_claude};
use an_core::agents::Agent;
use std::sync::{Arc, Mutex};
use std::time::Instant;

#[test]
#[ignore = "실제 claude · 사용량 — 개발 PC 에서 --ignored"]
fn live_run_reports_steps_before_result() {
    // 이 테스트를 부른 셸(예: Claude Code 세션)의 환경이 실행에 섞이지 않게.
    for (k, _) in std::env::vars() {
        if k.starts_with("CLAUDE") {
            // SAFETY: 이 테스트 실행 파일의 유일한 테스트 — 다른 스레드가 환경을 읽지 않는다.
            unsafe { std::env::remove_var(k) };
        }
    }
    let work = std::env::temp_dir().join(format!("an-steps-live-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&work);
    std::fs::create_dir_all(&work).unwrap();
    std::fs::write(work.join("note.txt"), "코드명: 노을-42\n").unwrap();
    let steps: Arc<Mutex<Vec<(Instant, an_net::proto::StepDetail)>>> = Arc::default();
    let s = steps.clone();
    let input = RunInput {
        agent: Agent {
            id: 1,
            name: "live".into(),
            folder: work.display().to_string(),
            tools: "Read,Grep,Glob".into(),
            prompt: String::new(),
            group: None,
            uid: String::new(),
        },
        session: Session::New(an_agent::chat::new_uuid()),
        text: "note.txt 를 Read 도구로 읽고 코드명만 답하라.".into(),
        model: "haiku".into(),
        effort: "low".into(),
        steps: StepSink::new(move |d| s.lock().unwrap().push((Instant::now(), d.clone()))),
    };
    let out = run_claude(&find_claude(), &input).unwrap();
    let done = Instant::now();
    let steps = steps.lock().unwrap();
    eprintln!("steps: {:?}", steps.iter().map(|(_, d)| d).collect::<Vec<_>>());
    eprintln!("out: {out:?}");
    assert!(!out.is_error, "{out:?}");
    assert!(out.text.contains("노을-42"), "{out:?}");
    assert!(!out.session.is_empty());
    let (at, last) = steps.last().expect("단계 요약이 1번 이상");
    assert!(*at <= done && last.tools.contains_key("Read"), "{last:?}");
    assert!(last.turns >= 2, "도구 결과 뒤 턴: {last:?}");
}
