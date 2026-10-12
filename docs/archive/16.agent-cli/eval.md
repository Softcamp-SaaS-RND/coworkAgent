# eval.md — 16.agent-cli 적대적 평가

## 평가 원장

<!-- tools/check_gates.py 산출 — 손으로 고치지 않는다. 단 CHECK 가 없는 수동 gate 의
     EVIDENCE 와 ABANDON 줄은 사람이 적고, 재실행해도 보존된다.
     WEAK 줄은 기계가 매 실행 다시 단다 — 약한 gate 를 셀 수 있게 하는 표시이며
     판정에는 영향이 없다. -->

측정 커밋: 61f548b4fd751c03a92e67bdf177eccf5c287896

- [x] E1: 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

- [x] E2: 우리 코드 경고 0(clippy -D warnings)
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.23s

- [x] E3: A5 — 화면 로직 node 테스트(CLI 라벨: 내 글 · 상대 글 · 타임라인)
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# fail 0/
  EVIDENCE: # fail 0

- [x] E4: A5 — 스키마 8 이전(paths 9 → 10건: 7 → 8 이전 · via 열)
  CHECK: cargo test -p an-core --test paths
  EXPECT: test result: ok. 10 passed; 0 failed
  EVIDENCE: test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s

- [x] E5: A5 — 양쪽 이력 via(cli_origin 3건: ask_via_cli_marks_both_sides · say_via_cli_marks_both_sides · payload_without_via_is_human)
  CHECK: cargo test -p an-agent --test cli_origin
  EXPECT: test result: ok. 3 passed; 0 failed
  EVIDENCE: test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s

- [x] E6: A1·A2·A3·A4·A7·A8 — CLI(cli 16건: peers_lists_paired_with_json · peer_name_ambiguous_errors · ask_new_conversation_waits_for_reply · ask_conv_continues_same_conversation · ask_no_wait_prints_conversation_id · ask_agent_ambiguous_errors · ask_remote_failure_exit_code · ask_timeout_exit_code · say_defaults_to_latest_conversation · say_without_conversation_errors · history_lists_and_shows_timeline · not_running_exits_with_code · bad_token_rejected · unknown_command_exe_exits_usage · old_primary_refused_before_send · post_lost_response_is_not_not_running)
  CHECK: cargo test -p coworkagent-app --test cli
  EXPECT: test result: ok. 16 passed; 0 failed
  EVIDENCE: test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.22s

- [x] E7: A6 — 릴리스 빌드(콘솔 없는 서브시스템)를 파이프로 부르면 출력과 종료 코드가 온다(인스턴스 유무와 무관 — 실행 중이면 rc 0, 이전 버전이 실행 중이면 rc 4, 아니면 rc 3 — 0 이 아니면 stderr 사유)
  CHECK: cargo build --release -p coworkagent-app --target-dir target/gate && python -c "import subprocess,os;r=subprocess.run([os.path.abspath('target/gate/release/coworkAgent.exe'),'peers','--json'],capture_output=True);print(f'rc={r.returncode} piped={bool(r.stdout.strip()) if r.returncode==0 else bool(r.stderr.strip())}')"
  EXPECT: /rc=(0|3|4) piped=True/
  EVIDENCE: rc=4 piped=True

- [ ] E8: A2·A3·A5 — 실PC 두 대 · 실제 claude(수동): AI 에이전트가 `ask` · `say` 를 파이프로 부르고 양쪽 대화 화면 · 타임라인에 CLI 라벨 확인
  EVIDENCE: pending

ABANDON: E8 실PC 두 대 · 실제 claude 확인은 원격 세션이라 불가 — 프로토타입(Phase 1 끝) 인수 Task 19.prototype-acceptance 에 TC 를 더해 수동 테스트 일괄(masterPlan D28)

## 적대적 평가 — 라운드 1/2

- 수단: codex-review 세션 `/codex:adversarial-review --base 385ea3e --scope branch` · 실행 HEAD `61092d9`(요청 `f72f5cb` 와 차이는 STATE.md 만) · 2026-10-12
- Verdict: needs-attention

### 원문

```text
- [high] 모르는 명령이 종료 코드 2 대신 트레이 앱을 실행한다 (crates/app/src/main.rs:23-30)
  `main`은 첫 인자가 알려진 4개 명령일 때만 CLI로 분기합니다. 따라서 `coworkAgent peerz` 같은 오타는 `Cli::parse`의 "모르는 명령" 분기에 도달하지 않고 `Tray::new()`와 기존 인스턴스 handoff로 진행합니다. 실행 중이면 웹UI를 열고 성공 종료할 수 있고, 미실행이면 장기 실행 트레이가 떠 파이프 호출자가 멈출 수 있습니다. plan의 "모르는 명령 → 2" 계약을 위반합니다. `bad_token_rejected` 테스트도 주석과 달리 unknown 명령이 아니라 인자 없는 `ask`만 검사해 이 결함을 놓칩니다.
  Recommendation: 인자가 하나라도 있으면 CLI parser로 보내 unknown 명령을 `EXIT_USAGE`로 거절하거나, 지원할 비-CLI activation 인자만 명시적으로 허용하십시오. 실제 executable을 `peerz`로 실행해 주 인스턴스 유무 모두 종료 코드 2와 stderr를 단정하는 테스트를 추가하십시오.
- [high] 실행 중인 이전 버전 primary가 `via`를 조용히 버려도 CLI는 성공한다 (crates/app/src/cli.rs:357-383)
  `Api::open`은 `instance.json`의 port/token만 신뢰하며 primary의 schema 또는 CLI-origin capability를 확인하지 않습니다. 기준 커밋의 API는 unknown JSON 필드 `via`를 무시하고 기존 `Chat::send`/`post`를 호출하므로, 새 CLI가 이전 버전 tray에 연결하면 요청은 2xx로 성공하지만 양쪽 이력에는 사람 글로 저장됩니다. 이는 A5와 'CLI 호출은 전부 이력에서 구분'이라는 수용 위험의 핵심 보완책을 조용히 무력화합니다. 또한 `Api::call`은 임의의 2xx JSON만 통과시키므로 stale port의 다른 HTTP 서비스도 coworkAgent로 오인할 수 있습니다.
  Recommendation: CLI 실행 전 인증된 capability/schema handshake를 수행해 schema 8 및 `via=cli` 지원을 확인하고, 미지원 primary는 명시적 비정상 종료로 거절하십시오. endpoint별 typed response 검증도 적용하고, 이전 버전 primary 및 valid-HTTP stale-port를 대상으로 혼합 버전 테스트를 추가하십시오.

Next steps:
- unknown 명령 executable 회귀 테스트와 분기 수정
- primary capability handshake 및 typed API 응답 검증
- 혼합 버전·stale valid-HTTP 시나리오를 E6/E7에 추가 후 재측정
```

### 대조 · 조치

| 지적 | 판단 | 조치 |
|---|---|---|
| [high] 모르는 명령이 트레이를 띄운다(`main` 이 4개 명령일 때만 CLI) | 사실 — `coworkAgent peerz` 는 `Tray::new()` 로 간다. plan 종료 코드 표 "모르는 명령 → 2" 위반 | 인자가 있으면 전부 CLI(`main.rs`). 실제 실행 파일로 `peerz` · `--json` · `chat --peer x` → 2 + 사용법(`unknown_command_exe_exits_usage`). 수정 전 실행 파일 mutation 은 트레이를 띄워(사용자 PC 의 실행 중 인스턴스에 웹UI 열기) 돌리지 않았다 |
| [high] 이전 버전 주 인스턴스가 `via` 를 버려도 성공 · 다른 HTTP 서비스 오인 | 사실 — 이전 API 는 모르는 필드 `via` 를 무시한다(Windows 는 실행 중 exe 를 덮지 못해 흔하진 않지만 다른 경로 설치 시 가능) | `/api/status` 에 `cli_api: 1`, CLI 는 보내기 전 확인 — 아니면 4 · 아무것도 보내지 않음(`old_primary_refused_before_send`, 가짜 이전 버전 서버가 받은 요청 = `GET /api/status` 1건). handshake 제거 mutation → FAIL. endpoint 별 typed 검증은 handshake 로 갈음 |

plan 개정 3(E6 13 → 15 · E7 rc 4 허용 — 2026-10-12 사용자 승인 "plan 개정 승인"). 재측정 E7 = `rc=4 piped=True` — 개발 PC 에 떠 있는 트레이 앱이 이전 빌드(`cli_api` 없음)라 handshake 가 거절한 것으로, 수정이 의도대로 동작한 결과다.

## 적대적 평가 — 라운드 2/2

- 수단: codex-review 세션(라운드 1 과 같은 세션) `/codex:adversarial-review --base 385ea3e --scope branch` · 실행 HEAD `c3beb8b` · 2026-10-12
- Verdict: needs-attention

### 원문

```text
- [high] POST 처리 후 응답 유실을 미실행으로 오인해 질문을 중복 실행할 수 있음 (crates/app/src/cli.rs:384-395)
  `local_request`가 요청을 전부 쓴 뒤 응답을 읽는 중 `ConnectionReset`/`ConnectionAborted`를 받아도 `Api::call`은 이를 `EXIT_NOT_RUNNING`으로 분류합니다. 따라서 primary가 질문을 저장·원격 전송한 직후 종료되면 CLI에는 대화 ID나 전달 불명 경고가 남지 않습니다. 호출자가 코드 3을 보고 재시도하면 새 UUID로 같은 원격 에이전트를 다시 실행합니다. 이는 ACK 유실 시 중복 실행을 피한다는 대화 전송 불변식과 충돌합니다.
  Recommendation: POST의 write 완료 후 발생한 read 오류를 명시적인 '전달 여부 불명' 결과로 구분하고 재시도 금지·history 확인을 안내하십시오. 가능하면 CLI 생성 idempotency key를 API에 전달해 재호출 시 기존 conversation/message 결과를 반환하도록 하십시오.
- [medium] E7이 성공 경로를 실행하지 않아 A6 회귀를 통과시킴 (docs/tasks/16.agent-cli/plan.md:152-154)
  E7은 `rc=(0|3|4)`와 선택된 스트림의 비어 있지 않음만 검사합니다. 현재 증거도 `rc=4 piped=True`라서 consoleless release의 정상 `peers --json` stdout 경로는 전혀 실행되지 않았습니다. 성공 stdout이 비어 있거나 정상 실행이 항상 코드 4로 실패해도 gate가 통과합니다. 개정 3은 이 완화를 사용자 미승인으로 기록하고 있습니다.
  Recommendation: 격리된 데이터 경로에서 동일 release primary를 띄우고 release CLI가 `rc=0`, 유효한 JSON stdout, 빈 stderr를 내는 검사를 E7의 필수 경로로 추가하십시오. 이전 primary의 `rc=4` 검사는 별도 호환성 gate로 유지하십시오.

Next steps:
- Q1: POST 전달 불명 상태에 idempotency key를 추가할까요?
- Q2: E7을 동일 release primary의 rc=0 성공 경로로 강화할까요?
- Q3: 수정 후 plan 개정 3 승인을 다시 받을까요?
```

### 대조 · 조치

| 지적 | 판단 | 조치 |
|---|---|---|
| [high] POST 를 보낸 뒤 응답 유실(`ConnectionReset` 등)을 "실행 중 아님"(3)으로 알려 재시도 → 중복 실행 | 사실 — `local_request` 가 읽기 단계 오류를 그대로 넘겨 연결 거부와 구분되지 않았다 | 읽기 단계 오류 · 빈 응답은 `UnexpectedEof`, CLI 는 POST 면 4 + "전달 여부를 모른다 — 다시 보내지 말고 history 로 확인"(`post_lost_response_is_not_not_running`, 가짜 앱이 POST 를 받고 응답 없이 닫음). 분기 무력화 mutation → FAIL. idempotency key 는 미조치(아래) |
| [medium] E7 이 성공 경로(rc 0 · JSON stdout)를 실행하지 않는다 — 개정 3 ③ 의 rc 4 허용이 완화 | 사실 — 지금 증거 `rc=4 piped=True` 는 거절 경로다 | 미조치(아래) — 19 수동 TC 로 이관 |

plan 개정 3 ④(E6 15 → 16 — 2026-10-12 사용자 승인). 라운드 상한 2 — 재리뷰하지 않고 종료.

## 미조치 지적

| 지적 | 판단 | 사유 |
|---|---|---|
| E8 수동 gate(실PC 두 대 · 실제 claude — `ask` · `say` · 양쪽 CLI 라벨) 미충족 | 사실 | 프로토타입 인수 Task 로 이관(D28). ABANDON 기록 · 19 TC-J |
| 라운드 2 [high] 권고 중 CLI idempotency key(재호출 시 같은 대화 · 글 돌려주기) | 사실(보강안) | 전달 불명을 4 + 재전송 금지 안내로 드러내는 것까지 반영. 키는 로컬 API · 피어 프로토콜 계약 변경이라 범위 밖 — 원격 실행은 D31 읽기 전용이라 중복의 피해는 사용량 |
| 라운드 2 [medium] E7 이 consoleless 릴리스의 성공 경로(rc 0 · JSON stdout · 빈 stderr)를 실행하지 않는다 | 사실 | 성공 경로는 측정 `24c52b6`(handshake 전)에서 실행 중 트레이 앱 상대로 `rc=0 piped=True` 를 얻었으나 지금 gate 증거는 거절 경로(rc 4). 격리 데이터 경로로 릴리스 주 인스턴스를 띄우는 gate 는 트레이 · 브라우저 열기가 따라와 CHECK 로 두기 어렵다. 개정 3 ③(rc 4 허용)은 사용자 승인 — 성공 경로(새 빌드 트레이 + 릴리스 CLI 파이프 → rc 0 · JSON stdout · 빈 stderr)는 19 TC-J 로 이관 |
