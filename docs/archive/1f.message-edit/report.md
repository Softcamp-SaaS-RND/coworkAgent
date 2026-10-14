# report — 1f.message-edit

종결: 2026-10-14 · 브랜치 `task/1f-message-edit` → main

## 결과

대화 내용을 바로 옮겨 쓰고, 끝난 대화를 사용자가 스스로 정리할 수 있다. 말풍선 머리줄 오른쪽 끝(우상단)에 **늘 보이는 복사 아이콘** — 누르면 본문
markdown 원문이 클립보드로("복사했다"). 입력창 위 **"편집"** 을 누르면 편집 모드 — 입력창 자리가 "n개 선택 · 삭제 · 완료" 막대가 되고, **끝난 내
사람 글**(웹UI · CLI)에만 체크박스 · 연필이 붙는다. 여러 글을 골라 삭제(흔적 없이 — 답은 남는다), 연필로 본문 편집("(편집됨)"). 삭제 · 저장 때마다
"내 창에서만" · "상대 창에도" 를 고른다. 상대 창 반영은 새 프레임 `Edit` · `Delete` 로 outbox 를 거쳐(꺼져 있으면 나중에) 가고, 받는 쪽은 그 상대가 쓴
사람 글에만 적용한다. 거절 · 24시간 포기는 내 활동 "상대 반영 실패". 편집 · 삭제는 실행 · 다시 보내기 · 알림을 일으키지 않는다.

| 성공 기준 | 판정 | 근거 |
|---|---|---|
| A1 복사 아이콘 늘 보임 · "복사했다" · 작업 막대 복사 없음 | 충족(문구 · 로직) · 화면 19 TC | E3 `message_actions` · E7 `tidy_literals` · E8 ABANDON |
| A2 입력창 위 "편집" · 편집 막대 · 지금 대화만 | 충족(문구 · 자리) · 화면 19 TC | E7(편집 단추가 입력창 앞) · E3 `keep_picked` · E8 ABANDON |
| A3 끝난 내 사람 글만 · API 도 거절 | 충족 | E4 `only_finished_own_human_messages` · E6 `patch_text_rules_and_mirror` · `delete_all_or_nothing` |
| A4 내 창에서만 완전 삭제 · 답은 남음 · 상대 그대로 | 충족 | E4 `delete_mine_only_here` · `deleting_message_removes_its_edit_failure_note` · `reused_id_after_delete_stays_unread` · E6 |
| A5 상대 창에도 · 꺼져 있으면 나중에 · 실패면 내 활동 | 충족(두 PC 통합 시험) · 실측 19 TC | E4 `delete_mirrored_leaves_peer_request_list` · `edit_mirrored_keeps_order_and_state` · `offline_peer_gets_it_later` · `give_up_and_refusal_noted_in_activity` · `refusal_note_and_row_removal_are_atomic` · E6 `activity_lists_sync_failed` |
| A6 "(편집됨)" · lc · 요청 상태 그대로 | 충족 | E4 `edit_mirrored_keeps_order_and_state` · E3 `edited_tag` · E6 `messages_carry_editable_and_edited_at` |
| A7 실행 · 알림 없음 | 충족 | E4 `tidy_runs_and_notifies_nothing` |
| A8 그 상대가 쓴 사람 글에만 | 충족 | E5 `forged_edit_delete_refused` |
| A9 검증 명령 | 충족 | E1(347) · E2 · E3(50) · `check_readme` · `check_crates` · `check_ddd_docs` |

- 적대적 평가 2 라운드(상한, codex-review 세션) — 라운드 1: medium 1(거절 행 지우기와 실패 기록이 따로 — 중단 · 쓰기 실패면 둘 다 사라짐) — 사실 ·
  응답 정산 한 트랜잭션으로 고침(plan 개정 1). 라운드 2: medium 1(편집 실패 기록의 본문 미리보기가 글을 지운 뒤에도 남음) — 사실 · 기록을 그 글에
  이어 함께 지움(plan 개정 2). 미조치 0. 종결 전 자체 검토로 맨 끝 글 삭제 뒤 id 재사용 → 새 글이 읽음으로 보이던 결함을 고침(plan 개정 3).
- E8(두 PC 사람 확인)은 이 PC 에서 사용자 앱이 실행 중(PID 7364)이고 두 번째 PC 가 없어 → 19 TC(`ABANDON`).
- masterPlan: 비목표 "메시지 수정/삭제 동기화" 삭제 · **D34 추가** · S3 보강 · D32 적용(새 프레임 · 스키마 15).

## 산출

- `an-core`: 스키마 15 `messages.edited_at` · `edit_text` · `delete_message` · `RelayMark::SyncFailed(Option<글 id>)` · `ActivityKind::SyncFailed` ·
  `outbox::EDIT` · `DELETE`(`for_message` 가 뺀다).
- `an-net`: `Request::Edit` · `Request::Delete`.
- `an-agent`: `Chat::tidy_ok` · `edit_message` · `delete_messages` · `acked`(응답 정산 한 트랜잭션) · `inbound_edit` · `inbound_delete`.
- `coworkagent-app`: `PATCH /api/messages/{id} {text, mirror}` · `POST /api/messages/delete` · `editable` · `edited_at` · 웹UI 복사 아이콘 · 편집 모드 ·
  반영 범위 `<dialog>`.
- 시험: `agent/tests/message_tidy.rs`(11) · `app/tests/message_tidy_api.rs`(4) · `ux_literals` `tidy_literals` · `view.test.mjs`(+3, 갱신 1).
- 문서: README(주요 기능) · user-guide(7.5 · 7.6 · 16) · developer-guide(5.2 · 6.6 · 7.2 · 7.3 · 8.9) · an_core · an_net · an_agent · an_app · PRODUCT ·
  masterPlan.

## 남은 것

- 19 인수 TC — 두 PC: 복사 아이콘 · 편집 모드 켜고 끄기 · 여러 글 삭제(내 창 · 상대 창) · 편집(내 창 · 상대 창 "(편집됨)") · 상대를 끈 채 편집 · 삭제 뒤
  켜면 반영 · 받은 요청 목록에서 빠짐 · 다른 대화로 가면 편집 모드 꺼짐 · 고치는 중 5초 새로 고침에 입력이 지워지지 않음(E8 이관).
- 범위 밖 관찰: 기존 outbox 종류(`req_state` 거절 등)도 "행을 지운 뒤 정산" 순서다 — 같은 원자성 보강이 필요한지는 별도 판단(eval 라운드 1 대조).
- GitHub 반영(push)은 사용자 확인 뒤.
