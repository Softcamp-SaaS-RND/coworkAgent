# report — 1z-6.home-zoom

종결: 2026-10-22 · 브랜치 `task/1z-6-home-zoom` → main(squash)

## 결과

개요 미리보기 머리에 **"미리보기 크기"** 드롭다운 100 · 125 · 150 · 175 · 200%(기본 100 = 지금 크기, 축소 없음 — 사용자). 고르면 카드 말풍선 칸 높이가 배율만큼
(화면 138 → 276px), 말풍선 줄 수 2 · 3 · 3 · 4 · 4 — 글자 크기 · 카드 폭 · 한 줄 열 수는 그대로. 설정 `ui_home_zoom` 에 기억(목록 밖 400), 미리보기 보기에서만 보인다.
받는 글 6 → 12개(200% 본문도 채움).

| 성공 기준 | 판정 | 근거 |
|---|---|---|
| A1 드롭다운 100 ~ 200% · 미리보기에서만 | 충족 | E5 · E7 |
| A2 본문 높이 · 줄 수만, 글자 · 폭 · 열 그대로 | 충족 | E3(`preview_zoom`) · E5 · E7(138/173/207/242/276px · 2/3/3/4/4줄 · 12px · 폭 378 · 2열) |
| A3 설정 기억 · 목록 밖 400 | 충족 | E4(`home_zoom_setting_persists`) · E7(새로 고침 뒤 200) |
| A4 회귀 없음 · 문서 | 충족 | E1 · E2 · E6 |

- 적대적 평가 2 라운드(상한, Orca 가 라운드마다 새로 띄운 Codex) — R1 medium 1(다른 설정의 늦은 응답이 배율을 되돌림), R2 medium 1(연속 선택의 동시 저장 순서) 반영. 미조치 없음([eval](eval.md)).
- 범위 밖 관찰: R1 의 경합(설정 저장 응답 전체를 `fillPrefs` 가 적용)은 테마 · 타일 크기 등 다른 `PREFS` 설정에도 이 Task 전부터 있다 — 고치려면 설정 응답에 차례 번호 또는 요청한 키만 적용(후보).
- 구현자 해석(사용자 미확인 — intent 결정): 받는 글 12개(4) · 드롭다운 이름 · 자리(5) · Task 번호(6).
- 작업 메모: `cargo test --workspace` 첫 실행 한 번이 결과 4줄에서 멈춤(링커 경합 추정 — 바로 다시 돌려 80개 모두 ok).

## 산출

- core: `run_opts` `HOME_ZOOM_KEY` · `HOME_ZOOMS` · `home_zoom`. app: 설정 `ui_home_zoom`(검사 · 쓰기 · 읽기).
- web: `view.js` `HOME_ZOOMS` · `previewZoom` · `PREVIEW_MSGS` 12, `app.js` `applyHomeZoom` · 고름 처리(`SendLatest` 저장) · `showSettings` 적용 · `#home_zoom_pick` 보이기,
  `index.html` `label#home_zoom_pick`, `app.css` `--pv-scale` · `--pv-lines`.
- 시험: `ux_api` `home_zoom_setting_persists` · `view.test.mjs` `preview_zoom` · `ux_literals`(#1z-6 여섯 줄).
- 문서: web-ui · an_app · an_core · 사용자 가이드 · PRODUCT · masterPlan S4 · 종결 목록 · 개정 기록.

## 남은 것

- 버전 그대로(1.3.0) — 다음 서명 릴리스에 들어간다.
