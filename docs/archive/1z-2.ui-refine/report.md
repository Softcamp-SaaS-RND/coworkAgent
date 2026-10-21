# report — 1z-2.ui-refine

종결: 2026-10-21 · 브랜치 `task/1z-2-ui-refine` → main(#1z 종결과 함께 squash)

## 결과

사이드바 '에이전트 응답'(옛 '에이전트 대화' — '나' 에만), 개요 아코디언 사용자 · 에이전트 아이콘, 레일 "+ 사용자" · "+ 그룹", 대화 목록 한 줄(로봇 아이콘 · 제목),
레일 타일 놓는 자리 = 가장 가까운 타일(사용자 간 위치 변경이 안 되던 경우 — 보통 크기에서 타일 아래 이름 · 틈에 놓을 때), 그룹설정 시안 B(카드 격자 + 사용자 표 · 줄 펼침 ·
포인터 넣기), 주고받은 메시지 시안 Peer(원형 아바타 · 에이전트 줄 바탕).

| 성공 기준 | 판정 | 근거 |
|---|---|---|
| A1 에이전트 응답 '나' 에만 · 이름 | 충족 | E4(`side_nav*`) · E5 · E8 |
| A2 상대 타일 → 개요 · 배지에서 에이전트 대화 제외 | 충족 | E4(`side_nav_agents` · `tile_unread`) · E5 · E8 |
| A3 개요 머리 사용자 · 줄 로봇 아이콘 | 충족 | E5 · E8(6/6 · 12/12) |
| A4 레일 "+ 사용자" · "+ 그룹" · 접근 이름 | 충족 | E5 · E8 |
| A5 대화 줄 한 줄 | 충족 | E5 · E8(2행 0 · 40px) |
| A6 가장 가까운 자리로 놓기 | 충족 | E4(`rail_slot`) · E6 · E8(보통 이름 · 작게 끝 밖 · 위 틈, 서버 순서 = 화면) |
| A7 그룹설정 B · 펼침 · 유지 | 충족 | E6 · E8 · R1 · R2 반영(펼친 다른 사용자 · 저장 응답 대기 중 저장 전 값 보존) |
| A8 포인터로 카드에 넣기 · HTML 끌기 없음 | 충족 | E6 · E8("이미 그룹 … 에 있습니다" — 데이터 바꾸지 않는 경로) |
| A9 주고받은 메시지 줄 | 충족 | E4(`avatar_kind` · `inbox_rows_author`) · E6 · E8 |
| A10 회귀 없음 | 충족 | E1 · E2 · E3(node 160) · E7 |

- 적대적 평가 2 라운드(상한, Orca 가 라운드마다 새로 띄운 Codex) — R1 medium 1(여러 사용자를 펼친 채 다른 사용자 저장 시 저장 전 모델 · effort 사라짐), R2 medium 1(R1 수정의 잔여 —
  저장 응답 대기 중 고친 값까지 지움) 모두 반영 · 실 화면 확인. 미조치 1행(그 두 경로의 DOM 회귀 시험 없음 — #1z 와 같은 판단, 원장 `#1z-2 R1·R2-D1` 수용)([eval](eval.md)).
- 구현자 해석(intent 결정 2): 상대 타일 배지에서 에이전트 대화 안 읽음을 뺐다(그 타일에서 열 수 없는 대화).
- plan 개정: E6 CHECK 보정(EXPECT 그대로).
- plan 밖: 시험 중 서버 상대 순서가 바뀌어 있던 것을 `PUT /api/order` 로 시작 순서(1,2,3,5,4,6)로 되돌림(원인 — 중단된 시험 실행으로 추정, 확인 필요).

## 산출

- 웹UI `view.js`: `railSlot` · `peerLeaves` · `tileUnread` · `avatarKind` · `inboxRows.mine`, `sideNav` 개정, `groupSettingsPick` 삭제.
- 웹UI `app.js`: `tileGrab`(놓을 자리) · `convGrab`(`join` · `lostpointercapture`) · `convItem`(한 줄) · `renderHome`(아이콘) · 그룹설정 B(`gsGroupsEl` · `gsUsersEl` · `gsToggle` · `gsDot` ·
  `gsJoin` · `gsDrafts`) · `peerRowEls`(시안 Peer).
- `index.html` · `app.css`(`--agent-soft` · 시안 B · `.pm-*` · 죽은 규칙 삭제), 시험 node 새 3 · 갱신 4 · 삭제 1(160), `ux_literals`.
- 문서: masterPlan D35 · D37 · S4 · 개정 기록 · 종결 목록, PRODUCT, README, 사용자 가이드, web-ui.

## 남은 것

- DOM 이벤트 경로 자동 시험 없음(#1z 와 같은 판단 — 원장 #1z).
- 그룹 타일 대화 목록 묶음 머리 "에이전트 대화"(메뉴 이름은 "에이전트 응답") — 같은 개념 두 이름, 사용자 확인 필요.
- 미확인(실 화면): 그림 아이콘 타일 끌기 · 다크 테마 대비 · 실제 그룹 넣기(데이터 변경을 피함).
