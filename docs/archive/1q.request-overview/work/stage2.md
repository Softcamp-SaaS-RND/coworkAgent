# stage2 — 내 활동(받은 요청 · 내 요청 · 받은 답 · 실패)

## 목표

plan Stage 2 — 해석 6 · 7: 내 활동을 받은 요청 · 내 요청 · 받은 답 · 실패로 좁히고, 상대가 내 요청 상태를 바꾸면 안 본 항목으로. 화면 거르기 = 탭.

## 산출 파일

- `crates/core/src/chat.rs` — `ActivityFilter`(All · Request · MyRequest · Answer · Failed) · `ActivityKind`(Message · Failed · MyRequest · SyncFailed — RelayStart ·
  RelayEnd 지움) · `RECV_REQUEST` · `RECV_ANSWER` · `MINE_FAILED` · `MY_REQUEST` · `activity_messages()` · `note_req_state` · `activity` · `activity_unseen`.
- `crates/agent/src/chat.rs` — `inbound_req_state` 가 상태를 저장한 뒤 `note_req_state`(같은 상태 · 메모 재전송은 그 전에 돌아가 기록 없음).
- `crates/app/src/chat_api.rs` — `kind=all|request|mine|answer|failed`(옛 human · agent · relay 400).
- `crates/app/web/{index.html,app.js,view.js}` — `#act_tabs` 탭(`ACT_KINDS`) · `state.actKind` · `activityRows` 새 라벨 · 메모 줄 · 라벨 태그.
- 시험: core `store.rs` `activity_scope` · agent `requests.rs` `activity_my_request_state_unseen` · app `ux_api.rs` `activity_kinds_scope`(옛
  `activity_merges_theirs_failures_relay_events` 를 새 범위로) · `activity_unseen_by_peer_and_mark_seen`(받은 글을 요청으로 · 자동 왕복 기록 대신 상대 반영
  실패 + 실패) · `ask_kind_recorded_and_filtered`(`kind=request` 200) · `message_tidy_api` · `models_tools`(request 200 · query 400) · core `paths.rs` ·
  agent `relay.rs`(자동 왕복 기록은 `relay_events` 로 단정, 내 활동에 없음) · node `activity_rows` · `activity_rows_tabs` · `row_peer` · `ask_marks` · `ux_literals` 1q 행.

## 설계 결정

- 받은 답 = `reply_to` 가 내 사람 글인 상대 에이전트 답 — 자동 왕복에서 내 에이전트 글에 단 답은 빠진다(첫 답만 받은 답).
- 내 요청 시각 = `coalesce(req_at, created_at)`, 안 본 = 그 글의 `req_state` 기록 id > `activity_seen_event`. 안 본 수는 요청마다 하나(`count(DISTINCT)`).
- 자동 왕복 시작 · 끝 기록(`relay_events`)은 그대로 남는다 — 내 활동 목록 · 안 본 수에서만 뺐다.

## 테스트 결과

- RED: core `activity_scope` — `no variant or associated item named MyRequest`(컴파일 실패). agent `activity_my_request_state_unseen` — `note_req_state` 를 막으면
  `panicked at crates\agent\tests\common\mod.rs:111:9` · `FAILED. 0 passed; 1 failed`. node `activity_rows` — `# pass 0 # fail 1`.
- GREEN: `cargo test --workspace --no-fail-fast` → `test result: ok` 75 묶음 · FAILED 0. `cargo clippy --workspace --all-targets -- -D warnings` → `Finished`.
  `node --test …` → `# pass 116` `# fail 0`.
- mock 눈 확인(다크): 내 활동 탭 전체 · 받은 요청 · 내 요청 · 받은 답 · 실패, "받은 요청 · 대기" · "내 요청 · 수용"(메모) · "받은 답" 태그, JS 오류 없음.

## 다음 단계

stage3 — 주고받은 메시지(요약 · 페이지 API · 화면 · 북마크). 같은 때 들어온 지시 `coworkAgent.exe /?` 는 plan 개정 1(Stage 5).
