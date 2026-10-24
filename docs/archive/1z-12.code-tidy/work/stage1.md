# stage 1 — Rust 도메인 모듈

## 목표

plan "설계 · stage 1" — `an-core::chat` · `an-agent::chat` · `app::chat_api` 를 도메인 하위 모듈로(옮기기만 — 본문 바이트 그대로, 공개 경로 그대로).

## 산출 파일

- `crates/core/src/chat/`: `mod.rs` · `conversation.rs` · `message.rs` · `request.rs` · `activity.rs` · `search.rs` · `relay.rs` · `session.rs` — `mod.rs` 의 `pub use <모듈>::*` 로 `an_core::chat::X` 경로 그대로.
- `crates/agent/src/chat/`: `mod.rs` · `queue.rs` · `tx.rs` · `delivery.rs` · `send.rs` · `inbound.rs` · `request.rs` · `tidy.rs` · `relay.rs` · `execute.rs` · `context.rs` · `sharing.rs` · `lifecycle.rs` — 모듈마다 `impl Chat`.
- `crates/app/src/chat_api/`: `mod.rs`(`routes` · 응답 · JSON 도우미) · `agent.rs` · `conversation.rs` · `message.rs` · `run.rs` · `activity.rs` · `tx.rs`.

## 설계 결정

- 이름: plan 표의 `outbox` · `share` → `delivery` · `sharing`(`an_core::{outbox, share}` import 와 이름이 겹침), `chat_api` 의 `agents` → `agent`(`an_core::agents` 와 겹침) · 나머지 단수형.
- 나누는 스크립트(scratchpad `rssplit.py`): 항목 = 최상위 항목 · `impl Chat` 메서드 + 바로 위 문서 주석 · 속성. 표에 없는 비공개 항목 중 한 모듈만 쓰는 것은 그 모듈로(자동), 여럿이 쓰면 `mod.rs`.
- 가시성: 다른 하위 모듈이 부르는 메서드만 `pub(super)`(컴파일 오류로 찾아 붙임 — `an-agent` 47곳), `chat_api` 하위 모듈의 처리기 · 요청 타입은 `routes` 가 쓰므로 `pub(super)`.

## 옮기기만 확인

`rsverify.py` — 나누기 전 파일의 항목 본문이 배정된 파일에 바이트 그대로 있고(`pub(super) ` 만 빼고 비교), 그 밖에 더한 줄은 모듈 머리 · `mod` · `use` · `impl Chat {` · `}` 뿐:

- `an-core chat`: items 142 · files 8 · problems 0 (전 항목을 이어 붙이면 원본과 같음 — 70,208 글자)
- `an-agent chat`: items 212 · files 13 · problems 0 (원본과 다른 줄 = `impl Chat {` · 끝 `}` 둘 — 모듈마다 다시 씀)
- `app chat_api`: items 100 · files 7 · problems 0

## 배정 표

`an-core chat/` (142 항목)

| 모듈 | 항목 |
| --- | --- |
| `mod` | `Dir` · `Author` · `State` · `ReqState` · `impl ReqState` · `AskFilter` · `Conversation` · `impl Conversation` · `Relay` · `Message` · `ARCHIVE_DAYS` · `impl Message` · `unix_now` · `archived_sql` · `Summary` · `LastMessage` · `PREVIEW_CHARS` · `TITLE_CHARS` · `impl Dir` · `impl Author` · `impl State` · `state_of` · `bad` · `MSG_COLS` · `prefixed` · `like_pattern` · `msg_row` · `lower_marks` · `stamp` |
| `request` | `MAX_REQ_NOTE` · `clean_req_note` · `note_to_wire` · `note_from_wire` · `note_req_state` · `set_request` · `set_req_state` · `set_req_archived` · `set_refused` · `ReqCounts` · `impl ReqCounts` · `req_counts` · `waiting_requests` · `add_agent_request` |
| `conversation` | `CONV_COLS` · `CONV_N` · `conv_row` · `open_conversation` · `find_own` · `find_open` · `find` · `by_id` · `list` · `set_title` · `set_starred` · `marks` · `mark_read` · `LEFT` · `left` · `gone` · `mark_gone` · `empty` · `busy` · `close_pending` · `leave` · `set_peer_left` · `delete_conversation` · `clear_conversation` · `set_agent` · `set_cmd` · `ConvInfo` · `info` · `pending_conversations` · `summaries` |
| `activity` | `MSG_N` · `ActivityKind` · `impl ActivityKind` · `ActivityFilter` · `Activity` · `RECV_REQUEST` · `RECV_ANSWER` · `MINE_FAILED` · `MY_REQUEST` · `activity_messages` · `activity` · `activity_unseen` · `activity_upto` · `mark_activity_seen` |
| `message` | `set_saved` · `edit_text` · `delete_message` · `Scope` · `impl Scope` · `saved` · `set_via` · `VIA_CLI` · `message_by_id` · `add_message` · `fill_mine_exec` · `add_human` · `MAX_LC` · `observe` · `notes_between` · `last_exec_before` · `set_state` · `messages` · `messages_tail` · `messages_across` · `messages_page` · `peer_message_counts` · `message_by_uuid` · `pending` · `add_reply` · `set_flags` · `note_failed` |
| `session` | `start_run` · `running` · `agent_session` · `set_agent_session` · `set_ctx_tokens` · `add_context` · `finish_context` · `sessions` · `clear_sessions` · `set_ctx_seed` · `ctx_seed` · `drop_ctx_seed` · `set_session` · `set_run` |
| `relay` | `RelayMark` · `add_relay_event` · `set_relay` · `mark_relayed` · `relay_on` · `relay_step` · `set_blocked` |
| `search` | `Found` · `FoundAgent` · `SEARCH_OTHERS` · `closed_sql` · `search` · `search_messages` |

`an-agent chat/` (212 항목)

| 모듈 | 항목 |
| --- | --- |
| `mod` | `Run` · `MAX_RELAY_CAP` · `DEFAULT_RELAY_CAP` · `RELAY_CAP_KEY` · `RELAY_END_CAP` · `default_relay_cap` · `MAX_RUNS_KEY` · `DEFAULT_MAX_RUNS` · `MAX_RUNS` · `max_runs` · `Progress` · `MESSAGE` · `RELAY_OPEN` · `RELAY_STEP` · `NOTE` · `REPLY` · `REPLY_POST` · `RELAY_LAST` · `REQ_STATE` · `RELAY` · `PROGRESS` · `CTX` · `CTX_DONE` · `SHARE` · `SHARE_NOTICE` · `LEAVE` · `CLEAR` · `READ_ONLY` · `CLOSED` · `CLEAR_UNKNOWN` · `CTX_UNSUPPORTED` · `UNREACHABLE` · `STOPPED` · `RETRY` · `Shipped` · `TxView` · `Lane` · `Watch` · `Retry` · `PREVIEW_CHARS` · `INTERRUPTED` · `MAX_NOTES` · `MAX_NOTES_BYTES` · `Chat` · `new_uuid` · `preview` · `fit_to` · `Compose` · `no_group` · `REQUEST_NO_RELAY` · `FOCUS` · `refused` · `addr` · `new` · `attach` · `recover` · `db` · `notify` · `turn` · `writable` · `conversation` |
| `send` | `MIRROR` · `MENTION_ONLY` · `REQUEST_NO_MENTION` · `REQUEST_NO_CLI` · `request_ok` · `paired_peer` · `remote_agents` · `send` · `send_via` · `send_with` · `open_session` · `send_human` · `post` · `post_via` · `post_with` |
| `lifecycle` | `LEAVE_BUSY` · `CLEAR_BUSY` · `CLEAR_SENDING` · `leave_conversation` · `delete_conversation` · `clear_conversation` · `inbound_clear` · `inbound_leave` |
| `context` | `CTX_FAILED` · `peer_code` · `MAX_SUMMARY` · `Seed` · `summary_text` · `context` · `request_context` · `inbound_context` · `context_refused` · `run_context` · `compact` |
| `inbound` | `MAX_CTX_TOKENS` · `run_meta` · `MessageExtra` · `handle` · `agents` · `inbound_message` · `inbound_post` · `inbound_ask` · `inbound_progress` · `inbound_reply` |
| `execute` | `FRAMES` · `unframe` · `fit` · `Ran` · `Failure` · `impl Failure` · `RUN_FAILED` · `RECORD_FAILED` · `RESULT_ERROR` · `EMPTY_ANSWER` · `FOLDER_BLOCKED` · `wake` · `process` · `focused` · `resent` · `refused_by_peer` · `execute` · `note_cmd` · `with_notes` · `record` |
| `queue` | `run_queue` · `tell_waiting` · `set_max_runs` · `progress_of` · `admit` · `send_progress` · `send_step` · `set_step_every` |
| `tx` | `set_watch` · `tx_by_key` · `tx_of_message` · `keyed` · `tx_resume` · `tx_view` · `watch_tx` |
| `delivery` | `set_retry` · `stage` · `ship` · `dispatch` · `enqueue` · `send_to` · `backoff` · `kick` · `lane` · `deliver` · `acked` · `conv_of` · `settle` · `give_up` · `peer_back` · `outbox_of` · `ask_status` · `inbound_status` |
| `relay` | `start_relay` · `start_relay_with` · `relay_end` · `inbound_relay` · `block_relay` · `relay` · `relay_forward` · `relay_stopped` |
| `request` | `set_req_state` · `req_state_refused` · `inbound_req_state` |
| `tidy` | `tidy_ok` · `tidy_targets` · `edit_message` · `delete_messages` · `mirror_tidy` · `note_sync_failed` · `theirs` · `inbound_edit` · `inbound_delete` |
| `sharing` | `share_add` · `share_head` · `share_room` · `share_remove` · `share_leave` · `reshare` · `watching` · `unshare` · `notice_watchers` · `fan_share` · `fan_share_in` · `share_change` · `stage_share` · `share_post` · `share_refused` · `inbound_share` · `live_copy` · `inbound_share_post` · `inbound_share_delete` · `inbound_share_end` · `inbound_share_notice` |

`app chat_api/` (100 항목)

| 모듈 | 항목 |
| --- | --- |
| `mod` | `MAX_AGENT_NAME` · `DELETE_MAX` · `SEARCH_MAX_CHARS` · `SEARCH_DEFAULT` · `SEARCH_MAX` · `SavedQuery` · `MessageQuery` · `MESSAGES_DEFAULT` · `MESSAGES_MAX` · `routes` · `fail` · `scope_of` · `db_fail` · `bad` · `via_of` · `to_of` · `conversation_json` · `message_json` · `conversation_ref` · `summary_map` · `message_in` · `summary_json` |
| `agent` | `NewAgent` · `AgentPatch` · `agent_json` · `checked_folder` · `list_agents` · `add_agent` · `update_agent` · `codex_options` · `remove_agent` · `remote_agents` |
| `conversation` | `NewConversation` · `ConversationPatch` · `ReadUpto` · `RelaySwitch` · `ContextOp` · `share_json` · `list_conversations` · `start_conversation` · `patch_conversation` · `read_conversation` · `conversation_info` · `switch_relay` · `context_op` · `NewObserver` · `add_observer` · `remove_observer` · `leave_conversation` · `delete_conversation` · `ClearReq` · `clear_conversation` |
| `message` | `MessagePatch` · `MessageDelete` · `TailQuery` · `ArchivePatch` · `list_messages` · `list_across` · `message_page` · `peer_since` · `message_peers` · `save_message` · `archive_message` · `delete_messages` |
| `activity` | `SearchQuery` · `ActivityQuery` · `ActivitySeen` · `req_counts` · `search` · `activity` · `activity_seen` · `saved_list` |
| `tx` | `NewMessage` · `continue_conversation` · `tx_key_of` · `tx_again` · `chat_conv` · `sent_tx` · `tx_json` · `get_tx` · `resume_tx` |
| `run` | `run_json` · `list_runs` · `set_runs` |

## 테스트 결과

- `cargo clippy --workspace --all-targets -- -D warnings` → `Finished`.
- `cargo test --workspace --no-fail-fast` → `FAILED` · `panicked` 0줄 · `test result: ok` 83줄.

## 다음 단계

stage 2 — 웹 ES 모듈.