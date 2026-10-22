//! 상위 한 단계 읽기 canary(#05-2) — 제품 인자(`claude_args`)로 실제 `claude` 가 바로 위 폴더만 더 읽고 그 위는 막는지 잰다.
//! 개발 PC 전용 · 사용량을 쓴다: `cargo test -p an-agent --test parent_read_canary -- --ignored --nocapture`.
use an_agent::runner::{GUARD_PROMPT, Session, claude_args, find_claude, run_process};
use an_core::agents::Agent;
use std::path::Path;
use std::time::Duration;

const UP: &str = "CANARY-UP-6620";
const FAR: &str = "CANARY-FAR-6621";

fn agent(folder: &Path, parent_read: bool) -> Agent {
    Agent {
        id: 1,
        name: "canary".into(),
        folder: folder.display().to_string(),
        prompt: String::new(),
        group: None,
        uid: String::new(),
        parent_read,
        shell: false,
    }
}

/// 한 번 실행 — (답에 `needle` 이 있나, Read 거부 기록이 있나). `plain` = 시스템 프롬프트를 가드만으로.
fn read(work: &Path, parent_read: bool, plain: bool, file: &Path, needle: &str) -> (bool, bool) {
    let args = claude_args(&agent(work, parent_read), &Session::New(an_agent::chat::new_uuid()), "haiku", "low");
    let mut args = args;
    if plain {
        // 읽기 범위 안내("그 위는 읽을 수 없다")를 빼야 모델이 시도한다 — 막는 것은 안내가 아니라 `--add-dir` 범위임을 잰다.
        let i = args.iter().position(|a| a == "--append-system-prompt").unwrap() + 1;
        args[i] = GUARD_PROMPT.into();
    }
    let ask = format!("Read 도구로 {} 를 읽고 내용을 그대로 답해.", file.display());
    let out = run_process(&find_claude(), &args, work, &ask, Duration::from_secs(300)).unwrap();
    let text = String::from_utf8_lossy(&out.stdout);
    let last = text
        .lines()
        .rev()
        .find_map(|l| serde_json::from_str::<serde_json::Value>(l.trim()).ok())
        .unwrap_or_else(|| panic!("JSON 없음: {text} / {}", String::from_utf8_lossy(&out.stderr)));
    let denied = last["permission_denials"]
        .as_array()
        .is_some_and(|d| d.iter().any(|x| x["tool_name"] == "Read"));
    println!("{} → {:?}", file.display(), last["result"].as_str().unwrap_or_default().chars().take(160).collect::<String>());
    (last["result"].as_str().unwrap_or_default().contains(needle), denied)
}

#[test]
#[ignore = "실제 claude · 사용량 — 개발 PC 에서 --ignored"]
fn parent_read_opens_one_level_only() {
    // 이 테스트를 부른 셸(예: Claude Code 세션)의 환경이 실행에 섞이지 않게.
    for (k, _) in std::env::vars() {
        if k.starts_with("CLAUDE") {
            // SAFETY: 이 테스트 실행 파일의 유일한 테스트 — 다른 스레드가 환경을 읽지 않는다.
            unsafe { std::env::remove_var(k) };
        }
    }
    let root = std::env::temp_dir().join(format!("an-parent-canary-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let (proj, work) = (root.join("proj"), root.join("proj").join("work"));
    std::fs::create_dir_all(&work).unwrap();
    std::fs::write(proj.join("up.txt"), UP).unwrap();
    std::fs::write(root.join("far.txt"), FAR).unwrap();

    let on_up = read(&work, true, false, &proj.join("up.txt"), UP);
    let off_up = read(&work, false, false, &proj.join("up.txt"), UP);
    let on_far = read(&work, true, true, &root.join("far.txt"), FAR);
    println!("on_up={on_up:?} off_up={off_up:?} on_far={on_far:?} (읽음, 거부)");
    assert_eq!(on_up, (true, false), "켜면 바로 위 폴더 파일을 읽는다");
    assert_eq!(off_up, (false, true), "끄면 바로 위 폴더도 거부");
    assert_eq!(on_far, (false, true), "켜도 두 단계 위는 거부");
    std::fs::remove_dir_all(&root).unwrap();
}
