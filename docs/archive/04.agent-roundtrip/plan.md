# plan — 04.agent-roundtrip

> 근거: [intent](intent.md)(승인 2026-10-11 — 자동 진행 위임 + 확인 답). **구현자는 이 문서만 보고 일한다.** D·S 는 [masterPlan](../../masterPlan.md).

## 목표

사람 → 원격 에이전트(`claude -p`) 1왕복 · 양쪽 이력 · 이어 쓰기 · 원격 메시지 알림.

### 성공 기준(intent 사본)

- A1: 에이전트 프로필을 등록 · 조회 · 삭제할 수 있고, 새 프로필의 허용 도구 기본값은 Read · Grep · Glob 이다.
- A2: 페어링된 상대만 에이전트 목록 조회 · 메시지 · 답 전달을 할 수 있다(미페어링은 `not_paired`). 등록되지 않은 에이전트 이름은 거부된다.
- A3: `claude -p` 실행 인자는 D22 를 지킨다 — `--restricted --strict-mcp-config --permission-prompts none --permission-mode dontAsk`,
  프로필의 `--tools` · `--allowedTools`, `--output-format json`, 작업 폴더 = 프로필 폴더. 원격 텍스트는 인자가 아니라 표준 입력으로 넘긴다.
- A4: 사람이 보낸 메시지에 수신 PC 에이전트의 답이 돌아와 양쪽 이력에 남고, 같은 대화의 다음 메시지는 저장된 세션으로 `--resume` 한다.
- A5: 실행은 대화별로 직렬이고, 30분을 넘으면 종료되어 오류 답이 돌아간다. 수신 처리는 실행을 기다리지 않는다(바로 응답).
- A6: 받은 요청 · 받은 답이 알림(S9) 이벤트로 나간다.
- A7: 루트 `PROJECT.md` 검증 명령의 테스트 · 경고 0 · 의존 경계 행이 통과한다.

### intent 결정 · 제외(사본)

- R1 `an-agent`(domain) · R2 대화 UUID 공유, 세션 ID 는 에이전트 쪽만(`--session-id` → `--resume`) · R3 `message` → 즉시 `ok` → 실행 →
  `reply`(고정 키), 실패 시 "미전달" · R4 본문 16 KiB · R5 `claude` 경로 탐색 · R6 `dontAsk` + `--allowedTools` = `--tools`.
- 제외: 사람끼리 대화 · 자동 왕복(Phase 1) · 오프라인 재전송(Phase 2) · 수동 확인(D28).

## 설계

### 저장(스키마 4, `an-core`)

```sql
CREATE TABLE agents (id INTEGER PRIMARY KEY, name TEXT NOT NULL UNIQUE, folder TEXT NOT NULL, tools TEXT NOT NULL,
  prompt TEXT NOT NULL DEFAULT '', created_at INTEGER NOT NULL);
CREATE TABLE conversations (id INTEGER PRIMARY KEY, uuid TEXT NOT NULL UNIQUE, peer_id INTEGER NOT NULL, agent TEXT NOT NULL,
  dir TEXT NOT NULL, session TEXT, created_at INTEGER NOT NULL);           -- dir = 'out'(내가 원격 에이전트에) | 'in'(내 에이전트가 받음)
CREATE TABLE messages (id INTEGER PRIMARY KEY, uuid TEXT NOT NULL UNIQUE, conversation_id INTEGER NOT NULL, author TEXT NOT NULL,
  text TEXT NOT NULL, state TEXT NOT NULL, created_at INTEGER NOT NULL);   -- author 'human' | 'agent', state 'sent' 'queued' 'running' 'done' 'error' 'undelivered'
```

- `an_core::agents` · `an_core::chat` — CRUD · 대화 찾기/만들기(uuid + peer) · 메시지 추가/상태 · 목록.

### 프로토콜(`an-net`)

- 요청 추가: `agents` → `agents{names}` · `message{conv, msg, agent, text}` → `ok` | `error(unknown_agent | bad_request)` ·
  `reply{conv, msg, reply_to, text, ok}` → `ok` | `error(unknown)`. 모두 **페어링된 상대만**(아니면 `not_paired`).
- `Node::on_request(f)` — 페어링 확인 뒤 위 요청을 `f(&Peer, Request) -> Response` 로 넘긴다(한 번 설정, 막히면 안 됨). 설정 전에는
  `error(unsupported)`. `PeerEvent::Message{name, host, agent, preview}` 추가.

### 실행기 · 조율(`an-agent`)

- `runner::claude_args(&Agent, session: Session) -> Vec<String>` — `-p --output-format json --restricted --strict-mcp-config
  --permission-prompts none --permission-mode dontAsk --tools <t> --allowedTools <t> [--append-system-prompt <p>]` + `--session-id <uuid>`
  (첫 실행) | `--resume <id>`. 프롬프트는 표준 입력.
- `runner::parse_output(stdout) -> RunOutput{text, session, is_error}` — `--output-format json` 의 `result` · `session_id` · `is_error`.
- `runner::run_process(program, args, cwd, stdin, timeout) -> io::Result<Output>` — 시간 초과면 자손까지(`ProcessTree`: Job Object ↔ 프로세스 그룹) 죽이고 `TimedOut`, 정상 종료 뒤 남은 자손도 종료.
- `runner::find_claude()` — R5.
- `Chat::new(conn, node, run)` — `run: Fn(RunInput) -> io::Result<RunOutput>`(운영 = claude, 테스트 = 대역, `spawn_blocking` 에서 부름).
  - `send(peer_id, agent, conv: Option<i64>, text)` — 대화(out) 만들기/찾기 → 대화별 보내기 잠금 → human 메시지 저장 · `message` 전송
    (상대 거절 → `error`, 전송 오류 → `sent` 유지 + "전달 확인 안 됨" 오류 — 늦은 답을 받는다).
  - `handle(&Peer, Request)` — `message`: 에이전트 확인 · 대화(in) · 메시지(queued) 저장(= 대기열 등록) 뒤 대화 작업을 깨우고 즉시
    `ok` · 이벤트. `reply`: 대화(out) · `reply_to` 가 그 대화의 `sent` 사람 메시지인지 확인 → agent 메시지 저장 + 원 메시지 done/error
    (한 트랜잭션) · 이벤트, 아니면 `unknown`. `agents`: 프로필 이름.
  - 작업: 대화별 tokio 작업 하나가 DB 에서 id 순으로 꺼내 `running` 표시 → 실행 → 세션 · 사람 상태 · agent 메시지 한 트랜잭션(실패 →
    오류 답) → `reply` 전송(실패 → `undelivered`). `recover()`(시작 시): `queued` 실행, `running` 은 오류 답.
  - 시간 제한 `RUN_TIMEOUT` = 30분.

### 웹UI · API(`agentnetwork-app`)

- `GET/POST /api/agents` · `DELETE /api/agents/{id}` · `GET /api/peers/{id}/agents` · `GET /api/conversations` ·
  `GET /api/conversations/{id}/messages` · `POST /api/conversations {peer_id, agent, text}` · `POST /api/conversations/{id}/messages {text}`.
- 웹UI: "에이전트" 섹션(등록 폼 · 목록) · "대화" 섹션(상대 · 원격 에이전트 선택 · 대화 목록 · 메시지 · 이어 쓰기). 텍스트로만.
- `Notifier` 가 `PeerEvent::Message` 를 알림으로.

## Stage

| stage | 목표 | 산출 | gate |
|---|---|---|---|
| 1 | 스키마 4 · 저장 | 테스트(store) | E3 |
| 2 | 실행기 | 테스트(runner) | E4 |
| 3 | 프로토콜 · 조율 · 왕복 | 테스트(roundtrip) | E5 · E6 |
| 4 | API · 웹UI · 알림 | 테스트(chat_api) | E1 · E2 · E7 · E8 |

## 평가 항목

- [ ] E1: A7 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E2: A7 — 우리 코드 경고 0(clippy -D warnings)
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: pending
- [ ] E3: A1·A4 — 저장 3건(agent_defaults_read_only_tools · conversation_by_uuid_per_peer · messages_in_order_with_state)
  CHECK: cargo test -p an-core --test store
  EXPECT: test result: ok. 3 passed; 0 failed
  EVIDENCE: pending
- [ ] E4: A3·A5 — 실행기 5건(args_follow_d22 · resume_uses_saved_session · parse_claude_json · timeout_kills_process · process_tree_is_killed)
  CHECK: cargo test -p an-agent --test runner
  EXPECT: test result: ok. 5 passed; 0 failed
  EVIDENCE: pending
- [ ] E5: A2·A4·A5·A6 — 왕복 7건(human_to_remote_agent_roundtrip · followup_resumes_session · unpaired_or_unknown_agent_refused · inbound_does_not_wait_for_run · forged_or_duplicate_reply_refused · restart_recovers_queue · storage_failure_sends_error_reply · unconfirmed_delivery_keeps_waiting_for_reply · concurrent_sends_keep_order)
  CHECK: cargo test -p an-agent --test roundtrip
  EXPECT: test result: ok. 9 passed; 0 failed
  EVIDENCE: pending
- [ ] E6: A7 — 의존 경계: 크레이트 4개 · 내부 간선 6개 · 위반 0
  CHECK: python tools/check_crates.py
  EXPECT: crates=4 edges=6 violations=0
  EVIDENCE: pending
- [ ] E7: A1·A4·A6 — 웹UI API 2건(agents_crud_api · conversation_api_roundtrip)
  CHECK: cargo test -p agentnetwork-app --test chat_api
  EXPECT: test result: ok. 2 passed; 0 failed
  EVIDENCE: pending
- [ ] E8: A3·A4 — 실동작(수동, PC 2대): 실제 `claude -p` 로 원격 에이전트 답 · 이어 쓰기 맥락 유지 · 읽기 전용 프로필에서 파일 수정 요청이 거부됨
  EVIDENCE: pending

## 종결 시 할 일

- `PRODUCT.md` 모듈 표(`an-agent`) · 도메인 규칙(Claude 실행 · 이력) · 매뉴얼 `an_agent` 신규 · masterPlan `#04` · `check_crates` LAYER.

## 개정 기록

- 개정 1(2026-10-11, 적대적 평가 라운드 1 반영): 대기열 = DB(`queued` → `running` → `done/error`, id 순, 시작 시 `Chat::recover` —
  `queued` 는 실행 · `running` 은 오류 답) · 실행 후 기록은 한 트랜잭션, 실패 시 오류 답 · `reply` 는 `reply_to` 가 그 대화의 답 없는
  내 메시지일 때만(받으면 그 메시지 `done/error`) · 시간 제한 · 종료 시 자손 프로세스까지 종료(Windows Job Object ↔ 프로세스 그룹,
  두 OS 같은 동작 — D30 의 "같은 동작을 낼 때만" OS 연동). 스키마 값 `running` 추가(열 추가 없음 — 버전 4 유지). E4 4 → 5건, E5 4 → 7건.
- 개정 2(2026-10-11, 적대적 평가 라운드 2 반영): 보내기는 대화별 잠금으로 저장 · 전송 · 응답을 한 번에 하나(도착 순서 = 이력 순서),
  전송 오류는 실패로 확정하지 않는다(`sent` 유지 — ACK 손실 시 늦은 답 수용 · 중복 실행 방지 안내). 웹UI 는 늦게 온 다른 대화 조회 응답을
  버린다. E5 7 → 9건.
