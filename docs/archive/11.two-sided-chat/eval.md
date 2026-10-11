# eval.md — 11.two-sided-chat 적대적 평가

## 평가 원장

<!-- tools/check_gates.py 산출 — 손으로 고치지 않는다. 단 CHECK 가 없는 수동 gate 의
     EVIDENCE 와 ABANDON 줄은 사람이 적고, 재실행해도 보존된다.
     WEAK 줄은 기계가 매 실행 다시 단다 — 약한 gate 를 셀 수 있게 하는 표시이며
     판정에는 영향이 없다. -->

측정 커밋: f18d13d3ed0e4c3d22aba0a0a6d5b8e9c5fe44f6

- [x] E1: A6 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

- [x] E2: A6 — 우리 코드 경고 0(clippy -D warnings)
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.05s

- [x] E3: A5 — 스키마 6 이전(schema6_fills_mine_exec · 기존 이전 테스트 포함 paths 5건)
  CHECK: cargo test -p an-core --test paths
  EXPECT: test result: ok. 5 passed; 0 failed
  EVIDENCE: test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s

- [x] E4: A1·A2·A3·A4 — 양쪽 발화 왕복(two_sided 테스트 7건: human_note_both_ways · receiver_asks_own_agent · notes_prepended_to_run · post_refused_when_foreign · concurrent_notes_same_order_both_sides · own_run_not_failed_by_mirror · late_queued_request_not_skipped)
  CHECK: cargo test -p an-agent --test two_sided
  EXPECT: test result: ok. 7 passed; 0 failed
  EVIDENCE: test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.36s

- [x] E5: A6 — 기존 왕복 회귀 없음(roundtrip 11건)
  CHECK: cargo test -p an-agent --test roundtrip
  EXPECT: test result: ok. 11 passed; 0 failed
  EVIDENCE: test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.26s

- [x] E6: A1·A2 — 웹UI API(conversation_post_targets 포함 chat_api 4건)
  CHECK: cargo test -p agentnetwork-app --test chat_api
  EXPECT: test result: ok. 4 passed; 0 failed
  EVIDENCE: test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.28s

- [x] E7: A1·A2·A5 — 화면 로직 node 테스트(발화자 이름 · 실행 글만 타임라인)
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# fail 0/
  EVIDENCE: # fail 0

- [ ] E8: A1·A2 — 실화면(수동): 양쪽 입력창 · 대상 선택 · 양쪽 이력 일치 · 받은 쪽 실행
  EVIDENCE: pending

ABANDON: E8 실화면 확인은 원격 세션이라 불가 — 프로토타입(Phase 1 끝) 인수 Task 에서 수동 테스트 일괄(masterPlan D28, 사용자 지시 2026-10-11 "사용자 테스트는 완료 후 진행")

## 적대적 평가 — 라운드 1/2

- 수단: codex-review 세션 `/codex:adversarial-review --base 30f0ac9 --scope branch` · 대상 HEAD `6baf2cc` · 2026-10-11
- Verdict: needs-attention

### 원문

```text
- [high] 양쪽의 동시 발화가 서로 반대 순서로 저장된다 (crates/agent/src/chat.rs:270-295)
  각 노드는 자기 글을 로컬 DB에 먼저 저장한 뒤 상대에게 전송한다. 반대편 `inbound_post`도 별도 로컬 순서로 저장하며 공유 sequence가 없다. A와 B가 동시에 작성해 양쪽 로컬 INSERT가 먼저 끝나면 A 이력은 A→B, B 이력은 B→A가 된다. 조회도 로컬 `id` 순이므로 A1/A2의 동일 순서 계약을 위반하고, 실행 요청과 메모가 경합하면 에이전트에 포함되는 맥락까지 달라질 수 있다. 추가 테스트는 양방향을 순차 호출할 뿐 이 경합을 다루지 않는다.
  Recommendation: 대화별 authoritative sequence 또는 `(logical_clock, origin)` 같은 양쪽 공통 정렬 키를 프로토콜과 DB에 저장하고 그 키로 조회하라. barrier로 양쪽 발화를 동시에 시작해 양쪽 UUID 순서와 실행 입력이 동일함을 검증하는 회귀 테스트를 추가하라.
- [high] 전송 실패를 반환한 뒤에도 에이전트를 실행해 재시도가 중복 실행된다 (crates/agent/src/chat.rs:292-320)
  `to_agent` 요청은 미러 전송 결과와 무관하게 `Queued`로 바꾸고 `wake`한다. 이후 timeout이나 명시적 거절이면 함수는 오류를 반환하고 API는 502를 내보낸다. 따라서 호출자나 자동 retry는 실패로 판단하지만 첫 요청은 이미 실행 중이다. 재시도는 새 UUID로 두 번째 실행을 만들며, 도구 권한이 있는 에이전트라면 외부 변경도 중복 수행할 수 있다.
  Recommendation: 실행을 수락했다면 transport 결과와 분리해 202/성공 및 안정적인 operation ID를 반환하고, 같은 idempotency key 재시도는 기존 요청을 반환하라. 미러 성공이 실행 전제라면 ACK 후에만 `Queued` 전이와 `wake`를 수행하라. timeout·거절·응답 유실 후 retry에서 실행 횟수가 1임을 검증하라.
```

### 대조 · 조치

| 지적 | 판단 | 조치 |
|---|---|---|
| 동시 발화 양쪽 순서 분기 | 사실 | 램포트 시계 `lc`(쓴 쪽 = 대화 최대 + 1, 받은 쪽은 그대로) · 정렬 `(lc, uuid)` — 양쪽 같은 키라 같은 순서. 실행 맥락(`last_exec_before` · `notes_between`)도 같은 순서. 회귀 `concurrent_notes_same_order_both_sides`(5 라운드 동시 발화) — "id 순 정렬" mutation 3회 모두 FAIL |
| 미러 실패를 오류로 돌려 재시도 중복 실행 | 사실 | 실행을 받아들인 요청은 미러 실패여도 201 + `warning`("다시 보내지 마세요"), 실행 1회. 사람 글(실행 없음)은 지금처럼 실패 반환. 회귀 `own_run_not_failed_by_mirror`(실행 1회 단정). 같은 키 재시도(idempotency key)는 미조치 후보 |

## 적대적 평가 — 라운드 2/2 (상한 — 재평가 없음)

- 수단: codex-review 세션 `/codex:adversarial-review --base 30f0ac9 --scope branch` · 대상 HEAD `37a7794` · 2026-10-11
- Verdict: needs-attention

### 원문

```text
- [high] 늦게 `queued`된 실행 요청이 재시작 전까지 영구 누락된다 (crates/agent/src/chat.rs:300-304)
  … 이후 낮은 `id` 행을 `queued`로 바꿔도 `next_pending`의 `id > after` 조건에서 영구 제외되어 …
- [high] 원격 `lc` 최댓값이 대화 저장소를 영구 손상시킨다 (crates/core/src/chat.rs:278-297)
  … `lc = i64::MAX`를 받은 뒤 다음 메시지의 `max(lc) + 1`은 … REAL 값이 된다 …
```

### 대조 · 조치

| 지적 | 판단 | 조치 |
|---|---|---|
| 늦게 `queued` 된 낮은 id 누락 | 사실 | 커서(가장 큰 id) 제거 — 대기열을 매번 처음부터 보고(`chat::pending`) 작업이 꺼낸 것의 집합만 건너뛴다(상태 저장 실패 시 되풀이 방지는 유지). 회귀 `late_queued_request_not_skipped`(낮은 id `sent` → 높은 id 실행 → 낮은 id `queued` → 실행) — "가장 큰 id 뒤만" mutation FAIL |
| 원격 `lc` 최댓값으로 순서 손상 | 사실 | 원격 값은 1 ~ `MAX_LC`(2^40)만 받고 밖이면 무시(이 PC 가 매긴 값 유지) — `max + 1` 이 정수 범위 안. 회귀 store `remote_order_value_bounded`(`i64::MAX` · `MAX - 1` · 0 · 음수 무시, 상한 뒤 다음 값 `MAX_LC + 1`) — "양수면 받음" mutation FAIL |

## 미조치 지적

| 지적 | 판단 | 사유 |
|---|---|---|
| E8 수동 gate(실화면) 미충족 | 사실 | 프로토타입(Phase 1 끝) 인수 Task 로 이관(D28). ABANDON 기록 |
| 같은 키 재시도(idempotency key)(라운드 1) | 사실 — 실행 1회까지 | 실행을 받아들인 요청은 미러 실패여도 성공 + 경고로 재시도 원인을 없앴다. 응답 유실 후 재시도의 중복 판정 키는 원장 #04 R2-F1(같은 uuid 재전송)과 함께 Phase 2 |
