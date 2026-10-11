# eval.md — 13.notify-settings 적대적 평가

## 평가 원장

<!-- tools/check_gates.py 산출 — 손으로 고치지 않는다. 단 CHECK 가 없는 수동 gate 의
     EVIDENCE 와 ABANDON 줄은 사람이 적고, 재실행해도 보존된다.
     WEAK 줄은 기계가 매 실행 다시 단다 — 약한 gate 를 셀 수 있게 하는 표시이며
     판정에는 영향이 없다. -->

측정 커밋: 293e5cb7f31e3b3c2ffbf9ef02276a649324d666

- [x] E1: A4 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

- [x] E2: A4 — 우리 코드 경고 0(clippy -D warnings)
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.26s

- [x] E3: A1·A2 — 거르기(notifier 9건: 기존 6 + kinds_filtered · relay_modes · preview_off)
  CHECK: cargo test -p agentnetwork-app --test notifier
  EXPECT: test result: ok. 9 passed; 0 failed
  EVIDENCE: test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.10s

- [x] E4: A2·A3 — 설정 API(settings_api 3건: 기존 2 + prefs_patch_validated_and_persisted)
  CHECK: cargo test -p agentnetwork-app --test settings_api
  EXPECT: test result: ok. 3 passed; 0 failed
  EVIDENCE: test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s

- [x] E5: A1 — 자동 왕복 알림 종류(relay 9건: 기존 7 + relay_events_start_step_end · relay_events_first_run_fails)
  CHECK: cargo test -p an-agent --test relay
  EXPECT: test result: ok. 9 passed; 0 failed
  EVIDENCE: test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.68s

- [x] E6: A3 — 상한 기본값(chat_api 6건: 기존 5 + relay_cap_default_from_settings)
  CHECK: cargo test -p agentnetwork-app --test chat_api
  EXPECT: test result: ok. 6 passed; 0 failed
  EVIDENCE: test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.32s

- [x] E7: A4 — 화면 로직 node 테스트 회귀 없음
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# fail 0/
  EVIDENCE: # fail 0

- [ ] E8: A1·A2·A3 — 실화면(수동): 설정 → 알림 항목 · 자동 왕복 시작 · 끝 알림 · 미리보기 끄기 · 상한 기본값
  EVIDENCE: pending

ABANDON: E8 실화면 확인은 원격 세션이라 불가 — 프로토타입(Phase 1 끝) 인수 Task 에서 수동 테스트 일괄(masterPlan D28, 사용자 지시 2026-10-11 "사용자 테스트는 완료 후 진행")

## 적대적 평가 — 라운드 1/2

- 수단: codex-review 세션 `/codex:adversarial-review --base 2e0348b --scope branch` · 대상 HEAD `873f706` · 2026-10-11
- Verdict: needs-attention

### 원문

```text
- [high] 자동 왕복 시작 알림이 양쪽에서 정확히 1건 발생하지 않음 (crates/agent/src/chat.rs:824-826)
  첫 자동 왕복 응답은 요청 작성자가 Human이라는 이유로 Reply로 분류됩니다. 실제 추가 테스트도 연 쪽 이벤트를 [Reply, RelayStep, RelayStep, RelayEnd], 받은 쪽을 [Request, RelayStart, RelayStep, RelayEnd]로 단정합니다. 따라서 start_end 기본 모드에서 연 쪽은 RelayStart를 전혀 받지 않고, 받은 쪽은 일반 Request와 RelayStart를 모두 받아 A1의 '양쪽 PC 시작 1건·끝 1건뿐' 계약을 깨뜨립니다. 첫 실행이 실패하면 받은 쪽은 RelayStart/RelayEnd 모두 못 받을 수도 있습니다.
  Recommendation: 초기 Request에 자동 왕복 시작임을 나타내는 프로토콜 메타데이터를 싣고, 양쪽 최초 인바운드 이벤트를 RelayStart로 분류하면서 중복 Request/Reply 알림을 억제하십시오. Notifier 최종 출력 기준으로 양쪽 정확히 [RelayStart, RelayEnd]인지와 첫 실행 실패 경로를 회귀 테스트하십시오.
- [medium] 설정 PATCH가 두 트랜잭션으로 분리되어 부분 저장됨 (crates/app/src/notify.rs:188-200)
  세부 설정은 첫 SQLite 트랜잭션에서 커밋된 뒤 notify_inbound가 별도 set_enabled 호출로 저장됩니다. 두 번째 쓰기가 실패하면 API는 500을 반환하지만 앞선 변경은 재시작 후에도 남습니다. 두 호출 사이 이벤트 처리나 동시 PATCH도 끼어들 수 있어 메모리·DB 설정과 알림 동작이 찢어질 수 있습니다. 이는 '하나라도 실패하면 아무것도 안 바꿈, 한 트랜잭션' 계약 위반입니다.
  Recommendation: notify_inbound를 writes에 포함해 모든 키를 단일 SQLite 트랜잭션으로 커밋하고, 성공 후 Status 전체를 잠금 안에서 한 번에 교체하십시오. 두 번째 쓰기 실패 및 동시 PATCH를 주입해 부분 저장이 없음을 테스트하십시오.
```

### 대조 · 조치

| 지적 | 판단 | 조치 |
|---|---|---|
| 양쪽 시작 알림이 1건이 아님 | 사실 | 자동 왕복을 여는 사람 요청에 `message.relay = true` — 받은 쪽은 그것을 `RelayStart`, 뒤 에이전트발 요청은 `RelayStep`. 연 쪽은 자동 왕복 중 사람 요청에 온 첫 답을 `RelayStart`. 받은 쪽 내 에이전트가 에이전트발 요청에 실패하면 `RelayEnd`. 회귀: `relay_events_start_step_end`(양쪽 시작 1 · 중간 · 끝 1) · `relay_events_first_run_fails`(연 쪽 시작 · 끝, 받은 쪽 시작) — "relay 표시 무시" · "연 쪽 첫 답 = Reply" mutation 모두 FAIL |
| 설정 PATCH 부분 저장 | 사실 | `notify_inbound` 까지 한 트랜잭션, 세부 설정 · 켜짐 · 건수를 한 상태 잠금 안에서 바꿈. 회귀: 틀린 값과 함께 온 `notify_inbound` 도 저장 안 됨. 쓰기 실패 주입 테스트는 미조치 |

## 적대적 평가 — 라운드 2/2 (상한 — 재평가 없음)

- 수단: codex-review 세션 `/codex:adversarial-review --base 2e0348b --scope branch` · 대상 HEAD `a619b9d` · 2026-10-11
- Verdict: needs-attention

### 원문

```text
- [high] 첫 실행 실패 시 받은 쪽의 RelayEnd가 누락됨 (crates/agent/src/chat.rs:975-977)
  자동 왕복의 첫 요청은 Author::Human으로 저장되지만 종료 알림 조건은 Author::Agent만 허용합니다. 따라서 첫 원격 에이전트 실행이 실패하면 연 쪽은 RelayStart와 RelayEnd를 받지만 받은 쪽은 RelayStart만 받습니다. 실제 회귀 테스트도 이 비대칭을 명시적으로 기대합니다. 사용자는 받은 PC에서 자동 왕복이 끝났는지 알 수 없고, intent.md A1의 '양쪽 PC 시작 1건·자동 멈춤 1건'을 충족하지 못합니다.
  Recommendation: 초기 relay 요청 여부를 대화나 메시지에 영속화하고, 그 요청의 실행 실패에도 받은 쪽에서 RelayEnd를 발생시키십시오. relay_events_first_run_fails가 양쪽 모두 [RelayStart, RelayEnd]를 요구하도록 수정하십시오.
```

### 대조 · 조치

| 지적 | 판단 | 조치 |
|---|---|---|
| 첫 실행 실패 시 받은 쪽 끝 알림 없음 | 사실 | `message.relay` 를 연 쪽 에이전트 이름으로 바꾸고, 받은 쪽은 `relay` · `by_agent` 를 받으면 대화를 자동 왕복으로 기록(`mark_relayed` — 스키마 변경 없이 `relay_agent` 만). 받은 쪽 실행 실패 끝 알림은 에이전트발 요청 + 자동 왕복을 연 첫 사람 요청(그 앞 실행 요청 없음). 회귀 `relay_events_first_run_fails` 를 양쪽 `[RelayStart, RelayEnd]` 로 — "첫 사람 요청 판정 제거" · "기록 제거" mutation FAIL. 덤: 받은 쪽 화면이 상대 에이전트 이름을 보인다 |

## 미조치 지적

| 지적 | 판단 | 사유 |
|---|---|---|
| E8 수동 gate(실화면) 미충족 | 사실 | 프로토타입(Phase 1 끝) 인수 Task 로 이관(D28). ABANDON 기록 — 19 TC-I |
| 설정 쓰기 실패 · 동시 PATCH 주입 테스트(라운드 1) | 사실 — 코드는 고침 | 한 트랜잭션 · 한 상태 잠금. 쓰기 실패 주입은 DB 고리가 필요 — 비용 대비 이득 작음 |
