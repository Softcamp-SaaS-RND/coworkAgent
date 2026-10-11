# eval.md — 02.inbound-toast 적대적 평가

## 평가 원장

<!-- tools/check_gates.py 산출 — 손으로 고치지 않는다. 단 CHECK 가 없는 수동 gate 의
     EVIDENCE 와 ABANDON 줄은 사람이 적고, 재실행해도 보존된다.
     WEAK 줄은 기계가 매 실행 다시 단다 — 약한 gate 를 셀 수 있게 하는 표시이며
     판정에는 영향이 없다. -->

측정 커밋: da6fba07a40a32a641bd25f486acebf8ce096654

- [x] E1: A5 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

- [x] E2: A5 — 우리 코드 경고 0(clippy -D warnings)
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.25s

- [x] E3: A1 — 인바운드 이벤트 2건(accepted_hello_emits_pair_request · applied_approve_emits_approved_only)
  CHECK: cargo test -p an-net --test events
  EXPECT: test result: ok. 2 passed; 0 failed
  EVIDENCE: test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s

- [x] E4: A1·A2·A3 — Notifier 5건(disabled_shows_nothing · enabled_shows_event · toast_xml_escapes_remote_text · blocked_os_does_not_block_events · turning_off_discards_queued_toasts)
  CHECK: cargo test -p agentnetwork-app --test notifier
  EXPECT: test result: ok. 5 passed; 0 failed
  EVIDENCE: test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.24s

- [x] E5: A2 — 설정 API 2건(notify_default_off · notify_toggle_persists)
  CHECK: cargo test -p agentnetwork-app --test settings_api
  EXPECT: test result: ok. 2 passed; 0 failed
  EVIDENCE: test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

- [x] E6: A5 — 의존 경계 3 · 3 · 0
  CHECK: python tools/check_crates.py
  EXPECT: crates=3 edges=3 violations=0
  EVIDENCE: crates=3 edges=3 violations=0

- [ ] E7: A4 — Windows 실동작(수동): 알림 켬 → 다른 PC 의 초대로 페어링 요청 → 토스트 표시(출처 agentNetwork) · 클릭 시 피어 화면 · 집중 지원 켬 상태에서 배너 억제(알림 센터에는 남음)
  EVIDENCE: pending

ABANDON: E7 원격 세션이라 화면 확인 불가 — 프로토타입(Phase 1 끝) 인수 Task 에서 수동 테스트 일괄(masterPlan D28, 사용자 지시 2026-10-11)

## 적대적 평가 — 라운드 1/2

- 수단: codex-review 세션 `/codex:adversarial-review --base dfab977 --scope branch`(02 + 01 개정 3) · 대상 HEAD `6d9fa12` · 2026-10-11
- Verdict: needs-attention

### 원문

```text
- [high] 동기식 `osascript`가 피어 응답과 Tokio 런타임을 막음 (crates/app/src/notify.rs:192-194)
  `Node::handle`은 응답 전 동기식 이벤트 콜백을 호출하며, macOS sink는 여기서 `Command::status()`로 `osascript` 종료를 무기한 기다립니다. 이 블로킹 호출 중에는 바깥 `INBOUND_DEADLINE`도 polling되지 않습니다. 알려진 피어가 반복 `Hello`를 보내면 출처당 최대 4개 worker를 점유할 수 있고, 같은 runtime을 쓰는 피어 서버와 웹 API까지 정지할 수 있습니다. 정상 환경에서도 느린 `osascript`가 성공한 페어링 요청을 클라이언트 timeout으로 바꿉니다.
  Recommendation: 프로토콜 응답을 먼저 기록하고 이벤트는 bounded queue로 넘기십시오. OS 표시는 전용 worker 또는 `spawn_blocking`에서 실행하고, 자식 프로세스 timeout·kill과 queue 포화 정책을 추가하십시오. 멈춘 notifier 중에도 피어 응답과 웹 API가 진행되는 회귀 테스트가 필요합니다.
- [medium] 동일 세대 `Approve` 재시도가 승인 알림을 계속 재발행함 (crates/net/src/node.rs:344-350)
  수신 코드는 같은 `round`이면 매번 `Approved` 이벤트를 만듭니다. `set_their_ok`의 SQL에는 `their_ok = 0` 조건이 없어 이미 승인된 행도 다시 성공으로 반환됩니다. SQLite는 동일값 UPDATE도 일치한 행을 change count 1로 계산합니다([SQLite 재현](https://sqlite.org/forum/info/481faf0b61804bc1?t=c)). 따라서 첫 승인 응답이 유실되어 정상적인 '다시 알리기'가 수행되거나 상대가 요청을 반복하면 상태 전이 없이 OS 토스트가 중복됩니다. 이는 A1의 '승인이 반영되면 한 번'과 retry idempotency를 위반합니다.
  Recommendation: `UPDATE ... WHERE spki = ?1 AND round = ?2 AND their_ok = 0`처럼 실제 false→true 전이만 성공으로 반환하고 그때만 이벤트를 발행하십시오. 동일 `Approve`를 두 번 보내도 응답은 idempotent하고 이벤트는 한 번뿐인 회귀 테스트를 추가하십시오.
```

### 대조 · 조치

| 지적 | 판단 | 조치 |
|---|---|---|
| 동기 osascript 가 런타임을 막음 | 사실(Windows WinRT 호출도 같은 경로) | `Notifier` 가 전용 스레드 + 유한 대기열(16, 가득 차면 버림 · 로그)로 표시, `on_event` 는 `try_send` 만. `osascript` 5초 뒤 kill. `Node::on_event` 문서에 "막히면 안 된다" 계약. 회귀 `blocked_os_does_not_block_events`(멈춘 표시 중 100건 < 0.5초 · 표시 ≤ 17건, E4). mutation "막히는 send" 에서 테스트가 끝나지 않음(600초 초과로 중단) 확인 |
| 같은 세대 approve 중복 알림 | 사실 | `set_their_ok` 에 `their_ok = 0` — 전이 때만 true · 이벤트. 같은 세대 재통지는 `ok`(이벤트 없음), 다른 세대는 `stale`. 회귀: `applied_approve_emits_approved_only` 에 재통지 단정 추가(E3). mutation 2종(전이 조건 제거 · 재통지 = stale) FAIL 확인 |

## 적대적 평가 — 라운드 2/2 (상한 — 재평가 없음)

- 수단: codex-review 세션 · 같은 base `dfab977` · 대상 HEAD `d07d7fc` · 2026-10-11 — 01 개정 3 은 blocker 없음
- Verdict: needs-attention

### 원문

```text
- [medium] OFF 전환 뒤 대기 중인 토스트가 계속 표시됨 (crates/app/src/notify.rs:55-58)
  켜진 상태에서 큐에 들어간 Toast는 worker가 `enabled`를 다시 확인하지 않고 무조건 `show`한다. OS 표시가 막혀 큐가 쌓인 뒤 사용자가 성공적으로 OFF로 바꿔도 최대 16건이 이후 표시될 수 있고, 아직 첫 Windows 표시 전이라면 OFF 상태에서 AUMID 레지스트리 등록도 발생한다. A2의 "꺼져 있으면 아무것도 표시하지 않는다"와 N5를 위반한다.
  Recommendation: 큐 항목에 설정 generation을 넣거나 worker가 공유 enabled/generation을 검사해 OFF 이전 대기 항목을 폐기하라. blocked sink에서 enqueue → OFF → release 후 in-flight 이외 추가 표시가 0건임을 단정하는 회귀 테스트를 추가하라.
- [medium] 큐 포화 경로가 동기 파일 로그로 다시 수신 처리를 막음 (crates/app/src/notify.rs:85-88)
  Round 1의 `try_send`는 포화 시 즉시 반환하지만 곧바로 `log::warn!`를 실행한다. 운영 `FileLogger::log`는 전역 Mutex를 잡고 `File::write_all`을 동기 호출한다. OS notifier가 막힌 상태에서 알려진 피어가 반복 Hello를 보내면 매 이벤트가 이 경로를 타며, 느리거나 막힌 로그 I/O가 `Node::handle`의 응답 전 콜백을 다시 정지시킨다. 현재 `blocked_os_does_not_block_events`는 logger가 설치되지 않아 이 경로를 검증하지 않는다.
  Recommendation: drop 경로에서는 Atomic counter만 증가시키고 worker나 별도 주기 작업에서 집계 로그를 남겨라. 포화 상태와 blocking logger를 함께 둬도 이벤트 콜백이 제한 시간 내 반환하는 회귀 테스트를 추가하라.
- [medium] 설정 저장 실패 시 UI는 OFF지만 backend는 계속 ON (crates/app/web/app.js:144-149)
  체크박스는 change 전에 이미 새 값으로 바뀐다. PUT이 SQLite 잠금 등으로 실패하면 `act`가 오류만 표시하고 이 handler는 이전 값 복원이나 GET 재조회 없이 끝난다. 따라서 OFF로 보이는 체크박스와 달리 `Notifier.enabled`는 계속 true여서 알림이 무기한 표시될 수 있다.
  Recommendation: 변경 전 authoritative 값을 보관하고 요청 중 checkbox를 비활성화하라. 실패 시 이전 값으로 복원하거나 GET `/api/settings`로 재동기화하고, 실패한 OFF PUT 뒤 checkbox가 다시 ON임을 검증하라.
```

### 대조 · 조치

| 지적 | 판단 | 조치 |
|---|---|---|
| OFF 뒤 대기 토스트 표시 | 사실 | 표시 스레드가 공유 `enabled` 를 매 건 확인 — 끈 뒤 대기분은 버린다(레지스트리 등록도 표시 경로라 함께 막힘). 회귀 `turning_off_discards_queued_toasts`(막힌 1건 + 대기 4건 → 끈 뒤 1건만, E4). mutation "표시 스레드 켜짐 무시" FAIL 확인 |
| 포화 경로 동기 로그 | 사실 | 이벤트 쪽은 `AtomicUsize` 로 버린 수만 세고, 표시 스레드가 다음 건에서 모아 `log::warn!`. 막히는 로거를 둔 회귀 테스트는 미추가(아래 미조치) |
| 설정 저장 실패 시 UI 불일치 | 사실 | 요청 중 체크박스 비활성, 실패하면 `GET /api/settings` 로 서버 값을 다시 읽어 되돌림(읽기도 실패하면 이전 값). JS 테스트 미추가(아래 미조치) |

## 미조치 지적

| 지적 | 판단 | 사유 |
|---|---|---|
| E7 수동 gate 미충족 | 사실 | 프로토타입(Phase 1 끝) 인수 Task 로 이관(D28). ABANDON 기록 |
| 포화 + 막히는 로거 회귀 테스트(라운드 2) | 사실 — 코드는 고침 | 전역 로거는 프로세스당 한 번만 설치돼 테스트 격리가 어렵다. 이벤트 경로에 로그 호출이 없음은 코드로 확인 |
| 설정 UI 실패 되돌림 테스트(라운드 2) | 사실 — 코드는 고침 | JS 테스트 기반이 없다 — Phase 1 UI Task 에서 도입 시(#00 R1-F4 와 같은 조건) |

