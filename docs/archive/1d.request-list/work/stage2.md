# stage2 — 요청 = 실행 없음 · 상태 바꾸기 · `ReqState` 동기화

> 근거: [plan](../plan.md) Stage 2. 사용자 검증은 D28 — 19.prototype-acceptance 에서 일괄.

## 목표

요청 글은 받는 PC 에이전트를 실행하지 않고 받는 PC 요청 목록에 '대기' 로 남는다(집중 중에도 — A3). 받은 사람만 대기 → 수용 · 반려, 수용 → 완료로
바꾸고(한 줄 메모) 그 상태가 outbox 로 요청자에게 간다(A4). 대상 gate: E1 · E2 · E4 · E5 · E6.

## 산출 파일

- `crates/core/src/chat.rs` — `ReqState`(`as_str` · `parse` · `can_become`) · `MAX_REQ_NOTE` · `clean_req_note` · `AskFilter`(질의 · 요청 + 상태),
  `Message.req_state` · `req_note` · `req_at`(`done_at` 대체, `MSG_N` 23), `set_request` 가 요청이면 `req_state = coalesce(…, 'wait')` · 질의면 `NULL`,
  `set_req_state`(`set_done` 대체), `messages_across` 의 `request: Option<bool>` → `ask: Option<AskFilter>`(인자 수 그대로), `ActivityFilter::Query` ·
  `Request` 삭제.
- `crates/net/src/proto.rs` — `Request::ReqState{conv, msg, state, note}`(`Done` 삭제) · `ids_ok`. `node.rs` — 처리기 목록 · `MessageKind::Ask` · `AskState`.
- `crates/agent/src/chat.rs` — `request_ok`(요청 + 멘션 · CLI 거부: `REQUEST_NO_MENTION` · `REQUEST_NO_CLI`), `send_human` 요청은 `exec = false` · outbox
  종류 `note`(닿으면 `done`), `post_with` 요청은 `to_agent = false`(보낸 대화는 `Message`, 받은 대화는 실행 없는 미러), `inbound_message` 요청은 집중
  판정 앞에서 `inbound_ask`(기록만 · 알림 `Ask`), `inbound_post` 요청 + 실행 거부 · 알림 `Ask`, `set_req_state`(`mark_done` 대체 — 대화 차례 안에서
  확인 · 상태 · 저장 · 보내기 행 한 트랜잭션), `inbound_req_state`(`inbound_done` 대체 — 알림 `AskState`), outbox 종류 `REQ_STATE`(`DONE` 삭제).
- `crates/app/src/chat_api.rs` — `PATCH /api/messages/{id}` 의 `done` → `req_state` + `note`(`wait` · 메모만 400), 메시지 JSON `req_state` · `req_note`
  · `req_at`(`done` 삭제), `GET /api/messages` 의 `req`(`ask=request` 와 함께만), 내 활동 `kind=query` · `request` 400. `notify.rs` — `Ask` · `AskState` →
  `notify_request`, 제목 "요청 받음 — …" · "요청 상태 — …".
- 테스트 — `agent/tests/requests.rs`(새, 6) · `core/tests/requests.rs`(+3 → 4) · `app/tests/models_tools.rs`(+2 → 4). 바뀐 동작에 맞춘 기존 시험:
  `roundtrip` `ask_kind_and_done_sync_both_sides` 삭제(요청 실행 · 완료 표시 — `requests.rs` 6건이 새 규칙으로 덮음) · 미페어링 목록 `Done` →
  `ReqState`, `ux_api` `ask_kind_recorded_and_filtered`(요청은 답 없음 · `req` 거르기 · 활동 400) · `done_only_by_receiver` →
  `req_state_only_by_receiver`, `net/tests/remote_input.rs`(`ReqState` 형식 검사), `core/tests/paths.rs`(`req_state`).

## 설계 결정

- **plan 개정 2** — 상태 보내기 행은 키로 합치지 않는다(바뀔 때마다 한 행). 요청자가 꺼져 있는 동안 '수용(메모)' 을 건너뛰지 않게.
- **보낸 대화의 요청은 `Message` 그대로** — 받는 쪽에 대화가 없을 수 있다(새 대화의 첫 글). 받는 쪽이 `request` 를 보고 실행 대신 `inbound_ask`.
- **요청은 집중 판정 앞에서 가른다** — 집중은 "새 실행 요청" 거부(D17 개정)라 실행이 없는 요청은 해당하지 않는다.
- **요청자 쪽은 옮김 검사를 하지 않는다** — 받은 쪽이 규칙을 지키고 outbox 가 대화 순서를 지킨다. 형식(상태 넷 · 메모 규칙)만 본다.
- **보낸 요청 글의 `exec = false`** — 답을 기다리지 않는다(화면의 "응답 대기" · CLI 감시 대상 아님). 닿으면 `done`, 거절이면 `error`(기존 `note` 처리).
- 인라인 heredoc 파이썬으로 한글 패턴을 고칠 때 표준 입력 인코딩 때문에 일치하지 않는 일이 있어 스크립트 파일 · Edit 도구로만 고쳤다.

## 테스트 결과

```text
$ cargo test --workspace            → 314 passed, 0 failed (합)
$ cargo clippy --workspace --all-targets -- -D warnings → 경고 0
$ cargo test -p an-core --test requests            → test result: ok. 4 passed; 0 failed
$ cargo test -p an-agent --test requests           → test result: ok. 6 passed; 0 failed
$ cargo test -p coworkagent-app --test models_tools → test result: ok. 4 passed; 0 failed
```

## 다음 단계

Stage 3 — 웹UI: `#requests` 화면 · 사이드바 '요청' 배지 · 대화의 요청 태그 · 상태 버튼(메모) · 입력창 안내 · 내 활동 종류 정리 · node 시험 5건 이상 ·
매뉴얼 · 실화면(E8).
