//! Codex 실물(#1z-9 A4 · E8) — 실제 `codex exec` 로 새 세션 답 · 단계 요약 · 이어 쓰기 기억 · 쓰기 막힘을 잰다.
//! 개발 PC 전용 · 사용량을 쓴다: `cargo test -p an-agent --test codex_live -- --ignored --nocapture`.
use an_agent::runner::{RunInput, Session, StepSink, find_codex, run_codex};
use an_core::agents::Agent;
use std::sync::{Arc, Mutex};

#[test]
#[ignore = "실제 codex · 사용량 — 개발 PC 에서 --ignored"]
fn live_codex_answers_and_resumes() {
    let work = std::env::temp_dir().join(format!("an-codex-live-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&work);
    std::fs::create_dir_all(&work).unwrap();
    std::fs::write(work.join("note.txt"), "코드명: 노을-42\n").unwrap();
    let steps: Arc<Mutex<Vec<an_net::proto::StepDetail>>> = Arc::default();
    let s = steps.clone();
    let agent = Agent {
        id: 1,
        name: "live".into(),
        folder: work.display().to_string(),
        prompt: String::new(),
        group: None,
        uid: String::new(),
        parent_read: false,
        shell: false,
        color: "agent-tile".into(),
        engine: "codex".into(),
        skip_git: true,
    };
    let first = RunInput {
        agent: agent.clone(),
        session: Session::New("ignored".into()),
        text: "note.txt 를 읽고 코드명만 답하라. 그다음 이 폴더에 w.txt 를 만들어 보고, 만들었는지 한 줄 덧붙여라.".into(),
        model: String::new(),
        effort: String::new(),
        steps: StepSink::new(move |d| s.lock().unwrap().push(d.clone())),
    };
    let out = run_codex(&find_codex(), &first).unwrap();
    println!("1st: {out:?}\nsteps: {:?}", steps.lock().unwrap());
    assert!(!out.is_error && out.text.contains("노을-42"), "{out:?}");
    assert!(!work.join("w.txt").exists(), "읽기 전용 샌드박스 — 쓰기 막힘");
    assert!(!steps.lock().unwrap().is_empty(), "명령 단계 요약이 왔다");
    let again = RunInput {
        session: Session::Resume(out.session.clone()),
        text: "앞에서 답한 코드명을 다시 말하라. 파일은 읽지 마라.".into(),
        steps: StepSink::default(),
        ..first
    };
    let out2 = run_codex(&find_codex(), &again).unwrap();
    println!("2nd: {out2:?}");
    assert!(!out2.is_error && out2.text.contains("노을-42") && out2.session == out.session, "{out2:?}");
    let gone = RunInput { session: Session::Resume("11111111-2222-3333-4444-555555555555".into()), ..again };
    let e = run_codex(&find_codex(), &gone).unwrap_err();
    assert!(an_agent::runner::session_missing(&e), "{e}");
    let _ = std::fs::remove_dir_all(&work);
}
