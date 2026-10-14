# plan — 1f.message-edit

> 근거: [intent](intent.md)(승인 2026-10-14 — 라운드 1). **구현자는 이 문서만 보고 일한다.** D·S 는 [masterPlan](../../masterPlan.md).
> 기준 커밋: `a7ef944`(`task/1f-message-edit` — 분기점 `43474bd` main). plan 승인: 2026-10-14 — 사용자 원문 "승인. 개발진행. 완료 후 readme ,
> 매뉴얼 문서 업데이트"(intent 승인과 함께 개발 진행 위임 — 아래 해석은 A1 ~ A9 안의 선택이라 따로 묻지 않고 최종 보고에 목록화).

## 목표

말풍선 우상단에 늘 보이는 복사 아이콘. 입력창 위 "편집" 단추로 편집 모드 — **내 사람 글** 중 끝난 글만 체크박스 · 연필, 여러 개 골라 삭제 ·
하나씩 내용 편집, 그때마다 "내 창에서만 · 상대 창에도". 편집 · 삭제는 실행을 다시 일으키지 않는 이력 정리다.

### 성공 기준(intent 사본)

- A1: 모든 말풍선 우상단에 늘 보이는 복사 아이콘이 있고, 누르면 본문 markdown 원문이 클립보드에 들어가며 "복사했다" 가 잠깐 보인다. 작업 막대의
  "복사" 글자 단추는 없다.
- A2: 입력창 위 "편집" 단추로 편집 모드를 켜고 끈다. 켜면 입력창 자리가 "n개 선택 · 삭제 · 완료" 막대가 되고(보내기 없음) 지금 열린 대화에만 적용된다.
- A3: 체크박스 · 연필은 내가 쓴 사람 글(웹UI · CLI) 중 끝난 글에만 보인다 — 에이전트 답 · 상대 글 · 보내는 중 · 실행 대기 · 실행 중 글에는 없고,
  API 로 직접 요청해도 거절한다.
- A4: 여러 글을 골라 "내 창에서만" 삭제하면 이 PC 에서 완전히 사라진다(대화 · 저장된 항목 · 검색 · 보낸 메시지 · 요청 목록 · 내 활동). 그 요청의 답은
  남고 상대 창은 그대로다.
- A5: "상대 창에도" 삭제 · 편집이면 상대 PC 에서도 같은 글이 지워지거나 같은 본문 + "(편집됨)" 이 된다(상대 요청 목록 포함). 상대가 꺼져 있으면
  outbox 로 다시 보내 켜질 때 반영하고, 24시간 안에 못 하면 그 대화에 "상대 반영 실패" 를 알린다.
- A6: 내용을 편집해 저장하면 본문이 바뀌고 "(편집됨)" 이 보인다. 순서(lc) · 요청 상태는 그대로다.
- A7: 편집 · 삭제는 양쪽 어디에서도 에이전트 실행 · 다시 보내기 · 자동 왕복 · 알림을 일으키지 않는다.
- A8: 상대에게서 받은 편집 · 삭제는 **그 상대가 쓴 사람 글**일 때만 적용한다 — 내 글 · 에이전트 답 · 다른 상대의 대화를 겨냥하면 거절한다.
- A9: 루트 `PROJECT.md` 검증 명령이 통과한다.

## 코드 사실(실측)

- 글 표 `messages` — 열 목록 `MSG_COLS` · 위치 읽기 `msg_row`, 뒤에 붙는 열은 `MSG_N`(=23) 기준(`activity`). 스키마 `user_version = 14`(`db.rs`).
- 내 글 상태: 실행 요청 = 보냄 `sent` → 답이 오면 `done`(받은 대화에서 내 에이전트에 쓴 글은 `queued` → `running` → `done`). 실행 없는 글 · 요청
  (#1d) = `sent` → 상대가 받으면 `done`. 보내지 못함 = `error` + `refused = unreachable`. 상대에 닿기 전 글은 outbox `pending` · `inflight` 행이 있다
  (`outbox::for_message` — 웹UI "보내는 중").
- 글을 가리키는 것: 답의 `reply_to`(로컬 id) · `cli_tx.message_id`(NOT NULL) · `relay_events.message_id`(`failed` 행) · `outbox.msg_id`.
- 상대 반영 틀 = #1d `ReqState`: 상태 + `stage()` 를 한 트랜잭션, 커밋 뒤 `dispatch()`(대화 차례 `turn()` 안), ack 뒤 `settle()` · 포기
  `give_up()`. 받는 쪽은 `chat::find(conv, peer.id)` 로 그 상대 소유 대화만.
- 피어 프로토콜은 D32(배포 전 하위 호환 없음) — 새 프레임에 이전 버전 대비 장치를 두지 않는다.
- 웹UI 대화는 5초 새로 고침마다 서명(`messagesSigOf`)이 바뀌면 `replaceChildren` 으로 다시 그린다. 작업 막대 `.msg-actions` 는 말풍선 위
  (`top:-14px; right:8px`)에 마우스를 올릴 때만.
- "이어서 작업" 실행 맥락은 Claude 세션(`--resume`) + 그 사이 실행 없는 사람 글(`notes_between`) — 편집 · 삭제한 글은 **다음** 실행 맥락에 바뀐 채
  들어간다(이미 돈 실행은 그대로).

## intent 대비 해석 (plan 승인 대상)

1. **편집 · 삭제 가능(`tidy`)** = 사람 글 ∧ `mine` ∧ 상태 `done` · `error` · `undelivered` ∧ 그 글을 실어 나르는 outbox 행(`pending` · `inflight`)
   없음. 한 곳(`Chat::tidy_ok`)에서 정하고 글 JSON 에 `editable` 로 싣는다 — 웹UI 는 그 값만 본다.
2. **편집 표시** = 새 열 `messages.edited_at`(스키마 15). JSON `edited_at`. 편집은 `text` · `edited_at` 만 바꾼다(uuid · lc · 요청 상태 · 저장 그대로).
   본문 규칙은 보내기와 같다(공백만 · `MAX_TEXT` 초과 거절).
3. **완전 삭제** = 그 글 행 + 그 글의 `cli_tx` · `relay_events` · 실패 보관 outbox 행. 답은 남는다(`reply_to` 는 없는 id 를 가리킨다 — 화면은 답만).
   빈 대화는 남는다.
4. **새 프레임** `Request::Edit { conv, msg, text }` · `Request::Delete { conv, msg }` — 글마다 하나(outbox 행 = 글 하나, 대화 순서 그대로). outbox
   종류 `msg_edit` · `msg_delete`(`outbox::for_message` 는 이 둘을 빼고 본다 — 반영 대기가 글을 "보내는 중" 으로 바꾸지 않게).
5. **받는 쪽(A8)** — 대화 = `(conv, 보낸 상대)`, 글 = 그 대화의 사람 글 ∧ `!mine`. 아니면 `unknown`. 그 글이 실행 대기 · 실행 중이면 `busy`. 본문
   규칙 위반은 `bad_request`. 같은 본문 재전송 · 이미 없는 글 삭제는 `Ok`(다시 보내기 멱등). 알림 · 실행 없음(A7).
6. **상대 반영 실패** — 상대 거절(ack 가 `Ok` 아님) · 24시간 포기 모두 `relay_events` 새 종류 `sync_failed`(사유 "편집 — {미리보기}" ·
   "삭제") → 내 활동 항목 "상대 반영 실패 — {사유}"(전체 · 실패 거르기, 안 본 수에 듦). 바로 보낸 결과가 거절이면 응답 `warning` 도.
7. **API** — `PATCH /api/messages/{id}` 에 `text` + `mirror`(다른 항목과 함께 못 씀) · `POST /api/messages/delete {ids, mirror}`(한 대화의 글만,
   하나라도 `tidy` 아니면 400 · 아무것도 지우지 않음).
8. **웹UI** — 복사 아이콘은 머리줄(`.msg-meta`) 오른쪽 끝, 작업 막대는 그 왼쪽으로 비켜 선다. 입력창 위 줄 "편집" 단추 · 편집 중 입력창 대신
   `#edit_bar`("n개 선택" · 삭제 · 완료). 체크박스는 말풍선 왼쪽, 연필은 복사 아이콘 옆(편집 가능한 글만). 연필 → 그 말풍선 본문 자리에 입력칸 ·
   저장 · 취소(편집 중엔 5초 새로 고침이 그 대화를 다시 그리지 않는다). 확인 창 = `<dialog>` 하나("내 창에서만" · "상대 창에도" · 취소). 대화를
   바꾸면 편집 모드가 꺼진다.

## 설계

### `an-core`
- `db.rs` — v15: `messages.edited_at INTEGER`.
- `chat.rs` — `Message.edited_at` · `MSG_COLS` 끝에 열 · `MSG_N` 24 · `edit_text(conn, id, text)` · `delete_message(conn, id)` ·
  `RelayMark::SyncFailed`(`sync_failed`) · `ActivityKind::SyncFailed` · `activity` · `activity_unseen` 반영.
- `outbox.rs` — `EDIT` · `DELETE` 종류 상수 · `for_message` 가 둘을 뺀다.

### `an-net`
- `proto.rs` — `Request::Edit` · `Request::Delete`.

### `an-agent`
- `chat.rs` — `tidy_ok(&Message) -> bool` · `edit_message(id, text, mirror) -> (Message, Option<경고>)` · `delete_messages(ids, mirror) -> (n,
  Option<경고>)` · `inbound_edit` · `inbound_delete` · `settle` · `give_up` 의 두 종류 → `sync_failed`.

### `an-app`
- `chat_api.rs` — `MessagePatch.text` · `mirror` · `POST /api/messages/delete` · 글 JSON `editable` · `edited_at` · 내 활동 `sync_failed`.
- 웹UI — `index.html`(편집 단추 · `#edit_bar` · `<dialog id="scope_dialog">`) · `app.css` · `view.js`(`messageActions` 에서 복사 빼기 ·
  `keepPicked` · 내 활동 라벨 · 서명에 `edited_at` · `editable`) · `app.js`(아이콘 · 편집 모드 · 편집 · 삭제).

### 문서
- README(무엇을 할 수 있나), 매뉴얼: user-guide(대화 — 복사 · 편집 모드) · developer-guide(피어 프레임 표 · API) · an_core · an_net · an_agent ·
  an_app. PRODUCT 도메인 규칙, masterPlan(비목표 개정 · D34 · S3 · S4 보강 · 개정 기록).

## Stage

| stage | 목표 | 산출 | gate |
|---|---|---|---|
| 1 | 저장 · 프로토콜 · 편집 · 삭제 · 상대 반영 · API | `message_tidy` 두 PC 시험 · API 시험 | E1 · E2 · E4 · E5 · E6 |
| 2 | 웹UI — 복사 아이콘 · 편집 모드 | node 시험 갱신 · 문구 시험 | E1 ~ E3 · E7 |
| 3 | README · 매뉴얼 · masterPlan | 문서 체커 | E1 ~ E3 · E8 |

## 평가 항목

- [ ] E1: A9 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E2: A9 — 우리 코드 경고 0(clippy -D warnings)
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: pending
- [ ] E3: A1·A2·A9 — 화면 로직 node 테스트 50건 이상 · 실패 0(작업 막대에 복사 없음 · 고른 글 거르기 · 내 활동 '상대 반영 실패')
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# pass ([5-9][0-9])\s+# fail 0/
  EVIDENCE: pending
- [ ] E4: A3·A4·A5·A6·A7 — 두 PC 편집 · 삭제 10건 + E5 위조 1건 = 파일 11건(편집 가능 판정 — 답 · 상대 글 · 보내는 중 · 실행 중 거절 / 내 창에서만 삭제 — 저장 · 검색 · 요청 목록 · 내 활동에서 빠지고 답은 남고 상대 그대로 / 상대 창에도 삭제 — 상대 요청 목록에서도 빠짐 / 편집 — 양쪽 본문 · 편집 시각, lc · 요청 상태 그대로 / 편집 · 삭제가 실행 · 알림을 만들지 않음 / 상대가 꺼져 있으면 돌아올 때 반영 / 포기 · 거절이면 내 활동 '상대 반영 실패' / 거절 정산 원자성 — 기록 쓰기 실패를 주입하면 행이 남고 다시 보내 기록과 함께 지움(개정 1) / 편집 실패 기록은 그 글을 지우면 함께 사라짐 · 삭제 실패 기록은 남음(개정 2) / 맨 끝 글을 지운 뒤 같은 id 로 온 글이 안 읽음 · 안 본 것(개정 3))
  CHECK: cargo test -p an-agent --test message_tidy
  EXPECT: test result: ok. 11 passed; 0 failed
  EVIDENCE: pending
- [ ] E5: A8 — 위조 거절 1건(내 글 · 답 · 다른 상대 대화 · 없는 대화를 겨냥한 편집 · 삭제는 `unknown`, 실행 중 글은 `busy`, 빈 본문은 `bad_request` — 기록 그대로)
  CHECK: cargo test -p an-agent --test message_tidy forged
  EXPECT: test result: ok. 1 passed; 0 failed
  EVIDENCE: pending
- [ ] E6: A3·A4·A6 — API 4건(`PATCH text` · `mirror` 다른 항목과 함께면 400 · 편집 불가 글 400 / `POST /api/messages/delete` 하나라도 불가면 아무것도 안 지움 · 다른 대화 섞이면 400 / 글 JSON `editable` · `edited_at` / 내 활동 `sync_failed` 항목)
  CHECK: cargo test -p coworkagent-app --test message_tidy_api
  EXPECT: test result: ok. 4 passed; 0 failed
  EVIDENCE: pending
- [ ] E7: A1·A2 — 웹UI 문구 · 자리(`index.html` 에 편집 단추 · `#edit_bar` · `scope_dialog` 세 선택지, `app.js` 복사 아이콘 · "복사했다")
  CHECK: cargo test -p coworkagent-app --test ux_literals
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E8: A1·A2·A5 — 두 PC 실측(사람 확인): 복사 아이콘 · 편집 모드 · 다중 삭제 · 편집 · 상대 창 반영 · 상대가 꺼졌다 켜질 때 반영
  EVIDENCE: pending

## 종결 시 할 일

- masterPlan: 비목표에서 "메시지 수정/삭제 동기화" 빼기 · D34 추가 · S3(이력 정리) · S4(복사 아이콘 · 편집 모드) 보강 · D32 적용 기록(새 프레임
  `Edit` · `Delete` — 이전 버전 대비 없음) · 개정 기록.
- PRODUCT 도메인 규칙(이력 정리), README, 매뉴얼.
- E8 은 두 PC 가 필요 — 19 acceptance TC 로 넘긴다(`ABANDON`, D28).
- GitHub 반영(push)은 사용자 확인 뒤.

## 개정 기록

- 개정 1(2026-10-14, 적대적 평가 라운드 1/2 — medium 1 사실): 해석 6 — 편집 · 삭제 반영 행의 응답 정산(`Chat::acked`)은 거절이면 '상대 반영 실패'
  기록과 행 지우기를 **한 트랜잭션**으로, 그 트랜잭션이 실패하면 행을 줄로 되돌려 다시 보낸다. 24시간 포기도 실패 보관과 기록을 한 트랜잭션으로.
  E4 8 → 9건(쓰기 실패 주입 시험).
- 개정 2(2026-10-14, 적대적 평가 라운드 2/2 — medium 1 사실): 해석 6 — 편집 반영 실패 기록(`sync_failed`, 사유에 본문 미리보기)은 그 글 id 를
  `relay_events.message_id` 에 잇는다(`RelayMark::SyncFailed(Option<글 id>)`) — 나중에 그 글을 지우면 `delete_message` 가 기록도 지워 본문이 남지
  않는다(A4 완전 삭제). 삭제 반영 실패 기록은 본문이 없고 글도 이미 없어 남는다. E4 9 → 10건.
- 개정 3(2026-10-14, 종결 전 자체 검토 — 재리뷰 아님): 해석 3 — 완전 삭제가 맨 끝 글이면 SQLite 가 그 id 를 다음 글에 다시 준다(`INTEGER PRIMARY KEY`,
  AUTOINCREMENT 없음). 읽음(`conversations.read_id`) · 내 활동 본 표시(`peers.activity_seen_msg` · `activity_seen_event`)는 `id >` 비교라 새 글이 이미
  읽은 것으로 보인다 → `delete_message` 가 같은 트랜잭션에서 세 표시를 남은 최대 id 로 낮춘다. 응답 정산(`acked`)이 실패하면 어느 종류의 행이든 줄로
  되돌려 다시 보낸다(주석을 동작에 맞춤 — 프레임은 uuid 로 멱등). E4 10 → 11건.
