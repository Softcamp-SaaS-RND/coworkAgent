# stage2 — API · 설정 · 화면(머리 단추 · 메뉴 · 구분선) · 문서 · mock · 실제 claude

> 근거: [plan](../plan.md) Stage 2.

## 목표

해석 2 · 3 · 6 · 11(설정 화면)과 문서 · mock 화면 · 실제 claude 1회. 대상 gate: E3 ~ E9.

## 산출 파일

- `crates/app/src/chat_api.rs` — `POST /api/conversations/{id}/context {op}`(틀린 값 400 · 없는 대화 404 · 진행 중 409 — `Chat::context` 의 `AlreadyExists`) ·
  `message_json` 에 `ctx_tokens` · `ctx_op`. `notify.rs` `SettingsPatch.auto_compact` · `Notifier::auto_compact` · `settings_json`.
- `crates/app/web/view.js` — `CONTEXT_LIMIT` · `contextSize`(끝난 구분선 뒤 세션별 마지막 값의 최대 · 구분선 뒤 답 없음 0 · 모름 `null`) · `contextLabel`
  ("42k / 1M" · `ok` · `warn` 60% · `high` 80% · `unknown`) · `contextBusy` · `ctxMenu` · `contextNote`(종류 · 상태 · 실패 사유).
- `crates/app/web/app.js` — `renderCtx`(머리 표시 — `renderConvHead` 끝) · `openCtxMenu` · `ctxConfirm`(확인 한 번 → `POST …/context` → 대화 다시 읽기) ·
  `contextEl`(구분선 · "요약 보기" 펼침) · `PREFS` 에 `auto_compact`.
- `crates/app/web/index.html` — 대화 머리 `#ctx_btn`(북마크 왼쪽, `#ctx_text`) · 설정 실행 카드 "자동 압축 — 컨텍스트 80% 넘으면" 체크 · '?' 안내.
- `crates/app/web/app.css` — 아이콘 `gauge` · `compress` · `reset` · `.ctx-btn[data-level]` 색 · `.ctx-break` · `.ctx-summary` · `.ctx-text`.
- 시험 — `view.test.mjs` 100 → 104(`context_size` · `context_label` · `ctx_menu` · `context_note`) · `ux_literals.rs` `context_literals`(20) · `ux_api`
  `context_api_and_auto_compact_setting` · 실제 claude `crates/agent/tests/context_live.rs`(`--ignored`).
- 문서 — 사용자 가이드 7.4 "컨텍스트 — 크기 · 압축 · 비우기" · 13장 자동 압축 · developer-guide(5.2 프로토콜 `Context` · `Pong.ctx` · outbox · 6.3 압축 ·
  비우기 · 7.2 스키마 19 표 · 설정 키 · 7.5 v19) · an_net · an_core · an_agent · an_app · PRODUCT(컨텍스트 문단) · README(기능 줄) · 캡처 `workspace`
  (가상 데이터 mock7 — 머리 "42k / 1M").

## 설계 결정 · plan 편차

- **메뉴 꺼짐 조건** — 실행 대기 · 실행 중 글, 상대 답을 기다리는 실행 요청(`sent` · `exec`), 진행 중 구분선이 있거나 에이전트 답이 아직 없으면 두 항목 다 꺼지고
  이유 한 줄을 보인다.
- **an_app 매뉴얼 묵은 문구** — 1l 라운드 2 에서 `color-mix` 를 `::before` 층으로 바꿨는데 매뉴얼에 `color-mix` 가 남아 있어 함께 고쳤다.
- **실제 claude(E9)** — 시험 하나가 두 앱(실제 실행기)을 띄워: 파일을 읽어 답 → 창 사용량 6157 → 압축(요약에 "노을-42" · 규칙) → 새 세션에서 "파일을 다시
  읽지 말고" 물음 → "노을-42" 로 답(실행 입력에 `<대화 요약>`). haiku · low, 13초.

## 검증

- mock(가상 데이터, scratchpad `m1/m1.mjs` · `mock7.py`) 12/12 — 42k / 1M · 650k 주황 · 830k 붉음 · 알 수 없음 · 0.4.0 거절 구분선 · 자동 압축 구분선 · 요약 펼침 ·
  메뉴 두 항목 · 확인 · 실행 → "요약 중…" · 진행 중 꺼짐 · 실행 중 · 답 없음 꺼짐 · 설정 체크 · JS 오류 0.
- `cargo test -p an-agent --test context_live -- --ignored` 1 passed(실제 claude 2.1.299).
