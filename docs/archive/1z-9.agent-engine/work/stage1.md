# stage 1 — 저장 · 실행기 · API

## 목표

plan "설계 · stage 1" — 실행 도구 저장(스키마 27), Claude 인자 결정 9, Codex 실행기(결정 2 · 3 · 4 · 5 · 8), 실행 도구별 실패 문구. API(stage 2 앞부분)도 함께 —
등록 인자가 바뀌어 빌드가 API 와 같이 움직인다(plan 개정 ①).

## 산출 파일

- core `agents.rs` — `ENGINES` · `CODEX` · `WEB_TOOLS` · `EDIT_TOOLS`, `Agent.engine` · `skip_git`, `add_with(…, engine, skip_git)` · `set_skip_git`. `db.rs` 스키마 27.
- agent `runner.rs` — `claude_args`(결정 9) · `CODEX_GUARD_PROMPT` · `run_agent` · `codex_args` · `parse_codex` · `codex_session_gone` · `count_codex_step` · `find_codex` ·
  `MIN_CODEX` · `parse_codex_version` · `run_codex`. `chat.rs` — Codex 면 model · effort 빈 값(답 · meta 에 싣지 않음), `Failure::run(e, codex)`.
- app `main.rs`(`run_agent`) · `chat_api.rs`(`NewAgent.engine` 필수 · `skip_git` · `AgentPatch.engine` 400 · `skip_git` · `codex_options` · `agent_json` + engine · skip_git).
- 시험: core `paths.rs` `schema27_agent_engine` · agent `runner.rs`(결정 9 인자 · `codex_args_follow_decisions` · `parse_codex_json` · `count_codex_steps` · `codex_session_and_version`) ·
  새 `codex_live.rs`(`#[ignore]`) · `tools_canary`(허용 · 막힘 대조로) · `shell_canary`(셸 끔 대조 뺌) · `restricted_canary` · `parent_read_canary` 삭제 · app `chat_api.rs` `agents_engine_api` ·
  등록 본문에 `engine`(chat_api · hardening · models_tools · ux_api) · `ux_literals.rs` CRLF 정규화.
- 버전 네 크레이트 1.3.5.

## 설계 결정

- 결정 9 의 MCP: 미리 허용하지 않음(intent 결정 9 구현자 해석 — 쓰기 · 지우기 MCP 도구가 섞임). 허용 = 읽기 도구 + 사용자 settings 규칙.
- Codex 세션 id 는 우리가 못 정한다 — `Session::New(id)` 의 id 는 무시하고 `thread.started.thread_id` 를 `RunOutput.session` 으로 돌려 기존 기록 경로(`set_session` ·
  `set_agent_session`)에 그대로 남긴다. 다음 실행은 `Session::Resume(thread_id)`.
- Codex 출력 줄은 답 · 세션 · 실패 줄만 남긴다(명령 출력 항목은 크다), 턴 수는 단계 셈에서. `ctx_tokens` 없음(턴 합 — 실측).
- `find_codex` 는 npm 전역 실제 실행 파일을 먼저(`codex.cmd` 는 배치 — 인자 처리 위험), 없으면 `codex.exe`(PATH). PATH 를 먼저 훑는 설계(plan)는 줄임 — Rust `Command` 가 PATH 의 `.exe` 를 찾는다.
- 지운 시험: `restricted_canary`(폴더 밖 읽기 · 프로젝트 훅 막힘) · `parent_read_canary`(상위 밖 거부) — 결정 9 로 사라진 동작이라 잴 것이 없다. 편집 막힘 · 셸 끔은 `tools_canary` 가 잰다.
- 기존 결함(#1z-8): `ux_literals` 의 줄바꿈 든 문구가 CRLF 체크아웃(`core.autocrlf=true`)에서 실패 — 새 작업 폴더에서 드러남, `\r\n` → `\n` 정규화로 고침.

## 테스트 결과

- `cargo test --workspace --no-fail-fast` → `test result: FAILED` 0줄 · ok 83줄.
- `cargo test -p an-agent --test runner` → `test result: ok. 16 passed; 0 failed`.
- `cargo test -p an-agent --test codex_live -- --ignored --nocapture`(실제 codex 0.163.0) → 1st `text: "노을-42\nw.txt 생성 안 함: …"`, `turns: Some(2)`, steps `[StepDetail { turns: 2, tools: {"Shell": 1}, secs: 19 }]`,
  2nd(resume) `text: "노을-42"` · 같은 session, 없는 thread = `SessionMissing` → `ok. 1 passed`.
- `cargo test -p an-agent --test tools_canary -- --ignored --nocapture`(실제 claude 2.1.309) → 셸 끔 목록에 Read · Grep · Glob · WebFetch · WebSearch 있음 · Edit · Write · NotebookEdit · Bash · PowerShell 없음,
  셸 켬 = Bash · PowerShell 있음 → `ok. 1 passed`.
- `cargo clippy --workspace --all-targets -- -D warnings` → Finished.
