# eval.md — 1e.autostart-watchdog 적대적 평가

## 평가 원장

<!-- tools/check_gates.py 산출 — 손으로 고치지 않는다. 단 CHECK 가 없는 수동 gate 의
     EVIDENCE 와 ABANDON 줄은 사람이 적고, 재실행해도 보존된다.
     WEAK 줄은 기계가 매 실행 다시 단다 — 약한 gate 를 셀 수 있게 하는 표시이며
     판정에는 영향이 없다. -->

측정 커밋: 996c2799db394af66d7805753f9d996348397465

- [x] E1: A7 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

- [x] E2: A7 — 우리 코드 경고 0(clippy -D warnings)
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.23s

- [x] E3: A4·A7 — 화면 로직 node 테스트 47건 이상 · 실패 0(설정 목차에 `startup`)
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# pass (4[7-9]|[5-9][0-9])\s+# fail 0/
  EVIDENCE: # skipped 0 / # todo 0 / # duration_ms 139.8971

- [x] E4: A1·A2·A3·A6 — 감시 6건(정상 종료면 한 번 띄우고 끝 / 비정상이면 다시 띄우고 횟수가 늘어 자식에 전달 / 60초 안 3번이면 멈춤 · 사유 / 간격이 60초를 넘는 비정상 종료는 계속 / 자식을 띄우지 못하면 멈춤 / 실제 실행 파일: 인자가 있으면 감시 없이 CLI 사용법 오류(2)로 바로 끝나고, 그 실행 파일을 자식으로 감시하면 비정상 3번에 멈춤)
  CHECK: cargo test -p coworkagent-app --test watchdog
  EXPECT: test result: ok. 6 passed; 0 failed
  EVIDENCE: test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s

- [x] E5: A5 — 패닉 메시지 · 파일 · 줄이 로그 파일에 남는다(1건)
  CHECK: cargo test -p an-core --test panic_log
  EXPECT: test result: ok. 1 passed; 0 failed
  EVIDENCE: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

- [x] E6: A4 — 자동 시작 8건(시험 키에 등록 · 같은 경로면 쓰지 않음 · 다른 경로면 고침 · 해제 / LaunchAgent plist 내용 · 경로 XML 이스케이프 / 설정 API 기본 켜짐 · 끄면 훅(false) 뒤 저장 · 다시 읽어도 꺼짐 / 훅 실패면 500 · 저장 안 함 / 켜기 뒤 저장 실패면 등록을 되돌림 · 끄기는 저장 실패면 OS 그대로 / 순서 — 켜기 훅 때 저장 꺼짐 · 끄기 훅 때 저장 이미 꺼짐(개정 2) / 동시 요청 뒤 마지막 등록 = 저장 값 / plist 통째 교체 — 실패하면 원래 파일 그대로 — 개정 1)
  CHECK: cargo test -p coworkagent-app --test autostart
  EXPECT: test result: ok. 8 passed; 0 failed
  EVIDENCE: test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s

- [ ] E7: A1·A2·A3·A4 — Windows 릴리스 실측: 실행 → 감시 · 앱 두 프로세스 · `HKCU\…\Run` 에 그 경로 / 앱 강제 종료(`taskkill /F`) → 다시 뜸 · 로그 "다시 띄움(1번째)" · 남은 outbox 이어 보냄 / 1분 안 3번 강제 종료 → 오류 트레이 / 트레이 "종료" → 두 프로세스 모두 없음 / 실행 중 다시 실행 → 브라우저 열림 · 감시 하나 / 설정에서 끄기 → Run 값 없음 · 다시 실행해도 없음(사람 확인은 19 TC)
  EVIDENCE: pending

- [ ] E8: A4 — macOS 실측: LaunchAgent plist 생성 · 로그인 시 실행 · 끄기 → 파일 없음
  EVIDENCE: pending

ABANDON: E7 이 PC 에 사용자 coworkAgent 가 실행 중(PID 7364) — 실측은 같은 데이터 폴더 · instance.lock · HKCU Run 을 써서 격리 불가, 새 실행은 그 앱에 열기를 넘기고 끝난다. plan 승인 조건대로 끄지 않고 19.prototype-acceptance TC 로 이관(D28)
ABANDON: E8 macOS 환경 없음 — Windows 에서만 개발·확인 가능. masterPlan D25 의 macOS TODO 로 이관

## 적대적 평가 — 라운드 1/2

- 2026-10-14 · `/codex:adversarial-review --base d58b291` · 세션 codex-review · 실행 HEAD `df1dd3f`(요청과 일치) · Verdict needs-attention(high 1 ·
  medium 1).

### 지적 원문

- [high] 자동 시작 OS 변경과 DB 저장이 원자적이지 않다 (crates/app/src/notify.rs:296-312) — OS 등록/삭제를 먼저 수행한 뒤 별도 잠금과 SQLite
  트랜잭션으로 설정을 저장한다. DB 잠금·디스크 오류가 발생하면 OS만 변경된다. 동시 PUT도 hook 실행 순서와 DB 잠금 획득 순서가 달라질 수 있다.
  예를 들어 enable hook 뒤 DB 저장이 실패하면 UI/DB는 false지만 다음 로그인에 앱이 실행된다. 반대 경우에는 DB가 true인데 실행되지 않는다. 시작 시
  false 상태는 `sync(false)`를 호출하지 않아 이 불일치가 자동 복구되지 않는다.
- [medium] LaunchAgent 갱신이 기존 등록 파일을 파괴할 수 있다 (crates/app/src/autostart.rs:31-34) — macOS plist를 대상 파일에 직접
  `std::fs::write`한다. 이 호출은 기존 파일을 truncate한 뒤 쓰므로 디스크 부족·프로세스 종료 같은 부분 실패에서 정상 plist가 빈 파일이나 부분
  XML로 바뀔 수 있다. 다음 로그인에 앱이 실행되지 않으므로 스스로 경로를 복구할 기회도 사라진다.

### 대조 · 처리

| 지적 | 판단 | 처리 |
|---|---|---|
| high · OS 등록과 DB 저장이 따로 | 사실 — plan 해석 4 가 "꺼져 있으면 아무것도 하지 않는다" 로 정해 어긋남이 고쳐질 길이 없었다 | ① 시작마다 설정 값으로 `sync`(꺼짐 = 지움) ② `update` 가 훅 잠금을 저장 끝까지 쥔다(등록 순서 = 저장 순서) ③ 저장 실패면 `f(저장된 값)` 으로 되돌림(되돌리기 실패는 `warn` — 다음 시작이 고침). 시험 `autostart` `save_failure_restores_registration`(구현 전 FAILED 확인) · `concurrent_puts_end_consistent` — plan 개정 1 |
| medium · plist 덮어쓰기 | 사실 | `autostart::write_atomic` — 같은 폴더 `.tmp` 에 쓰고 `sync_all` 뒤 `rename`, 실패하면 `.tmp` 지우고 원래 파일 그대로. 시험 `write_atomic_keeps_old_on_failure`(임시 자리를 폴더로 막아 실패 주입) — plan 개정 1 · E6 7건 |

- `concurrent_puts_end_consistent` 는 고치기 전 코드에서도 통과했다(경쟁이 시험 안에서 재현되지 않음) — 회귀 방지용, 직렬화의 근거는 코드(잠금
  범위)다.

## 적대적 평가 — 라운드 2/2

- 2026-10-14 · `/codex:adversarial-review --base d58b291` · 세션 codex-review(라운드 1 과 같은 세션) · 실행 HEAD `11ee5d5`(요청과 일치) · Verdict
  needs-attention(high 1). 라운드 상한 — 이 반영 뒤 재리뷰하지 않는다.

### 지적 원문

- [high] 자동 시작 전환이 crash-consistent하지 않다 (crates/app/src/notify.rs:300-317) — OS 등록/삭제를 먼저 수행한 뒤 SQLite를 커밋한다. 두 작업
  사이에 프로세스 강제 종료·패닉·전원 손실이 발생하면 보상 코드가 실행되지 않는다. 켜기 중단은 OS=true/DB=false가 되어 다음 로그인에 한 번 실행된
  뒤 스스로 등록을 지우고, 끄기 중단은 OS=false/DB=true가 되어 다음 로그인 자체가 없어 시작 시 동기화도 실행되지 않는다. 결과적으로 UI의 저장값과
  실제 자동 시작이 장기간 어긋난다. (권고: durable transition 상태 · 외부 pending marker · 단계별 강제 종료 테스트)

### 대조 · 처리

| 지적 | 판단 | 처리 |
|---|---|---|
| high · 중단 때 어긋남 | 사실(끄기 쪽) — 켜기 중단의 "OS 켜짐 · 저장 꺼짐" 은 다음 로그인 시작 맞추기가 저장 값(꺼짐)으로 수렴시키지만, 끄기 중단의 "저장 켜짐 · OS 꺼짐" 은 실행 기회가 없어 남는다. 강제 종료 · 패닉은 감시 프로세스가 곧 다시 띄워 시작 맞추기가 고치지만 전원 손실 · 두 프로세스 모두 종료는 남는다 | pending marker 대신 **순서**로 닫았다 — 켜기 = OS 등록 → 저장, 끄기 = 저장 → OS 지우기. 어느 지점에서 중단돼도 남는 꼴은 "OS 켜짐 · 저장 꺼짐" 뿐(다음 로그인에 실행 → 시작 맞추기가 지움). 끄기 훅 실패는 저장을 이전 값으로 되돌리고 500. 시험 `autostart` `toggle_order_never_leaves_saved_on_os_off`(훅이 부를 때의 저장 값으로 중단 지점의 상태를 단정 — 고치기 전 코드에서 FAILED 확인) · `save_failure_restores_registration` 갱신 — plan 개정 2 · E6 8건 |

- 권고 중 채택하지 않은 것: 외부 pending marker(순서만으로 위험한 꼴이 생기지 않아 둘째 저장소가 필요 없다) · 실제 프로세스 강제 종료 통합 시험
  (중단 지점의 상태는 훅 시점의 저장 값으로 결정되므로 순서 시험이 같은 것을 잰다) · 불일치 웹UI 노출(남는 꼴이 다음 시작에 저절로 고쳐진다).

## 미조치 지적

없음
