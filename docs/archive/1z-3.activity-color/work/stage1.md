# stage 1 — 에이전트 줄 여백 · 내 활동 종류 색(시안 A)

## 목표

intent 결정 1 ~ 4 — 줄 오른쪽 여백 24px, 내 활동 이름표 · 탭 종류 색.

## 산출 파일

- `crates/app/web/view.js`(`activityRows` `tone`) · `app.js`(`activityEls` 칩 클래스) · `app.css`(토큰 둘 + 다크, `.tag.act-*`, `#act_tabs` 색, `li.has-edit` 여백, `.tag.plain` 삭제).
- `crates/app/web/view.test.mjs`(`activity_rows` 에 `tone`). 문서 — web-ui · 사용자 가이드 · masterPlan S4.

## 설계 결정

- 색 키를 탭 `kind` 와 같은 이름으로 — 칩 · 탭이 같은 규칙(`act-{kind}` / `[data-kind=kind]`)을 쓴다.
- 고른 탭의 기존 규칙(`.tabs a[aria-current="page"]` 글자 `--text`)보다 `#act_tabs` 규칙이 더 구체적이라 고른 탭도 종류 색 글자를 유지한다.
- 대비(실측): 밝은 칩 5.14 ~ 7.53, 다크 칩 5.94 ~ 9.24(WCAG AA 4.5 이상).

## 테스트 결과

```text
node --test "crates/app/web/*.test.mjs"   → pass 160 · fail 0
실제 화면(ego-browser, 디버그 빌드 · 시험 데이터 폴더) → li.has-edit padding-right 24px · 연필 ↔ 줄 끝 24px,
  탭 request/mine/answer/failed 글자 rgb(128,77,12)/(16,74,151)/(12,108,104)/(182,50,45) · 전체 무채색, 칩 바탕 = 토큰,
  받은 요청 탭 고름 → 바탕 rgb(253,241,224) · 굵기 600, 다크 → 글자 rgb(234,187,121) · 바탕 rgb(48,36,18)
```
