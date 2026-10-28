use an_agent::runner::{
    CODEX_GUARD_PROMPT, GUARD_PROMPT, MIN_CLAUDE, MIN_CODEX, SHELL_GUARD_PROMPT, Session, codex_args, codex_session_gone,
    count_codex_step, parse_codex, parse_codex_version, parse_output, parse_version, run_process, shell_env, version_ok,
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
        engine: "claude".into(),
        skip_git: true,
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
    assert!(args.iter().any(|a| a == "-p"), "{args:?}");
    // #1z-9 결정 9 — 로컬 사용과 맞춤: 제한 모드 · MCP 설정 무시 · 도구 목록 고정 없음.
    for flag in ["--restricted", "--strict-mcp-config", "--tools"] {
        assert!(!args.iter().any(|a| a == flag), "{flag} 있음: {args:?}");
    }
    assert_eq!(value_of(&args, "--output-format"), Some("stream-json"));
    assert!(
        args.iter().any(|a| a == "--verbose"),
        "stream-json 은 --verbose 필요"
    );
    assert_eq!(value_of(&args, "--permission-prompts"), Some("none"));
    assert_eq!(value_of(&args, "--permission-mode"), Some("dontAsk"));
    assert_eq!(
        value_of(&args, "--allowedTools"),
        Some("Read,Grep,Glob,WebFetch,WebSearch")
    );
    let denied = if cfg!(windows) { "Edit,Write,NotebookEdit,Bash,PowerShell" } else { "Edit,Write,NotebookEdit,Bash" };
    assert_eq!(value_of(&args, "--disallowedTools"), Some(denied));
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
    // 프로필 지시는 가드 앞에(맨 앞 — 잘 따르게), 가드는 그대로 뒤에.
    let args = claude_args(&agent("짧게 답해"), &Session::New("u".into()));
    let system = value_of(&args, "--append-system-prompt").unwrap();
    assert!(system.starts_with("[수신자 추가 지시]\n짧게 답해\n\n"), "{system}");
    assert!(system.ends_with(GUARD_PROMPT), "{system}");
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
    // 미리 허용 = 탐색 · 웹 읽기, 편집 도구는 늘 막음(#1z-9 결정 9 — 그 밖은 사용자 settings 허용 규칙). 체크 칸이 없다(#1d).
    let args = claude_args(&agent("짧게"), &Session::New("u".into()));
    assert_eq!(value_of(&args, "--allowedTools"), Some("Read,Grep,Glob,WebFetch,WebSearch"));
    let denied = value_of(&args, "--disallowedTools").unwrap();
    assert!(denied.starts_with("Edit,Write,NotebookEdit"), "{denied}");
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
        "Read,Grep,Glob,WebFetch,WebSearch,Bash,PowerShell"
    } else {
        "Read,Grep,Glob,WebFetch,WebSearch,Bash"
    };
    assert_eq!(value_of(&args, "--allowedTools"), Some(want));
    assert_eq!(value_of(&args, "--disallowedTools"), Some("Edit,Write,NotebookEdit"), "셸 켬 = 셸은 막지 않음");
    assert_eq!(value_of(&args, "--append-system-prompt"), Some(SHELL_GUARD_PROMPT));
    assert!(!SHELL_GUARD_PROMPT.contains("명령 · 스크립트 실행") && SHELL_GUARD_PROMPT.contains("workflow") && SHELL_GUARD_PROMPT.contains("삭제 · 편집 · 이동"));
    assert!(args.iter().any(|a| a == "dontAsk"), "{args:?}");
    let off = claude_args(&agent(""), &Session::New("u".into()));
    assert_eq!(value_of(&off, "--allowedTools"), Some("Read,Grep,Glob,WebFetch,WebSearch"));
    assert!(value_of(&off, "--disallowedTools").unwrap().ends_with("Bash") || value_of(&off, "--disallowedTools").unwrap().ends_with("PowerShell"), "셸 끔 = 셸 막음");
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

/// Codex 실행 인자(#1z-9 결정 4 · 5 · 8 · 2) — 새 세션 `-C <폴더> -s read-only`, 이어 쓰기 `resume … <thread>`(샌드박스는 설정), Git 스위치, 개발자 지시(TOML 문자열),
/// 모델 · effort · 사용자 설정 무시 인자 없음, 프롬프트는 표준 입력.
#[test]
fn codex_args_follow_decisions() {
    let cx = Agent { engine: "codex".into(), folder: r"D:\work\p".into(), ..agent("짧게 \"답\"") };
    let new = codex_args(&cx, &Session::New("ignored".into()));
    assert_eq!(&new[..6], ["exec", "-C", r"D:\work\p", "-s", "read-only", "--json"], "{new:?}");
    assert!(new.contains(&"--skip-git-repo-check".to_string()) && new.last().unwrap() == "-", "{new:?}");
    for no in ["--ignore-user-config", "-m", "--model", "resume", "ignored"] {
        assert!(!new.iter().any(|a| a == no), "{no} 있음: {new:?}");
    }
    let dev = value_of(&new, "-c").unwrap();
    let toml = dev.strip_prefix("developer_instructions=").unwrap();
    let text: String = serde_json::from_str(toml).unwrap();
    assert!(text.starts_with("[수신자 추가 지시]\n짧게 \"답\"\n\n") && text.contains(CODEX_GUARD_PROMPT) && text.ends_with(r"[읽기 범위] 이 폴더 D:\work\p 아래만 읽는다. 그 밖의 파일은 읽지 않는다."), "{text}");
    for must in ["문의", "삭제 · 편집 · 이동", "비밀"] {
        assert!(CODEX_GUARD_PROMPT.contains(must), "{must}");
    }
    let resume = codex_args(&Agent { skip_git: false, ..cx }, &Session::Resume("t-1".into()));
    assert_eq!(&resume[..4], ["exec", "resume", "-c", "sandbox_mode=\"read-only\""], "{resume:?}");
    assert_eq!(&resume[resume.len() - 2..], ["t-1", "-"], "{resume:?}");
    assert!(!resume.iter().any(|a| a == "--skip-git-repo-check" || a == "-C" || a == "-s"), "Git 끔 · resume 은 -C · -s 없음: {resume:?}");
}

/// `codex exec --json` 출력(0.163.0 실측 모양) — thread = 세션, 마지막 agent_message = 답, 실패 이벤트 = 오류, 사용량은 싣지 않음.
#[test]
fn parse_codex_json() {
    let ok = r#"{"type":"thread.started","thread_id":"01a1-t"}
{"type":"turn.started"}
{"type":"item.completed","item":{"id":"i0","type":"agent_message","text":"먼저 볼게요"}}
{"type":"item.completed","item":{"id":"i1","type":"agent_message","text":"ZEBRA 답"}}
{"type":"turn.completed","usage":{"input_tokens":27303,"cached_input_tokens":13440,"output_tokens":74}}
"#;
    let out = parse_codex(ok.as_bytes()).unwrap();
    assert_eq!((out.session.as_str(), out.text.as_str(), out.is_error, out.ctx_tokens), ("01a1-t", "ZEBRA 답", false, None));
    let failed = r#"{"type":"thread.started","thread_id":"t2"}
{"type":"turn.failed","error":{"message":"rate limited"}}
"#;
    let out = parse_codex(failed.as_bytes()).unwrap();
    assert_eq!((out.text.as_str(), out.is_error), ("rate limited", true));
    let silent = r#"{"type":"thread.started","thread_id":"t3"}
"#;
    assert!(parse_codex(silent.as_bytes()).unwrap().is_error, "답 없음 = 오류");
    // R1: 중간 답만 남고 끊김(turn.completed 없음) = 실패 — 상대에 성공 답 · 자동 왕복 · 압축 요약으로 가지 않게.
    let cut = r#"{"type":"thread.started","thread_id":"t4"}
{"type":"item.completed","item":{"id":"i0","type":"agent_message","text":"먼저 볼게요"}}
"#;
    let out = parse_codex(cut.as_bytes()).unwrap();
    assert!(out.is_error && out.text.contains("turn.completed 없음"), "{out:?}");
    assert_eq!(parse_codex(b"plain text").unwrap_err().kind(), ErrorKind::InvalidData);
}

/// 단계 요약(#1c 꼴) — 끝난 명령 · MCP · 웹 검색만 센다, 답 · 시작 이벤트는 아님.
#[test]
fn count_codex_steps() {
    let mut d = an_net::proto::StepDetail { turns: 1, ..Default::default() };
    let ev = |t: &str| serde_json::json!({ "type": "item.completed", "item": { "type": t, "command": "type secret.txt", "aggregated_output": "x" } });
    assert!(count_codex_step(&mut d, &ev("command_execution")));
    assert!(count_codex_step(&mut d, &ev("command_execution")));
    assert!(count_codex_step(&mut d, &ev("mcp_tool_call")));
    assert!(!count_codex_step(&mut d, &ev("agent_message")));
    assert!(!count_codex_step(&mut d, &serde_json::json!({ "type": "item.started", "item": { "type": "command_execution" } })));
    assert_eq!((d.turns, d.tools.get("Shell"), d.tools.get("MCP")), (4, Some(&2), Some(&1)));
}

/// 없는 thread 이어 쓰기(0.163.0 실측 stderr) · 버전 출력.
#[test]
fn codex_session_and_version() {
    let err = b"Reading additional input from stdin...\nError: thread/resume: thread/resume failed: no rollout found for thread id 1111 (code -32600)\n";
    assert!(codex_session_gone(err).unwrap().contains("no rollout found"));
    assert_eq!(codex_session_gone(b"Error: rate limited"), None);
    assert_eq!(parse_codex_version("codex-cli 0.163.0\n"), Some((0, 163, 0)));
    assert_eq!(parse_codex_version("2.1.309 (Claude Code)"), None);
    assert!(parse_codex_version("codex-cli 0.163.0").unwrap() >= MIN_CODEX);
}

/// R1: 비정상 종료(종료 코드 ≠ 0)면 완료 이벤트 · 답이 있어도 실패, 중간 답만 남고 끊겨도 실패 — 가짜 codex(버전 · JSON 줄 · 종료 코드).
#[test]
fn codex_abnormal_exit_is_error() {
    use an_agent::runner::{RunInput, StepSink, run_codex};
    let dir = std::env::temp_dir().join(format!("an-fake-codex-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let fake = |name: &str, body: &str, code: i32| {
        if cfg!(windows) {
            let p = dir.join(format!("{name}.cmd"));
            let lines: String = body.lines().map(|l| format!("echo {l}\r\n")).collect();
            std::fs::write(&p, format!("@echo off\r\nif \"%1\"==\"--version\" (echo codex-cli 0.163.0& exit /b 0)\r\n{lines}exit /b {code}\r\n")).unwrap();
            p
        } else {
            let p = dir.join(name);
            let lines: String = body.lines().map(|l| format!("echo '{l}'\n")).collect();
            std::fs::write(&p, format!("#!/bin/sh\n[ \"$1\" = --version ] && echo codex-cli 0.163.0 && exit 0\ncat >/dev/null\n{lines}exit {code}\n")).unwrap();
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o755)).unwrap();
            }
            p
        }
    };
    let input = RunInput {
        agent: Agent { engine: "codex".into(), folder: dir.display().to_string(), ..agent("") },
        session: Session::New("x".into()),
        text: "q".into(),
        model: String::new(),
        effort: String::new(),
        steps: StepSink::default(),
    };
    let done = r#"{"type":"thread.started","thread_id":"t1"}
{"type":"item.completed","item":{"type":"agent_message","text":"ok"}}
{"type":"turn.completed","usage":{}}"#;
    let out = run_codex(&fake("good", done, 0), &input).unwrap();
    assert!(!out.is_error && out.text == "ok" && out.session == "t1", "{out:?}");
    let out = run_codex(&fake("crash", done, 3), &input).unwrap();
    assert!(out.is_error && out.text.contains("비정상 종료"), "종료 코드 3 = 실패: {out:?}");
    let cut = r#"{"type":"thread.started","thread_id":"t2"}
{"type":"item.completed","item":{"type":"agent_message","text":"half"}}"#;
    let out = run_codex(&fake("cut", cut, 0), &input).unwrap();
    assert!(out.is_error, "완료 이벤트 없음 = 실패: {out:?}");
    std::fs::remove_dir_all(&dir).unwrap();
}

/// 실행 명령(#1z-10) — 인자는 띄울 때와 같은 함수, 따옴표는 OS 셸(Windows = PowerShell · 그 밖 POSIX), 원격 본문(표준 입력)은 없다.
#[test]
fn launch_line_matches_spawn_args() {
    use an_agent::runner::{RunInput, agent_args, launch_line, shell_line};
    let input = |a: Agent| RunInput {
        agent: a,
        session: Session::Resume("s-1".into()),
        text: "원격 본문 SECRET-TEXT".into(),
        model: "sonnet".into(),
        effort: "high".into(),
        steps: Default::default(),
    };
    let cl = input(agent("추가"));
    assert_eq!(agent_args(&cl), claude_args(&cl.agent, &cl.session), "Claude = claude_args 그대로");
    let mut cx = agent("");
    cx.engine = "codex".into();
    let cx = input(cx);
    assert_eq!(agent_args(&cx), codex_args(&cx.agent, &cx.session), "Codex = codex_args 그대로");
    let line = launch_line(&cl);
    assert!(line.contains("--resume") && line.contains("s-1"), "{line}");
    assert!(!line.contains("SECRET-TEXT"), "원격 본문은 넣지 않는다");
    assert!(launch_line(&cx).contains("resume"));

    let args: Vec<String> = ["-p", "a b", "it's", r#"say "hi""#, "줄1\n줄2", "", "k=v,x", r"C:\a/b.c:d=e"].map(String::from).to_vec();
    assert_eq!(
        shell_line(std::path::Path::new(r"C:\Program Files\claude.exe"), &args, true),
        "& 'C:\\Program Files\\claude.exe' -p 'a b' 'it''s' 'say \"hi\"' '줄1\n줄2' '' 'k=v,x' C:\\a/b.c:d=e",
        "PowerShell — 확인 줄 없음 · 작은따옴표 · 안의 ' 는 '' · 쉼표(배열)는 따옴표 · 역슬래시는 그대로"
    );
    // R1 — 둥근 따옴표도 PowerShell 작은따옴표라 두 번(인용 탈출 → 명령 주입 막음).
    let smart: Vec<String> = vec!["don\u{2019}t".into(), "p\u{2018}; Write-Output X; #".into()];
    assert!(
        shell_line(std::path::Path::new(r"C:\x’y\c.exe"), &smart, true)
            .ends_with("& 'C:\\x’’y\\c.exe' 'don’’t' 'p‘‘; Write-Output X; #'"),
        "{}",
        shell_line(std::path::Path::new(r"C:\x’y\c.exe"), &smart, true)
    );
    assert_eq!(
        shell_line(std::path::Path::new("/usr/bin/claude"), &args, false),
        "'/usr/bin/claude' -p 'a b' 'it'\\''s' 'say \"hi\"' '줄1\n줄2' '' 'k=v,x' 'C:\\a/b.c:d=e'",
        "POSIX — 안의 ' 는 '\\'' · 역슬래시는 따옴표"
    );
}

/// R1 — 복사한 명령을 실제 PowerShell 7 에서 돌리면 실행 파일이 받는 인자가 원래 인자와 같다(argv 왕복 — 둥근 따옴표 주입 · 큰따옴표 · 빈 인자 · 줄바꿈 · `$` `@` `%`),
/// Windows PowerShell 5.1 도 인용을 빠져나가지 않는다(인자 손실은 사용자 몫). 실행 파일 대역 = node(인자를 JSON 으로 찍음). pwsh · node 가 없으면 건너뜀.
#[cfg(windows)]
#[test]
fn powershell_line_roundtrip() {
    use an_agent::runner::shell_line;
    use std::path::{Path, PathBuf};
    use std::process::Command;
    let which = |n: &str| {
        Command::new("where.exe")
            .arg(n)
            .output()
            .ok()
            .filter(|o| o.status.success())
            .and_then(|o| String::from_utf8_lossy(&o.stdout).lines().next().map(|l| PathBuf::from(l.trim())))
    };
    let (Some(pwsh), Some(node)) = (which("pwsh"), which("node")) else {
        eprintln!("pwsh · node 없음 — 건너뜀");
        return;
    };
    let code = r#"console.log(JSON.stringify(process.argv.slice(1)).replace(/[\u007f-￿]/g, (c) => '\\u' + c.charCodeAt(0).toString(16).padStart(4, '0')))"#;
    let args: Vec<String> = vec![
        "-e".into(),
        code.into(),
        "it's".into(),
        "don\u{2019}t".into(),
        "p\u{2019}; Write-Output INJECTED; #".into(),
        r#"sandbox_mode="read-only""#.into(),
        String::new(),
        "줄1\n줄2 \"q\"".into(),
        "a,b".into(),
        "$HOME @x %PATH%".into(),
    ];
    let dir = std::env::temp_dir().join(format!("an-psline-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let script = dir.join("line.ps1");
    let mut bytes = vec![0xEF, 0xBB, 0xBF];
    bytes.extend(shell_line(&node, &args, true).as_bytes());
    std::fs::write(&script, bytes).unwrap();
    let run = |exe: &Path| {
        let o = Command::new(exe)
            .args(["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-File"])
            .arg(&script)
            .output()
            .unwrap();
        (o.status.success(), String::from_utf8_lossy(&o.stdout).into_owned(), String::from_utf8_lossy(&o.stderr).into_owned())
    };
    let (ok, out, err) = run(&pwsh);
    assert!(ok, "pwsh 실패: {err}");
    assert!(!out.lines().any(|l| l.trim() == "INJECTED"), "인용 탈출 없음(따로 실행된 명령 출력 없음): {out}");
    let line = out.lines().rev().find(|l| l.starts_with('[')).unwrap_or_else(|| panic!("node 출력 없음: {out} {err}"));
    let got: Vec<String> = serde_json::from_str(line).unwrap();
    assert_eq!(got, args[2..].to_vec(), "PowerShell 7 = 인자 그대로");
    // 인자 전달 Standard 도 그대로. 5.1 · Legacy 는 인자를 잃지만 확인 줄 없이 사용자 몫(2026-10-24 사용자 결정) — 인용 탈출만 없는지 본다.
    let o = Command::new(&pwsh)
        .args(["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command"])
        .arg(format!("$PSNativeCommandArgumentPassing = 'Standard'; & '{}'", script.display()))
        .output()
        .unwrap();
    let out = String::from_utf8_lossy(&o.stdout);
    let line = out.lines().rev().find(|l| l.starts_with('[')).unwrap_or_else(|| panic!("Standard: node 출력 없음 {out}"));
    assert_eq!(serde_json::from_str::<Vec<String>>(line).unwrap(), args[2..].to_vec(), "Standard = 인자 그대로");
    if let Some(ps) = which("powershell") {
        let (_, out, _) = run(&ps);
        assert!(!out.lines().any(|l| l.trim() == "INJECTED"), "5.1 도 인용 탈출 없음: {out}");
    }
    let _ = std::fs::remove_dir_all(&dir);
}
