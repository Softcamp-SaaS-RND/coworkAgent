# an_agent

## 개요

`an-agent`(`crates/agent`) — domain Layer. 에이전트 왕복: `claude -p` 실행기(`runner`)와 대화 조율(`chat`). 원격 메시지는 신뢰하지
않는 입력 — 텍스트는 표준 입력으로만, 권한은 프로필의 허용 도구만. 의존: `an-core`(프로필 · 대화 · 메시지 표) · `an-net`(노드).
Windows 는 `windows-sys`(Job Object — 자손 프로세스 종료, 프로세스 그룹과 같은 동작).

## 공개 계약

- `runner::claude_args(&Agent, &Session)` — `-p --output-format json --restricted --strict-mcp-config --permission-prompts none
  --permission-mode dontAsk --tools <t> --allowedTools <t> [--append-system-prompt <p>]` + `--session-id <uuid>`(`Session::New`) |
  `--resume <id>`(`Session::Resume`). `--bare` · `bypassPermissions` 없음(D20 · D22).
- `runner::GUARD_PROMPT`(D31 문의 응답 전용 — 개입 · 편집 · 삭제 · 명령 실행 금지, 비밀 싣지 않기) — `claude_args` 가 항상
  `--append-system-prompt`(가드 + `[수신자 추가 지시]` 프로필 지시) · `--system-prompt-snapshot off`(재개 세션에도 지금 가드) · 도구는
  `agents::read_only`. `MIN_CLAUDE`(2.1.267) · `parse_version` · `version_ok` — `run_claude` 는 낮은 버전이면 실행하지 않는다.
- `runner::parse_output(stdout) -> RunOutput{text, session, is_error, turns, duration_ms}` — 마지막 JSON 줄(`num_turns` · `duration_ms`
  는 없으면 `None`), `session_id` 없으면 오류.
- `runner::run_process(program, args, cwd, stdin, timeout)` — 표준 입력 쓰기 · 출력 읽기를 스레드로. 시간 초과면 `TimedOut`, 끝나거나
  죽인 뒤 `ProcessTree` 로 자손까지 종료(Windows kill-on-close Job Object ↔ 그 밖 프로세스 그룹). Windows 는 `CREATE_NO_WINDOW`.
- `runner::RUN_TIMEOUT`(30분, D24 개정 — 유일한 실행 한도) · `find_claude()`(`~/.local/bin/claude(.exe)`, 없으면 PATH) ·
  `run_claude(program, &RunInput)`(작업 폴더 = 프로필 폴더).
- `chat::Chat::new(conn, node, run: Run)` — `Run = Arc<dyn Fn(RunInput) -> io::Result<RunOutput>>`(운영 `run_claude`, 테스트 대역,
  `spawn_blocking` 에서 부름). `attach()`(노드 대화 요청 처리기) · `recover()`(시작 시, 런타임 안에서 — `queued` 실행 · `running` 은
  오류 답) · `send(peer_id, agent, Option<conv>, text) -> (Conversation, Message)` · `remote_agents(peer_id)` · `db()`(await 를 넘겨
  쥐지 않는다) · `new_uuid()`.
- 받기 규칙: 본문 1 ~ `MAX_TEXT`(16 KiB) · 등록된 에이전트(`unknown_agent`) · 대화 uuid 는 그 상대 소유 · 같은 메시지 uuid 는 한 번만.
  저장(`queued`)이 대기열 등록 — 대화별 작업이 id 순으로 `running` → 실행 → 세션 · 상태 · 답을 한 트랜잭션(실패 → 오류 답) →
  `reply`(실행 정보 · 원 요청 대응을 싣는다, 못 보내면 `undelivered`).
- `Chat::post(conv, text, to_agent) -> (Conversation, Message, 경고)` — 양쪽 사람 발화(D18 · D23): 보낸 대화의 실행 요청은 `send`, 그 밖은
  내 이력 + `post` 미러. 받은 쪽 실행 요청은 사람 글을 먼저 미러한 뒤 실행(미러 실패여도 실행 · 성공 + 경고), 답은 `post`(agent).
  받기 `post`: 그 상대 소유 대화만, 에이전트 답 · 실행 요청 글은 에이전트가 상대 쪽인 대화(`out`)만, 에이전트 답은 상대가 쓴 `sent`
  실행 요청에만. 실행 입력에 지난 실행 요청 뒤 사람 글을 `[대화 메모 — 실행 요청 아님]` · `[요청 — {발화자}]` 로 앞붙임.
- 보내기 규칙: 대화별 잠금으로 저장 · 전송 · 응답을 한 번에 하나(도착 순서 = 이력 순서). 상대 거절만 `error`, 전송 오류는 `sent` 유지 +
  "전달 확인 안 됨" 오류(상대가 받았을 수 있다 — 늦은 답을 받는다).
- 답 받기 규칙: `reply_to` 가 같은 대화의 `sent` 실행 요청(사람 글 · 자동 왕복의 내 에이전트 답)일 때만(아니면 `unknown`), 받으면 그 메시지 `done/error`.
- 자동 왕복(D24): `Chat::start_relay(peer_id, agent, my_agent, cap, text)`(상한 1 ~ `MAX_RELAY_CAP` 50, 기본 `DEFAULT_RELAY_CAP` 10) ·
  `Chat::relay(conv, on)`(연 쪽 멈춤 · 재개 — 카운트 0 · 받은 쪽 거절 해제, 받은 쪽은 멈춤만). 연 쪽이 구동: 성공한 상대 답 + 상한 전이면
  답 기록과 한 트랜잭션으로 실행 요청 등록 → 내 에이전트 실행(`relay_step`) → `message{by_agent}` 로 넘김. 상한 n = 양쪽 n 번씩 —
  n 번째 내 답은 `post`(agent, 실행 없음)로 상대 이력에 기록만. 재개는 넘기지 못한 내 답을 같은 uuid 로 다시 넘긴다(받은 쪽은 기록만
  받은 글을 실행 요청으로). 실패 답 · 상한 · 거절이면 멈춤,
  전송 오류는 `sent` 유지(늦은 답 수신). 사람이 쓰면 멈춤(연 쪽 끔 · 받은 쪽 거절 + `relay{on:false}` 알림 — 기다리지 않음). 받은 쪽은
  거절 중이면 에이전트발 요청에 `stopped`. 실행 입력에 `[요청 — 상대 에이전트({상대})]`.
- 이벤트: 받은 요청 · 받은 답 → `PeerEvent::Message{name, host, agent, preview, kind}`(알림 S9). 자동 왕복은 양쪽 모두 시작 1건
  (받은 쪽 = `relay` 표시 사람 요청 · 재개 알림, 연 쪽 = 그 요청에 온 첫 답) · 중간(`RelayStep`) · 저절로 멈춤 1건(`RelayEnd` — 상한 ·
  받은 쪽 자동 왕복 요청 실행 실패(받은 쪽은 `mark_relayed` 로 자동 왕복 대화를 기록) · 상대/내 실행 실패 · 거절 · 전달 확인 안 됨 · 상태 저장 실패 · 상대가 멈춤). 사람이 멈춘 것은 내지 않는다.
- 상한 기본값: `RELAY_CAP_KEY`(`relay_cap_default`) · `default_relay_cap(conn)`(없거나 범위 밖이면 10).

## 사용 예

```rust
let chat = Chat::new(db::open(&paths.db_file())?, node.clone(), Arc::new(|input| run_claude(&find_claude(), &input)));
chat.attach();
{
    let _rt = runtime.enter();
    chat.recover()?;
}
let (conv, msg) = chat.send(peer_id, "review", None, "이 폴더 구조 설명해 줘").await?; // 답은 나중에 이력에
```

## 금지 사항

- 원격 텍스트 · 에이전트 이름을 명령 인자 · 셸에 넣지 않는다(표준 입력만). 자동 테스트에서 실제 `claude -p` 를 부르지 않는다
  (사용자 구독 사용량).
- `Chat::db()` 잠금을 await 너머로 쥐지 않는다.
