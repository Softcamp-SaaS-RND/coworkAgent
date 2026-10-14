# eval.md — 1f.message-edit 적대적 평가

## 평가 원장

<!-- tools/check_gates.py 산출 — 손으로 고치지 않는다. 단 CHECK 가 없는 수동 gate 의
     EVIDENCE 와 ABANDON 줄은 사람이 적고, 재실행해도 보존된다.
     WEAK 줄은 기계가 매 실행 다시 단다 — 약한 gate 를 셀 수 있게 하는 표시이며
     판정에는 영향이 없다. -->

측정 커밋: 521ddf2567c5d049aebb528988e6207ab599a16b

- [x] E1: A9 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

- [x] E2: A9 — 우리 코드 경고 0(clippy -D warnings)
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.22s

- [x] E3: A1·A2·A9 — 화면 로직 node 테스트 50건 이상 · 실패 0(작업 막대에 복사 없음 · 고른 글 거르기 · 내 활동 '상대 반영 실패')
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# pass ([5-9][0-9])\s+# fail 0/
  EVIDENCE: # skipped 0 / # todo 0 / # duration_ms 88.0974

- [x] E4: A3·A4·A5·A6·A7 — 두 PC 편집 · 삭제 10건 + E5 위조 1건 = 파일 11건(편집 가능 판정 — 답 · 상대 글 · 보내는 중 · 실행 중 거절 / 내 창에서만 삭제 — 저장 · 검색 · 요청 목록 · 내 활동에서 빠지고 답은 남고 상대 그대로 / 상대 창에도 삭제 — 상대 요청 목록에서도 빠짐 / 편집 — 양쪽 본문 · 편집 시각, lc · 요청 상태 그대로 / 편집 · 삭제가 실행 · 알림을 만들지 않음 / 상대가 꺼져 있으면 돌아올 때 반영 / 포기 · 거절이면 내 활동 '상대 반영 실패' / 거절 정산 원자성 — 기록 쓰기 실패를 주입하면 행이 남고 다시 보내 기록과 함께 지움(개정 1) / 편집 실패 기록은 그 글을 지우면 함께 사라짐 · 삭제 실패 기록은 남음(개정 2) / 맨 끝 글을 지운 뒤 같은 id 로 온 글이 안 읽음 · 안 본 것(개정 3))
  CHECK: cargo test -p an-agent --test message_tidy
  EXPECT: test result: ok. 11 passed; 0 failed
  EVIDENCE: test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 8.61s

- [x] E5: A8 — 위조 거절 1건(내 글 · 답 · 다른 상대 대화 · 없는 대화를 겨냥한 편집 · 삭제는 `unknown`, 실행 중 글은 `busy`, 빈 본문은 `bad_request` — 기록 그대로)
  CHECK: cargo test -p an-agent --test message_tidy forged
  EXPECT: test result: ok. 1 passed; 0 failed
  EVIDENCE: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 10 filtered out; finished in 0.37s

- [x] E6: A3·A4·A6 — API 4건(`PATCH text` · `mirror` 다른 항목과 함께면 400 · 편집 불가 글 400 / `POST /api/messages/delete` 하나라도 불가면 아무것도 안 지움 · 다른 대화 섞이면 400 / 글 JSON `editable` · `edited_at` / 내 활동 `sync_failed` 항목)
  CHECK: cargo test -p coworkagent-app --test message_tidy_api
  EXPECT: test result: ok. 4 passed; 0 failed
  EVIDENCE: test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.29s

- [x] E7: A1·A2 — 웹UI 문구 · 자리(`index.html` 에 편집 단추 · `#edit_bar` · `scope_dialog` 세 선택지, `app.js` 복사 아이콘 · "복사했다")
  CHECK: cargo test -p coworkagent-app --test ux_literals
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

- [ ] E8: A1·A2·A5 — 두 PC 실측(사람 확인): 복사 아이콘 · 편집 모드 · 다중 삭제 · 편집 · 상대 창 반영 · 상대가 꺼졌다 켜질 때 반영
  EVIDENCE: pending

ABANDON: E8 두 PC 사람 확인이 필요 — 이 PC 에는 사용자 coworkAgent 가 실행 중(PID 7364)이라 같은 데이터 폴더로 두 번째 인스턴스를 띄울 수 없고 두 번째 PC 도 없다. 기계 범위는 E4 ~ E7(두 PC 통합 시험 · API · 화면 로직 · 문구)이 덮는다. 19.prototype-acceptance TC 로 이관(D28)

## 적대적 평가 — 라운드 1/2

- 2026-10-14 · `/codex:adversarial-review --base 43474bd` · 세션 codex-review · 실행 HEAD `1ba79d0`(요청과 일치) · Verdict needs-attention(medium 1).

### 지적 원문

- [medium] 거절 응답을 실패 기록보다 먼저 삭제해 silent divergence가 발생할 수 있음 (crates/agent/src/chat.rs:830-855) — `deliver`는 응답 종류와
  무관하게 outbox 행을 먼저 삭제한 뒤 `settle`을 호출합니다. 새 edit/delete 분기는 그 후에야 `sync_failed` 이벤트를 별도 DB 작업으로 기록합니다.
  따라서 상대가 요청을 거절한 뒤 프로세스가 중단되거나 이벤트 INSERT가 실패하면, 재시도할 outbox 행도 실패 알림도 남지 않습니다. 특히 백그라운드
  재전송에서는 API 경고도 없어 사용자가 양쪽 이력이 일치한다고 오인합니다.

### 대조 · 처리

| 지적 | 판단 | 처리 |
|---|---|---|
| medium · 거절 행 지우기와 실패 기록이 따로 | 사실 — 바로 보내기(`dispatch`)와 줄(`deliver`) 모두 행을 지운 뒤 기록했다. 24시간 포기도 실패 보관과 기록이 따로였다 | `Chat::acked` — 응답 정산을 한 트랜잭션(거절된 편집 · 삭제 반영이면 `sync_failed` 기록 + 행 지우기), 실패하면 행을 줄로 되돌려 다시 보낸다(`backoff`). 두 경로 모두 이것을 쓴다. 포기도 `outbox::fail` + 기록을 한 트랜잭션. 시험 `refusal_note_and_row_removal_are_atomic`(`relay_events` 쓰기를 트리거로 막아 실패 주입 — 고치기 전 코드에서 FAILED "기록 못 했으면 행이 남는다" 확인) — plan 개정 1 · E4 9건 |

- 다른 종류의 행(기존 `req_state` 거절 등)도 "지운 뒤 정산" 순서다 — 이번 범위(#1f 반영 행) 밖이라 바꾸지 않았다. 기존 동작 그대로.

## 적대적 평가 — 라운드 2/2

- 2026-10-14 · `/codex:adversarial-review --base 43474bd` · 세션 codex-review(라운드 1 과 같은 세션) · 실행 HEAD `cb64379`(요청과 일치) · Verdict
  needs-attention(medium 1 — 라운드 1 지적 재지적 없음). 상한 도달 — 이 라운드 수정 뒤 재리뷰하지 않는다.

### 지적 원문

- [medium] 삭제된 메시지의 본문 미리보기가 `sync_failed` 활동에 잔존 (crates/agent/src/chat.rs:2189-2197) — 상대 반영 편집이 거절되거나
  포기되면 `note_sync_failed`가 `편집 — {preview}`를 `relay_events`에 기록하지만 원본 `message_id`를 저장하지 않습니다. 이후 사용자가 해당 글을
  로컬 삭제해도 `delete_message`는 `message_id`가 일치하는 이벤트만 삭제하므로 이 행은 제거되지 않습니다. `activity()`가 이를 계속 반환해 A4의 완전
  삭제 계약을 위반하며, 삭제한 본문 일부가 UI와 DB에 잔존합니다.

### 대조 · 처리

| 지적 | 판단 | 처리 |
|---|---|---|
| medium · 편집 실패 기록에 본문 미리보기가 남음 | 사실 — A4 "내 활동에서도 사라진다" 위반 | `RelayMark::SyncFailed(Option<글 id>)` — 편집 실패는 `item.msg_id` 를 `relay_events.message_id` 에 적어 `delete_message` 가 함께 지운다. 삭제 실패 기록("삭제", 본문 없음)은 글이 지워진 뒤 생기므로 남는다. 시험 `deleting_message_removes_its_edit_failure_note`(고치기 전 코드에서 FAILED 확인) — plan 개정 2 · E4 10건 |

## 종결 전 자체 검토 (라운드 밖 — 재리뷰 없음)

- 맨 끝 글을 지우면 SQLite 가 id 를 다시 줘 새 글이 읽음 · 본 것으로 보이던 결함 — `delete_message` 가 읽음 · 본 표시를 남은 최대 id 로 낮춤. 시험
  `reused_id_after_delete_stays_unread`(고치기 전 코드에서 FAILED "새 글 3 · 읽음 3 · 본 3" 확인) — plan 개정 3 · E4 11건.
- `Chat::acked` 주석이 동작과 달랐다(그 밖 행도 정산 실패면 다시 보냄) — 주석을 동작에 맞춤. 다시 보내기는 uuid 로 멱등.

## 미조치 지적

없음

## 이 PC 실측 (종결 뒤 · 2026-10-14)

사용자 지시 "빌드하고 현재 pc에 반영 / 이 pc에서 확인가능한 테스트 진행" — 실행 중이던 앱을 끄고(정상 종료 신호에 응답하지 않아 강제 종료) DB 를 같은 폴더
`coworkAgent.db.pre-1f.bak` 으로 보존한 뒤 main `88fed5d` 릴리스 빌드로 다시 띄웠다(스키마 13 → 15, 앱이 `.v13.bak` 를 남김). 실제 앱 화면을 headless
Edge(CDP)로 눌러 확인 — **삭제 · 저장 확인 창은 늘 '취소'**(사용자 이력 · 상대 PC 를 바꾸지 않음).

| 확인 | 결과 |
|---|---|
| A1 모든 말풍선에 복사 아이콘 · 마우스 없이 보임 · 우상단 · 작업 막대에 복사 없음 | PASS |
| A1 누르면 본문 markdown 원문이 클립보드 · "복사했다" | PASS |
| A2 "편집" 단추가 입력창 위 · 켜면 입력창 숨김 + 편집 막대 · "0개 선택" → 고르면 "1개 선택" · 완료 · 다른 대화로 가면 꺼짐 | PASS |
| A3 체크박스 · 연필 = API `editable` 글과 같음(보낸 대화의 내 글 2건 · 받은 대화 0건) | PASS |
| A4 · A5 삭제 → 확인 창 "내 창에서만" · "상대 창에도" · "취소", 취소면 글 수 그대로 | PASS |
| A6 연필 → 말풍선 안 입력칸(원문) · 5초 새로 고침을 넘겨도 입력 보존 · 취소면 그대로 | PASS |
| 화면 JS 오류 | 없음 |

- 22/22 PASS. 실제로 지우기 · 고치기(내 창 · 상대 창)와 상대가 꺼졌다 켜질 때 반영은 이 PC 이력과 연결된 상대 PC 를 바꾸므로 하지 않았다 — E8 은
  `ABANDON` 그대로(19 TC).
