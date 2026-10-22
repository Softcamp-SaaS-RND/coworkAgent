use an_agent::runner::{
    GUARD_PROMPT, MIN_CLAUDE, SHELL_GUARD_PROMPT, Session, parse_output, parse_version, run_process, shell_env,
    version_ok,
};
use an_core::agents::Agent;
use std::io::ErrorKind;
use std::time::{Duration, Instant};

fn agent(prompt: &str) -> Agent {
    Agent {
        id: 1,
        name: "review".into(),
        folder: "/work".into(),
        prompt: prompt.into(),
        group: None,
        uid: String::new(),
        parent_read: false,
        shell: false,
        color: "agent-tile".into(),
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
        &agent(""),
        &Session::New("11111111-2222-3333-4444-555555555555".into()),
    );
    for flag in ["-p", "--restricted", "--strict-mcp-config"] {
        assert!(args.iter().any(|a| a == flag), "{flag} 없음: {args:?}");
    }
    assert_eq!(value_of(&args, "--output-format"), Some("stream-json"));
    assert!(
        args.iter().any(|a| a == "--verbose"),
        "stream-json 은 --verbose 필요"
    );
    assert_eq!(value_of(&args, "--permission-prompts"), Some("none"));
    assert_eq!(value_of(&args, "--permission-mode"), Some("dontAsk"));
    assert_eq!(
        value_of(&args, "--tools"),
        Some("Read,Grep,Glob")
    );
    assert_eq!(
        value_of(&args, "--allowedTools"),
        Some("Read,Grep,Glob")
    );
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
    let args = claude_args(&agent(""), &Session::New("u".into()));
    assert_eq!(
        value_of(&args, "--append-system-prompt"),
        Some(GUARD_PROMPT)
    );
    for must in ["문의", "수정", "삭제", "명령", "비밀"] {
        assert!(GUARD_PROMPT.contains(must), "{must}");
    }
    // 프로필 지시는 그 뒤에.
    let args = claude_args(&agent("짧게 답해"), &Session::New("u".into()));
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
        let args = claude_args(&agent(""), &session);
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
fn tools_are_all_read_only() {
    // 프로필과 무관하게 탐색 도구 3개 — 체크 칸이 없다(#1d · #05-3 없는 이름 LS · NotebookRead 뺌).
    let args = claude_args(&agent("짧게"), &Session::New("u".into()));
    let all = "Read,Grep,Glob";
    assert_eq!(value_of(&args, "--tools"), Some(all));
    assert_eq!(value_of(&args, "--allowedTools"), Some(all));
}

#[test]
fn resume_uses_saved_session() {
    let args = claude_args(&agent(""), &Session::Resume("sess-9".into()));
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
fn parse_context_tokens() {
    // 2.1.299 실측 꼴 — `usage` 는 실행 합계, 창 사용량은 마지막 호출(iterations 끝)의 입력 + 캐시 + 출력.
    let multi = r#"{"type":"result","is_error":false,"result":"x","session_id":"s","usage":{"input_tokens":9,"cache_creation_input_tokens":900,"cache_read_input_tokens":90000,"output_tokens":99,"iterations":[{"input_tokens":2,"cache_creation_input_tokens":30000,"cache_read_input_tokens":0,"output_tokens":50},{"input_tokens":3,"cache_creation_input_tokens":400,"cache_read_input_tokens":30050,"output_tokens":20}]}}"#;
    assert_eq!(
        parse_output(multi.as_bytes()).unwrap().ctx_tokens,
        Some(3 + 400 + 30050 + 20)
    );
    let flat = r#"{"type":"result","is_error":false,"result":"x","session_id":"s","usage":{"input_tokens":2,"cache_creation_input_tokens":30379,"output_tokens":166}}"#;
    assert_eq!(
        parse_output(flat.as_bytes()).unwrap().ctx_tokens,
        Some(2 + 30379 + 166)
    );
    let none = r#"{"type":"result","is_error":false,"result":"x","session_id":"s"}"#;
    assert_eq!(parse_output(none.as_bytes()).unwrap().ctx_tokens, None);
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

/// 상위 한 단계 읽기(#05-2) — 켜면 `--add-dir <바로 위>` + 읽기 범위 안내, 끄면 · 막힌 위(드라이브 루트)면 둘 다 없다.
#[test]
fn parent_read_adds_dir() {
    let root = std::env::temp_dir().join(format!("an-parent-read-{}", std::process::id()));
    let work = root.join("work");
    std::fs::create_dir_all(&work).unwrap();
    let with = |folder: &std::path::Path, on: bool| Agent {
        folder: folder.display().to_string(),
        parent_read: on,
        ..agent("")
    };
    let args = claude_args(&with(&work, true), &Session::New("u".into()));
    assert_eq!(value_of(&args, "--add-dir"), Some(root.display().to_string().as_str()));
    let system = value_of(&args, "--append-system-prompt").unwrap();
    assert!(system.starts_with(GUARD_PROMPT) && system.contains("[읽기 범위]"), "{system}");
    for off in [with(&work, false), with(std::path::Path::new(if cfg!(windows) { r"C:\Windows" } else { "/usr" }), true)] {
        let args = claude_args(&off, &Session::New("u".into()));
        assert!(!args.iter().any(|a| a == "--add-dir"), "{args:?}");
        assert_eq!(value_of(&args, "--append-system-prompt"), Some(GUARD_PROMPT));
    }
    std::fs::remove_dir_all(&root).unwrap();
}

/// 셸 명령 실행 허용(#05-3) — 켜면 도구에 셸(Windows `Bash` · `PowerShell`, 그 밖 `Bash`) · 셸용 가드, 끄면 읽기 전용 5개 · 가드 그대로.
#[test]
fn shell_adds_tools() {
    let on = Agent { shell: true, ..agent("") };
    let args = claude_args(&on, &Session::New("u".into()));
    let want = if cfg!(windows) {
        "Read,Grep,Glob,Bash,PowerShell"
    } else {
        "Read,Grep,Glob,Bash"
    };
    assert_eq!(value_of(&args, "--tools"), Some(want));
    assert_eq!(value_of(&args, "--allowedTools"), Some(want));
    assert_eq!(value_of(&args, "--append-system-prompt"), Some(SHELL_GUARD_PROMPT));
    assert!(!SHELL_GUARD_PROMPT.contains("명령 · 스크립트 실행") && SHELL_GUARD_PROMPT.contains("workflow") && SHELL_GUARD_PROMPT.contains("삭제 · 편집 · 이동"));
    for flag in ["--restricted", "--strict-mcp-config", "dontAsk"] {
        assert!(args.iter().any(|a| a == flag), "{flag} 없음: {args:?}");
    }
    let off = claude_args(&agent(""), &Session::New("u".into()));
    assert_eq!(value_of(&off, "--tools"), Some("Read,Grep,Glob"));
    assert_eq!(value_of(&off, "--append-system-prompt"), Some(GUARD_PROMPT));
    // 상위 읽기와 함께 켜면 "그 위는 읽을 수 없다" 를 싣지 않는다(셸은 범위 밖도 읽는다).
    let root = std::env::temp_dir().join(format!("an-shell-scope-{}", std::process::id()));
    let work = root.join("work");
    std::fs::create_dir_all(&work).unwrap();
    let both = Agent { folder: work.display().to_string(), parent_read: true, ..on };
    let system = claude_args(&both, &Session::New("u".into()))
        .into_iter()
        .skip_while(|a| a != "--append-system-prompt")
        .nth(1)
        .unwrap();
    assert!(system.starts_with(SHELL_GUARD_PROMPT) && system.contains("[읽기 범위] 파일 도구는") && !system.contains("그 위는 읽을 수 없다"), "{system}");
    std::fs::remove_dir_all(&root).unwrap();
}

/// 셸 실행 환경(#05-3) — 기본 목록만 남고 키 · 토큰 변수는 빠진다. Windows 는 이름 대소문자 무시.
#[test]
fn shell_env_keeps_base_only() {
    let vars = [("Path", r"C:\bin"), ("HOME", "/h"), ("ANTHROPIC_API_KEY", "sk-x"), ("GITHUB_TOKEN", "t"), ("HTTPS_PROXY", "p"), ("CLAUDECODE", "1")]
        .map(|(k, v)| (std::ffi::OsString::from(k), std::ffi::OsString::from(v)));
    let kept: Vec<String> = shell_env(vars).into_iter().map(|(k, _)| k.to_string_lossy().into_owned()).collect();
    let want: &[&str] = if cfg!(windows) { &["Path", "HOME", "HTTPS_PROXY"] } else { &["HOME", "HTTPS_PROXY"] };
    assert_eq!(kept, want);
}
