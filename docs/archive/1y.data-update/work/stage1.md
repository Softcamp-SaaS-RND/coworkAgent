# stage 1 — 서버(스키마 23 · archive · 기간 · VACUUM · 대기 되돌리기)

## 한 일

- an-core: 스키마 23(`messages.req_archived`), `db::vacuum_if_sparse` · `VACUUM_MIN_FREE`, `chat::ARCHIVE_DAYS` · `Message::archived(now)` · `archived_sql` · `set_req_archived` ·
  `unix_now`, `AskFilter::Request(state, archived)`, `req_counts` 의 `archived`, `set_req_state` 가 상태가 바뀌면 손 표시를 지움, `messages_page` · `peer_message_counts` 의 `since`,
  `run_opts::PEER_WEEKS_*` · `peer_weeks`.
- an-app: `MessageQuery.archived`(`ask=request` 와 함께만) · `POST /api/messages/{id}/archive` · 메시지 JSON `archived` · `req_counts.archived`, `side=both` · 상대별 요약에 기간,
  설정 `ui_peer_weeks`(1 ~ 12), 시작 때 VACUUM(실패 = 경고), `PATCH` `req_state` 가 `wait` 도 받음(해석 7).

## plan 과 다른 점

- `Message` 의 필드는 `archived: bool` 이 아니라 **원래 열 `req_archived: Option<bool>` + `Message::archived(now)`** — `MSG_COLS` 는 별칭을 붙이는 열 목록이라 식을 넣지 못한다.
  규칙은 SQL(`archived_sql`)과 Rust(`archived`) 두 곳 — 시험 `request_archive_auto`(목록 · JSON) · `schema23_req_archived`(경계 1초)가 둘을 함께 본다.
- 기존 시험 전제 고침: `requests.rs` `req_counts_beyond_page` 의 완료 501건이 `created_at` 100 대(1970년)라 이제 archive 로 센다 → `req_at = unixepoch()`(방금 완료).
  `store.rs` `schema22_messages_autoincrement` 의 `user_version` 22 → `SCHEMA_VERSION`, `paths.rs` 22 → 23, `AskFilter::Request` 둘째 인자 · `messages_page` · `peer_message_counts` `since` 인자.

## 테스트 결과

```text
cargo test -p an-core --test store -- schema23_req_archived vacuum_ schema22  → test result: ok. 4 passed; 0 failed
cargo test -p coworkagent-app --test ux_api -- peer_weeks_setting_persists peer_messages_within_weeks request_archive_ req_state_back_to_wait → test result: ok. 8 passed; 0 failed
cargo test --workspace → 78 바이너리 479 passed · 0 failed
cargo clippy --workspace --all-targets -- -D warnings → 경고 0
```
