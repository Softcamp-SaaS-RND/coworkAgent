//! 셸 명령 실행 허용 canary(#05-3) — 제품 실행 경로(`run_claude`)로 실제 `claude` 가 켜면 셸을 쓰고(폴더 밖 파일도 읽는다) 앱 환경 변수는 셸에 보이지 않는지,
//! 끄면 셸이 없는지 잰다. 개발 PC 전용 · 사용량을 쓴다: `cargo test -p an-agent --test shell_canary -- --ignored --nocapture`.
use an_agent::runner::{RunInput, Session, StepSink, find_claude, run_claude};
use an_core::agents::Agent;
use std::path::Path;

const UP: &str = "CANARY-SHELL-UP-7731";
const LEAK: &str = "CANARY-SHELL-LEAK-7732";
/// 대조군 — 넘기는 목록(`NO_PROXY`)에 든 변수는 셸에 보여야 한다(모델이 값을 숨겨서 통과하는 것을 가린다).
const KEEP: &str = "canary-keep-7733.invalid";

fn ask(work: &Path, shell: bool) -> String {
    let input = RunInput {
        agent: Agent {
            id: 1,
            name: "canary".into(),
            folder: work.display().to_string(),
            prompt: String::new(),
            group: None,
            uid: String::new(),
            parent_read: false,
            shell,
            color: "agent-tile".into(),
            engine: "claude".into(),
            skip_git: true,
        },
        session: Session::New(an_agent::chat::new_uuid()),
        text: "셸 도구(Bash 또는 PowerShell)로 다음 두 명령을 실행하고 출력을 그대로 답해. 셸 도구가 없으면 NO_SHELL 이라고만 답해.\n\
               1) cat ../up.txt\n2) echo \"[$AN_CANARY_MARK][$NO_PROXY]\""
            .into(),
        model: "haiku".into(),
        effort: "low".into(),
        steps: StepSink::default(),
    };
    let out = run_claude(&find_claude(), &input).unwrap();
    println!("shell={shell} → {:?}", out.text.chars().take(240).collect::<String>());
    out.text
}

#[test]
#[ignore = "실제 claude · 사용량 — 개발 PC 에서 --ignored"]
fn shell_runs_with_clean_env() {
    // 이 테스트를 부른 셸(예: Claude Code 세션)의 환경이 실행에 섞이지 않게 + 앱 환경에 비밀을 심는다.
    // SAFETY: 이 테스트 실행 파일의 유일한 테스트 — 다른 스레드가 환경을 읽지 않는다.
    unsafe {
        for (k, _) in std::env::vars_os() {
            if k.to_string_lossy().starts_with("CLAUDE") {
                std::env::remove_var(k);
            }
        }
        std::env::set_var("AN_CANARY_MARK", LEAK);
        std::env::set_var("NO_PROXY", KEEP);
    }
    let root = std::env::temp_dir().join(format!("an-shell-canary-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let work = root.join("work");
    std::fs::create_dir_all(&work).unwrap();
    std::fs::write(root.join("up.txt"), UP).unwrap();

    // #1z-9 결정 9 — 파일 도구도 폴더 밖을 읽어(제한 모드 없음) "끄면 폴더 밖 못 읽음" 대조는 없앴다. 셸 끔 = 셸 도구 없음은 `tools_canary`.
    let on = ask(&work, true);
    assert!(on.contains(UP), "켜면 셸로 폴더 밖 파일을 읽는다: {on}");
    assert!(on.contains(KEEP), "넘기는 목록의 변수는 보인다(대조군): {on}");
    assert!(!on.contains(LEAK), "목록 밖 앱 환경 변수는 셸에 보이지 않는다: {on}");
    std::fs::remove_dir_all(&root).unwrap();
}
