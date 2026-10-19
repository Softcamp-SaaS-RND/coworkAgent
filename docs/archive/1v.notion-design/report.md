# report — 1v.notion-design

종결: 2026-10-19 · 브랜치 `task/1v-notion-design` → main

## 결과

Notion UI/UX 시안 A(톤 스킨) · B(페이지형 대화) · C(워크스페이스 골격)를 Design 아티팩트로 게시한 뒤, 사용자 결정은 **기본 디자인 유지 + C 의 요청 보드 + 사이드바 재배치 + 피드백 말풍선 +
A 의 입력창 모양 + 웹UI 편집 모드 삭제**. 웹UI 만 바꿨다(서버 · 저장 · 프레임 변경 없음). 대화 나가기 · 삭제는 별도 Task 1w(intent 승인, 브랜치 `task/1w-conv-leave`).

| 성공 기준 | 판정 | 근거 |
|---|---|---|
| A1 보드 열 · 건수 | 충족 | E4 node(`req_board_columns`) |
| A2 허용 열만 놓기 · 보낸 요청 읽기 전용 | 충족 | E4 node(`req_board_drop`) |
| A3 놓으면 메모 칸 → 확인 시 요청자에게 · 취소면 그대로 | 19 인수로 이월 | E12 ABANDON(D28), 원장 `#1v E12` 조건 대기 |
| A4 목록 / 보드 전환 · 목록 보기 그대로 | 충족(자동) · 실화면 이월 | E3(node 141) · E12 |
| A5 나 · 상대 타일 = 내 대화만 · 공유 대화 머리 · 그룹 그대로 | 충족 | E5 node(`conv_list_mine` · `conv_list_mine_sections`) |
| A6 에이전트 › 등록된 에이전트 · 에이전트 대화 | 충족 | E6 node(`side_nav*`) |
| A7 에이전트 대화 화면 = 요청자별 절 | 충족 | E7 node(`agent_convs_sections`) |
| A8 피드백 말풍선 · 위 메모 없음 | 충족 | E8 node(`req_feedback`) · E9 |
| A9 입력창(세그먼트 · 안내 없음 · placeholder · 860px) | 충족 | E10 node(`composer_a` — 런타임 재대입 금지 포함) |
| A10 회귀 없음 | 충족 | E1 · E2 · E3 · E11 |
| A11 웹UI 편집 모드 없음 | 충족 | E13 |

- 적대적 평가 2 라운드(상한, `codex-review`) — 라운드 1 medium 1(빈 상대 절 · 빈 검색 결과 → `visibleConvRows`), 라운드 2 medium 1(placeholder 런타임 덮어쓰기 → 삭제 ·
  재대입 금지 검사). 미조치 1(DOM 시험 — [eval](eval.md)) — 사용자 수용("종결 진행"), 원장 `#1v R2-D1`.
- 구현자 해석(plan 승인 때 함께 승인): @멘션 기능 유지(문구만 삭제) · 보기 기억 = `ui_folded` 열쇠 `reqboard:me` · 피드백 쓴 사람 "나" / 상대 이름 · 기존 `conv_split_rows` · `share_groups` 유지.
- 구현 중 편차(plan 개정 기록): 세그먼트 = `:has()` 대신 형제 선택자(Safari 15.0 ~ 15.3), 새 대화 질의 / 요청도 같은 세그먼트, 그림자 토큰 `--shadow-sm` · `--dark-shadow-sm`.
- 화면에서 바뀐 것(사용자 확인 필요할 수 있음): 입력창 placeholder 가 "{에이전트} 에게 보내기" → "메시지 입력"(A9 · 라운드 2).

## 산출

- `view.js` `BOARD_COLS` · `boardColumns` · `canDrop` · `reqFeedback` · `visibleConvRows` · `convListParts` · `agentConvSections`, `sideNav` 에이전트 머리 · 하위, `route("#agents/convs")`, `keepPicked` 삭제.
- `app.js` 요청 보드(`boardEls` · `requestItem` · `reqBoard`) · 사이드바 머리 · 하위 · 안 읽음 합 · `renderAgentConvs` · `feedbackEl` · 편집 모드 삭제 · `renderAskHint`.
- `index.html` · `app.css` — 요청 보기 세그먼트 · `#req_board` · `#agent_convs`, 입력창 A 모양, 편집 줄 · 막대 · 대화상자 · `#mention_hint` 삭제.
- 시험: node 8 새 + 4 고침(133 → 141), `ux_literals` 있음 · 없음 표 갱신.
- 문서: masterPlan D34 · D35 개정 · D39 ① · S4 보강 · Phase 1 후보 줄(U2 로 교체) · 개정 기록, PRODUCT 2줄, web-ui 사이드바 · 요청 · 대화 목록 · 대화, 원장 4행.

## 남은 것

- 19 인수 TC: 1v E12(보드 끌어 놓기 실화면 · 입력창 placeholder · 에이전트 대화뿐인 상대 타일).
- 다음: 1w.conv-leave — 착수 때 D40 신설 → plan(intent 승인 2026-10-19).
- 후보: 요청 보드 보기 설정(U2 — 그룹 · 정렬 · 거르기, 전용 설정 키).
