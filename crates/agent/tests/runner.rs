use an_agent::runner::{
    GUARD_PROMPT, MIN_CLAUDE, Session, parse_output, parse_version, run_process, version_ok,
};
use an_core::agents::Agent;
use std::io::ErrorKind;
use std::time::{Duration, Instant};

fn agent(tools: &str, prompt: &str) -> Agent {
    Agent {
        id: 1,
        name: "review".into(),
        folder: "/work".into(),
        tools: tools.into(),
        prompt: prompt.into(),
        group: None,
        uid: String::new(),
    }
}

fn claude_args(agent: &Agent, session: &Session) -> Vec<String> {
    an_agent::runner::claude_args(agent, session, "sonnet", "high")
}

/// `flag` 바로 뒤 값.
fn value_of<'a>(args: &'a [String], flag: &str) -> Option<&'a str> {
    args.iter()
        .position(|a| a == flag)
        .map(|i| args[i + 1].as_str())
}

#[test]
fn args_follow_d22() {
    let args = claude_args(
        &agent("Read,Grep,Glob", ""),
        &Session::New("11111111-2222-3333-4444-555555555555".into()),
    );
    for flag in ["-p", "--restricted", "--strict-mcp-config"] {
        assert!(args.iter().any(|a| a == flag), "{flag} 없음: {args:?}");
    }
    assert_eq!(value_of(&args, "--output-format"), Some("stream-json"));
    assert!(args.iter().any(|a| a == "--verbose"), "stream-json 은 --verbose 필요");
    assert_eq!(value_of(&args, "--permission-prompts"), Some("none"));
    assert_eq!(value_of(&args, "--permission-mode"), Some("dontAsk"));
    assert_eq!(value_of(&args, "--tools"), Some("Read,Grep,Glob"));
    assert_eq!(value_of(&args, "--allowedTools"), Some("Read,Grep,Glob"));
    assert_eq!(
        value_of(&args, "--session-id"),
        Some("11111111-2222-3333-4444-555555555555")
    );
    assert!(!args.iter().any(|a| a == "--resume"));
    // 구독 OAuth 를 버리는 --bare · 전부 허용은 쓰지 않는다(D20 · D22).
    assert!(!args.iter().any(|a| a == "--bare"
        || a == "bypassPermissions"
        || a == "--dangerously-skip-permissions"));
}

#[test]
fn guard_prompt_always() {
    // 프로필 지시가 없어도 문의 응답 전용 지시는 항상(D31).
    let args = claude_args(&agent("Read", ""), &Session::New("u".into()));
    assert_eq!(
        value_of(&args, "--append-system-prompt"),
        Some(GUARD_PROMPT)
    );
    for must in ["문의", "수정", "삭제", "명령", "비밀"] {
        assert!(GUARD_PROMPT.contains(must), "{must}");
    }
    // 프로필 지시는 그 뒤에.
    let args = claude_args(&agent("Read", "짧게 답해"), &Session::New("u".into()));
    let system = value_of(&args, "--append-system-prompt").unwrap();
    assert!(system.starts_with(GUARD_PROMPT), "{system}");
    assert!(
        system.ends_with("[수신자 추가 지시]\n짧게 답해"),
        "{system}"
    );
    // 재개 때도 기록된 옛 프롬프트가 아니라 지금 가드로 — 스냅샷 끔.
    for session in [
        Session::New("u".into()),
        Session::Resume("old-session".into()),
    ] {
        let args = claude_args(&agent("Read", ""), &session);
        assert_eq!(
            value_of(&args, "--system-prompt-snapshot"),
            Some("off"),
            "{session:?}"
        );
        assert_eq!(
            value_of(&args, "--append-system-prompt"),
            Some(GUARD_PROMPT)
        );
    }
}

#[test]
fn tools_limited_to_read_only() {
    // 이전에 저장된 쓰기 도구는 실행 인자에서 빠진다.
    let args = claude_args(
        &agent("Read,Edit,Bash,Grep,Write", ""),
        &Session::New("u".into()),
    );
    assert_eq!(value_of(&args, "--tools"), Some("Read,Grep"));
    assert_eq!(value_of(&args, "--allowedTools"), Some("Read,Grep"));
    let args = claude_args(&agent("Bash", ""), &Session::New("u".into()));
    assert_eq!(
        value_of(&args, "--tools"),
        Some("Read,Grep,Glob"),
        "남는 것이 없으면 기본"
    );
}

#[test]
fn resume_uses_saved_session() {
    let args = claude_args(&agent("Read", ""), &Session::Resume("sess-9".into()));
    assert_eq!(value_of(&args, "--resume"), Some("sess-9"));
    assert!(!args.iter().any(|a| a == "--session-id"));
}

#[test]
fn parse_claude_json() {
    let ok = r#"{"type":"result","subtype":"success","is_error":false,"duration_ms":18400,"num_turns":3,"result":"안녕하세요","session_id":"abc-1","total_cost_usd":0.01}"#;
    let out = parse_output(ok.as_bytes()).unwrap();
    assert_eq!(
        (out.text.as_str(), out.session.as_str(), out.is_error),
        ("안녕하세요", "abc-1", false)
    );
    assert_eq!((out.turns, out.duration_ms), (Some(3), Some(18400)));
    let failed = br#"{"type":"result","subtype":"error_during_execution","is_error":true,"result":"","session_id":"abc-2"}"#;
    let failed = parse_output(failed).unwrap();
    assert!(failed.is_error);
    assert_eq!(
        (failed.turns, failed.duration_ms),
        (None, None),
        "실행 정보가 없으면 None"
    );
    // 앞에 다른 줄이 섞여도 마지막 JSON 줄을 읽는다.
    let noisy =
        b"warn: something\r\n{\"is_error\":false,\"result\":\"x\",\"session_id\":\"s\"}\r\n";
    assert_eq!(parse_output(noisy).unwrap().text, "x");
    assert!(parse_output(b"not json").is_err());
    assert!(
        parse_output(br#"{"result":"x"}"#).is_err(),
        "session_id 없으면 오류"
    );
}

#[test]
fn timeout_kills_process() {
    let dir = std::env::temp_dir();
    // 표준 입력 전달 · 표준 출력 수집 — 원격 텍스트(따옴표 · 개행 · 한글)가 인자를 거치지 않고 그대로 들어간다.
    #[cfg(windows)]
    let (prog, args) = ("findstr", vec!["^".to_string()]);
    #[cfg(not(windows))]
    let (prog, args) = ("cat", Vec::<String>::new());
    let input = "a \"; rm -rf / \"\nline2 $(whoami)\n";
    let out = run_process(prog.as_ref(), &args, &dir, input, Duration::from_secs(10)).unwrap();
    let text = String::from_utf8_lossy(&out.stdout).replace("\r\n", "\n");
    assert_eq!(text, input, "표준 입력이 그대로 전달돼야 한다");
    // 오래 도는 프로세스는 시간 제한에서 죽인다.
    #[cfg(windows)]
    let (prog, args) = (
        "ping",
        vec!["-n".to_string(), "30".into(), "127.0.0.1".into()],
    );
    #[cfg(not(windows))]
    let (prog, args) = ("sleep", vec!["30".to_string()]);
    let started = Instant::now();
    let err = run_process(prog.as_ref(), &args, &dir, "", Duration::from_millis(800)).unwrap_err();
    assert_eq!(err.kind(), ErrorKind::TimedOut);
    assert!(
        started.elapsed() < Duration::from_secs(5),
        "{:?}",
        started.elapsed()
    );
}

#[test]
fn process_tree_is_killed() {
    let dir = std::env::temp_dir();
    // 자식(셸)이 띄운 손자가 출력 파이프를 쥐고 30초 돈다 — 손자까지 죽여야 읽기가 끝나 바로 돌아온다.
    #[cfg(windows)]
    let (prog, timed, lingering) = (
        "cmd",
        vec!["/c".to_string(), "ping -n 30 127.0.0.1".into()],
        vec!["/c".to_string(), "start /b ping -n 30 127.0.0.1".into()],
    );
    #[cfg(not(windows))]
    let (prog, timed, lingering) = (
        "sh",
        vec!["-c".to_string(), "sleep 30; true".into()],
        vec!["-c".to_string(), "sleep 30 &".into()],
    );
    // 시간 초과: 자식 · 손자 모두 종료.
    let started = Instant::now();
    let err = run_process(prog.as_ref(), &timed, &dir, "", Duration::from_millis(800)).unwrap_err();
    assert_eq!(err.kind(), ErrorKind::TimedOut);
    assert!(
        started.elapsed() < Duration::from_secs(5),
        "{:?}",
        started.elapsed()
    );
    // 정상 종료: 자식은 바로 끝나도 남은 손자를 정리한다.
    let started = Instant::now();
    run_process(prog.as_ref(), &lingering, &dir, "", Duration::from_secs(20)).unwrap();
    assert!(
        started.elapsed() < Duration::from_secs(5),
        "{:?}",
        started.elapsed()
    );
}

#[test]
fn claude_version_gate() {
    assert_eq!(parse_version("2.1.296 (Claude Code)\n"), Some((2, 1, 296)));
    assert_eq!(parse_version("garbage"), None);
    assert_eq!(parse_version(""), None);
    // 경계 — `--system-prompt-snapshot` 이 없는 버전에서는 돌리지 않는다.
    assert_eq!(MIN_CLAUDE, (2, 1, 267));
    assert!(!version_ok((2, 1, 266)));
    assert!(version_ok((2, 1, 267)));
    assert!(version_ok((2, 2, 0)));
    assert!(!version_ok((1, 9, 999)));
}
