# eval.md — 1o.chat-list-split 적대적 평가

## 평가 원장

<!-- tools/check_gates.py 산출 — 손으로 고치지 않는다. 단 CHECK 가 없는 수동 gate 의
     EVIDENCE 와 ABANDON 줄은 사람이 적고, 재실행해도 보존된다.
     WEAK 줄은 기계가 매 실행 다시 단다 — 약한 gate 를 셀 수 있게 하는 표시이며
     판정에는 영향이 없다. -->

측정 커밋: 9577d25a56555de6ebcd52079e546e7475214253

- [x] E1: A4·A5 — 워크스페이스 테스트 전부 통과(문구 · 대비 · 기존 시험 회귀 없음)
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

- [x] E2: A5 — 우리 코드 경고 0(clippy -D warnings)
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.23s

- [x] E3: A1·A2·A4·A5 — 화면 로직 node 테스트 108건 이상 · 실패 0(기존 104 + `conv_split` 4 — 검색 · 별 · 칩 · 끌기 기존 시험 포함)
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# pass (10[8-9]|1[1-9][0-9])\s+# fail 0/
  EVIDENCE: # skipped 0 / # todo 0 / # duration_ms 126.2546

- [x] E4: A1·A2·A4 — 나눔 시험만 따로 4건 이상(내 대화 · agent 대화 · 자동 왕복 = 내 대화 · 빈 묶음 머리 숨김 · 상대 타일 / 그룹 / '나' 같은 규칙 · 다른 묶음 놓기 불가 · 즐겨찾기 그대로)
  CHECK: node --test --test-isolation=none --test-reporter=tap --test-name-pattern="^conv_split" crates/app/web/view.test.mjs
  EXPECT: /# pass [4-9]\s+# fail 0/
  EVIDENCE: # skipped 0 / # todo 0 / # duration_ms 16.4565

- [x] E5: A1·A3·A4 — 문구 · 규칙 자리(머리 문구 · 접힘 열쇠 · 즐겨찾기 그대로 · 말풍선 층 0.14 / 0.07 · 오른쪽 띠, 없어진 것 0 — 강조색 꽉 채운 내 글 · 층 0.12 · color-mix)
  CHECK: cargo test -p coworkagent-app --test ux_literals chat_list_split
  EXPECT: test result: ok. 1 passed; 0 failed
  EVIDENCE: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 23 filtered out; finished in 0.00s

- [x] E6: A3 — 말풍선 글자 대비 36 경우(2 테마 × 내 글 · 상대 글 5색 × 본문 · muted · 링크) 모두 4.5:1 이상
  CHECK: cargo test -p coworkagent-app --test ux_literals bubble_contrast
  EXPECT: test result: ok. 1 passed; 0 failed
  EVIDENCE: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 23 filtered out; finished in 0.00s

- [x] E7: A5 — 문서 규약
  CHECK: python tools/check_ddd_docs.py
  EXPECT: PASS
  EVIDENCE: PASS — DDD 문서 규약 위반 없음

- [x] E8: A5 — README 사용자 관점
  CHECK: python tools/check_readme.py
  EXPECT: readme ok
  EVIDENCE: readme ok

- [x] E9: A5 — 의존 경계
  CHECK: python tools/check_crates.py
  EXPECT: /crates=4 edges=[0-9]+ violations=0/
  EVIDENCE: crates=4 edges=6 violations=0

- [ ] E10: A1·A2·A3·A4 — 실화면 확인(실제 상대 대화, 다크 · 라이트): 상대 타일 · 그룹 · '나'(상대별 절 안) 목록에 "내 대화 n" · "agent 대화 n" 머리, 한쪽만이면 머리 없음 ·
  EVIDENCE: pending

- [x] E11: A7·A9·A10 — 세 앱 통합 시험(이력 · 새 글 · 답 · 편집 · 삭제 따라감 · 꺼진 관람자 보관 · 빼기 · 나가기 뒤 안 감 · 사본에 실행 없음 · 위조 Share 거절 · 구버전 거절)
  CHECK: cargo test -p an-agent --test share
  EXPECT: /test result: ok\. ([6-9]|[1-9][0-9]) passed; 0 failed/
  EVIDENCE: test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 11.91s

- [x] E12: A8 — 원 상대 알림(관람자 수 · 이름 없음)
  CHECK: cargo test -p an-agent --test share notice
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 8 filtered out; finished in 0.51s

- [x] E13: A7·A9·A10 — API(관람자 추가 · 빼기 · 나가기 · 사본 거절 · B 본인 거절 · 구버전 409 · 대화 JSON 필드)
  CHECK: cargo test -p coworkagent-app --test chat_api share
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 7 filtered out; finished in 0.62s

- [x] E14: A7·A10 — 화면 로직(공유 대화 묶음 · 읽기 전용 · 발화자 · 자동 왕복 표시 없음) node 3건 이상
  CHECK: node --test --test-isolation=none --test-reporter=tap --test-name-pattern="^share_" crates/app/web/view.test.mjs
  EXPECT: /# pass [3-9]\s+# fail 0/
  EVIDENCE: # skipped 0 / # todo 0 / # duration_ms 17.5775

- [x] E15: A10 — 스키마 20 올리기(19 → 20 · 기존 대화 dir 그대로)
  CHECK: cargo test -p an-core --test store share
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 13 filtered out; finished in 0.11s

- [ ] E16: A6·A7·A8·A9 — 실화면 확인(실제 세 PC): 레일 빈칸 · 타일 우클릭에 브라우저 메뉴 없음 · 관람자 추가 → 관람자 PC "공유 대화" 에 이력 · 새 글 따라옴 · 입력창 없음 ·
  EVIDENCE: pending

ABANDON: E10 실제 상대 · 두 PC 실화면 확인 — 수동 테스트는 프로토타입 인수 Task 에서 일괄(masterPlan D28, plan 해석 8 · 종결 시 할 일). 19 TC 로 넘김 · 구현 중 mock 확인은 work/stage4.md(8/8)
ABANDON: E16 실제 세 PC 실화면 확인(레일 우클릭 브라우저 메뉴 · 관람자 이력 · 원 상대 알림 · 빼기 · 나가기) — masterPlan D28, plan 종결 시 할 일. 19 TC 로 넘김 · 자동 시험은 E11 ~ E14, mock 8/8

## 적대적 평가 — 라운드 1/2

- 2026-10-17 · `/codex:adversarial-review --base 8c715bd --scope branch` + focus(intent A1 ~ A10 ↔ plan 해석 1 ~ 23 · E1 ~ E16, 공유 위조 · 사본 끼어들기 · 커서 ·
  꺼진 관람자 · 상한 · 빼기 뒤 · 이름 노출 · 구버전 · 스키마, 목록 묶음 · 대비) · 세션 codex-review(워크트리에서 실행) · HEAD `d9ee597`(요청과 일치) · job
  `review-mvbu3yn3-q0cpvp` — Verdict needs-attention(high 3). base `8c715bd` = plan 승인(구현 직전).

원문:

```text
- [high] 재사용된 message id의 새 글이 관람자에게 영구 누락된다 (crates/agent/src/chat.rs:3944-3956)
  `sent_upto`를 재사용 가능한 `messages.id`와 비교하고 `sent_upto >= top`이면 전송을 생략한다. `messages.id`는 AUTOINCREMENT가 아니므로 마지막 글 삭제 후
  같은 id가 재사용될 수 있다. 새 UUID의 글이어도 cursor와 id가 같아 `SharePost`가 생성되지 않는다. 실제 테스트도 id 재사용을 전제로 하지만 이 분기는 새
  글을 건너뛴다.
  Recommendation: 공유 전용 단조 증가 sequence를 저장하거나 `messages.id` 재사용을 막는 migration을 적용한다. 최대 id 삭제·재사용 뒤 관람자 사본에 새
  UUID와 본문이 실제 도착하는 회귀 테스트를 추가한다.
- [high] 관람자 해제와 inflight 전송이 겹치면 해제 뒤에도 글이 전달된다 (crates/core/src/share.rs:40-49)
  해제 시 `state = 'pending'`인 공유 행만 삭제한다. `deliver()`가 이미 행을 `inflight`로 바꾼 순간 해제하면 해당 행은 남는다. 전송 실패 뒤 `backoff()`가
  이를 다시 `pending`으로 되살리며, 나중에 추가된 `ShareEnd`보다 먼저 재전송한다. 따라서 관람자가 돌아왔을 때 해제된 대화 내용이 먼저 도착하고 그 뒤 해제
  표시가 적용될 수 있다.
  Recommendation: 공유 세대 또는 revocation tombstone을 두고 전송·재시도 직전에 현재 관람자 세대를 검증한다. 해제와 지연된 inflight 실패를 교차시켜
  `ShareEnd` 이후 어떤 `SharePost`도 적용되지 않는 통합 테스트를 추가한다.
- [high] 꺼진 관람자의 공유 payload가 무제한 누적되어 DB와 상대별 전송 줄을 고갈시킨다 (crates/agent/src/chat.rs:827-839)
  각 공유 글을 전체 payload를 가진 outbox 행으로 저장하면서 `SHARE`와 `SHARE_NOTICE`는 give-up 대상에서 영구 제외한다. 삭제·편집까지 별도 행으로 쌓이고
  용량·기간 상한이 없다. 관람자 최대 5명이 장기간 꺼지면 DB가 대화량의 최대 5배로 계속 증가한다. 전송은 peer별 단일 FIFO라 복귀 후 일반 대화도 누적 공유
  행 뒤에서 대기한다.
  Recommendation: 본문 복제 outbox 대신 관람자별 durable cursor만 유지하고 복귀 시 원본 DB에서 재생한다. 최소한 byte/row quota, compact/coalesce, 일반
  메시지와 공유 catch-up 간 공정 스케줄링 및 초과 상태의 사용자 노출을 추가한다.
```

판단 · 조치:

- **[high] 재사용 id 누락** — 판단: **오탐**. 글 삭제의 유일한 길 `core::chat::delete_message`(chat.rs 463 — `DELETE FROM messages` 는 여기뿐)가 지운 뒤
  `UPDATE shares SET sent_upto = max(messages.id) WHERE sent_upto > max(messages.id)` 로 커서를 낮춘다(471 ~ 472). 그래서 다시 받은 id 는 늘 커서보다 크다.
  회귀 시험이 이미 있다 — `history_and_followups_reach_viewer`(share.rs 161 ~ 174: 끝 글 지움 → 새 글이 **같은 id** 를 받음을 단정 → 사본에 "다시 쓴 글"
  도착). 고치지 않음.
- **[high] 빼기 ↔ 보내던 중 글** — 판단: **사실**. **고침** — `deliver()` 가 공유 행을 보내기 직전에 그 상대가 아직 그 대화의 관람자인지 보고(`watching`),
  아니면 행을 지운다(`ShareEnd` 는 예외 — 빼기 표시는 보낸다). 다시 보낼 차례(되살아난 행)도 같은 길을 지난다. 시험 `removed_mid_flight_not_delivered`(공유 행을
  `inflight` 로 만든 뒤 빼고 `pending` 으로 되살림 → 사본에 그 글 없음 · 공유 행 0 · 사본 빠짐). 확인 ~ 보내기 사이(ms) 창은 ponytail 주석.
- **[high] 꺼진 관람자 몫 무제한** — 판단: **사실**. **고침(plan 개정 — 사용자 미승인)** — 공유 행의 "포기 없음" 을 없앴다. 관람자에 24시간(`give_up`)
  닿지 않으면 그 관람자의 공유 행을 지우고 관람자에서 뺀다(`unshare` — 원 상대에 관람자 수), 원 상대 알림 행은 버린다. 그 뒤 글은 줄에 쌓이지 않는다. 시험
  `unreachable_viewer_dropped_after_give_up`(포기 2초). 커서 재생(원본 DB 에서 다시 만들기) · 공정 스케줄링은 하지 않았다 — 미조치 표.

## 적대적 평가 — 라운드 2/2

- 2026-10-17 · `/codex:adversarial-review --base 8c715bd --scope branch` + 라운드 1 focus(+ 라운드 1 고침 · 오탐 반박 확인) · 세션 codex-review(라운드 1 과 같음,
  워크트리) · HEAD `263d9ac`(요청과 일치) · job `review-mvbuqtn0-j4jufe` — Verdict needs-attention(high 2). 라운드 1 "재사용 id 누락" 반박은 타당하다고 확인됨.

원문:

```text
- [high] 관람자 해제 반환 뒤에도 SharePost가 전송될 수 있다 (crates/agent/src/chat.rs:963-992)
  `deliver()`는 `watching()`을 한 번 확인한 뒤 `knows_share()` 네트워크 왕복과 실제 `send_to()`를 수행한다. 이 사이 `share_remove()`가 관람자를 지우고 반환해도
  기존 작업은 계속 전송한다. 주석도 이 TOCTOU 창을 인정한다. 라운드 1 시험은 inflight 행을 다시 pending으로 만든 경우만 검사하므로 이 실제 교차 실행을 다루지
  않는다. 결과적으로 해제 뒤 새 대화 내용이 관람자에게 노출될 수 있다.
  Recommendation: 관람자별 직렬화 잠금으로 최종 권한 확인과 전송을 `share_remove()`와 선형화하거나, 공유 generation을 프레임에 넣어 수신자가 폐기된 generation의
  글을 거절하게 한다. `knows_share()` 중간에 해제를 완료시키는 barrier 기반 회귀 시험을 추가한다.
- [high] 24시간 포기가 관람자 사본을 활성 상태로 영구 방치한다 (crates/agent/src/chat.rs:841-879)
  give-up은 모든 공유 payload를 삭제한 뒤 `unshare(..., false)`를 호출하므로 C에게 `ShareEnd`를 남기지 않는다. 기존 사본의 `share_ended`는 계속 false지만 A는
  관람자 행을 삭제해 이후 글도 보내지 않는다. C 화면에는 활성 공유처럼 보이는 영구 정지 사본이 남는다. 또한 승인된 intent A9의 '꺼진 관람자에게 보관했다
  보낸다'를 사용자 미승인 plan 개정으로 뒤집은 구현이다.
  Recommendation: 승인된 A9대로 payload 대신 durable cursor를 보관해 복귀 시 원본에서 재생한다. 시간 제한을 채택하려면 먼저 계약 승인을 받고, payload를 비운 뒤에도
  coalesced `ShareEnd` tombstone 한 건은 ack까지 유지해 복귀한 C가 반드시 `공유 해제됨` 상태가 되게 한다.
```

판단 · 조치(상한 도달 — 고친 뒤 재리뷰하지 않는다):

- **[high] 확인 ~ 보내기 사이 해제** — 판단: **사실(남은 창)**. **일부 고침** — `watching` 확인을 능력 확인(Ping, 네트워크 왕복) 뒤 보내기 바로 앞으로 옮겨 창을 ms 로
  줄였다. 관람자별 잠금 · 공유 세대는 하지 않았다 — 미조치 표.
- **[high] 24시간 포기 뒤 사본 방치 · A9 충돌** — 판단: **사실**. **고침** — 포기 때 빼기 표시(`ShareEnd`) 한 행은 남기고(`unshare(…, true)`, 포기 대상에서 뺌)
  돌아온 관람자 사본이 "공유 해제됨" 이 된다. 시험 `unreachable_viewer_dropped_after_give_up` 에 돌아온 관람자 단정(사본 `ended` · 꺼진 동안 글 없음 · 남은 행 1)
  을 더했다. A9 충돌(무기한 보관 → 24시간)은 plan 개정 "사용자 미승인" 으로 남아 있다 — 사용자 결정.

재검증: `cargo test -p an-agent --test share` 9 passed · `cargo test --workspace` 실패 0 · clippy Finished · gate 원장(위, 측정 커밋 갱신).

## 미조치 지적

| 지적 원문(요약) | 판단 | 미조치 사유 |
|---|---|---|
| R1 [high] 꺼진 관람자 몫을 본문 복제 outbox 대신 커서로 두고 돌아오면 원본에서 다시 만들기 · 공유 · 보통 글 공정 스케줄링 | 사실(권고) | 24시간 포기 + 빼기 표시로 쌓임 상한을 두었다. 커서 재생 · 따로 줄은 줄 구조(상대별 한 줄)를 바꾸는 일 — 다음 Task 후보. 돌아온 관람자의 보통 글이 밀린 공유 글(최대 24시간치) 뒤에서 기다린다 |
| R2 [high] `watching` 확인 ~ 보내기 사이(ms)에 빼면 그 한 글이 간다 — 관람자별 잠금 또는 프레임 공유 세대 | 사실(남은 창) | 창을 Ping 뒤 보내기 바로 앞으로 줄였다. 잠금 · 세대는 프레임 · 받는 쪽 검사를 바꾸는 일 — 빼기 순간 이미 보내던 글 한 건이라 위험 수용 제안 |
| R2 [high] 24시간 포기는 승인된 A9("꺼진 관람자에게는 보관했다 보낸다")와 충돌 | 사실(계약) | plan 개정 "사용자 미승인" — 사용자가 24시간 한정을 승인하거나 무기한(+ 커서 재생)으로 되돌릴지 결정 |
