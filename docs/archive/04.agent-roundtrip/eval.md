# eval.md — 04.agent-roundtrip 적대적 평가

## 평가 원장

<!-- tools/check_gates.py 산출 — 손으로 고치지 않는다. 단 CHECK 가 없는 수동 gate 의
     EVIDENCE 와 ABANDON 줄은 사람이 적고, 재실행해도 보존된다.
     WEAK 줄은 기계가 매 실행 다시 단다 — 약한 gate 를 셀 수 있게 하는 표시이며
     판정에는 영향이 없다. -->

측정 커밋: 1314b6ec2efd4293a6d7d24f526ea96aec6dc6fa

- [x] E1: A7 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

- [x] E2: A7 — 우리 코드 경고 0(clippy -D warnings)
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.22s

- [x] E3: A1·A4 — 저장 3건(agent_defaults_read_only_tools · conversation_by_uuid_per_peer · messages_in_order_with_state)
  CHECK: cargo test -p an-core --test store
  EXPECT: test result: ok. 3 passed; 0 failed
  EVIDENCE: test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

- [x] E4: A3·A5 — 실행기 5건(args_follow_d22 · resume_uses_saved_session · parse_claude_json · timeout_kills_process · process_tree_is_killed)
  CHECK: cargo test -p an-agent --test runner
  EXPECT: test result: ok. 5 passed; 0 failed
  EVIDENCE: test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.88s

- [x] E5: A2·A4·A5·A6 — 왕복 7건(human_to_remote_agent_roundtrip · followup_resumes_session · unpaired_or_unknown_agent_refused · inbound_does_not_wait_for_run · forged_or_duplicate_reply_refused · restart_recovers_queue · storage_failure_sends_error_reply · unconfirmed_delivery_keeps_waiting_for_reply · concurrent_sends_keep_order)
  CHECK: cargo test -p an-agent --test roundtrip
  EXPECT: test result: ok. 9 passed; 0 failed
  EVIDENCE: test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.19s

- [x] E6: A7 — 의존 경계: 크레이트 4개 · 내부 간선 6개 · 위반 0
  CHECK: python tools/check_crates.py
  EXPECT: crates=4 edges=6 violations=0
  EVIDENCE: crates=4 edges=6 violations=0

- [x] E7: A1·A4·A6 — 웹UI API 2건(agents_crud_api · conversation_api_roundtrip)
  CHECK: cargo test -p agentnetwork-app --test chat_api
  EXPECT: test result: ok. 2 passed; 0 failed
  EVIDENCE: test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s

- [ ] E8: A3·A4 — 실동작(수동, PC 2대): 실제 `claude -p` 로 원격 에이전트 답 · 이어 쓰기 맥락 유지 · 읽기 전용 프로필에서 파일 수정 요청이 거부됨
  EVIDENCE: pending

ABANDON: E8 실제 `claude -p` · PC 2대 확인은 원격 세션 · 구독 사용량 문제로 자동 불가 — 프로토타입(Phase 1 끝) 인수 Task 에서 수동 테스트 일괄(masterPlan D28, 사용자 지시 2026-10-11)

## 적대적 평가 — 라운드 1/2

- 수단: codex-review 세션 `/codex:adversarial-review --base d5fadb4 --scope branch` · 대상 HEAD `e01bf0d` · 2026-10-11
- Verdict: needs-attention

### 원문

```text
- [high] ACK된 작업이 durable FIFO가 아니다 (crates/agent/src/chat.rs:266-283)
  `queued` 저장 후 DB 잠금을 놓고 별도 메모리 queue에 넣은 다음 `Ok`를 반환한다. 저장과 enqueue 사이 종료되거나 앱이 재시작되면 `Chat::new`는 빈 queue만 만들므로 해당 메시지는 영구히 `queued`로 남고 실행·답장이 없다. 동시 수신 시 DB 저장 순서와 enqueue 순서도 역전될 수 있어 같은 Claude session에 질문이 잘못된 순서로 들어간다.
  Recommendation: DB에 실행 순서와 처리 상태를 transaction으로 확정하고, 시작 시 미완료 작업을 복구하거나 명시적 오류로 종결하라. 대화별 dispatcher가 DB 순서대로 dequeue하도록 만들고 durable scheduling 이후에만 `Ok`를 반환하라.
- [high] 실행 후 DB 실패를 무시하고 성공 답장을 전송한다 (crates/agent/src/chat.rs:376-392)
  `set_session`, human message 상태 변경, agent message 저장 오류가 `let _`와 `.ok()`로 폐기된다. DB 잠금 초과·디스크 부족 같은 실패가 발생하면 다음 작업은 stale session으로 실행되고 로컬 이력은 빠질 수 있지만, 코드는 계속 상대에게 정상 reply를 전송한다. 양쪽 이력과 resume 상태가 조용히 불일치하며 복구 근거도 남지 않는다.
  Recommendation: session·human state·agent reply 저장을 하나의 transaction으로 처리하고 모든 오류를 전파·기록하라. durable commit 실패 시 성공 reply를 보내지 말고 retry 가능한 상태 또는 명시적 error reply로 전환하라.
- [high] 30분 timeout이 Claude의 자식 프로세스를 종료하지 않는다 (crates/agent/src/runner.rs:131-137)
  timeout 경로는 `child.kill()`로 직접 실행한 `claude` PID만 종료한다. Unix process group이나 Windows Job Object가 없어, 허용된 도구가 실행 중인 자식 프로세스는 timeout 오류가 반환된 뒤에도 파일 변경이나 명령 실행을 계속할 수 있다. 자식이 pipe를 보유하면 분리된 reader thread도 남는다. 자식이 deadline 시점에 존재한다는 조건부 추론이지만, 도구 실행이 제품 기능이라 현실적인 A5 위반이다.
  Recommendation: Unix에서는 별도 process group/session을 만들고 group 전체를 종료하며, Windows에서는 kill-on-close Job Object에 프로세스 tree를 묶어라. 손자 프로세스와 reader thread까지 종료되는 회귀 테스트를 추가하라.
- [medium] `reply_to`를 버려 unsolicited reply를 정상 이력으로 수용한다 (crates/agent/src/chat.rs:217-223)
  `Request::Reply.reply_to`를 `..`로 폐기하고, 이후에는 peer·conversation과 새 `msg` UUID만 검사한다. 따라서 신뢰하지 않는 paired peer가 어떤 outbound conversation에도 원 요청과 무관한 agent reply를 여러 개 삽입하고 알림을 발생시킬 수 있다. 이력의 요청-응답 대응과 중복 방지가 깨진다.
  Recommendation: `reply_to`가 같은 conversation의 아직 응답되지 않은 human message인지 검증하고, reply correlation을 DB에 저장하라. `(conversation_id, reply_to)` uniqueness와 원 메시지 상태 전환을 같은 transaction에 포함하라.
```

### 대조 · 조치

| 지적 | 판단 | 조치 |
|---|---|---|
| ACK 작업 비내구 · 순서 역전 | 사실 | 대기열 = DB. 저장(`queued`)이 곧 등록이고 그 뒤에만 `ok`. 대화 작업은 깨우기 신호만 받고 `next_pending`(id 순)으로 꺼낸다 — 저장 순서 = 실행 순서. 실행 전 `running` 표시, 시작 시 `Chat::recover`(`main`): `queued` 는 실행, `running`(실행 도중 종료)은 중복 실행하지 않고 오류 답. 회귀 `restart_recovers_queue`(E5) — "running 도 다시 실행" mutation FAIL. 동시 수신 순서 역전은 구조로 없앴고 별도 경쟁 테스트는 없음 |
| 기록 실패 은폐 후 성공 답 | 사실 | 세션 · 사람 메시지 상태 · 에이전트 답을 한 트랜잭션(`record`). 실패하면 로그 + 사람 메시지 `error` + 오류 답("받는 PC 기록 오류"). 회귀 `storage_failure_sends_error_reply`(트리거로 답 저장 실패 주입, E5) — "트랜잭션 없이" · "실패해도 성공 답" mutation 각각 FAIL |
| timeout 시 자손 미종료 | 사실 | `ProcessTree` — Windows kill-on-close Job Object ↔ 그 밖 프로세스 그룹(`kill -KILL -pgid`). 시간 초과 · 정상 종료 모두 묶음 종료 후 읽기 스레드 회수. 두 OS 같은 동작이라 D30 의 OS 연동 조건에 맞는다(UI 아님). 회귀 `process_tree_is_killed`(E4, 셸 → 손자 ping/sleep) — "자식만 kill" mutation 시 29초로 FAIL. macOS 경로는 이 PC 에서 실행 확인 못 함(Phase 4) |
| `reply_to` 미검증 | 사실 | 답은 `reply_to` 가 같은 대화 · 사람 · `sent` 인 내 메시지일 때만, 받으면 그 메시지를 `done/error` 로(한 트랜잭션) — 두 번째 답 · 요청 없는 답은 `unknown`. 회귀 `forged_or_duplicate_reply_refused`(E5) — "sent 확인 제거" mutation FAIL |

## 적대적 평가 — 라운드 2/2 (상한 — 재평가 없음)

- 수단: codex-review 세션 `/codex:adversarial-review --base d5fadb4 --scope branch` · 대상 HEAD `6383ed9` · 2026-10-11
- Verdict: needs-attention

### 원문

```text
- [high] ACK 손실을 실행 실패로 확정해 원격 작업이 숨겨지고 재시도 시 중복 실행된다 (crates/agent/src/chat.rs:195-206)
  수신 측이 메시지를 `queued`로 영구 저장한 뒤 `Response::Ok` 전송에 실패하면 작업은 이미 실행되지만, 송신 측은 transport error를 받아 원 메시지를 즉시 `Error`로 바꾼다. 이후 정상 도착한 reply도 `inbound_reply`가 원 메시지 상태를 `Sent`로 제한해 `unknown`으로 거절한다. 사용자는 실패로 오인해 새 UUID로 재전송하고, 비용·파일 변경 같은 비가역 작업을 중복 실행할 수 있다.
  Recommendation: 명시적 원격 거절과 ACK 여부가 불명확한 transport failure를 분리하라. 불명확 상태에서는 correlated reply를 계속 수용하고, 같은 message UUID로 안전하게 재조회·재전송하도록 하라. 수신 저장 직후 ACK 연결을 끊는 회귀 테스트를 추가하라.
- [high] 동일 대화의 동시 전송이 원격 Claude 세션에서 역순 실행될 수 있다 (crates/agent/src/chat.rs:172-203)
  송신 측은 메시지를 로컬 DB에 순서대로 저장한 뒤 잠금을 풀고 각각 독립된 TCP 요청을 비동기로 보낸다. 두 `send` 호출이 겹치면 두 번째 연결이 먼저 도착할 수 있으며, 수신 측 FIFO는 도착 후 생성된 DB id만 사용한다. 결과적으로 송신 이력은 A→B인데 원격 Claude 세션은 B→A로 실행되어 문맥과 답변이 어긋난다.
  Recommendation: 대화별 outbound dispatcher로 저장과 전송 순서를 직렬화하거나 protocol에 monotonic sequence를 추가해 수신 측이 순서대로 처리하게 하라. 첫 요청 응답을 지연시킨 상태에서 두 `send` future를 동시에 실행하는 회귀 테스트를 추가하라.
- [high] 오래된 메시지 조회 응답이 다른 대화 화면을 덮어 잘못된 피어로 답할 수 있다 (crates/app/web/app.js:287-310)
  `showMessages`는 요청 전에 `current.id`를 읽지만 응답 후 현재 대화가 그대로인지 확인하지 않는다. 대화 A 조회가 느린 동안 사용자가 B를 열면 A의 늦은 응답이 B 제목 아래 렌더링될 수 있다. reply handler는 그 시점의 `current.id`인 B로 전송하므로 사용자가 A 내용에 대한 민감한 답을 B 피어에게 보낼 수 있다.
  Recommendation: 조회 시작 시 conversation id를 캡처하고 응답 시 `current?.id`와 다르면 폐기하라. 요청 세대 번호나 `AbortController`로 stale 요청을 취소하고, reply 대상 id를 실제 렌더링된 대화와 결합하라.
```

### 대조 · 조치

| 지적 | 판단 | 조치 |
|---|---|---|
| ACK 손실 → 실패 확정 · 중복 실행 | 사실 | 상대의 명시적 거절만 `error`. 전송 오류는 `sent` 유지 + "전달 확인 안 됨 — 상대가 받았을 수 있다. 답을 기다린 뒤 다시 보내세요" 오류 → 늦게 온 답을 받는다. 회귀 `unconfirmed_delivery_keeps_waiting_for_reply`(닫힌 포트로 전송 오류 → `sent` 유지 → 늦은 답 수용, E5) — "전송 오류도 error 로" mutation FAIL. 같은 uuid 재전송 · 재조회는 미조치(아래) |
| 동시 보내기 역순 도착 | 사실 | 대화별 보내기 잠금(`outbound`) — 저장 · 전송 · 응답까지 한 번에 하나 → 도착 순서 = 이력 순서. 회귀 `concurrent_sends_keep_order`(첫 연결만 0.3초 늦추는 TCP 중계로 결정적 재현, E5) — "잠금 제거" mutation 3회 모두 FAIL(처음 쓴 무작위 동시 전송 테스트는 mutation 을 못 잡아 중계 방식으로 바꿨다) |
| 늦은 조회 응답이 다른 대화를 덮음 | 사실 | 조회 시작 시 대화를 잡아 두고 응답 때 `current` 와 다르면 버림, 대화를 바꾸면 이전 메시지를 먼저 지움. `node --check` 통과, JS 테스트 없음(아래) |

## 미조치 지적

| 지적 | 판단 | 사유 |
|---|---|---|
| E8 수동 gate 미충족 | 사실 | 프로토타입(Phase 1 끝) 인수 Task 로 이관(D28). ABANDON 기록 |
| 같은 메시지 uuid 재전송 · 재조회(라운드 2) | 사실 — 일부 조치 | 불확실 상태를 실패로 확정하지 않는 것까지 했다. 같은 uuid 재전송은 오프라인 재전송(Phase 2, masterPlan 제외 항목)과 함께 — 수신 측은 이미 같은 uuid 를 한 번만 실행한다 |
| 수신 저장 직후 ACK 끊기 회귀 테스트(라운드 2) | 사실 — 근사 테스트 | 같은 코드 경로(전송 오류)를 닫힌 포트로 시험했다. 저장 뒤 응답만 끊는 주입은 노드 응답 경로에 시험용 고리가 필요하다 |
| 동시 수신 순서 역전 경쟁 테스트(라운드 1) | 사실 — 구조로 해소 | 실행 순서를 DB id 로 정하고, 보내는 쪽이 대화별로 직렬화(라운드 2) — 도착 순서 자체가 하나 |
| 프로세스 그룹 종료 macOS 실행 확인(라운드 1) | 사실 | 이 PC 는 Windows — Phase 4(macOS) 빌드 · 인수에서 확인 |
| 웹UI 늦은 응답 폐기 JS 테스트(라운드 2) | 사실 — 코드는 고침 | JS 테스트 기반 도입 시(#00 R1-F4 와 같은 조건) |
