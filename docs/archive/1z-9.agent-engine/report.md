# report — 1z-9.agent-engine

종결: 2026-10-23 · 브랜치 `task/1z-9-agent-engine` → main(squash — 사용자 승인 뒤)

## 결과

에이전트마다 실행 도구 **Claude Code / Codex** 를 등록 때 하나 고른다(필수 · 등록 뒤 바꿀 수 없음). Codex 에이전트는 그 PC 의 `codex exec` 로 답한다. Claude 실행은 로컬 사용과 맞추고 편집 도구만 막는다
(사용자 지시 "로컬 에이전트가 동작하는 것과 최대한 맞춤. 편집, 삭제만 제외." · "--strict-mcp-config 옵션도 제거"). 버전 1.3.5 · 스키마 27.

- 등록(시안 B — 사용자 선택, 권장안 A 아님): 1단계 도구 타일(Claude Code · Codex — 읽기 · 셸/Git · 모델 비교) → 2단계 도구 칩 · "도구 다시 고르기" · 이름 · 폴더 · 좌우 옵션 블록
  (안 고른 도구 블록 = 점선 · 비활성 · "… 에이전트는 쓰지 않음") · 추가 지시. 상세 = 실행 도구 칩(읽기 전용) · 같은 좌우 블록. 목록 부제 "도구 · 폴더".
- Codex: `codex exec -C <폴더> -s read-only --json` · 이어 쓰기 `exec resume <thread_id>` · 가드 = `-c developer_instructions` · "Git 저장소 아닌 폴더 허용" 스위치 기본 켬(`--skip-git-repo-check`) ·
  사용자 Codex 설정 그대로 · 모델 인자 없음. 세션 = `thread_id`, 완료 이벤트 없거나 비정상 종료 = 실패, 없는 thread = 새 세션 한 번 더.
- Claude(D22 개정): `--restricted` · `--strict-mcp-config` · `--tools` 뺌, `--allowedTools Read,Grep,Glob,WebFetch,WebSearch`(+ 셸), `--disallowedTools Edit,Write,NotebookEdit`(+ 셸 끔이면 셸).
- API: `engine` 필수(없으면 422 · 밖 400) · PATCH `engine` 400 · Codex 에 상위 탐색 · 셸 400 · `skip_git`(세대 그대로).

| 성공 기준 | 판정 | 근거 |
|---|---|---|
| A1 2단계 · 도구 없이 등록 불가(웹 · API) | 충족 | E5(`agents_engine_api` 422) · E6 · E3 · E8 |
| A2 실행 도구 못 바꿈 · 늘 그 도구 | 충족 | E5(PATCH 400) · core `schema27_agent_engine`(고쳐도 그대로) · `run_agent` 분기 |
| A3 안 쓰는 도구 블록 비활성 · 문구 | 충족 | E3(`agent_engine`) · E6 · E8(`:disabled` · 눌러도 그대로) |
| A4 Codex 답 · 이어 쓰기 · 단계 · 실패 | 충족 | E4(`codex_args` · `parse_codex` · `codex_abnormal_exit_is_error`) · E8(`codex_live` 실측) |
| A5 Git 스위치 기본 켬 · 끄면 빠짐 | 충족 | E4 · E5 · E8(끔 저장 유지) |
| A6 기존 에이전트 Claude 그대로 · 스키마 27 | 충족 | core `schema27_agent_engine` · E1 |
| A7 회귀 없음 · 문서 | 충족 | E1 · E2 · E7 |
| A8 Claude 결정 9 · Codex 결정 8 인자 | 충족 | E4(`args_follow_d22` · `codex_args_follow_decisions`) · E8(`tools_canary` 실측) |

- 적대적 평가 2 라운드(상한, Orca 가 라운드마다 새로 띄운 Codex) — R1 medium 1(Codex 중간 답을 비정상 종료 뒤에도 성공 처리 → `turn.completed` · 종료 코드 검사 · 가짜 codex 회귀 시험) 반영,
  R2 approve(지적 없음 — R1 수정 확인). 미조치 없음.
- 실제 화면에서 스스로 찾아 고침 — 960px 아래 블록 2열(규칙 순서). 기존 결함 — `ux_literals` 의 줄바꿈 문구가 CRLF 체크아웃에서 실패(#1z-8 — 새 작업 폴더에서 드러남, 정규화).
- 구현자 해석(사용자 미확인):
  - MCP 미리 허용 안 함(결정 9 — 선택 화면 미리보기에는 `mcp__*` 를 적었으나 쓰기 · 지우기 MCP 가 섞여 "편집, 삭제만 제외" 와 어긋남 → 사용자 settings 허용 규칙만).
  - Codex 가 없는 PC 에 Codex 에이전트를 등록하면 등록은 되고 답할 때 "받는 PC 에서 Codex CLI 를 찾지 못했다"(결정 6 — 사용자 답은 "묻는 PC 에는 없어도 됨").
  - Codex 실행 환경 변수는 그대로 물려줌(Claude 셸 켬처럼 거르지 않음 — 로컬과 맞춤, Codex 셸 정책이 KEY · TOKEN 변수를 거름).
  - Codex 창 사용량(컨텍스트) 표시 없음 — `turn.completed.usage` 는 턴 합이라 뜻이 다름. 자동 압축 기준에 닿지 않는다.
  - 승인 = 인터뷰 답과 "개발 진행해" 로 intent · plan 갈음, `restricted_canary` · `parent_read_canary` 삭제(잴 동작이 사라짐 — plan 개정 ④).
- 보안 영향(사용자가 실측을 보고 고름 — intent 결정 5 · 9): Claude · Codex 모두 **등록 폴더 밖도 읽는다**. Claude 실행에 그 PC 사용자의 MCP · 스킬 · 훅이 걸린다(원격 요청으로 훅이 돈다).
  폴더 범위 · 편집 금지 중 막는 장치는 편집 도구 차단(Claude) · 읽기 전용 샌드박스(Codex)뿐이고 나머지는 가드 문구다 — 매뉴얼 · 보안 표 · 사용자 가이드에 적음.

## 산출

- core: `agents::ENGINES` · `CODEX` · `WEB_TOOLS` · `EDIT_TOOLS` · `Agent.engine` · `skip_git` · `add_with(…, engine, skip_git)` · `set_skip_git`, 스키마 27.
- agent: `runner::claude_args`(결정 9) · `CODEX_GUARD_PROMPT` · `run_agent` · `codex_args` · `parse_codex` · `codex_session_gone` · `count_codex_step` · `find_codex` · `MIN_CODEX` · `parse_codex_version` ·
  `run_codex`, `chat` 의 Codex model · effort 빈 값 · `Failure::run(e, codex)`. app: `main` `run_agent`, `chat_api` engine · skip_git.
- web: `index.html` 등록 2단계 · `view.js` `engineLabel` · `agentEngineBlocks` · `TOOLS_NOTE` · `agentPatch` skip_git · `app.js` `showNewEngine` · `agentDetail` 블록 · `app.css` `.engine-*`.
- 시험: core `schema27_agent_engine` · agent `runner`(17) · 새 `codex_live`(`#[ignore]`) · `tools_canary` 개정 · `shell_canary` 개정 · app `agents_engine_api` · `view.test.mjs` `agent_engine` · `ux_literals` #1z-9.
- 문서: developer-guide · an_app · an_core · web-ui · user-guide §6 · README(1.3.5) · PRODUCT · overview/security-review · masterPlan D16 · D19 · D21 · D22 · D31 · 종결 목록 · 개정 기록.
  시안 캔버스 https://claude.ai/artifact/2BKZnhDQXWmwG2oskYiMfT.

## 남은 것

- main squash 반영 · push · 1.3.5 서명 릴리스 — 사용자 결정.
- 실제 두 PC 왕복(상대가 Codex 에이전트에 묻기 · 자동 왕복 · 멘션) · macOS(`codex` PATH) — 19 acceptance · Mac 실기 조건 대기(D28).
- 후보: Codex 모델 · effort 고르기(결정 3 — 다음 Task), Codex 창 사용량 표시.
