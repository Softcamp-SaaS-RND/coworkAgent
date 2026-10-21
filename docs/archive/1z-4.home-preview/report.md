# report — 1z-4.home-preview

종결: 2026-10-22 · 브랜치 `task/1z-4-home-preview` → main(squash)

## 결과

개요 기본 보기 = **미리보기**(시안 B — 상대마다 작은 대화 창: 머리 · 대화방 탭 3개 + "+n개" · 최근 글 말풍선, 상대 타일은 대화방마다 창). 열 수 = 브라우저 폭(CSS `auto-fill`,
창 최소 360px · 높이 320px) · 넘치면 세로 스크롤. "목록 | 미리보기" 는 설정 `ui_home_view`(기본 `preview`)에 기억. 미리보기 글은 `GET …/messages?limit=6`(SQL 에서 자름),
다시 읽기는 요약 변화 · 변경 신호 · 개요에 들어옴 · 다시 붙음 · 탭 고름 때만, 읽음은 바꾸지 않는다.

| 성공 기준 | 판정 | 근거 |
|---|---|---|
| A1 기본 미리보기 · 목록 기억 · 틀린 값 400 | 충족 | E4(`home_view_setting_persists`) · E5 · E7 |
| A2 상대마다 창 · 처음 탭 최근 · 고른 탭 유지 | 충족 | E3(`preview_windows`) · E7 |
| A3 상대 타일 = 대화방마다 창 | 충족 | E7 |
| A4 열 수 = 폭 · 세로 스크롤 | 충족 | E5 · E7(380/800/1200/1600 → 1/1/3/4열, 자연 폭 820 → 2열) |
| A5 마지막 글만 · 필요한 대화만 다시 · 읽음 불변 | 충족(남은 경우 1 — 원장) | E4(`limit`) · E3 · E7(가만히 11초 읽기 0회 · 새 글 = 그 대화만 1회 · 안 읽음 그대로) |
| A6 회귀 없음 · 문서 | 충족 | E1 · E2 · E6 |
| A7 탭 3개 + "+n개" | 충족 | E3(`preview_tabs_and_reload`) · E7("+2개" → 창 5) |

- 적대적 평가 2 라운드(상한, Orca 가 라운드마다 새로 띄운 Codex) — R1 medium 2(마지막이 아닌 글 변경 · 실패 뒤 재시도 서명), R2 medium 2(신호 다시 붙음 · 첫 연결 안내) 반영.
  미조치 1행 — 원장 `#1z-4 R1-1` 조건 대기(사용자 "Q3 반영"으로 5초 전체 읽기 대신 신호 · 요약 기반, 이 PC 변경 신호가 생기면 해소)([eval](eval.md)).
- 구현자 해석(사용자 미확인 — intent 결정): 그룹 개요 = 상대별 창(7) · 상대 순서 지금 그대로(8) · Task 번호(9) · Q3 의 남은 경우 처리(11).
- 사용자 중간 지시 "Q2, Q3 반영" — plan 개정 기록 2 · intent 결정 11 · A7 신설.
- 작업 메모: `cargo fmt` 는 이 저장소에서 쓰지 않는다(손대지 않은 파일까지 바뀜). Bash 셸의 `cargo test` 는 링커 LNK1104/1201 이 잦아 PowerShell 로 돌렸다.

## 산출

- core: `run_opts` `HOME_VIEW_KEY` · `HOME_VIEWS` · `home_view`, `chat::messages_tail`.
- app: 설정 `ui_home_view`(검사 · 쓰기 · 읽기), `GET /api/conversations/{id}/messages?limit=N`.
- web: `view.js` `PREVIEW_MSGS` · `PREVIEW_TABS` · `previewWindows` · `previewTabs` · `previewKey` · `previewStaleOf` · `previewBubbles`, `app.js` 보기 분기 · `renderHomePreview` ·
  `previewWindowEl` · `pickPreviewTab` · `setHomeView` · 신호 · 다시 붙음 무효화, `index.html` `.seg` · `#home_preview`, `app.css` `.pv-*`.
- 시험: `chat_api`(limit) · `ux_api`(`home_view_setting_persists`) · `view.test.mjs`(3) · `ux_literals`(7줄).
- 문서: web-ui · an_app · an_core · 사용자 가이드 · PRODUCT · masterPlan D35 · 종결 목록 · 개정 기록.

## 남은 것

- 미조치 1행(원장 조건 대기). 두 PC 실제 상대의 변경 신호 · 신호 끊김/복구 실측은 미실행(19 acceptance 와 함께).
