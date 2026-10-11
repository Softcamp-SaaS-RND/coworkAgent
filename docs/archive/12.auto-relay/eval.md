# eval.md — 12.auto-relay 적대적 평가

## 평가 원장

<!-- tools/check_gates.py 산출 — 손으로 고치지 않는다. 단 CHECK 가 없는 수동 gate 의
     EVIDENCE 와 ABANDON 줄은 사람이 적고, 재실행해도 보존된다.
     WEAK 줄은 기계가 매 실행 다시 단다 — 약한 gate 를 셀 수 있게 하는 표시이며
     판정에는 영향이 없다. -->

측정 커밋: 06103b99f615a04f29bbacfa2142d9dc619ea1e6

- [x] E1: A5 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

- [x] E2: A5 — 우리 코드 경고 0(clippy -D warnings)
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.21s

- [x] E3: A1·A2·A3·A4 — 자동 왕복(relay 테스트 7건: relay_runs_until_cap · human_write_stops_relay · receiver_stop_refuses_agent_requests · resume_restarts_count · remote_failure_stops_relay · late_reply_after_transport_error_accepted · stop_during_run_then_resume)
  CHECK: cargo test -p an-agent --test relay
  EXPECT: test result: ok. 7 passed; 0 failed
  EVIDENCE: test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.68s

- [x] E4: A5 — 스키마 7 이전(paths 6건)
  CHECK: cargo test -p an-core --test paths
  EXPECT: test result: ok. 6 passed; 0 failed
  EVIDENCE: test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s

- [x] E5: A1·A2·A3 — 웹UI API(relay_api 포함 chat_api 5건)
  CHECK: cargo test -p agentnetwork-app --test chat_api
  EXPECT: test result: ok. 5 passed; 0 failed
  EVIDENCE: test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.29s

- [x] E6: A1·A2 — 화면 로직 node 테스트(자동 왕복 문구)
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# fail 0/
  EVIDENCE: # fail 0

- [ ] E7: A1·A2·A3 — 실화면 · 실제 claude(수동): 두 에이전트 자동 왕복 · 상한 · 양쪽 멈춤 · 재개
  EVIDENCE: pending

ABANDON: E7 실화면 · 실제 claude 확인은 원격 세션이라 불가 — 프로토타입(Phase 1 끝) 인수 Task 에서 수동 테스트 일괄(masterPlan D28, 사용자 지시 2026-10-11 "사용자 테스트는 완료 후 진행")

## 적대적 평가 — 라운드 1/2

- 수단: codex-review 세션 `/codex:adversarial-review --base 0dec99a --scope branch` · 대상 HEAD `ef67630` · 2026-10-11
- Verdict: needs-attention

### 원문

```text
- [high] ACK 손실 시 실제 실행 결과를 영구 거부한다 (crates/agent/src/chat.rs:930-954)
  `relay_forward`는 전송 오류가 발생하면 요청을 `Undelivered`로 확정한다. 그러나 요청은 상대에게 이미 저장·실행됐고 ACK만 손실됐을 수 있다. 이후 정상 `Reply`가 와도 `inbound_reply`는 원 요청 상태가 `Sent`인 경우만 허용하므로 `unknown`으로 거부한다. 양쪽 이력이 갈라지고 실행 결과가 유실되며 재개도 해당 답을 이용하지 못한다. 또한 line 930의 `set_flags` 실패를 무시하고 전송하므로, 로컬 상태 저장 실패에서도 같은 문제가 발생한다.
  Recommendation: 전송 전에 `set_flags(..., Sent)` 성공을 필수화하고, transport `Err`에서는 `Sent` 또는 별도 `delivery_unknown` 상태를 유지해 늦은 답을 허용한다. 명시적 `Response::Error`만 실패로 확정하고, 요청 수신 후 ACK만 끊긴 상황의 회귀 테스트를 추가한다.
- [high] 원격 실행 실패도 다음 relay 실행으로 전달된다 (crates/agent/src/chat.rs:719-734)
  `Reply { ok: false }`를 `Error`로 기록한 직후에도 `ok`를 검사하지 않고 해당 답을 `Queued` 실행 요청으로 바꾼다. 따라서 상대 Claude 실행 실패 문구가 내 Claude 입력이 되고 자동 왕복이 상한까지 계속될 수 있다. 이는 계획의 "실행 실패면 멈춤" 계약을 깨고 불필요한 유료 실행과 오도된 대화를 만든다.
  Recommendation: relay 큐잉 조건에 `ok`를 포함하고 `ok == false`이면 답은 `Error`로 보존한 채 `relay_on = false`로 전환한다. 원격 실패 후 양쪽 실행 횟수가 증가하지 않는 테스트를 추가한다.
- [medium] 수신 답 저장과 relay 큐 등록이 원자적이지 않아 재시작 후 영구 정지한다 (crates/agent/src/chat.rs:720-735)
  답과 원 요청 상태를 커밋한 뒤 별도 UPDATE로 답을 `Queued`로 바꾼다. 두 작업 사이의 프로세스 종료·I/O 오류는 `relay_on = true`인 채 실행 대기 메시지가 없는 DB를 남긴다. 시작 복구는 `Queued`/`Running`만 찾으므로 자동 왕복은 진행 중으로 표시되지만 다시 깨워지지 않는다. 상한 도달 시 `relay_on`을 끄는 작업도 같은 커밋 밖에 있다.
  Recommendation: 답 기록, 원 요청 상태, relay 큐 플래그 또는 relay 종료를 하나의 SQLite transaction으로 커밋하고 커밋 후에만 `wake`한다. 중간 상태 DB를 다시 열어 복구되는 fault-injection 테스트를 추가한다.
```

### 대조 · 조치

| 지적 | 판단 | 조치 |
|---|---|---|
| ACK 손실 시 늦은 답 거부 | 사실 | 넘기기 전 `sent` 저장이 실패하면 보내지 않고 멈춤. 전송 오류는 `sent` 유지(늦은 답 수신), 명시적 거절만 `error`. 회귀 `late_reply_after_transport_error_accepted`(넘기는 중 주소 끊김 → 늦은 `reply` 수신 · 원 글 `done`) — "전송 오류면 undelivered" mutation FAIL |
| 상대 실행 실패 답도 넘김 | 사실 | 자동 전달 조건에 `ok` — 실패 답은 `error` 로 남기고 멈춤. 회귀 `remote_failure_stops_relay`(내 에이전트 0회 · 멈춤) — "`ok` 조건 제거" mutation FAIL |
| 답 기록과 자동 전달 등록 비원자 | 사실 | 답 · 원 요청 상태 · 자동 전달 등록(또는 멈춤)을 한 트랜잭션, 커밋 뒤 깨움. 중간 상태 주입 테스트는 미조치(트랜잭션 경계는 코드로 확인) |

## 적대적 평가 — 라운드 2/2 (상한 — 재평가 없음)

- 수단: codex-review 세션 `/codex:adversarial-review --base 0dec99a --scope branch` · 대상 HEAD `db64224` · 2026-10-11
- Verdict: needs-attention

### 원문

```text
- [high] 왕복 상한 도달 뒤에도 상대 에이전트를 한 번 더 실행한다 (crates/agent/src/chat.rs:916-944)
  `process`가 내 에이전트 실행 전에 `relay_count`를 증가시키지만, `relay_forward`는 증가된 count가 cap에 도달했는지 확인하지 않고 항상 `Request::Message`를 보낸다. 따라서 cap=1도 상대 답 1회 + 내 답 1회로 이미 1왕복을 채운 뒤 상대를 다시 실행한다. 테스트도 cap=2에서 실행 횟수 `(내 2, 상대 3)`을 명시해 결함을 고정한다(`crates/agent/tests/relay.rs:74-79`). 사용자 설정을 초과한 유료 실행과 상한 뒤의 dangling reply가 발생한다.
  Recommendation: 증가 후 count가 cap에 도달하면 내 답을 상대 이력에 실행 없는 에이전트 글로 미러하고 `relay_on`을 끈다. count < cap일 때만 실행 요청을 보낸다. cap=1 및 cap=N에서 양쪽 실행 횟수가 각각 정확히 1/N인지 경계 테스트를 추가한다.
- [high] 수동 멈춤 중 답이 생성되면 재개가 진행 중 상태로 영구 정지한다 (crates/agent/src/chat.rs:657-675)
  재개는 전체 이력의 마지막 한 건이 `상대 agent + !mine + !exec + done`일 때만 이를 다시 큐잉한다. 내 에이전트 실행 중 사람이 멈추면 더 최신인 내 에이전트 답이 `Done`으로 기록되지만 전달되지 않는다. 이 상태에서 재개하면 count=0, relay_on=true만 설정되고 작업은 전혀 깨우지 않아 UI는 진행 중인데 왕복은 멈춘다. `stopped` 거절로 마지막 내 답이 Error가 된 경우도 동일하다. 이는 A3의 수동 멈춤 후 재개 경로를 깨뜨린다.
  Recommendation: 미전달 relay handoff를 명시적으로 저장한다. 재개 시 완료됐지만 전달되지 않은 내 답은 동일 UUID로 재전송하고, 아직 실행하지 않은 상대 답만 큐잉한다. relay 상태 전환과 pending 등록을 한 트랜잭션으로 처리하고, 실행 중 멈춤→재개 및 `stopped` 거절→재개 회귀 테스트를 추가한다.
```

### 대조 · 조치

| 지적 | 판단 | 조치 |
|---|---|---|
| 상한 뒤 상대 1회 더 실행 | 사실 | 상한 n = 양쪽 n 번씩. n 번째 내 답은 실행 요청 대신 `post`(agent · 실행 없음)로 상대 이력에 기록만 하고 멈춤 — 받은 쪽은 내 에이전트 답에 단 답일 때만 받음. 회귀: `relay_runs_until_cap`(상한 2 → 2 · 2, 받은 쪽 마지막 글 실행 아님) · `resume_restarts_count`(상한 1 → 1 · 1) — "상한 기록 제거" mutation FAIL |
| 멈춤 중 생긴 내 답이 재개 때 넘어가지 않음 | 사실 | 재개 = 켜기 + 넘기지 않은 상대 답 대기열(한 트랜잭션) 또는 넘기지 못한 내 답(`done` · `undelivered` · 거절 `error` · `sent`)을 같은 uuid 로 다시 넘김. 받은 쪽은 기록만 받았던 같은 uuid 글을 실행 요청으로 올림. 회귀 `stop_during_run_then_resume`(실행 중 멈춤 → 재개 → 상대 실행 · 양쪽 이력 일치) — "다시 넘기기 제거" · "올리기 제거" mutation FAIL |

## 미조치 지적

| 지적 | 판단 | 사유 |
|---|---|---|
| E7 수동 gate(실화면 · 실제 claude) 미충족 | 사실 | 프로토타입(Phase 1 끝) 인수 Task 로 이관(D28). ABANDON 기록 |
| 답 기록 · 자동 전달 등록 중간 상태 주입 테스트(라운드 1) | 사실 — 코드는 고침 | 한 트랜잭션으로 묶음. 프로세스 중단 주입은 테스트 고리(실패 주입 지점)가 필요 — 비용 대비 이득 작음 |
| 넘기지 못한 넘김을 명시적으로 저장(라운드 2) | 사실 — 마지막 글까지 | 재개는 마지막 글로 판정(넘기지 못한 내 답 · 넘기지 않은 상대 답). 그 뒤에 사람 글이 있으면 재개해도 넘기지 않는다 — 사람이 새로 쓰면 된다. `stopped` 거절 → 재개 전용 회귀 테스트는 없음(같은 판정 경로) |
