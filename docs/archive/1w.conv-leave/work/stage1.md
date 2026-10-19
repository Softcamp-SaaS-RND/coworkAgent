# stage 1 — 저장 · 서버(스키마 21 · Leave 프레임 · 나가기 · 삭제 · 그룹 = 대화만)

## 목표

plan stage 1 — an-core 스키마 21 · 나가기 · 삭제 · 그룹 서버 정리, an-net `Leave` · `Pong.leave`, an-agent 나가기 · 삭제 · 받기 거절, an-app 경로 + Rust 시험(E1 · E2 · E4 ~ E7).

## 산출 파일

- `crates/core/src/db.rs` — 스키마 21: `conversations` AUTOINCREMENT 로 다시 만듦(스키마 17 `peers` 방법, 순번 = 대화 · 딸린 표 `conversation_id` · outbox `conv_id` 최대) ·
  `left_at` · `peer_left_at` · 묘비 `left_convs` · `group_peers` → `group_convs` 옮기고 DROP.
- `crates/core/src/chat.rs` — `Conversation.left_at` · `peer_left_at` · `closed()`, `LEFT` · `left` · `running` · `leave` · `set_peer_left` · `delete_conversation`(+ 낮추기 묶음
  `lower_marks` — `delete_message` 와 같이 씀), `Scope::sql` 상대 소속 갈래 삭제.
- `crates/core/src/groups.rs` — `Group.peers` · `unknown_peers` · `change_peers` · `change_peer_groups` · `remove` 의 상대 소속 지우기 삭제.
- `crates/core/src/outbox.rs` `LEAVE` · `crates/core/src/share.rs` `forget_peer` 가 나가기 행도 지움.
- `crates/net/src/proto.rs` — `Request::Leave{conv}`(`conv()` · `ids_ok`), `Pong.leave`, 오류 코드 `left`. `crates/net/src/node.rs` — 처리기 넘김 목록 · `Pong{leave: true}` · `knows_leave`.
- `crates/agent/src/chat.rs` — `leave_conversation` · `delete_conversation` · `inbound_leave`, `handle` 앞 나간 대화 거절, `deliver` 의 `leave`(모르는 상대 = 버림) · `backoff` 포기 제외,
  `refused()` `left` 문구, `writable` 이 나간 대화도 막음(+ 이력 정리 · 공유 넣기 · 자동 왕복), `process` 의 취소된 대기 글 건너뜀, `record` 가 지운 대화 = 답 버림 · 나간 대화 = 이력에만.
- `crates/app/src/chat_api.rs` — `POST /api/conversations/{id}/leave` = 모든 대화, `DELETE /api/conversations/{id}`, 실행 중 409(`ResourceBusy`), 대화 JSON · 글의 `conversation` 요약에
  `left_at` · `peer_left_at`. `crates/app/src/groups_api.rs` — 상대 소속 두 경로 · `group_json.peers` 삭제.
- 시험: `crates/agent/tests/conv_leave.rs`(새 10) · `common::old_peer`(share.rs 에서 옮김 — Pong 을 인자로), `crates/core/tests/store.rs` `schema21_*` 2 + 고침,
  `crates/core/tests/paths.rs` 버전 21, `crates/app/tests/groups_api.rs` 상대 소속 4 → `peer_routes_gone` · `group_json_has_no_peers`, `chat_api.rs` `leave_and_delete_routes`,
  `Pong` 모양 2(`context.rs` · `remote_input.rs`).

## 설계 결정

- 나가기 한 트랜잭션 — 관람자 공유 끝은 `unshare` 를 부르지 않고 같은 트랜잭션 안에서 `share::remove` + `ShareEnd`(`unshare` 는 자기 트랜잭션을 연다). 원 상대에게 관람자 수 알림은 보내지 않는다
  (그 대화는 바로 나감을 받는다 — "관람자 0" 알림만 늘어난다).
- 실행 중 확인은 나가기 트랜잭션(쓰기 잠금) 안에서 — `running` 표시와 같은 잠금. 오류 종류 `ResourceBusy` → 409(`fail` 에 추가).
- 나가며 버리는 보낼 글 = 그 대화 · 원 상대 앞 `pending` 행만(보내는 중 `inflight` 는 그 결과가 정한다, 실패 보관은 삭제 때 지운다).
- `delete_conversation`(core) 의 outbox — 관람자 앞 `ShareEnd` 행은 `conv_id = NULL` 로 떼어 남긴다. `Leave` 행은 처음부터 `conv_id` 없이 넣는다(plan). `deliver` · `settle` · `watching` 은
  `conv_id` 없는 공유 행을 이미 보낸다(ShareEnd 갈래는 `watching` 을 건너뛴다) — 고칠 곳 없음(plan 의 구현자 확인).
- `process` — 줄(실행 문)을 서는 동안 나가기가 대기 글을 취소했으면 실행하지 않는다(DB 상태를 다시 읽는다).

### 구현자 해석(사용자 확인 필요)

1. **나간 쪽의 '보낸 뒤 기다리는 내 글'도 닫는다** — plan 은 `chat::leave` 가 `queued` 만 취소한다고 적었다. 나가며 그 대화의 보낼 줄을 버리고 상대는 답을 이력에만 남기므로, 나간 쪽의 `sent` 내 글
   (상대 답 · 받음 확인을 기다림)은 영영 '보냄' 으로 남는다 → `leave` · `set_peer_left` 가 같은 규칙(`queued` 또는 `mine = 1 AND sent` → `error` + `refused = left`)으로 닫는다
   (`inbound_leave` 쪽은 plan 의 구현자 해석과 같다).
2. **공유 사본을 지울 때는 묘비를 남기지 않는다** — plan 은 묘비 범위를 정하지 않았다. 사본에 묘비를 남기면 대화를 가진 PC 의 다시 초대(`Share`)가 `left` 로 거절되고, 그 PC 는 그 코드로
   관람자를 빼지 않아(`share_refused` 는 `unsupported` · `not_paired` 만) 죽은 관람자가 남는다. 사본은 지금 규칙("다시 초대 = 새 사본")을 따른다(`delete_shared_copy` 가 다시 초대를 본다).
3. 상대가 `left` 로 거절해도 내 쪽 대화를 '상대가 나감' 으로 바꾸지는 않는다(plan 범위 밖 — 나가기 프레임만 그 표시를 만든다). 이전 버전 상대 몫은 보낼 때 받는 거절 문구로 안다.

### plan 밖 변경(기존 시험)

- `store.rs` `schema18_group_includes_peer_conversations` 삭제 — 상대 소속 API(`change_peers`)가 없어졌다(스키마 21 시험이 이전을 본다). `schema18_peer_groups_from_old_text` 는 상대 소속 기대를
  대화 소속(빈 목록 — 그 상대들의 대화가 없다)으로, 설정 단계는 `DROP TABLE IF EXISTS group_peers`. `schema20_share_tables_from_19` 의 버전 단정 20 → `SCHEMA_VERSION`.
- `chat_api.rs` `share_observer_api` — "사본 아닌 대화의 나가기 400" 단정 삭제(D40 — 모든 대화 나가기 204, `leave_and_delete_routes` 가 본다).

## 테스트 결과

- core: 구현 뒤 `cargo test -p an-core` 전부 통과, `schema21` 2 passed.
- conv_leave: 10 passed. 물기 확인(mutation) — `handle` 앞 거절을 끄고 `record` 의 나간 대화 거르기를 끈 빌드에서 `delete_not_revived` · `leave_refuses_inbound` ·
  `leave_old_peer_gets_refusal` · `leave_blocked_while_running` 4 FAILED → 되돌림.
- gate(이 stage):

```text
E4  cargo test -p an-agent --test conv_leave leave   → test result: ok. 6 passed; 0 failed; … 4 filtered out
E5  cargo test -p an-agent --test conv_leave delete  → test result: ok. 4 passed; 0 failed; … 6 filtered out
E6  cargo test -p an-core schema21                   → test result: ok. 2 passed; 0 failed; … 15 filtered out
E7  cargo test -p coworkagent-app -- peer_routes_gone group_json_has_no_peers leave_and_delete_routes
                                                     → chat_api ok. 1 passed · groups_api ok. 2 passed
E1  cargo test --workspace                           → test result 76 줄 모두 ok, failed 0
E2  cargo clippy --workspace --all-targets -- -D warnings → 경고 0
E10(서버 몫)                                          → group_peers_live=0 peer_api=0 (web=6 — stage 2)
```

## 다음 단계

stage 2 — 웹UI(그룹 = 대화만 · 설정 연결 목록 · 대화 메뉴 나가기 · 삭제 · 읽기 전용) · 문서, E3 · E8 ~ E11.
