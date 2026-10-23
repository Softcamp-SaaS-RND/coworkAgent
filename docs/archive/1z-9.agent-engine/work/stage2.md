# stage 2 — 웹UI(시안 B) · 문서

## 목표

plan "설계 · stage 2" — 등록 2단계(실행 도구 타일 → 폼, 도구 없으면 등록 없음 — A1), 상세 실행 도구 칩 · 좌우 옵션 블록(안 쓰는 도구 비활성 — A3), Codex Git 확인 건너뜀 스위치(A5),
매뉴얼 · README · PRODUCT · 보안 표 반영(A7). API 는 stage 1 에서 함께 했다(plan 개정 ①).

## 산출 파일

- web `index.html` — `#agent_new` = 1단계 `#agent_engine_step`(타일 `#agent_engine_claude` · `#agent_engine_codex` — 읽기 · 셸/Git · 모델 비교) · 2단계 `#agent_form`(처음 `hidden` — 도구 칩 ·
  "도구 다시 고르기" · 좌우 `fieldset` `#agent_claude_opts` / `#agent_codex_opts` · 스위치 `agent_skip_git` 기본 켬) · 단계별 아래 줄 `#agent_step1_foot` / `#agent_step2_foot`.
- `view.js` — `WEB_TOOLS` · `EDIT_TOOLS` · `TOOLS_NOTE`(결정 9 문구) · `ENGINES` · `engineLabel` · `agentEngineBlocks` · `SKIP_GIT_HINT` · `CODEX_READ_NOTE` · `agentPatch` 의 `skip_git` ·
  새로 만들기 "에이전트 등록" 초점 `agent_engine_claude`.
- `app.js` — `showNewEngine`(단계 · 칩 · 블록 비활성 · 문구, Codex 면 Claude 스위치 끔) · 타일 · 다시 고르기 · 취소, 등록 본문 `engine`(+ Codex `skip_git`) · 도구 없으면 보내지 않음,
  목록 부제 "도구 · 폴더", `agentDetail` 실행 도구 칩 · 좌우 블록 · Git 스위치(초안 `skip_git`).
- `app.css` — `.engine-*`(타일 · 칩 · 블록 · 비활성 점선), 960px 아래 1열(정의 뒤 `@media`).
- 시험: `view.test.mjs` `agent_engine` · 새로 만들기 초점 · `ux_literals.rs` #1z-9 행 13 · 도구 안내 행 개정.
- 문서: developer-guide(실행 인자 결정 9 · Codex 절 · 보안 회귀 검사 · 위협 표) · an_app(agents API) · an_core(스키마 27) · web-ui · user-guide §6 · README(1.3.5) · PRODUCT ·
  overview/security-review(막는 것 표).

## 설계 결정

- 2단계 폼은 처음부터 DOM 에 두고 `hidden` 만 바꾼다(1z-8 의 상세 칸 폼과 같은 꼴) — 등록 · 취소 뒤 1단계로 되돌림(`showNewEngine(null)`).
- 안 쓰는 블록은 `fieldset disabled` — 안의 스위치가 한 번에 꺼진다(클릭해도 바뀌지 않음 — 실제 화면 확인). Codex 를 고르면 Claude 스위치 값을 끔으로(서버가 Codex + 셸을 400).
- 960px 규칙을 기존 `@media` 블록에 넣었더니 뒤에 오는 `.engine-*` 정의가 덮음(실제 화면 900px 에서 2열) → 정의 뒤에 별도 `@media`.

## 테스트 결과

- `node --test --test-reporter=tap "crates/app/web/*.test.mjs"` → `# pass 171` · `# fail 0`.
- `cargo test -p coworkagent-app --test ux_literals --test css_tokens` → 3 · 5 passed.
- `cargo test --workspace --no-fail-fast` → FAILED 0 · ok 82. `cargo clippy --workspace --all-targets -- -D warnings` → Finished.
- 문서: `check_ddd_docs` PASS · `check_crates` violations=0 · `check_readme` ok · E7 수치 `ver=4 app=2 core=2 dev=2 guide=10`.
- 실제 화면(E8) — plan 평가 항목 E8 EVIDENCE.
