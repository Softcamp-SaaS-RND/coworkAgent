# plan — 1z-9.agent-engine

> 근거: [intent](intent.md)(승인 2026-10-23 — 결정 1 ~ 10). **구현자는 이 문서만 보고 일한다.** D·S 는 [masterPlan](../../masterPlan.md)(D16 · D19 · D21 · D22 · D31 · S4).
> 브랜치 `task/1z-9-agent-engine`(작업 폴더 `D:\dev\git\coworkAgent-1z-9`). 기준 커밋: `38c2c73`(main). plan 승인: 2026-10-23(결정 10 — 사용자 "개발 진행해." 로 갈음, 구현자 해석).
> 시안: https://claude.ai/artifact/2BKZnhDQXWmwG2oskYiMfT (시안 B).

## 목표

에이전트마다 실행 도구 Claude Code / Codex 를 등록 때 하나 고르고(필수 · 뒤에 못 바꿈), Codex 에이전트는 `codex exec` 로 답한다. Claude 실행은 로컬 사용과 맞추고 편집 도구만 막는다.
등록 화면 = 시안 B 2단계. 버전 1.3.5.

### 성공 기준(intent 사본)

- A1: 에이전트 등록 = 시안 B 2단계 — 1단계에서 실행 도구(Claude Code / Codex)를 고르지 않으면 2단계 · 등록으로 갈 수 없고, API 도 도구 없는 등록을 거부한다(시험 · 실제 화면).
- A2: 등록된 에이전트의 실행 도구는 API · 웹UI 어디서도 바뀌지 않고, 그 에이전트의 대화 실행은 늘 그 도구다(시험).
- A3: Codex 에이전트(등록 2단계 · 상세)는 Claude Code 옵션 블록(상위 한단계 탐색 · 셸 명령 실행 · 도구)이 비활성 · "Codex 에이전트는 쓰지 않음", Claude 에이전트는 Codex 블록이 비활성(시험 · 실제 화면).
- A4: Codex 에이전트에 묻고 답을 받는다 — `codex exec -C <등록 폴더> -s read-only`(표준 입력 프롬프트) · 새 대화 · 이어 쓰기 · 단계 요약 · 실패 표시(시험 · 실제 Codex 실측).
- A5: Codex 옵션 "Git 저장소 아닌 폴더 허용" 기본 켬 = `--skip-git-repo-check` 붙음, 끄면 빠짐(시험).
- A6: 기존 에이전트 = Claude Code 로 그대로 동작(스키마 27 올림, 회귀 없음).
- A8: Claude 실행 인자 = 결정 9, Codex 실행 = 결정 8(시험).
- A7: 회귀 없음 — PROJECT.md 검증 명령 PASS, 매뉴얼 · README · PRODUCT · masterPlan 반영.

## 설계

### stage 1 — 저장 · 실행기

- core `agents`: `ENGINES = ["claude", "codex"]`(첫 값 = 기존 행 기본), `Agent.engine: String` · `Agent.skip_git: bool`. 스키마 27 = `agents.engine TEXT NOT NULL DEFAULT 'claude'` ·
  `agents.skip_git INTEGER NOT NULL DEFAULT 1`. `add_with(…, engine, skip_git)`(`add` = claude · 켬). `update` 는 engine 을 받지 않는다(바꿀 길 없음), `set_skip_git(id, bool)` — 실행 인자만
  바뀌어 세대 · 세션 그대로(색과 같은 꼴). Codex 행의 `parent_read` · `shell` 은 늘 거짓(API 가 막음).
- agent `runner` — Claude(결정 9): `claude_args` 에서 `--restricted` · `--strict-mcp-config` · `--tools` 를 빼고 `--allowedTools`(Read · Grep · Glob · WebFetch · WebSearch + 셸 켬이면 셸)
  · `--disallowedTools`(Edit · Write · NotebookEdit + 셸 끔이면 셸). `READ_ONLY_TOOLS` → 허용 목록 `ALLOWED_TOOLS`, 새 `EDIT_TOOLS`.
- agent `runner` — Codex(결정 4 · 8 · 2): `codex_args(agent, session)` =
  새 세션 `exec -C <폴더> -s read-only --json [--skip-git-repo-check] -c developer_instructions=<TOML 문자열> -`,
  이어 쓰기 `exec resume --json [--skip-git-repo-check] -c sandbox_mode="read-only" -c developer_instructions=… <thread_id> -`(작업 폴더 = 프로세스 cwd). 모델 · effort 인자 없음.
  개발자 지시 = `CODEX_GUARD_PROMPT`(문의 응답 · 읽기 · 조회 셸 명령 · workflow 수행 괜찮음 · 폴더 밖 읽지 않음 · 편집 · 삭제 · 이동 · 설정 바꾸기 · 외부 전송 금지 · 원격 틀 문구 규칙 · 비밀) + 추가 지시.
  `parse_codex(stdout)` — `thread.started.thread_id` = 세션, 마지막 `agent_message` = 답, `turn.failed` · `error` = 실패, `ctx_tokens` = 없음(턴 합이라 뜻이 다름 — 실측), 턴 수 = 명령 · MCP 항목 수 + 1.
  `count_codex_step` — `item.completed` 의 `command_execution` → `Shell`, `mcp_tool_call` → `MCP`, `web_search` → `WebSearch`. 없는 세션 = stderr `no rollout found` → `SessionMissing`.
  `find_codex()` — Windows: PATH 의 `codex.exe` 가 없으면 npm 전역 `node_modules\@openai\codex\node_modules\@openai\codex-win32-{x64|arm64}\vendor\<triple>\bin\codex.exe`
  (`.cmd` 는 띄우지 않는다 — 배치 인자 처리), 그 밖 = PATH `codex`. `MIN_CODEX` = 0.163.0(실측 버전), `codex --version` = `codex-cli X.Y.Z`. 실행 파일이 없으면
  "이 PC 에서 Codex CLI 를 찾지 못했습니다"(결정 6). 환경 변수는 그대로 물려준다(로컬과 맞춤 — Codex 자기 셸 정책이 KEY · TOKEN 변수를 거른다).
  `run_codex(program, input)` · `run_agent(input)`(엔진에 따라 claude / codex) — `main.rs` 는 `run_agent`.
- agent `chat`: 실행 model · effort 는 Codex 에이전트면 빈 값 → 답 기록 · 상대에 보낼 meta 에 넣지 않음(`None`).

### stage 2 — API · 웹UI · 문서

- app `chat_api`: `NewAgent.engine: String`(필수 — 없으면 422, `ENGINES` 밖이면 400) · `skip_git: Option<bool>`(기본 켬). Codex + `parent_read` · `shell` 참 = 400.
  `AgentPatch.engine` 있으면 400 "실행 도구는 바꾸지 않는다 — 새로 등록", `skip_git` = `set_skip_git`, Codex 에이전트에 `parent_read` · `shell` 참 = 400. `agent_json` + `engine` · `skip_git`.
- 웹UI(시안 B): 등록 상세 = 1단계 `#agent_engine_step`(도구 타일 2 — Claude Code · Codex, 각 비교 3줄) → 고르면 2단계 `#agent_form`(도구 칩 · "도구 다시 고르기" · 이름 · 폴더 ·
  좌우 블록 `Claude Code 옵션`(상위 탐색 · 셸 · 도구 줄) / `Codex 옵션`(스위치 `agent_skip_git` "Git 저장소 아닌 폴더 허용" 기본 켬 · 읽기 범위 줄) — 안 고른 도구 블록 = `fieldset disabled` ·
  "… 에이전트는 쓰지 않음" · 추가 지시). 상세 = 머리 이름 옆 도구 칩 · 부제 "이름 · 실행 도구는 등록 뒤 바꿀 수 없음" · 같은 좌우 블록(초안 `agentDrafts` 에 `skip_git`). 목록 줄 부제 =
  "{Claude Code|Codex} · 폴더". `view.js` `engineLabel(e)` · `agentEngineBlocks(engine)`(블록마다 `disabled` · 문구).
- 문서: [an_app](../../manual/dev/an_app.md)(agents API) · [an_core](../../manual/dev/an_core.md)(스키마 27) · [web-ui](../../manual/dev/web-ui.md) · [developer-guide](../../manual/dev/developer-guide.md)
  (실행 인자 · Codex) · [사용자 가이드](../../manual/user/user-guide.md)(폴더 밖 읽기 · Codex 필요 PC) · PRODUCT · README(1.3.5) · masterPlan D16 · D19 · D21 · D22 · D31 · 개정 기록(종결).

### 바꾸는 시험(의도된 변경 — 약화 아님)

- `agent/tests/runner.rs`: `args_follow_d22`(`--restricted` · `--strict-mcp-config` 있음 → 없음 · `--disallowedTools` 편집 도구), `tools_are_all_read_only`(→ 허용 · 막는 목록), `shell_adds_tools`.
- `restricted_canary` · `parent_read_canary` · `tools_canary` · `shell_canary`(`#[ignore]` 실측): 폴더 밖 읽기 · 훅 금지를 재던 것 → 결정 9 동작(편집 막힘 · 셸 끔이면 셸 없음)으로.
- Agent 리터럴(시험 7곳) + `engine` · `skip_git`. `chat_api` 에이전트 등록 시험 본문에 `engine`.
- 새 시험: `codex_args`(새 · 이어 쓰기 · Git 스위치 · 개발자 지시 TOML · 모델 인자 없음) · `parse_codex` · `count_codex_step` · `codex_session_gone` · 스키마 27 기본값 ·
  API(`agents_engine` — 도구 없음 422 · 모르는 값 400 · 바꾸기 400 · Codex + 셸 400 · skip_git 기본 켬 · 끔) · `view.test.mjs`(`engineLabel` · `agentEngineBlocks`) · `codex_live`(`#[ignore]` 실측).

## 평가 항목

- [ ] E1: A7 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E2: A7 — clippy 경고 0
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: pending
- [ ] E3: A1·A3 — 웹UI 로직 시험(도구 라벨 · 블록 비활성)
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# fail 0/
  EVIDENCE: pending
- [ ] E4: A4·A5·A8 — 실행기 시험(Claude 인자 결정 9 · Codex 인자 · 출력 해석 · 단계 · 없는 세션)
  CHECK: cargo test -p an-agent --test runner
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E5: A1·A2·A3·A5·A6 — 에이전트 API(도구 필수 · 못 바꿈 · Codex + Claude 옵션 거부 · Git 스위치 · 기존 행 claude)
  CHECK: cargo test -p coworkagent-app --test chat_api agents_
  EXPECT: /test result: ok\. [2-9] passed; 0 failed/
  EVIDENCE: pending
- [ ] E6: A1·A3 — 화면 원본 literal(1단계 타일 · 2단계 블록 · 칩 · 문구)
  CHECK: cargo test -p coworkagent-app --test ux_literals --test css_tokens
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E7: A7 — 문서 규약 · 크레이트 경계 · 버전 · 매뉴얼 반영
  CHECK: python tools/check_ddd_docs.py && python tools/check_crates.py && python -c "import pathlib as P;r=lambda f:P.Path(f).read_text(encoding='utf-8');print('ver=%d app=%d core=%d dev=%d guide=%d'%(sum(r('crates/%s/Cargo.toml'%c).count('version = \x221.3.5\x22') for c in ('agent','app','core','net')),r('docs/manual/dev/an_app.md').count('engine'),r('docs/manual/dev/an_core.md').count('skip_git'),r('docs/manual/dev/developer-guide.md').count('codex exec'),r('docs/manual/user/user-guide.md').count('Codex')))"
  EXPECT: /ver=4 app=[1-9][0-9]* core=[1-9][0-9]* dev=[1-9][0-9]* guide=[1-9][0-9]*/
  EVIDENCE: pending
- [x] E8: A1 ~ A6 · A8 — 실제 화면 · 실제 실행: 디버그 빌드에서 등록 1단계(도구 안 고르면 등록 없음) → Codex 고름 → 2단계 Claude 블록 비활성 · Git 스위치 켬 → 등록 → 상세 칩 · 블록 ·
  Git 스위치 끔/켬 저장, Claude 에이전트 상세 Codex 블록 비활성, 실제 `codex_live`(새 세션 답 · 이어 쓰기 기억 · 단계 요약) · 실제 Claude 실행(결정 9 인자 — Write 막힘)
  EVIDENCE: Claude(구현자) 2026-10-23 — 디버그 빌드 `start --parent <대기 프로세스>` + `COWORKAGENT_DATA_DIR`=scratchpad `ui1z9\data`(빈 데이터), ego-browser 1440 × 900 · 900 × 900 DOM 판독 · 스크린샷.
  `#settings/agent_new` = "1 / 2 — 실행 도구 고르기" · 도구 타일 보임 · 폼 · 등록 줄 안 보임(도구 없이 등록 불가) → Codex 타일 = "2 / 2 — 이름 · 폴더 · 옵션" · 칩 "Codex" · Claude 블록
  `disabled` · "Codex 에이전트는 쓰지 않음" · Codex 블록 켬 · Git 스위치 켬 · 초점 `agent_name` → 다시 고르기 → Claude 타일 = 칩 "Claude Code" · Codex 블록 `disabled` → Codex 로 "cx" 등록 =
  "등록 완료" · 목록 줄 "Codex · C:\…\inside" · 상세 칩 "Codex" · 블록 [Claude 꺼짐 · Codex 켬] · 스위치 상위 · 셸 `:disabled` · Git 켬 → Git 끔 저장 → 새로 읽어 다시 고름 = Git 끔 유지 ·
  꺼진 "상위 한단계 탐색 허용" 눌러도 그대로 → Claude 로 "cl" 등록 = 상세 칩 "Claude Code" · Codex 블록 꺼짐 "Claude Code 에이전트는 쓰지 않음". 900px = 블록 1열(처음 2열 — 규칙 순서 → 고침).
  실제 실행: `cargo test -p an-agent --test codex_live -- --ignored --nocapture`(codex-cli 0.163.0) → 새 세션 "노을-42" · 단계 `{"Shell": 1}` · 폴더에 w.txt 없음 · resume 같은 thread "노을-42" ·
  없는 thread `SessionMissing` · ok. `cargo test -p an-agent --test tools_canary -- --ignored --nocapture`(claude 2.1.309, 결정 9 인자) → init 도구에 Read · Grep · Glob · WebFetch · WebSearch 있음,
  Edit · Write · NotebookEdit 없음, 셸 끔 = Bash · PowerShell 없음 · 켬 = 있음 · ok. intent 시험 실측(결정 9) — 폴더 밖 Read 성공 · Write "disabled for this session". 끝에 `coworkAgent stop`.

## 개정 기록

- 2026-10-23(구현 중 — 설계만, 평가 항목 · EXPECT 그대로): ① API(stage 2 앞부분)를 stage 1 커밋에 — 등록 함수 인자가 바뀌어 함께 움직임. ② `READ_ONLY_TOOLS` 는 그대로 두고 `WEB_TOOLS` · `EDIT_TOOLS` 를 더함
  (`ALLOWED_TOOLS` 새 이름 없음). ③ `find_codex` 는 npm 실제 실행 파일 → `codex.exe`(PATH) 순(PATH 먼저 훑기 없음). ④ `restricted_canary` · `parent_read_canary` 삭제(잴 동작이 결정 9 로 사라짐),
  `ux_literals` CRLF 정규화(#1z-8 기존 결함).
