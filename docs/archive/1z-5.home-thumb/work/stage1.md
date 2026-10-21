# stage 1 — 개요 미리보기 2차 시안 A(상대별 절 + 대화방 카드)

## 목표

intent 결정 1 ~ 5 — 미리보기 = 상대마다 절 + 대화방마다 축소 채팅 카드, '나' · 그룹은 절마다 한 줄(폭 맞춤) + "+n개", 빈 상대 점선 줄.

## 산출 파일

- `crates/app/web/view.js`(`previewSections` — `previewWindows` · `previewTabs` · `PREVIEW_TABS` 삭제) · `app.js`(`state.previewCols` · `renderHomePreview` ·
  `fitPreviewCols` · `ResizeObserver` · `previewSectionEl` · `previewCardEl` — `previewWindowEl` · `pickPreviewTab` · `state.previewTab` 삭제) ·
  `index.html`(`#home_preview.pv-list`) · `app.css`(`.pv-*` 다시 씀).
- 시험 `view.test.mjs`(`preview_sections` · `preview_reload`) · `ux_literals.rs`(#1z-5 다섯 줄). 문서 — web-ui · 사용자 가이드 · PRODUCT · masterPlan.

## 설계 결정

- 한 줄 열 수는 스크립트가 셈하지 않고 **그린 격자에서 잰다** — CSS `auto-fill`(카드 최소 240px)이 정한 트랙 수(`getComputedStyle(.pv-cards).gridTemplateColumns`)를
  `state.previewCols` 에 두고, 처음 값(4)과 다르면 다시 그린다. 안 보이는 격자는 풀지 않은 `repeat(…)` 를 주므로 트랙이 모두 `px` 일 때만 센다.
  창 폭이 바뀌면 `ResizeObserver(#home_preview)` 가 다시 잰다. 카드 수가 바뀌어도 한 줄이라 높이(세로 스크롤바)가 바뀌지 않아 열 수가 흔들리지 않는다.
- 넘치면 `cols - 1` 개 + 끝 칸 "+n개"(숨은 수) — 한 열 폭이면 카드 하나 + 다음 줄 "+n개".
- 다시 읽기 규칙은 #1z-4 그대로, 대상만 "창의 고른 대화" → "보이는 카드의 대화". 한 줄 밖 대화의 변경 신호 표시는 남겨 두었다가 보일 때 읽는다.
  전체 무효화(`previewAll`)도 한 줄 밖 숨은 카드의 받아 둔 글을 `previewStale` 로 남긴다(R1 — 폭을 줄인 채 다시 붙으면 넓혔을 때 옛 글이 남던 결함).
- 카드 전체가 링크(`a.pv-card`) — 안에 단추를 두지 않는다("+n개" · "{이름} 개요 →" 는 카드 밖 단추). 본문은 글자로만(`el` `text`).
- 대화 없는 상대 단추 = 기존 `SESSION_NEW`("세션 생성") — intent 결정 4.

## 테스트 결과

```text
node --test "crates/app/web/*.test.mjs"   → pass 163 · fail 0
cargo test -p coworkagent-app --test ux_literals --test css_tokens → ok 3 · ok 5
실제 화면(ego-browser, 디버그 빌드 · 시험 데이터 폴더 · #1z-4 시드) → E6 EVIDENCE
```
