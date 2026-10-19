# report — 1w.conv-leave

종결: 2026-10-19 · 브랜치 `task/1w-conv-leave` → main

## 결과

모든 대화(내 대화 · 에이전트 대화 · 공유 대화)의 ⋯ 메뉴에 "나가기" · "삭제"(D40), 그룹 = 대화만(D37 개정) · 설정 연결 목록 = 그룹 절(대화 줄) + "사용자 목록"(관리 줄 + 대화 트리,
끌어 놓기 = 복사). 저장 스키마 21 · 프레임 `Leave` · `Pong.leave` · 오류 코드 `left`.

| 성공 기준 | 판정 | 근거 |
|---|---|---|
| A1 ⋯ 나가기 · 삭제(나간 대화 = 삭제만) · 확인 | 충족(자동) · 실화면 이월 | E8 node(`conv_menu_items` · `conv_closed` · `refused_left`) · E12 |
| A2 나간 뒤 이력 그대로 · 읽기 전용 · 그 대화로 온 요청 거절 | 충족 | E4(`leave_marks_both_read_only` · `leave_refuses_inbound` · `leave_peer_left_refuses_raw`) |
| A3 상대 읽기 전용 · 이전 버전 상대는 보낼 때 거절 | 충족 | E4(`leave_old_peer_gets_refusal`) |
| A4 삭제 정리 · 상대 사본 그대로 · 되살아나지 않음 · 번호 재사용 없음 | 충족 | E5(`delete_cleans_local_keeps_peer` · `delete_not_revived` · `delete_id_not_reused` · `delete_race_guards` · `delete_while_waiting_leaves_no_rows`) · E6 |
| A5 공유 사본 나가기 = 공유 나가기 · 삭제 = + 사본 삭제 | 충족 | E4 `leave_shared_copy` · E5 `delete_shared_copy` |
| A7 그룹 = 대화만 · 이전 · 상대 소속 API 없음 | 충족 | E6 `schema21_group_peers_to_convs` · E7 · E10 |
| A8 설정 연결 목록 · 복사 | 충족(자동) · 실화면 이월 | E9 node(`settings_group_sections` · `conv_copy`) · E12 |
| A9 실행 중 막음 · 대기 취소 · 자동 왕복 끔 | 충족 | E4 `leave_blocked_while_running` · E7 `leave_and_delete_routes`(409) |
| A6 회귀 없음 | 충족 | E1 · E2 · E3(node 142) · E11 |

- 적대적 평가 2 라운드(상한, `codex-review`) — 라운드 1 high 3 · medium 1(묘비 우회 TOCTOU → 잠금 안 재확인 + `open_conversation` 묘비 거부, `peer_left_at` 미검사 → `chat::left` 에
  포함, 취소를 덮는 무조건 실행 표시 → `chat::start_run` 조건부, 상세 재개 단추), 라운드 2 high 1 · medium 1(편집 · 삭제 등 형제 받기 경로 → `chat::find_open`, 대기 중 지운 대화에 실행
  알림 행 → `start_run` 뒤 알림). 미조치 3(barrier 시험 · DOM 시험 · 진행 저장 확인 — [eval](eval.md)) — 사용자 수용("1w 종결 승인"), 원장 `#1w R1-D1` · `R1-D2` · `R2-D1`.
- 구현자 해석(사용자 승인 — 종결 승인 2026-10-19, [stage1](work/stage1.md)): ① 나갈 때 상대를 기다리던 내 글(`sent`)도 닫는다 ② 공유 사본 삭제는 묘비 없음(다시 초대 = 새 사본)
  ③ 상대의 `left` 거절은 내 쪽 대화를 '상대가 나감' 으로 바꾸지 않는다.
- plan 밖 변경: 기존 시험 정리(`store.rs` schema18 2 · `share_observer_api` 의 '사본 아닌 나가기 400'), 문서 범위 확장(an_core · an_net · an_agent · 개발자 가이드 · 사용자 가이드 — 틀린
  상대 단위 그룹 서술).

## 산출

- an-core: 스키마 21(`conversations` AUTOINCREMENT · `left_at` · `peer_left_at` · `left_convs` · `group_peers` → `group_convs`), `chat` `closed` · `left` · `running` · `start_run` · `leave` ·
  `set_peer_left` · `delete_conversation` · `find_open` · `lower_marks`, `open_conversation` 묘비 거부, `groups` 상대 소속 삭제, `outbox::LEAVE`.
- an-net: `Request::Leave` · `Pong.leave` · `knows_leave` · 코드 `left`.
- an-agent: `leave_conversation` · `delete_conversation` · `inbound_leave`, 받기 앞 · 잠금 안 거절, `deliver` `leave`, `record` · `process` 닫힌 · 지운 대화 처리, `writable` 확장.
- an-app: `POST …/leave` = 모든 대화 · `DELETE /api/conversations/{id}`(409 실행 중), JSON `left_at` · `peer_left_at`, 상대 소속 경로 삭제.
- 웹UI: `convMenuItems` · `convClosed` · `settingsGroupSections` · `convCopy` · `USER_LIST`, 읽기 전용, 설정 연결 목록 다시, `.ico-exit`.
- 시험: Rust `conv_leave.rs` 13 · `schema21_*` 2 · `peer_routes_gone` · `group_json_has_no_peers` · `leave_and_delete_routes`, node 새 6 · 고침 2 · 지움 5(141 → 142), `ux_literals` 표.
- 문서: masterPlan D37 개정 · D40 구현 보강 · 개정 기록 · 종결 목록, PRODUCT 2줄, web-ui · an_app · an_core · an_net · an_agent · 개발자 가이드(7.2 · 7.5 · 8.12) · 사용자 가이드(4.1 · 4.2 · 5 · 7.8),
  원장 10행.

## 남은 것

- 19 인수 TC: 1w E12(두 PC — 나가기 양쪽 읽기 전용 · 삭제 · 사용자 목록 → 그룹 절 복사).
- 지운 대화 묘비(`left_convs`)는 지우지 않는다(보관 기한 없음 — 행이 작다).
