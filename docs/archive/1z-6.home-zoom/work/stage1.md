# stage 1 — 개요 "미리보기 크기" 100 ~ 200%

## 목표

intent 결정 1 ~ 5 — 말풍선 칸 높이 · 줄 수만 배율, 글자 고정, 설정 `ui_home_zoom` 에 기억.

## 산출 파일

- `crates/core/src/run_opts.rs`(`HOME_ZOOM_KEY` · `HOME_ZOOMS` · `home_zoom`) · `crates/app/src/notify.rs`(`ui_home_zoom` 검사 · 쓰기 · 읽기).
- `crates/app/web/view.js`(`HOME_ZOOMS` · `previewZoom` · `PREVIEW_MSGS` 12) · `app.js`(`applyHomeZoom` · 고름 처리 · `showSettings` · `#home_zoom_pick` 보이기) ·
  `index.html`(`label#home_zoom_pick`) · `app.css`(`--pv-scale` · `--pv-lines`).
- 시험 `ux_api.rs`(`home_zoom_setting_persists`) · `view.test.mjs`(`preview_zoom`) · `ux_literals.rs`(네 줄). 문서 — web-ui · an_app · an_core · 사용자 가이드 · PRODUCT · masterPlan.

## 설계 결정

- 배율은 CSS 사용자 속성 둘(`--pv-scale` · `--pv-lines`)만 바꾼다 — 화면을 다시 그리지 않고, 카드 폭이 그대로라 한 줄 열 수(`fitPreviewCols`)도 그대로.
  높이는 절 `zoom`(1.15) 아래 단위라 100% 본문 = 화면 138px, 200% = 276px.
- 저장 = 고르면 바로 적용하고 `PUT /api/settings {ui_home_zoom}`(실패는 상태 줄), 시작할 때 `showSettings` 가 적용 — `ui_home_view` 와 같은 규칙.
  처음엔 `PREFS`(숫자 선택 `ui_peer_weeks` 의 길)였으나, 모든 설정 응답을 `fillPrefs` 가 적용해 다른 설정의 늦은 응답이 막 고른 배율을 되돌렸다(R1).
  저장은 `SendLatest`(접힘 저장과 같은 도구) — 연속으로 고르면 한 번에 하나만 보내고 마지막 값을 이어서(R2 — 동시 PUT 이 앞 값으로 끝나던 결함).
- 받는 글 12개 — 200% 본문 276px 에 2줄 말풍선 6 ~ 8개. 서버가 SQL `LIMIT` 로 자르므로 비용은 작다.

## 테스트 결과

```text
node --test "crates/app/web/*.test.mjs"   → pass 164 · fail 0
cargo test -p coworkagent-app --test ux_api --test ux_literals --test css_tokens → ok 48 · ok 3 · ok 5
실제 화면(ego-browser, 디버그 빌드 · 시험 데이터 폴더 · #1z-4 시드) → E7 EVIDENCE
```
