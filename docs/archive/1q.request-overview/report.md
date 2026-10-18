# report — 1q.request-overview

종결: 2026-10-18 · 브랜치 `task/1q-request-overview` → main

## 결과

'요청' 화면을 **받은 요청 · 보낸 요청 개요**(머리 = 전체 · 대기 · 수용 · 완료 · 반려 수, 서버 집계)로 열고, 탭을 누르면 그 목록만 · 다시 누르면 개요로,
상태 바꾸기를 본문 바로 아래 **글자 단추**(✓ 수용 · ✓ 완료 · ✕ 반려)로 옮겼다. **내 활동**을 받은 요청 · 내 요청(상대가 상태를 바꾸면 맨 위 · 안 본 것) ·
받은 답 · 실패로 좁히고(자동 왕복 · 상대 사람 일반 메시지 뺌) 거르기를 탭으로 바꿨다. **주고받은 메시지**는 상대별 절 처음 접힘 · 머리 요약(건수 · 마지막
메시지) · 펼치면 서버에서 20건씩 · "더 보기" · 행 북마크. 진행 중 사용자 지시로 **CLI 도움말 `coworkAgent /?`**(`-h` · `--help`)와 README "다른 AI 도구와 연동"
절을 더했다. 시안은 디자인 캔버스(비공개 https://claude.ai/artifact/F7pCWhi5WFvi3sQMDoZQ32)에서 골랐다. 스키마 · 통신 프레임 변경 없음.

| 성공 기준 | 판정 | 근거 |
|---|---|---|
| A1 요청 개요 두 절 · 머리 전체 · 상태별 건수 | 충족 | E3 · E4(`req_overview_*` 4) · `req_counts_beyond_page` · mock |
| A2 탭 = 그 목록만 · 다시 / 메뉴 = 개요 | 충족(자동 = 문구 · 자리 잠금) | E8 `ux_literals`(탭 토글 · 메뉴 다시 누름) · mock — 실제 클릭은 원장 `#1q PC` |
| A3 본문 바로 아래 글자 단추 → 메모 → 상태 | 충족 | E4 `req_overview_buttons` · E8 · 기존 `requests.rs` 상태 시험 · mock |
| A4 내 활동 = 받은 요청 · 내 요청 · 받은 답 · 실패, 안 본 수 같은 범위 | 충족 | E5 `activity_scope`(공유 사본 · 자동 왕복 답 · 일반 메시지 빠짐) · E7 `activity_kinds_scope` · `activity_unseen_by_peer_and_mark_seen` |
| A5 내 요청 상태 바뀜 = 안 본 항목 | 충족 | E6 `activity_my_request_state_unseen` · `activity_my_request_atomic_with_state` |
| A6 주고받은 메시지 접힘 요약 · 20건씩 | 충족 | E7 `peer_pages_summary_and_cursor` · `peer_pages_order` · node `peer_msgs_*` 5 · mock |
| A7 행 북마크 = 말풍선 · 북마크 화면과 같은 상태 | 충족 | node `peer_msgs_saved` · E8(`row-mark`) · 같은 `PATCH saved` |
| A8 회귀 없음 | 충족 | E1 · E2 · E3(node 121) · E9 · E10 · `check_gates` 13 중 미충족 0(E11 ABANDON D28) |
| A9 `coworkAgent /?` 명령 · 형식 · 종료 코드 · 0 · 앱 불필요 | 충족 | E12 `help_exe_prints_commands`(실제 실행 파일 · 파이프) |
| A10 README 연동 안내 | 충족 | E13(README `coworkAgent /?` 5곳 · `readme ok`) |

- 적대적 평가 2 라운드(상한, `codex-review` 세션) — 라운드 1 high 1 · medium 2(요청 500 상한 건수 · 거르기 → 서버 집계 · `req=` / 상태 + 기록 비원자 → 한
  트랜잭션 / 다시 그리기 서명), 라운드 2 medium 3(서명 발화자 값 / 새로 고침이 더 보기 페이지 덮음 → 한 줄 차례 + `mergePage` / id 순 → `(created_at, id)` ·
  복합 커서). 모두 고침, 미조치 1(요청 목록 커서 페이지 — 수용). 라운드 2 수정은 상한으로 재리뷰 없음([eval](eval.md), 원장 `#1q R1-*` · `R2-*`).
- plan 편차: intent 개정 1(CLI 도움말 — 사용자 병행 지시), plan 개정 기록 3줄 모두 승인(E3 EXPECT 121 → 119 — 사용자 "plan 개정 승인."). 구현자 해석으로 남은
  것: 결정 7(라운드 2 "의견 수용" → 받은 답 유지) · 결정 15(`-h` · `--help` 별칭) — 사용자가 명시 확인하지 않았다(뒤집으면 다음 Task).
- 해석 8(주고받은 메시지 정렬)은 처음 구현에서 id 순으로 벗어났다가 라운드 2 지적으로 plan 대로 되돌렸다. `view.peerMessages`(죽은 코드)는 지웠다.

## 산출

- core — `ActivityFilter`(All · Request · MyRequest · Answer · Failed) · `ActivityKind`(MyRequest 추가, RelayStart · RelayEnd 지움) · `activity` · `activity_unseen` ·
  `note_req_state` · `req_counts`/`ReqCounts` · `messages_page`(복합 커서) · `peer_message_counts`.
- agent — `inbound_req_state` 상태 + `req_state` 기록 한 트랜잭션.
- API — `/api/activity?kind=all|request|mine|answer|failed` · `/api/messages?side=both&before=&before_at=` · `/api/messages/peers` · `/api/messages/req_counts`.
- CLI — `cli::is_help` · `cli::help`(소개 + `USAGE` + 출력 형식 · 종료 코드 · 연동 예).
- 웹 — `reqBoxes` · `reqHead` · `REQ_BTN` · `textButton` · `foldEl` 처음 접힘(`open:` 열쇠) · `ACT_KINDS` · `activityRows` · `PEER_PAGE` · `peerSummaryRows` · `pageMore` ·
  `peerMsgsSig` · `mergePage` · `loadPeerPage`(상대마다 한 줄 차례) · `markRow` · `.ico-check` · `.ico-x` · `.text-btn`.
- 시험 — core `activity_scope` · `peer_pages_order` · `req_counts_beyond_page`, agent `activity_my_request_state_unseen` · `activity_my_request_atomic_with_state`,
  app `activity_kinds_scope` · `peer_pages_summary_and_cursor` · `help_exe_prints_commands` · 옛 거르기 시험 6곳 새 범위로, node `req_overview_*` 4 ·
  `activity_rows_tabs` · `peer_msgs_*` 5, `ux_literals` 1q 행.
- 문서 — README "다른 AI 도구와 연동" · 사용자 가이드 4 · 7 · 9 · 10 · 14장 · web-ui · an_app · 개발자 가이드 저장 표 · PRODUCT · masterPlan D14 · D35 · S4 ·
  원장 7행 · `design/mock.py`(가상 데이터).

## 남은 것

- 19 인수 TC — 실제 두 PC: 요청 개요 · 글자 단추 · 내 활동 탭 · 내 요청 상태 바뀜 · 주고받은 메시지 접힘 · "더 보기 여러 번 + 5초 기다리기" · 공유 대화가 있는
  상대의 정렬 · 행 북마크(원장 `#1q PC`).
- 다음 Task 후보 — 요청 목록 커서 페이지(원장 `#1q R1-F1b`) · 자동 왕복 끝 사유를 내 활동 밖(세부정보 자동 왕복 칸)에 보이기 · 릴리스 실행 파일을 콘솔에서 직접
  실행할 때 CLI 출력이 보이게(콘솔 붙이기 — 지금은 파이프 · `| more`, 직접 실행 확인 안 함).
- 릴리스 · GitHub 반영 · 이 PC 반영은 사용자 확인 뒤.
