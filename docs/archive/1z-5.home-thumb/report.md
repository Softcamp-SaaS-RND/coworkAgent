# report — 1z-5.home-thumb

종결: 2026-10-22 · 브랜치 `task/1z-5-home-thumb` → main(squash)

## 결과

개요 '미리보기' 보기 = **2차 시안 A**(상대마다 절 + 대화방마다 축소 채팅 카드) — #1z-4 의 상대별 창 · 대화방 탭(시안 B)을 대체. '나' · 그룹은 절마다 한 줄
(열 수 = 그린 카드 격자의 실제 열 — CSS `auto-fill`, 카드 최소 240px · 폭이 바뀌면 다시 잼), 넘치면 끝 칸 "+n개"(그 상대 개요 = 카드 전부). 대화 없는 페어링 상대 = 점선 줄 + "세션 생성".
기본 보기 · 보기 기억(`ui_home_view`) · 글 `?limit=6` · 다시 읽기 규칙 · 읽음 불변은 #1z-4 그대로. 서버 · API 바꿈 없음.

| 성공 기준 | 판정 | 근거 |
|---|---|---|
| A1 절 + 카드(제목 · 상태 · 말풍선 · 에이전트 · 시각 · 안 읽음) · 누르면 대화 · 글자만 | 충족 | E3 · E4 · E6 |
| A2 절마다 한 줄 · 열 수 = 폭 · "+n개" → 상대 개요 전부 | 충족 | E3(`preview_sections`) · E4 · E6(380/700/1000/1400 → 1/2/3/5열) |
| A3 빈 상대 점선 줄 + 새 세션 | 충족 | E6 |
| A4 기본 보기 · 기억 · 다시 읽기 그대로 · 안 읽음 불변 | 충족 | E1 · E3 · E6 |
| A5 회귀 없음 · 문서 | 충족 | E1 · E2 · E5 |

- 적대적 평가 2 라운드(상한, Orca 가 라운드마다 새로 띄운 Codex) — R1 medium 1(전체 무효화가 한 줄 밖 숨은 카드에 남지 않음) 반영, R2 approve(지적 없음). 미조치 없음([eval](eval.md)).
- 작업 메모: plan E4 CHECK 괄호 오타(첫 실행 SyntaxError)를 개정 기록과 함께 고침.
- 편차(intent 결정 4): 빈 상대 단추 글자 = 앱 기존 "세션 생성"(시안 · 질문 보기의 "+ 새 세션" 아님) — 사용자 "그대로 유지"(2026-10-22) 확정.
- 구현자 해석(사용자 미확인 — intent 결정): 카드 순서 = 최근 활동 순(3) · 한 열 폭이면 "+n개" 가 다음 줄(2) · Task 번호(6).

## 산출

- web: `view.js` `previewSections`(삭제 `previewWindows` · `previewTabs` · `PREVIEW_TABS`), `app.js` `state.previewCols` · `renderHomePreview` · `fitPreviewCols` ·
  `ResizeObserver` · `previewSectionEl` · `previewCardEl`(삭제 `previewWindowEl` · `pickPreviewTab` · `state.previewTab`), `index.html` `#home_preview.pv-list`, `app.css` `.pv-*`.
- 시험: `view.test.mjs` `preview_sections` · `preview_reload` · `ux_literals`(#1z-5 다섯 줄).
- 문서: web-ui · 사용자 가이드 · PRODUCT · masterPlan S4 · 종결 목록 · 개정 기록.

## 남은 것

- 원장 `#1z-4 R1-1`(조건 대기) 그대로 이어받음. 두 PC 실제 상대 실측은 미실행(19 acceptance 와 함께).
