# an_agent

## 개요

`an-agent`(`crates/agent`) — domain Layer. 에이전트 왕복: `claude -p` 실행기(`runner`)와 대화 조율(`chat`). 원격 메시지는 신뢰하지
않는 입력 — 텍스트는 표준 입력으로만, 도구는 읽기 전용 5개(`READ_ONLY_TOOLS` — #1d, 프로필과 무관)만. 의존: `an-core`(프로필 · 대화 · 메시지 표) · `an-net`(노드).
Windows 는 `windows-sys`(Job Object — 자손 프로세스 종료, 프로세스 그룹과 같은 동작).

## 공개 계약

- `runner::claude_args(&Agent, &Session, model, effort)` — `-p --output-format stream-json --verbose --restricted --strict-mcp-config --permission-prompts none
  --permission-mode dontAsk --tools <t> --allowedTools <t> [--append-system-prompt <p>]` + `--session-id <uuid>`(`Session::New`) |
  `--resume <id>`(`Session::Resume`) + 끝에 `--model <m> --effort <e>`. `--bare` · `bypassPermissions` 없음(D20 · D22).
  프로필 `parent_read` 이고 `an_core::paths::parent_read_dir` 가 `Some` 이면 `--add-dir <바로 위 폴더>` + 시스템 프롬프트에 `[읽기 범위]` 한 줄(#05-2 —
  `--restricted` 는 파일 도구를 작업 폴더 + `--add-dir` 안으로 가둔다, 허용 규칙 `Read(//…)` 로는 넘지 못한다 — 2.1.306 실측). `None` 이면 폴더만(로그 한 줄).
  `--add-dir` 은 값을 여러 개 받아 뒤의 위치 인자를 먹는다 — 원격 텍스트가 표준 입력이라 해당 없음.
- `runner::GUARD_PROMPT`(D31 문의 응답 전용 — 개입 · 편집 · 삭제 · 명령 실행 금지, 비밀 싣지 않기) — `claude_args` 가 항상
  `--append-system-prompt`(가드 + `[수신자 추가 지시]` 프로필 지시) · `--system-prompt-snapshot off`(재개 세션에도 지금 가드) · 도구는
  `agents::read_only`. `MIN_CLAUDE`(2.1.267) · `parse_version` · `version_ok` — `run_claude` 는 낮은 버전이면 실행하지 않는다(`--effort` 다섯 값
  모두 그 위에서 받는다 — 공식 CHANGELOG 근거는 `runner.rs` 주석).
- `runner::parse_output(stdout) -> RunOutput{text, session, is_error, turns, duration_ms, ctx_tokens}` — 마지막 JSON 줄(stream-json 의 `result` 이벤트 · 예전
  `json` 한 줄, `num_turns` · `duration_ms` 는 없으면 `None`), `session_id` 없으면 오류. `ctx_tokens`(#1m) = `usage.iterations` 마지막 항목(없으면 `usage`)의
  입력 + 캐시 쓰기 · 읽기 + 출력 토큰 — 실행 뒤 세션의 창 사용량. `RunOutput` 은 `Default`. `SUMMARY_PROMPT` — 압축 요약 지시.
- `runner::run_process_lines(program, args, cwd, stdin, timeout, on_line)`(#1c) — 표준 출력을 줄마다(끝 줄바꿈 뗀 채) `on_line` 에 넘기고 참을
  돌려준 줄만 `stdout` 에 남긴다. `runner::run_process(…)` = 모든 줄을 남기는 `run_process_lines` — 표준 입력 쓰기 · 출력 읽기를 스레드로. 시간 초과면 `TimedOut`, 끝나거나
  죽인 뒤 `ProcessTree` 로 자손까지 종료(Windows kill-on-close Job Object ↔ 그 밖 프로세스 그룹). Windows 는 `CREATE_NO_WINDOW`.
- `runner::RUN_TIMEOUT`(30분, D24 개정 — 유일한 실행 한도) · `find_claude()`(`~/.local/bin/claude(.exe)`, 없으면 PATH) ·
  `run_claude(program, &RunInput)`(작업 폴더 = 프로필 폴더 — 결과 줄 · JSON 아닌 줄만 남기고, 단계 요약이 바뀔 때마다 `RunInput.steps` 로).
- 단계 요약(#1c) — `RunInput.steps: StepSink`(상대가 보낸 실행 요청일 때만 채움, 비교 · 출력에서 무시) · `StepSink::new(f)` · `emit(&StepDetail)` ·
  `Default` = 아무것도 안 함. `count_step(&mut StepDetail, &event) -> 바뀜`(`assistant` 의 `tool_use` 이름별 +1 — `mcp__` 로 시작하면 `MCP`, 도구
  결과를 담은 `user` 마다 턴 +1 · 턴은 1 에서 시작 · `secs` 는 실행 시작부터. 입력 · 경로 · 글은 보지 않는다). 타입은 `an_net::proto::StepDetail`.
- `chat::Chat::new(conn, node, run: Run)` — `Run = Arc<dyn Fn(RunInput) -> io::Result<RunOutput>>`(운영 `run_claude`, 테스트 대역,
  `spawn_blocking` 에서 부름). `attach()`(노드 대화 요청 처리기) · `recover()`(시작 시, 런타임 안에서 — `queued` 실행 · `running` 은
  오류 답) · `send(peer_id, agent, Option<conv>, text) -> (Conversation, Message)` · `send_via(…, via, title)`(글의 출처 — `Some("cli")` 면 내 이력 · `message` 요청에 싣는다, `title` 은 새 대화에 이 PC 에서만 붙이는 제목) · `remote_agents(peer_id)` · `db()`(await 를 넘겨
  쥐지 않는다) · `new_uuid()`. `context(conv, op)`(#1m — `compact` · `clear`, 진행 중이면 `AlreadyExists`) — 구분선 글(`Author::Context`) +
  이 PC 세션이면 대화 줄(`run_context` → 비우기 · `compact` 요약 실행), 상대 쪽이면 outbox `context`(보내기 전 `knows_context` — 모르면 `unsupported`).
  공유(#1o): `share_add(conv, peer)`(async — 사본 · 원 상대 · 5명 `InvalidInput` · 이미 관람자 `AlreadyExists` · 모르는 상대 `Unsupported` · 없음 `NotFound`;
  관람자 행 · Share 머리 · 이력 한 트랜잭션) · `share_remove(conv, peer)` · `share_leave(conv)` · `fan_share`(커서 `sent_upto` — `dispatch` · `enqueue` · `handle` 끝) ·
  보내기 직전 `watching`(빠진 관람자 몫은 지움 — `ShareEnd` 제외) · 24시간 포기 = 그 관람 끝(`unshare`) ·
  받는 처리기 `inbound_share*`(그 상대의 Shared 사본만) · 원 상대 `ShareNotice`(바뀔 때만 알림). 사본 쓰기 = `InvalidInput` "공유 대화는 읽기 전용".
  나가기 · 삭제(#1w — D40): `leave_conversation(conv)`(사본 = `share_leave`, 도는 실행 `ResourceBusy` "실행이 끝난 뒤 나갈 수 있습니다", 아니면 한 트랜잭션 —
  `chat::leave` · 관람자마다 `share::remove` + `ShareEnd` · 원 상대 앞 `pending` 버림 · `Leave` 행 `conv_id` 없이, 이미 나갔으면 그대로) · `delete_conversation(conv)`
  (나가기 뒤 `chat::delete_conversation`) · `inbound_leave`(그 상대 대화에 `set_peer_left`, 없으면 `Ok`). `handle` 앞 — 나간 대화 · 묘비로 온 요청은 `Leave` · `ShareEnd`
  말고 `left`. `deliver` 의 `leave` 는 `knows_leave` 거짓이면 버림 · 포기 없음. 나간 대화 쓰기 = `InvalidInput` "나간 대화는 읽기 전용"(`writable` — 이력 정리 · 공유 넣기 ·
  자동 왕복도). `record` — 지운 대화면 답을 버리고 나간 대화면 이력에만, `process` — 줄을 서는 동안 취소된 대기 글은 실행하지 않는다. `refused()` `left` = "상대가 대화를 나갔습니다".
  세션 생성 · 기록 삭제(#1x — D40 개정, 웹UI 나가기 = `delete_conversation`): `open_session(peer_id, agent, relay: Option<(내 에이전트, 상한)>, Compose) -> Conversation`
  (보내지 않는다 — 상대 에이전트 빈 이름 · 상한 1 ~ `MAX_RELAY_CAP` 밖 · 없는 내 에이전트는 `InvalidInput`, 페어링 확인 뒤 한 트랜잭션 `open_conversation(Out)` + `set_relay` +
  `set_title` + 그룹 넣기. 상대 PC 는 첫 글이 갈 때 `inbound_message` 가 대화를 연다). `send_with(Some(conv))` — 보낸 대화가 글 0(`chat::empty`) + 자동 왕복 설정이면
  첫 글은 `send_human(.., Some(내 에이전트))`(자동 왕복 시작 — `start_relay_with` 와 같은 길, `via` · 멘션은 버림, 요청이면 `InvalidInput` `REQUEST_NO_RELAY`), 그 밖은
  지금처럼 사람이 쓰면 자동 왕복 끔. `clear_conversation(conv, all)`(async) — 사본 `READ_ONLY` · `all` 인데 닫힌 대화 `CLOSED` · `all` 이면 먼저 `knows_clear`
  (거짓 = `Unsupported` `CLEAR_UNKNOWN` "상대 앱이 이 기능을 모릅니다" — 아무것도 안 지움, 페어링 아님 = 그 오류, 그 밖 전송 오류 = 진행 — 줄이 나중에 보낸다) →
  한 트랜잭션: `chat::busy` 면 `ResourceBusy` `CLEAR_BUSY` "실행이 끝난 뒤 지울 수 있습니다" · `all` 인데 그 상대에게 보내는 중(`outbox::sending`)이면 `CLEAR_SENDING`(적대적 평가 R2) · `chat::clear_conversation` · `all` 이면 `Clear{conv, id}` 를 그 대화 줄(`conv_id` — 뒤에 쓴 글보다 먼저 가게) outbox(kind
  `clear`, `conv_id` 없이) + `reshare`(그 대화의 기다리는 `share_post` · `share_delete` 행을 버리고 관람자마다 머리 `Share` 를 다시 — 사본은 같은 uuid 머리를 받으면
  글을 비운다, 새 프레임 없음) → `kick`. `share_head(conn, conv)` = 공유 머리(제목 · 원 상대 이름 — `share_add` 와 같이 씀). 받기 `inbound_clear(peer, conv)` — `find_own`
  없음 · 사본이면 `unknown`(닫힌 대화는 `handle` 앞이 `left`), `busy` 면 `retry`(보낸 쪽 `backoff` 재시도 — 실행이 끝난 뒤 비움), 아니면 비움 · `reshare` · 알림
  `MessageKind::Clear`. `deliver` 의 `clear` — 보내기 전 `knows_clear`: 거짓이면 행을 버리고 warn 로그(그 사이 이전 버전이 된 상대 — 상대 쪽 기록은 그대로),
  페어링 아님이면 `not_paired` 로 정리, 전송 오류는 `backoff`. 24시간 포기 대상 아님(`leave` 와 같게).
  받은 요청은 다시 넘기지 않는다. 자동 압축 = `process` 뒤 창 사용량 ≥ `settings::AUTO_COMPACT_AT`(800k) · `settings::auto_compact`. 새 세션 첫 실행에
  `ctx_seeds` 요약을 `<대화 요약>` 으로 붙이고(`execute`) 세션을 남기면 지운다(`record`).
- 실행 문(#1b, D24 개정) — `gate::Gate`(FIFO): `enter(id) -> (Ticket, Option<앞 사람 수>)` · `Ticket::ready()` · 티켓 drop = 자리 놓기(실패 ·
  panic · 취소에도) · `set_cap`(늘면 바로 당김) · `snapshot()`. `Chat` 하나 = PC 전체 문 — `process` 첫 줄 `admit` 에서 줄을 서고 **차례가 온 뒤**
  대화 · 에이전트 · 자동 왕복 멈춤을 읽는다(사람 요청 · 자동 왕복 · 이 PC 사람 요청 모두 같은 문). 상한 `max_runs(conn)`(설정 `MAX_RUNS_KEY`
  `max_runs`, 기본 `DEFAULT_MAX_RUNS` 2 · 1 ~ `MAX_RUNS` 8, 못 읽으면 기본값 + warn) · `run_queue() -> (cap, 실행 중 id, 기다리는 id)` ·
  `set_max_runs(n)`(저장 + 문에 바로, 범위 밖 `InvalidInput`). 상대가 보낸 실행 요청이면 `Progress` 를 그 상대 보내기 줄(outbox, 키
  `progress:{글 uuid}` — 기다리는 진행은 마지막 값만)로: 기다리면 `queued`(앞 사람 수) → `running`, 바로면 `running`. 실행 중에는 실행기 단계 요약의
  마지막 값을 두고 작업 하나가 `step_every`(기본 10초 · 시험용 `set_step_every(Duration)`)마다 바뀐 것만 `running` + `detail` 로(10초 안에 끝나면
  보내지 않음). 보낸 쪽 `inbound_progress` — 그 상대 대화의 내 실행 요청 · `sent` 만(아니면 `unknown`, 상태 밖 `bad_request`, `detail` 은 `running`
  일 때만 · `cleaned`) → 메모리 `progress` · `progress_of(&Message) -> Option<Progress{state, ahead, detail}>`(답이 온 글은 지움, 앱을 다시 켜면
  사라짐).
- 받기 규칙: 본문 1 ~ `MAX_TEXT`(16 KiB) · 등록된 에이전트(`unknown_agent`) · 대화 uuid 는 그 상대 소유 · 같은 메시지 uuid 는 한 번만.
  저장(`queued`)이 대기열 등록 — 대화별 작업이 id 순으로 `running` → 실행 → 세션 · 상태 · 답을 한 트랜잭션(실패 → 오류 답) →
  `reply`(실행 정보 · 원 요청 대응 · 실행한 model · effort 를 싣는다 — 양쪽 답에 남는다, outbox 로 — 상대가 거절했거나 24시간 닿지 않으면 `undelivered`). 실행 옵션은 실행 직전 `run_opts::resolve`(그 상대 덮어쓰기 → 전역 →
  기본값) — 설정을 못 읽으면(DB 오류) 기본값으로 돌리지 않고 실패 답. 세션은 실행한 에이전트의 폴더가 그대로일 때만 남긴다(실행 도중 폴더가
  바뀌면 `agents::update` 가 지운 세션을 되살리지 않는다). 재개할 세션이 사라졌으면(`runner::session_gone` — 결과 줄 `errors` 의
  "No conversation found with session ID"(2.1.299) · 결과 줄 없으면 stderr(2.1.296)) `run_claude` 가 `SessionMissing` 을 돌려주고, 실행은 새 세션으로
  **한 번** 다시 돌린다 — 답 · 새 세션을 기록해 다음 요청은 새 세션을 잇는다(#1h — 원장 #15 R2-F2r). 다른 실패는 다시 돌리지 않는다.
- `Chat::post(conv, text, to_agent) -> (Conversation, Message, 경고)` · `post_via(…, via)` — 양쪽 사람 발화(D18 · D23): 보낸 대화의 실행 요청은 `send`, 그 밖은
  내 이력 + `post` 미러. 로컬 API 는 늘 `to_agent = true`(D23 개정 — 새 글은 모두 실행 요청), 실행 없는 사람 글 받기는 이전 이력용으로 남는다. 받은 쪽 실행 요청은 사람 글을 먼저 미러한 뒤 실행(미러가 거절되거나 보관조차 못 하면 실행 · 성공 + 경고, 닿지 않으면 보관 — 경고 없음), 답은 `post`(agent).
  받기 `post`: 그 상대 소유 대화만, 에이전트 답 · 실행 요청 글은 에이전트가 상대 쪽인 대화(`out`)만, 에이전트 답은 상대가 쓴 `sent`
  실행 요청에만. 실행 입력은 늘 `[요청 — {발화자}]` + `<원격 본문>` … `</원격 본문>`, 지난 실행 요청 뒤 사람 글은 `[대화 메모 — 실행 요청 아님]` + `<대화 메모>` …
  `</대화 메모>` 로 앞에(최근 `MAX_NOTES` 20 · `MAX_NOTES_BYTES` 32 KiB, 원격 글 안의 구분자는 지움 — #1a). 실행 실패는 내 쪽 글에 원문, 상대에게는
  정해진 문구(`Failure` — 시간 초과 · 버전 · 못 찾음 · 기록 오류 · 폴더 막힘 · 그 밖). 실행 직전 `agent_folder_problem` 으로 폴더를 다시 거른다.
- 받은 `message` · `post`(사람 글)의 `via` 는 글 · 순서와 한 트랜잭션으로 저장(`cli` 만). 자동 왕복 · 에이전트 답은 `via` 없음.
- 보내기 규칙: 대화별 잠금으로 저장 · 전송 · 응답을 한 번에 하나(도착 순서 = 이력 순서). 상대 거절만 `error`, 전송 오류는 오류가 아니다 —
  `sent` 유지 + 보관(아래).
- 보내기 보관(#1c — D7): 상대에게 가는 대화 프레임은 `an_core::outbox` 에 먼저 적는다. 내부 `ship`(행 `inflight` → 같은 대화에 앞선 행이 없고 그
  상대에 닿지 않고 있지 않으면 바로 한 번, 보내기 · 뒷정리는 따로 띄운 작업 — 부른 쪽이 취소돼도 끝난다. 응답이면 행 지움, 전송 오류 · 앞선
  행이면 `pending` · 보관) — 사람 실행 요청 · 실행 없는 글 · 미러 · 답 · 자동 왕복 넘김 · 상한 기록 · 완료 · 멈춤 알림. 내부 `enqueue`(줄에만 ·
  키 합침) — 진행. 사람 실행 요청 · 미러 · 답 · 자동 왕복 넘김은 `stage`(행 적기)를 **글을 적는 DB 트랜잭션 안에서** 하고 커밋 뒤
  `dispatch`(보내기) — 글만 남고 보낼 행이 없는 채로 꺼지지 않는다(적대적 평가 반영). 24시간 포기는 그 상대 줄의 `pending` 행만(보내는 중은 그 결과가 정함). 바로 보내는 것: `Agents` · `Status` · 자동 왕복 재개 `Relay{on: true}`. 행 종류(`kind`): `message` · `relay_open` · `relay_step` ·
  `note` · `mirror` · `reply` · `reply_post` · `relay_last` · `done` · `relay` · `progress`.
- 상대 줄(#1c): 상대마다 tokio 작업 하나(`lane` — 끝났으면 다시 띄움)가 맨 앞 `pending` 행을 `next_at` 에 보낸다. 응답이면 행을 지우고 `settle`(그
  자리의 응답 처리와 같게 — 거절이면 `refused_by_peer` · 자동 왕복 끔 · 넘김이면 끝 사유, `note` 는 `Ok` 면 `done` · 거절이면 `error`, 답 종류 거절은
  `undelivered`). 전송 오류 · 받는 쪽 `retry` 코드면 `backoff` — 5초부터 두 배 · 최대 5분, 기다린 합이 24시간이면 그 상대 줄의 행을 모두 `failed` +
  `give_up`(`message` · `relay_open` · `relay_step` · `note` → `error` + 거절 코드 `UNREACHABLE`(`unreachable`) · 자동 왕복 끔 · 넘김이면 끝 사유
  "상대에 닿지 않음", `reply` · `reply_post` · `relay_last` → `undelivered`, 그 밖은 기록만). 시험용 `set_retry(base, max, give_up)`(초). 그 상대에서
  무엇이든 받거나(`handle` · `on_seen` 의 `Ping`) 상태 질의가 닿으면 `peer_back` — 그 상대 줄을 지금 보낸다. `recover` 는 남은 상대 줄을 지금 다시
  띄우고 7일 지난 실패 보관을 지운다. `outbox_of(&Message) -> Option<outbox::Item>` — 내 글의 보관 상태(웹UI "보내는 중" · "보내지 못함").
- 받는 쪽 처리 오류(DB 잠금 등)는 거절이 아니라 `Error{RETRY}`(`retry`) — 보낸 쪽이 다시 보낸다.
- 상태 질의(#1c): `ask_status(&Message) -> Response` — 상대에 `Status` 를 바로(보관하지 않음), `queued` · `running` 이면 진행을 갱신. 받는 쪽
  `inbound_status` — 그 상대 대화의 상대 실행 요청 글이면 `queued`(실행 대기 목록의 앞 사람 수) · `running` · `done`(끝남 — 그 상대 줄을 지금
  보냄), 그 밖 `unknown`.
- CLI 트랜잭션(#1c): `Compose.tx_key` — `via` 가 있는 글은 글과 같은 트랜잭션에서 `cli_tx` 를 만든다(키가 겹치면 글도 없음) · `keyed()`(키 있는 CLI
  보내기 잠금 — API 가 같은 키 확인부터 보내기까지 잡는다) · `tx_of_message(id)` · `tx_by_key(key)` · `tx_resume(id)`(`wait` — 멈춤을
  풀고 다시) · `tx_view(id) -> Option<TxView{tx, state, message, reply, progress, outbox}>` — `state` 는 순서대로 `answered` · `failed`(실패 답 · 글
  실패) · `stalled` · `failed`(실패 보관) · `sending`(outbox 에 남음) · `queued` · `running` · `sent`. 끝나지 않았고 마지막 질의가 `every` 초 전이면 감시 작업을
  띄우고 바로 돌아온다(같은 트랜잭션은 겹쳐 묻지 않음) — `sending` 이면 그 상대 줄이 닿지 않고 있는지(`outbox::failing`), 그 밖은 `ask_status`.
  닿으면 `cli_tx::reached`, 아니면 `cli_tx::unreachable`(`stall_after` 초 이어지면 멈춤). 운영 60초 · 10분, 시험용 `set_watch(every, stall_after)`.
- 답 받기 규칙: `reply_to` 가 같은 대화의 `sent` 실행 요청(사람 글 · 자동 왕복의 내 에이전트 답)일 때만(아니면 `unknown`), 받으면 그 메시지 `done/error`.
- 자동 왕복 시작 · 끝은 알림 설정과 무관하게 `relay_events` 에 기록(내 활동).
- 집중(#18): 받는 순간 `settings::focus` 면 새 `message`(자동 왕복 차례 포함)를 `focus` 로 거부 — 기록 · 실행 · 알림 없음, 못 읽으면 받는다(15 R2),
  이미 받은 글의 재전송(같은 uuid · 승격 아님 — `Chat::resent`)은 `Ok`. 연 쪽 `inbound_reply` 는 집중이면 답을 넘기지 않고 끔(끝 사유 "집중 중").
  보낸 쪽 거절(`send_human` · `relay_forward`)은 `refused_by_peer` — 실패 + 거절 코드(`clean_code`), `focus` 면 그 상대 접속 · 집중, 자동 왕복을 여는
  글이면 켜짐을 거둔다 · 오류 문구 "상대가 집중 중 — 요청을 받지 않습니다. 나중에 다시 보내세요" · 끝 사유 "상대가 집중 중". 재개로 다시 보내면 거절
  코드를 지운다.
- 새 글의 선택(#18) — `Compose { via, title, request, mention }` · `send_with(peer, agent, conv?, text, Compose)` · `post_with(conv, text, to_agent,
  Compose)` · `start_relay_with(…, Compose)`(기존 `send` · `send_via` · `post` · `post_via` · `start_relay` 는 기본 `Compose` 로 위임). `request` 는
  `message` · `post` 에 실어 양쪽에 남는다.
- 요청(#1d) — `Compose.request` 글은 **실행하지 않는다**. 보낸 대화는 `Message{request: true}`(내 글 `exec = false` · outbox 종류 `note` — 닿으면
  `done`), 받은 대화는 실행 없는 미러(`post_with` 가 `to_agent = false` 로). 멘션 · CLI(`via`)와 함께면 `InvalidInput`(`REQUEST_NO_MENTION` ·
  `REQUEST_NO_CLI`), 자동 왕복을 열지 않는다. 받기: `inbound_message` 가 집중 판정 **앞에서** `inbound_ask` 로(대기열 · 실행 기록 없이 사람 글
  `exec = false` · `done` · `req_state = wait`, 알림 `Ask` — 자동 왕복 · 멘션과 함께면 `bad_request`), `inbound_post` 의 요청 + 실행은 `bad_request`.
- 요청 상태(#1d — #18 완료 표시 대체) — `set_req_state(id, state, note, saved?) -> (Message, 경고)`: 받은 요청(사람 글 · `mine = false` · `request`)만,
  옮김은 `ReqState::can_become`(같은 상태 · 의견 규칙 위반은 `InvalidInput` — 아무것도 쓰지 않는다). 대화 차례(`turn`) 안에서 확인 · 상태 · 저장 ·
  보내기 행(`ReqState`, 종류 `req_state` · 그 요청 글에 이음 — 바뀔 때마다 한 행, 합치지 않음)을 한 트랜잭션으로 쓰고 보낸다(닿지 않으면 보관).
  상대가 거절했을 때만 경고("이 PC 에는 바꿨다 — …"). 오래 닿지 않아 포기하면(`give_up`) 그 글 `refused = unreachable`, 요청자가 거절하면(바로 보낼 때 · 줄이 보낼 때
  `settle`) 그 거절 코드 — 같은 상태 · 의견으로 다시 부르면(`refused` 가 있을 때만 같은 상태 허용) 다시 보내고 표시를 지운다. 받기는 같은 상태 ·
  의견이면 아무것도 바꾸지 않고 `Ok`(시각 · 알림 그대로 — 재전송 멱등). 받기 `ReqState` 는 그 상대 대화의 내 요청 사람 글에만(그 밖 `unknown`, 모르는 상태 · 의견 규칙 위반은
  `bad_request`) — 옮김 검사 없이 받은 대로, 알림 `AskState`.
- 이력 정리(#1f) — `Chat::tidy_ok(conn, &msg)`(사람 글 ∧ `mine` ∧ `done` · `error` · `undelivered` ∧ 그 글을 실어 나르는 outbox 행이 기다리지
  않음 — 웹UI `editable` 의 정본) · `edit_message(id, text, mirror) -> (Message, 경고)` · `delete_messages(ids, mirror) -> (지운 수, 경고)`(한 대화의
  글만 · 하나라도 안 되면 `InvalidInput` — 아무것도 쓰지 않는다). 대화 차례(`turn`) 안에서 확인 · 쓰기 · 반영 행(`Edit` · `Delete`, 종류 `msg_edit` ·
  `msg_delete` — 글마다 한 행)을 한 트랜잭션으로 쓰고 커밋 뒤 보낸다(닿지 않으면 보관 — 경고 없음). 상대 거절 · 24시간 포기(`settle` ·
  `give_up`)는 그 대화 `relay_events` `sync_failed`(사유 "편집 — {미리보기}" · "삭제") — 바로 보낸 결과가 거절이면 경고("이 PC 에는 반영했다 — 상대 창에
  반영하지 못했다(…)")도. 받기 `inbound_edit` · `inbound_delete` 는 그 상대 소유 대화의 `!mine` 사람 글에만(그 밖 `unknown`, 실행 대기 · 실행 중
  `busy`, 빈 · 긴 본문 `bad_request`) — 실행 · 알림 없음, 같은 본문 · 이미 없는 글 삭제는 `Ok`.
- 멘션(#18) — `Compose.mention`: 보낸 대화 이어 쓰기 · 자동 왕복 아님에서만(새 대화 · 받은 대화 · 자동 왕복 · 빈 이름은 `InvalidInput`, 대화 에이전트
  이름이면 멘션 아님). 글 `agent` 와 `message.mention` 에 부른 이름(`message.agent` 는 늘 대화 에이전트). 받은 쪽은 `mention` 이 대화 에이전트가
  아니면 멘션(`relay` · `by_agent` 와 함께면 대화를 만들기 전 · 자동 왕복 대화면 `bad_request`, 없는 이름은 `unknown_agent`, 대화가 아직 없으면 대화
  에이전트로 연다) — 실행 대상 `asked.agent ?? 대화 에이전트`, 세션은 `agent_sessions`(대화 에이전트는 `conversations.session`), 답 `agent` = 요청의
  값(보낸 쪽도 답을 받을 때 요청의 값), 알림의 에이전트 = 부른 이름. 끝난 실행은 그 에이전트의 지금 세대(`Agent.uid`)가 실행 때와 같을 때만 세션을
  남긴다(실행 중 폴더 변경 · 지우고 다시 등록하면 남기지 않는다).
- 자동 왕복(D24): `Chat::start_relay(peer_id, agent, my_agent, cap, text, title)`(상한 1 ~ `MAX_RELAY_CAP` 50, 기본 `DEFAULT_RELAY_CAP` 10) ·
  `Chat::relay(conv, on)`(연 쪽 멈춤 · 재개 — 카운트 0 · 받은 쪽 거절 해제, 받은 쪽은 멈춤만). 연 쪽이 구동: 성공한 상대 답 + 상한 전이면
  답 기록과 한 트랜잭션으로 실행 요청 등록 → 내 에이전트 실행(`relay_step`) → `message{by_agent}` 로 넘김. 상한 n = 양쪽 n 번씩 —
  n 번째 내 답은 `post`(agent, 실행 없음)로 상대 이력에 기록만. 재개는 넘기지 못한 내 답을 같은 uuid 로 다시 넘긴다(받은 쪽은 기록만
  받은 글을 실행 요청으로). 실패 답 · 상한 · 거절 · 24시간 닿지 않음이면 멈춤,
  전송 오류는 멈추지 않는다(보관 — 닿으면 이어 감). 재개(`Relay{on: true}`)는 바로 보낸다. 사람이 쓰면 멈춤(연 쪽 끔 · 받은 쪽 거절 + `relay{on:false}` 알림 — 기다리지 않음). 받은 쪽은
  거절 중이면 에이전트발 요청에 `stopped` — 연 쪽은 `relay_stopped`(내 답을 넘기지 않은 답 `done` 으로 되돌리고 끔 · "상대가 멈춤", 멈춤 알림이
  먼저 온 것과 같은 결과 — 바로 보낼 때 · 줄이 보낼 때 모두). 실행 입력에 `[요청 — 상대 에이전트({상대})]`.
- 이벤트: 받은 요청 · 받은 답 → `PeerEvent::Message{name, host, agent, preview, kind}`(알림 S9). 자동 왕복은 양쪽 모두 시작 1건
  (받은 쪽 = `relay` 표시 사람 요청 · 재개 알림, 연 쪽 = 그 요청에 온 첫 답) · 중간(`RelayStep`) · 저절로 멈춤 1건(`RelayEnd` — 상한 ·
  받은 쪽 자동 왕복 요청 실행 실패(받은 쪽은 `mark_relayed` 로 자동 왕복 대화를 기록) · 상대/내 실행 실패 · 거절 · 상대에 닿지 않음 · 상태 저장 실패 · 상대가 멈춤). 사람이 멈춘 것은 내지 않는다.
- 상한 기본값: `RELAY_CAP_KEY`(`relay_cap_default`) · `default_relay_cap(conn) -> Result`(없거나 범위 밖이면 10, DB 오류는 그대로).

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
