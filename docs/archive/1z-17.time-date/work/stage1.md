# stage 1 — 설정 저장소 · 화면(web) · 버전

## 목표

[plan](../plan.md) stage 1 — 서버 설정 `ui_date_format`, 화면 시각 함수가 날짜 표시방식을 따름(실행 상세 초 · 대화 정보 포함), 설정 > 화면 '날짜 표시방식' 칸,
설정 타일 1열 · 최대 900px, 레일 상태 점 14px · 12px, 버전 1.4.2. (plan 개정) 에이전트 상세 저장 · 삭제 창, 시스템 프롬프트 추가 지시 맨 앞.

## 산출 파일

- `crates/core/src/run_opts.rs` — `DATE_FORMAT_KEY` · `DATE_FORMATS` · `date_format`.
- `crates/app/src/notify.rs` — 패치 필드 `ui_date_format` · 모르는 값 거절 · 저장 · `settings_json`, `Notifier::date_format`.
- `crates/app/web/js/logic/format.js` — `DATE_FORMATS` · `setDateFormat` · `dateText`(`dayLabel` 이 씀) · `formatTime(unix, { seconds, fmt, now })`.
- `crates/app/web/js/logic/message.js` `runSteps` — 요청 · 응답 시각 `formatTime(t, { seconds: true })`. `js/logic/conv.js` `convInfoRows` — `when` = `formatTime(t, { now })`.
- `crates/app/web/js/settings/prefs.js` — `PREFS` 에 `ui_date_format`, `fillPrefs` → `setDateFormat`, 바꿔 저장하면 `[data-sig]` 비움 + `refresh()`.
- `crates/app/web/index.html` — 화면 절 타일 '날짜 표시방식'(`#ui_date_format`, 옵션 넷).
- `crates/app/web/app.css` — `.set-tiles` 1열 · `max-width: 900px`(960px 미디어 질의에서 뺌), `.rail-dot` 14px · 작은 타일 12px.
- `crates/{core,net,agent,app}/Cargo.toml` · `Cargo.lock` — 1.4.2.
- `crates/app/web/js/logic/agent.js` — `agentSavedText` · `agentSaveFailText` · `agentDeleteAsk`. `js/settings/agents.js` — 저장 성공 · 실패 `window.alert`, 삭제 `window.confirm` 뒤에만.
- `crates/agent/src/runner.rs` — `extra_prompt`(추가 지시 머리), `claude_args` · `codex_args` 가 그것으로 시작.
- 시험: `crates/app/web/view.test.mjs`(format_time 새 시험 · run_steps · conv_info 기대값), `crates/app/tests/ux_api.rs`(`date_format_setting_persists`),
  `crates/app/tests/ux_literals.rs`(#1z-8 2열 literal → 1열 · 900px, PRESENT 10행), `view.test.mjs` agent_dialogs,
  `crates/agent/tests/runner.rs`(`guard_prompt_always` · `codex_args_follow_decisions` — 추가 지시로 시작 · 가드 뒤).

## 설계 결정

- 방식은 `format.js` 모듈 변수 하나(`setDateFormat`) — 호출처 15곳 남짓을 고치지 않고 `formatTime(x)` 그대로 둔다. 시험은 `fmt` · `now` 를 넘겨 고정.
- `formatClock` 은 그대로 — 디버깅 로그가 "MM-DD " 를 붙여 쓴다(제외 범위). 실행 상세만 `formatTime(…, { seconds: true })` 로 옮김.
- 다시 그리기: 목록 · 대화 · 미리보기는 `data-sig` 가 같으면 건너뛰므로(`needsRender` · `openConversation`) 방식을 바꿀 때 서명을 비우고 `refresh()` —
  숨은 화면은 다시 열 때 그린다. 처음 불러올 때(`showSettings`, 첫 그리기 전)는 `setDateFormat` 만.
- 저장 실패 창은 `act` 안에서 띄우고 다시 던진다 — 상세 아래 글자("에이전트 저장 실패: …")도 그대로 남는다. 삭제 실패는 창 없이 글자(intent 제외).
- 추가 지시 머리는 함수 하나(`extra_prompt`)로 Claude · Codex 가 함께 쓴다 — 두 곳에 같은 머리 글자를 두지 않게.
- 한계(구현자 해석): `relative`(오늘 · 어제)는 자정을 넘겨도 그린 글자가 그대로다 — 서명이 바뀌거나 화면을 다시 열 때 바뀐다. 날짜 구분선(`dayBreaks`)도 같은 동작이라 맞춤.

## 테스트 결과

```text
node --test --test-reporter=tap "crates/app/web/*.test.mjs" → RED 먼저: "does not provide an export named 'DATE_FORMATS'" (# fail 1) → # pass 178 / # fail 0
cargo test --workspace                       → test result 84개 전부 ok (첫 실행 date_format_setting_persists 1 FAILED — 시험 원본의 split 글자 깨짐, 고친 뒤 ok)
cargo test -p coworkagent-app --test ux_api date_format_setting_persists → test result: ok. 1 passed; 0 failed
cargo clippy --workspace --all-targets -- -D warnings → Finished
(plan 개정분) RED 먼저: runner guard_prompt_always · codex_args_follow_decisions FAILED(2), node "does not provide an export named 'agentDeleteAsk'"
  → cargo test --workspace: test result 84개 전부 ok · node # pass 179 / # fail 0
python tools/check_code_tidy.py              → PASS — 제품 원본 116개 · 최대 968줄 · 이력 표식 0
python tools/check_crates.py                 → crates=4 edges=6 violations=0
```

## 다음 단계

stage 2(`work/stage2.md`) — 디버그 빌드 실제 화면(두 테마) · 문서(web-ui · 사용자 가이드 · PRODUCT · README).
