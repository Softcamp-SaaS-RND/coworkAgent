# stage1 — 모델 드롭다운 · 탐색 도구 고정 · 스키마 14

> 근거: [plan](../plan.md) Stage 1. 사용자 검증은 D28 — 19.prototype-acceptance 에서 일괄.

## 목표

모델은 `opus` · `sonnet` · `haiku` 중 하나만 저장 · 실행(A1), 에이전트 도구는 모든 실행에 읽기 전용 5개로 고정하고 도구 칸을 없앤다(A2). 스키마 14 로
요청 상태 열을 더하고 이전 완료 표시 · 목록 밖 모델을 옮긴다(A1 · A6 — 요청 동작은 Stage 2). 대상 gate: E1 · E2 · E3 · E6(2/4) · E7 · E4(1/4).

## 산출 파일

- `crates/core/src/run_opts.rs` — `MODELS` · `valid_model`(목록 포함 여부).
- `crates/core/src/db.rs` — 스키마 14: `messages.req_state` · `req_note` · `req_at`, 요청 글 `done_at` → `done` · 나머지 → `wait`, 목록 밖
  `settings.claude_model` 삭제 · `peers.model` → `NULL`, outbox `kind = 'done'` 행 삭제(평가 라운드 1 에서 `ReqState` 프레임으로 바꾸기로 고침 — plan 개정 3).
- `crates/core/src/agents.rs` — `Agent.tools` · `DEFAULT_TOOLS` · `read_only` · `unknown_tools` 삭제, `add` · `update` 의 도구 인자 삭제(새 행은
  `agents.tools` 열에 5개를 적어 둔다 — 열은 `NOT NULL`, 읽지 않음).
- `crates/agent/src/runner.rs` — `--tools` · `--allowedTools` = `READ_ONLY_TOOLS.join(",")`.
- `crates/app/src/chat_api.rs` — 에이전트 JSON `tools` 없음 · `checked_tools` 삭제 · `NewAgent` 에 `deny_unknown_fields`. `notify.rs` · `peers_api.rs` —
  모델 오류 문구 "{목록} 중 하나".
- 웹UI — `index.html` 전역 모델 `<select>` · `datalist` 삭제 · 등록 폼 도구 칸 → 안내, `view.js` `MODELS` · `TOOLS_NOTE`, `app.js` 상대별 모델
  `<select>`(첫 항목 "기본값(…)") · `toolChecks` · `chosenTools` 삭제 · 편집 칸 도구 → 안내 · "내 에이전트 도구 편집" → "내 에이전트 편집".
- 테스트 — `core/tests/requests.rs`(새, 1/4 — 스키마 14) · `app/tests/models_tools.rs`(새, 2/4 — 모델 목록 · 도구 필드 없음). 바뀐 시그니처 ·
  동작에 맞춘 기존 시험: `agents::add` 4 인자(17 파일) · `runner.rs`(`tools_are_all_read_only` · `args_follow_d22` 5개) · `store.rs`
  (`read_only_tools_filter` 삭제 — 함수 없음) · `paths.rs`(스키마 14 · `update` 인자) · `chat_api.rs` · `ux_api.rs`(도구 입력 → 없음).

## 설계 결정

- **plan 개정 1** — 코드 대조 결과 ① `resolve` 는 이미 목록 밖 값을 쓰지 않고 기본값(D22 "실행 실패" 는 DB 오류) → 그대로. ② 도구 입력은 axum 본문
  거절로 422(수정 API 는 원래 `deny_unknown_fields`) — 등록도 맞췄다.
- **`agents.tools` 열은 남긴다** — `NOT NULL` 이라 새 행에 5개를 적는다. 열 삭제(`DROP COLUMN`)는 되돌리기 어려운 스키마 변경이라 하지 않음(D32 —
  사용자 데이터는 보존).
- **모델 표기 = 별칭 셋** — 전체 ID(`claude-opus-5-5` 등) · `[1m]` 접미는 거부. 이유: 사용자 지시 "opus, sonnet, haiku 등 … 드롭다운".
- `cargo fmt` 를 한 번 돌렸다가 이 Task 와 무관한 파일의 서식까지 바뀌어 되돌렸다(저장소가 rustfmt 일괄 적용 상태가 아님) — 손댄 줄만 남김.

## 테스트 결과

```text
$ cargo test --workspace            → 304 passed, 0 failed (합)
$ cargo clippy --workspace --all-targets -- -D warnings → Finished
$ node --test --test-reporter=tap "crates/app/web/*.test.mjs" → # pass 42 # fail 0
$ cargo test -p an-agent --test runner          → test result: ok. 8 passed; 0 failed
$ cargo test -p coworkagent-app --test models_tools → test result: ok. 2 passed; 0 failed (E6 2/4 — 나머지 Stage 2 · 3)
$ cargo test -p an-core --test requests         → test result: ok. 1 passed; 0 failed (E4 1/4 — 나머지 Stage 2)
```

## 다음 단계

Stage 2 — 요청 = 실행 없음(`inbound_message` · `post_with`) · `set_req_state` · `Request::ReqState` · outbox `req_state` · `requests` 시험(core 3 ·
agent 6) · `models_tools` 요청 거르기.
