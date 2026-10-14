# stage1 — 저장 · 프로토콜 · 편집 · 삭제 · 상대 반영 · API

> 근거: [plan](../plan.md) Stage 1. 사용자 검증은 D28 — 19.prototype-acceptance 에서 일괄.

## 목표

끝난 내 사람 글의 편집 · 삭제(A3 · A4 · A6), 상대 창 반영 · 꺼진 상대 · 실패 알림(A5), 실행 · 알림 없음(A7), 위조 거절(A8). 대상 gate: E1 · E2 ·
E4 · E5 · E6.

## 산출 파일

- `crates/core/src/db.rs` — 스키마 15(`messages.edited_at`, 한 트랜잭션) · `SCHEMA_VERSION` 15.
- `crates/core/src/chat.rs` — `Message.edited_at`(`MSG_COLS` 끝 · `MSG_N` 24) · `edit_text` · `delete_message` · `RelayMark::SyncFailed` ·
  `ActivityKind::SyncFailed` · `activity`(거르기별 기록 종류) · `activity_unseen`.
- `crates/core/src/outbox.rs` — `EDIT` · `DELETE` · `for_message` 가 둘을 뺀다.
- `crates/net/src/proto.rs` · `node.rs` — `Request::Edit` · `Request::Delete`(페어링된 상대만 · `ids_ok`).
- `crates/agent/src/chat.rs` — `tidy_ok` · `tidy_targets` · `edit_message` · `delete_messages` · `mirror_tidy` · `sync_failed` · `sync_failed_item` ·
  `theirs` · `inbound_edit` · `inbound_delete` · `settle` · `give_up` 분기.
- `crates/app/src/chat_api.rs` — `PATCH /api/messages/{id} {text, mirror}` · `POST /api/messages/delete` · `editable` · `edited_at`.
- 테스트 — `agent/tests/message_tidy.rs`(새, 8) · `app/tests/message_tidy_api.rs`(새, 4) · `core/tests/paths.rs` · `requests.rs`(스키마 15).

## 설계 결정

- **편집 가능 판정은 서버 한 곳**(`Chat::tidy_ok`) — 글 JSON `editable` 로 싣고 웹UI 는 그 값만 본다(규칙 중복 없음).
- **반영 행은 글마다 하나** — outbox 대화 순서(`has_before`)가 그대로 지켜진다(편집 뒤 삭제는 그 순서로 닿는다). 반영 행은 `for_message` 에서
  빼서 반영 대기 중인 글이 "보내는 중" 으로 보이거나 다시 편집 · 삭제가 막히지 않게 했다.
- **상대 반영 실패 = `relay_events` `sync_failed`** — 지운 글은 행이 없어 글에 표시할 수 없다. 편집 · 삭제를 한 길로(행의 `conv_id` · 프레임).
- 받는 쪽 실행 대기 · 실행 중 글은 `busy` — 실행 중인 글 행을 지우지 않게(보내는 쪽 판정상 생기지 않아야 하지만 원격 입력은 믿지 않는다).
- 새 함수는 `Chat` 연관 함수 · 메서드(파일 전용 자유 함수 없음).

## 테스트 결과

```text
$ cargo test -p an-agent --test message_tidy → test result: ok. 8 passed; 0 failed
$ cargo test -p coworkagent-app --test message_tidy_api → test result: ok. 4 passed; 0 failed
$ cargo clippy --workspace --all-targets -- -D warnings → Finished
```

RED: 구현 전에는 새 API(`Chat::tidy_ok` · `edit_message` · `delete_messages` · `Request::Edit`)가 없어 두 시험 파일이 컴파일되지 않는다.
첫 실행에서 `SCHEMA_VERSION` 을 올리지 않아 8건 모두 실패("DB 스키마 15 가 이 실행 파일(14)보다 새롭다") → 고침. 검색 단정이 에이전트 답(본문을
되돌림)에 걸려 실패 → 단정을 지운 글 id 기준으로 고침.
