# eval.md — 1b.inbound-capacity 적대적 평가

## 평가 원장

<!-- tools/check_gates.py 산출 — 손으로 고치지 않는다. 단 CHECK 가 없는 수동 gate 의
     EVIDENCE 와 ABANDON 줄은 사람이 적고, 재실행해도 보존된다.
     WEAK 줄은 기계가 매 실행 다시 단다 — 약한 gate 를 셀 수 있게 하는 표시이며
     판정에는 영향이 없다. -->

측정 커밋: 176f6771ec821146b3a77339177ecaca4d32fea7

- [x] E1: A10 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

- [x] E2: A10 — 우리 코드 경고 0(clippy -D warnings)
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.23s

- [x] E3: A3·A5·A10 — 화면 로직 node 테스트 37건 이상 · 실패 0(기존 35 + progressText · runsSummary)
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# pass (3[7-9]|[4-9][0-9])\s+# fail 0/
  EVIDENCE: # skipped 0 / # todo 0 / # duration_ms 82.4444

- [x] E4: A1·A2 — 실행 문 단위 테스트 5건(상한 · FIFO · 앞 사람 수 · 상한 늘리면 바로 당김 · 티켓 drop 이 실패에도 놓음)
  CHECK: cargo test -p an-agent --lib gate
  EXPECT: test result: ok. 5 passed; 0 failed
  EVIDENCE: test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s

- [x] E5: A1·A2·A3 — 두 PC 통합 6건(상한 2 · 대화 5건 동시 실행 최대 2 · 도착 순서 / 30건 모두 답 · 거절 0 / 보낸 쪽 진행 queued(앞 사람 수) → running → 답 오면 지움 / 바로 실행이면 running 만 / 남의 대화 · 내 글 아님 진행은 unknown / 같은 대화 후속 요청도 queued · 실행 목록 — 개정 1)
  CHECK: cargo test -p an-agent --test capacity
  EXPECT: test result: ok. 6 passed; 0 failed
  EVIDENCE: test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.31s

- [x] E6: A3·A4·A5 — 로컬 API · CLI 5건(`/api/runs` 목록 · `PUT` 1 ~ 8 · 저장 · 메시지 JSON `progress` · CLI 평문 · `--json` 진행 줄 · 대기 중 기다림 멈춤)
  CHECK: cargo test -p coworkagent-app --test capacity_api
  EXPECT: test result: ok. 5 passed; 0 failed
  EVIDENCE: test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 12.25s

- [x] E7: A6·A7·A9 — 포트 4건(잡힌 포트 → 임의 포트 + 잡은 프로세스 이름 · PID / 포트를 바꿔 다시 뜬 상대가 Ping 으로 승인 그대로 새 주소 / 다른 출처 IP 의 같은 키 Ping 이 IP 를 바꿈 · 페어링 안 된 키는 못 바꿈 / 열린 연결 수)
  CHECK: cargo test -p an-net --test ports
  EXPECT: test result: ok. 4 passed; 0 failed
  EVIDENCE: test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.83s

- [x] E8: A8 — 자식(`run_process`)이 살아 있어도 닫은 수신 소켓의 포트를 바로 다시 연다
  CHECK: cargo test -p an-agent --test port_release
  EXPECT: test result: ok. 1 passed; 0 failed
  EVIDENCE: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.12s

- [x] E9: A8 — 실물: 강제 종료 뒤 5초 안에 두 포트 풀림 · 남은 instance.json 인 채로 다시 띄워 47100 을 새 프로세스가 연다
  CHECK: python tools/port_check.py
  EXPECT: freed=1 relisten=1 instance_left=1
  EVIDENCE: freed=1 relisten=1 instance_left=1

- [x] E10: A6 — 실물: 47100 을 잡아 둔 채 띄우면 `/api/me` 에 잡은 프로세스(python) · 임의 포트, 끝나면 정상으로 다시 띄움
  CHECK: python tools/port_check.py --occupy
  EXPECT: fixed=0 holder=python
  EVIDENCE: fixed=0 holder=python

- [x] E11: A3·A5·A6·A9 — 실화면(헤드리스 Edge · 실행 중 앱): 메타 줄 진행 문구 · 실행 상세 "이 PC 실행" · 설정 동시 실행 상한 · 내 정보 포트 줄 · 트레이 툴팁(사람 확인은 19 TC)
  EVIDENCE: 2026-10-13 09:31 · 구현 세션(Claude) · `176f677` 릴리스 빌드(E9 · E10 뒤 실행 중 — 09:09 `3e94477` 에서 같은 결과) · 헤드리스 Edge(playwright-core msedge 채널) — 내 정보 웹UI 포트 = instance.json 포트 · 피어 포트 "47100(고정)" · 열린 피어 연결 "0" / 설정 동시 실행 상한 처음 "2" → 3 으로 바꾸면 `GET /api/runs` cap 3 → 2 로 되돌림 / 보낸 대화 글 목록 응답을 가로채 첫 내 글을 `sent` + `progress{queued,1}` 로 → 메타 줄 "상대 PC 에 처리 중인 작업이 많아 응답을 기다립니다(앞에 1건)" / 실행 상세 맨 위 "이 PC 실행 — 실행 중 0 · 대기 0" · "실행 · 대기 없음" / 콘솔 오류 0. 피어 포트 임의 표시는 E10(`/api/me` holder=python) · `view.test.mjs` portText, 트레이 툴팁은 헤드리스로 못 봄 — `tray::tip` 단위 시험 · 사람 확인 19 TC. 상세 [stage3](work/stage3.md)


WEAK: E11 CHECK 없음 — 수동 gate(사람이 적은 EVIDENCE 로 판정)

## 적대적 평가 — 라운드 1/2

- 2026-10-13 · `/codex:adversarial-review --base 35f1b8f` · 세션 codex-review · 실행 HEAD `0aa1182`(요청과 일치) · Verdict needs-attention(high 1 · medium 1).

### 지적 원문

- [high] 같은 대화의 후속 요청은 대기열·진행 상태에서 누락된다 (crates/agent/src/chat.rs:1476-1478) — 대화별 worker 는 `process` 가 끝나야 다음 메시지를
  꺼내고 gate 등록도 `process` 진입 때뿐이라, 실행 중인 요청 뒤 같은 대화 요청은 DB `queued` 로만 남아 `Gate` · `/api/runs` · `Progress::queued` 어디에도 없다.
  후속 CLI 는 `WAIT_LIMIT` 을 소진해 timeout 을 내지만 요청은 뒤에 실행 — 재전송이면 중복 실행.
- [medium] 진행 알림 실패가 영구 손실되고 다른 피어까지 지연시킨다 (crates/agent/src/chat.rs:323-335) — 모든 피어 알림이 unbounded channel 하나 · worker
  하나로 순차, 실패는 로그만(재시도 · 조회 · 확인 없음) — 일시 실패로 `queued` 가 유실돼 CLI 가 시간을 계속 세고, 닿지 않는 피어가 뒤 피어를 막는다.

### 대조 · 처리

| 지적 | 판단 | 처리 |
|---|---|---|
| high · 같은 대화 후속 요청 누락 | 사실(코드 순서 · 시험으로 RED 확인) | `Chat::tell_waiting` — 받은 실행 요청이 같은 대화의 앞 요청 뒤면 닿자마자 `queued`(앞 사람 수 = 같은 대화 앞 요청 + 문 줄). `run_queue` 기다림에 문에 닿지 않은 DB `queued` 실행 요청(id 순)을 문 줄 뒤로 붙임 → `/api/runs` 에도. 시험 `capacity` `same_conversation_followup_is_queued_and_listed`(고치기 전 FAILED 확인) · plan 개정 1(E5 6건). 중앙 scheduler 로 바꾸는 권고는 대화 직렬(D19) 구조 변경이라 하지 않음 |
| medium · 알림 유실 · 피어 간 막힘 | 막힘은 사실 · 유실은 사실(설계 — plan 해석 2 "보내기 실패는 로그만", 해석 3 "진행은 메모리") | **상대별 줄**로 나눔(`notices` = 상대 id → 줄) — 한 상대의 장애가 다른 상대 알림을 막지 않는다. 유실 · 재시도 · 지속 저장은 **미조치** — 명세(해석 2 · 3)가 정한 동작, CLI 시계가 도는 쪽으로만 틀린다(대기를 실행으로 착각하지 않음). 아래 미조치 |

## 적대적 평가 — 라운드 2/2

- 2026-10-13 · `/codex:adversarial-review --base 35f1b8f` · 세션 codex-review(라운드 1 과 같은 세션) · 실행 HEAD `a2f42d0`(요청과 일치) · Verdict
  needs-attention(medium 2). 상한 도달 — 반영 뒤 재리뷰하지 않는다.

### 지적 원문

- [medium] 진행 상태 변경이 메시지 UI를 다시 렌더링하지 않는다 (crates/app/web/app.js:1158-1159) — `messagesSig` 가 `progress.state` · `progress.ahead` 를
  포함하지 않아, 글 상태가 `sent` 그대로인 동안 `queued`→`running` 이 와도 DOM 교체를 건너뛴다(첫 조회가 알림보다 빠르면 문구가 아예 안 나타남).
- [medium] stale queued 상태가 CLI timeout을 영구 정지시킨다 (crates/app/src/cli.rs:540-580) — 마지막 조회가 `queued` 인 동안 `WAIT_LIMIT` 을 차감하지
  않으므로, 상대가 queued 통지 뒤 종료되거나 running/reply 가 유실되면 CLI 가 무기한 기다린다. 권고: 절대 wall-clock 상한 또는 진행 lease.

### 대조 · 처리

| 지적 | 판단 | 처리 |
|---|---|---|
| medium · 진행이 바뀌어도 다시 그리지 않음 | 사실(E11 은 첫 그리기만 봤다) | 서명을 `view.js` `messagesSigOf(peerName, ms)` 로 옮겨 `progressText` 를 넣음(app.js 는 위임). `view.test.mjs` `messagesSigOf` — 진행 없음 · queued 1 · queued 0 · running 이 모두 다른 서명, 저장은 서명 밖 |
| medium · stale queued 로 CLI 무기한 대기 | 사실 — 명세(plan 해석 5 "queued 동안 멈춤")가 정한 동작, Stage 1 리뷰 M 과 같은 지적 | **미조치 · 사용자 결정** — 아래 미조치 1행 |

## 미조치 지적

| 지적 | 판단 | 사유 |
|---|---|---|
| R2 · Stage 1 리뷰 M · `queued` 를 본 뒤 상대가 꺼지거나 running · 답이 유실되면 CLI `ask` 가 30분 시계가 멈춘 채 무기한 기다린다 | 사실 · 명세(해석 5) | **사용자 결정 필요** — 안: (a) 대기 중에도 함께 세는 절대 상한(예: 2시간) · (b) 상대가 오프라인(접속 확인 75초)이면 다시 센다 · (c) 지금대로. 그동안 `--no-wait` · Ctrl+C 로 빠질 수 있다 |
| R1 · 진행 알림 실패는 다시 보내지 않고 사라진다(재시도 · 지속 저장 · 확인 없음) | 사실 · 명세(해석 2 "실패는 로그만" · 해석 3 "메모리") | 피어 간 막힘은 상대별 줄로 고쳤다. 유실은 CLI 시계가 **도는** 쪽으로만 틀린다(대기를 놓쳐도 실행으로 착각하지 않음) — 재시도는 다음 Task "처리경과 콜백" 에서 진행 상태 모델과 함께 |
| Stage 2 리뷰 M · Ping 이 저장 주소를 연결 출처 IP 로 확인 없이 덮는다(다중 NIC · VPN · NAT) | 추정 · 명세(해석 8) | 갱신 대상은 TLS 인증 키의 자기 행뿐 · 30초마다 다시 맞춘다 · 사내망 단일 경로 전제 |
| Stage 1 리뷰 L · 상한이 모두 실행 중일 때 첫 대기자에 "앞에 0건" | 사실 · 명세 문구(해석 4 · E4) | 앞 사람 = 기다리는 줄 — 문구 보완은 19 인수 때 사용자 확인 |
| stage1 남은 위험 · 상한을 8 까지 올리면 같은 상대로 가는 답이 한순간에 겹쳐 받는 쪽 출처당 연결 상한 4 를 넘으면 `undelivered` | 사실 · 1b 이전부터(상한이 아예 없었다) | 답 전달 재시도는 이 Task 밖 — 기본 2 에서는 답 2 + 알림 1 ≤ 4 |
| Stage 2 리뷰 L · `port_holder` 가 시작 중 동기 · 시간 제한 없음 | 사실(멈춤 빈도 추정) | 47100 이 막힐 때만 부른다 · netstat 은 평소 1초 미만 |
| Stage 2 리뷰 L · tasklist 출력(OEM 코드페이지)을 UTF-8 로 읽어 한글 프로세스 이름이 깨질 수 있다 | 추정 | PID 는 정확, 표시 이름만 — 필요하면 `QueryFullProcessImageNameW` |
| Stage 2 리뷰 메모 · 같은 PC 프로세스가 `127.0.0.1:47100` 을 먼저 잡으면 우리 `0.0.0.0:47100` 바인드가 성공해 연결을 가로챌 수 있다(`SO_EXCLUSIVEADDRUSE` 없음) | 사실(리뷰어 실측) · 범위 밖(1a 경계) | 상대 키 고정이라 피해는 연결 방해 — 한 PC 한 사용자(D33) |
| Unix `port_holder`(`lsof`) · `port_release`(`sleep`) · `ports` 를 컴파일 · 실행하지 못함 | 확인 필요 | 이 PC 는 Windows 만 — Phase 4 macOS(D25) |
| 트레이 툴팁 · 실제 두 PC 대기 안내를 사람 눈으로 보지 않음 | 확인 필요 | 헤드리스 불가 — 19 인수 TC |
