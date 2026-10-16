# stage1 — 통신 · 스키마 19 · 실행기 크기 · 압축 · 비우기 · 자동 압축 · 자동 왕복 연동 · 0.4.0 보호

> 근거: [plan](../plan.md) Stage 1.

## 목표

해석 1 · 4 ~ 13 의 서버 쪽 전부(화면 · API · 설정 화면은 stage 2). 대상 gate: E1 · E2 · E7.

## 산출 파일

- `crates/net` — `proto.rs` `Request::Context{conv, msg, op, text}` · `CTX_OPS` · `Reply.ctx` · `Post.ctx`(없으면 싣지 않음) · `Pong.ctx` · `ids_ok`.
  `node.rs` 대화 요청 목록 · 갈래에 `Context`, `Pong{ctx: true}`, `knows_context(id)`(Ping 으로 묻기 — `ping` 과 같이 접속 상태 기록).
- `crates/core` — `db.rs` 스키마 19(`messages.ctx_tokens` · `ctx_op`, `ctx_seeds`). `chat.rs` `Author::Context` · `Message.ctx_tokens` · `ctx_op` ·
  `set_ctx_tokens` · `add_context` · `finish_context` · `sessions` · `clear_sessions` · `set_ctx_seed` · `ctx_seed` · `drop_ctx_seed`, `pending` 은 구분선 글을 맨
  앞에, 구분선 글은 내 활동 · 대화 미리보기 · 안 읽음 · 가로지른 글 · 마지막 실행 요청(메모 범위)에서 뺌. `settings.rs` `AUTO_COMPACT_KEY` · `auto_compact()`(기본 켜짐) ·
  `CONTEXT_LIMIT`(1M) · `AUTO_COMPACT_AT`(800k).
- `crates/agent` — `runner.rs` `RunOutput.ctx_tokens`(`Default`) · `parse_output` 이 `usage.iterations` 마지막 항목을 읽음 · `SUMMARY_PROMPT` · 가드에
  `<대화 요약>`. `chat.rs` `Chat::context(conv, op)`(화면) · `request_context`(구분선 글 + 대화 줄 · 상대 줄, 한 트랜잭션) · `inbound_context` ·
  `run_context`(비우기 · 압축 → 상대가 요청한 글이면 `done` · `failed` 회신) · `compact`(세션마다 요약 실행 · 실행 문 통과 · 하나라도 실패하면 세션 그대로) ·
  `context_refused`, `execute` 가 새 세션이면 요약 주입(`unframe`), `record` 가 크기 기록 · 요약 지움, `process` 뒤 자동 압축 판정, `deliver` 가 `context` 줄 앞
  `knows_context`(모르면 `unsupported` 로 정산, 닿지 않으면 다시 보냄), `settle` · `give_up` 컨텍스트 갈래, 답 · 글 프레임 `ctx`.
- 시험 — 새 `crates/agent/tests/context.rs` 6건(묻는 쪽 압축 → 상대 요약 · 양쪽 구분선 · 다음 실행 새 세션 + 요약 주입 · 요약 한 번 · 진행 중 거절 / 비우기 실행 없음 ·
  새 세션 / 자동 압축 800k · 설정 끔 · 문턱 아래 / 꺼진 상대 보관 뒤 전달 / 자동 왕복 양쪽 압축 · 되넘김 없음 / 0.4.0 `Pong` · 답 프레임 `ctx` 없음).
  `runner.rs` `parse_context_tokens`. 기존 시험 — `RunOutput` · 프레임 글자에 `ctx` 더함, 스키마 버전 18 → 19.

## 설계 결정 · plan 편차

- **요약 주입 키** — `ctx_seeds.agent` 빈 글자 = 그 대화의 `conversations.session` 주인(받은 대화 = 대화 에이전트, 자동 왕복을 연 쪽 = 내 왕복 에이전트), 멘션은 이름.
- **구분선 글 `exec`** — 이 PC 가 세션을 처리하면 1(대화 줄이 처리), 상대에만 요청하면 0(`sent` → 상대 `done` · `failed` 로 끝). 자동 왕복은 양쪽 다 1 — 상대
  결과 · 거절은 내 글을 바꾸지 않는다(내 처리 결과가 그 글).
- **진행 중 거절** — 같은 대화에 `queued` · `running` · `sent` 구분선 글이 있으면 새 요청 거절(중복 요약 방지).
- **꺼진 상대 · 0.4.0** — `context` 줄은 바로 보내지 않고(`enqueue`) 상대 줄이 보낸다. 줄은 보내기 전 Ping 으로 `ctx` 를 묻는다 — 닿지 않으면 보통 줄처럼 다시,
  모르면 `unsupported` 거절과 같게 정산(내 글 실패 + 코드).
- **자동 왕복 받은 쪽 크기** — 받은 쪽은 연 쪽 세션 크기를 왕복 중에는 모른다(넘김 프레임 `Message` 에 크기 없음 — 상한 기록 `Post` 에만). 받은 쪽 표시는 자기
  세션 기준. 연동 압축은 그대로 된다.

## 검증

- `cargo test --workspace` 실패 0 · `cargo clippy --workspace --all-targets -- -D warnings` Finished.
- 0.4.0 상대 거절은 e2e 시험 없음 — 0.4.0 노드를 띄울 수 없어 `Pong` · 답 프레임의 이전 꼴 읽기만 시험(`old_pong_has_no_context`). 실제 확인은 19 인수 TC.
