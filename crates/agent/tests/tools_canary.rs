//! 도구 이름 canary(#05-3) — 제품 인자(`claude_args`)로 실제 `claude` 가 알리는 도구 목록(`stream-json` 첫 `system` init 의 `tools`)이 넘긴 이름과 같은지 잰다.
//! claude 는 없는 도구 이름을 오류 없이 무시한다(2.1.308 실측 — `LS` · `NotebookRead`) — 이름이 사라지면 여기서 드러난다.
//! 개발 PC 전용 · 사용량을 쓴다: `cargo test -p an-agent --test tools_canary -- --ignored --nocapture`.
use an_agent::runner::{Session, claude_args, find_claude, run_process};
use an_core::agents::{Agent, READ_ONLY_TOOLS, SHELL_TOOLS};
use std::time::Duration;

fn tools(shell: bool) -> Vec<String> {
    let work = std::env::temp_dir();
    let agent = Agent {
        id: 1,
        name: "canary".into(),
        folder: work.display().to_string(),
        prompt: String::new(),
        group: None,
        uid: String::new(),
        parent_read: false,
        shell,
    };
    let args = claude_args(&agent, &Session::New(an_agent::chat::new_uuid()), "haiku", "low");
    let out = run_process(&find_claude(), &args, &work, "ok 라고만 답해.", Duration::from_secs(300)).unwrap();
    let text = String::from_utf8_lossy(&out.stdout);
    let init = text
        .lines()
        .filter_map(|l| serde_json::from_str::<serde_json::Value>(l.trim()).ok())
        .find(|v| v["type"] == "system" && v["subtype"] == "init")
        .unwrap_or_else(|| panic!("init 없음: {text} / {}", String::from_utf8_lossy(&out.stderr)));
    let mut got: Vec<String> = init["tools"].as_array().unwrap().iter().map(|t| t.as_str().unwrap().to_string()).collect();
    got.sort();
    println!("shell={shell} → {got:?}");
    got
}

#[test]
#[ignore = "실제 claude · 사용량 — 개발 PC 에서 --ignored"]
fn product_tools_exist() {
    // 이 테스트를 부른 셸(예: Claude Code 세션)의 환경이 실행에 섞이지 않게.
    for (k, _) in std::env::vars() {
        if k.starts_with("CLAUDE") {
            // SAFETY: 이 테스트 실행 파일의 유일한 테스트 — 다른 스레드가 환경을 읽지 않는다.
            unsafe { std::env::remove_var(k) };
        }
    }
    let want = |extra: &[&str]| {
        let mut w: Vec<String> = READ_ONLY_TOOLS.iter().chain(extra).map(|s| s.to_string()).collect();
        w.sort();
        w
    };
    assert_eq!(tools(false), want(&[]), "읽기 전용 도구 이름이 모두 실재");
    assert_eq!(tools(true), want(SHELL_TOOLS), "셸 도구 이름도 실재");
}
