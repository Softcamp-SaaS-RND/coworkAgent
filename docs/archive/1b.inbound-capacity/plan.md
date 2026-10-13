# plan — 1b.inbound-capacity

> 근거: [intent](intent.md)(승인 2026-10-13 — 라운드 1). **구현자는 이 문서만 보고 일한다.** D·S 는 [masterPlan](../../masterPlan.md).
> 기준 커밋: `b874176`(main — Merge Task #1a, `task/1b-inbound-capacity` 분기점). plan 승인: 2026-10-13 — 위임(사용자 원문 "권장방안 승인 개발
> 진행해줘" · masterPlan "자동 진행 위임").

## 목표

받는 PC 의 `claude -p` 동시 실행을 상한(기본 2)으로 묶고 넘는 요청은 거절 없이 도착 순서로 대기시킨다. 보낸 쪽에는 "처리 중인 작업이 많아 응답을
기다립니다(앞에 k건)" → "처리 중" 을 보이고(웹UI · CLI), 받는 쪽은 내 PC 의 실행 중 · 대기 목록과 상한 설정을 본다. 피어 포트 47100 을 못 열면 잡은
프로세스를 알리고, 임의 포트 · 바뀐 IP 를 페어링된 상대가 승인 초기화 없이 따라오게 하며, 포트 상태를 설정 화면에 보이고 종료 뒤 포트가 풀리는지
잰다.

### 성공 기준(intent 사본)

- A1: 동시 실행이 상한을 넘지 않는다 — 상한 2 에서 서로 다른 대화 5건이 동시에 오면 실행 2 · 대기 3, 끝나는 대로 도착 순서로 실행된다.
- A2: 요청이 몰려도 거절되지 않는다 — 상한 2 에서 서로 다른 대화 30건이 오면 30건 모두 대기열에 들어가 도착 순서로 실행되고 모두 답한다.
- A3: 대기에 들어간 요청을 보낸 쪽 글 아래 "처리 중인 작업이 많아 응답을 기다립니다(앞에 k건)" 이 보이고, 실행이 시작되면 "처리 중" 으로 바뀐다.
- A4: CLI `ask` 가 대기 · 처리 중 상태를 출력하고(`--json` 포함), 대기 시간 때문에 실행 전에 포기하지 않는다.
- A5: 받는 쪽 화면에 내 PC 의 실행 중 · 대기 건수와 목록이 보인다. 동시 실행 상한은 설정에서 1 ~ 8 로 바꾸고 이 PC 에 저장된다.
- A6: 47100 을 다른 프로세스가 잡고 있으면 시작 시 그 프로세스 이름 · PID 가 트레이 · 웹UI 에 보인다.
- A7: 임의 포트로 연 경우 페어링된 상대가 승인 초기화 없이 새 주소로 이 PC 에 닿는다(새 주소 통지는 키가 맞는 상대만 반영).
- A8: 정상 종료 · 강제 종료 · `claude` 실행 중 강제 종료 뒤 5초 안에 웹UI · 피어 포트가 모두 풀리고(다시 실행하면 47100 을 연다), 남은 `instance.json` 이 다음
  실행을 막지 않는다.
- A9: 설정 화면에 웹UI 포트 · 피어 포트(고정 / 임의) · 열린 연결 수가 보인다.
- A10: 기존 테스트 · 웹UI 테스트 green, 경고 0.

### intent 결정(사본)

거절 없음 — 몰리면 대기열 · 도착 순서(해석: 동시 실행 수는 상한, 완전 직렬은 상한 1) · 1a 와 나눈 1b · 상한 기본 2 · 1 ~ 8 · 이 PC 저장(D24 개정) · CLI 는
"대기 중" 동안 30분을 세지 않음 · 피어 포트 설정 없음 · 새 주소 통지는 포트 · IP 모두 · 1b 뒤 "처리경과 콜백" Task. 제외: 우선순위 · 공정 분배 · 대기열
재시작 보존 · 바쁨 거절 · 슬롯 고갈 DoS.

## intent 대비 해석 (plan 승인 대상)

1. **실행 문(gate)은 한 곳** — `an_agent::gate::Gate`(FIFO): `acquire(msg_id) -> Ticket`(자리가 있고 기다리는 줄이 비면 바로, 아니면 줄 끝 · 앞 사람 수
   `ahead`), 실행이 끝나면(티켓 drop) 다음 사람에게. `set_cap(n)` 은 늘면 그 자리에서 줄을 당긴다. 대화 안 직렬(D19)은 지금처럼 대화 작업이 지키고,
   문은 **대화 사이**를 묶는다. 사람 요청 · 자동 왕복 · 이 PC 사람이 자기 에이전트에 한 요청 모두 같은 문(같은 Max 구독). "도착 순서" = 문에 닿은 순서
   (대화 작업이 꺼낸 순서) — 같은 대화의 다음 글은 앞 글이 끝나야 문에 닿는다.
2. **보낸 쪽 진행 알림 = 새 프레임 `Progress { conv, msg, state, ahead }`**(받는 쪽 → 보낸 쪽, 페어링된 상대만 · id 는 UUID 꼴 — 1a `ids_ok`).
   `state` = `queued`(문에서 기다림 · `ahead` = 앞 사람 수) | `running`(실행 시작, `ahead` 0). 상대가 보낸 실행 요청(받은 대화 `in` · 상대 글)만 보낸다 —
   바로 실행되면 `running` 하나, 기다리면 `queued` 뒤 `running`. 보내기 실패는 로그만(답은 따로 온다). 앞 사람 수가 줄 때마다 다시 보내지 않는다.
3. **보낸 쪽은 진행을 메모리에** — `Chat` 의 `progress: msg_id → (state, ahead)`. 받는 조건: 그 상대 소유 대화(`find(conv, peer)`) · 내 글 · 실행 요청 ·
   상태 `sent`(아니면 `unknown`). 답이 오면 지운다. 앱을 다시 켜면 사라진다(진행 표시일 뿐 — 스키마 그대로). 메시지 JSON 에 `progress: {state, ahead}`
   (없으면 null).
4. **웹UI 문구** — `view.js` `progressText(m)`: 보낸 상태(`sent`)일 때만 `queued` → "상대 PC 에 처리 중인 작업이 많아 응답을 기다립니다(앞에 {k}건)" ·
   `running` → "처리 중". 메시지 메타 줄(상태 자리)에.
5. **CLI `ask`** — 기다리는 동안 진행이 바뀔 때마다 표준 오류에 한 줄: 평문 "대기 중 — 상대 PC 에 처리 중인 작업이 많다(앞에 {k}건)" · "처리 중",
   `--json` 이면 JSON 한 줄 `{"progress":"queued","ahead":k}` · `{"progress":"running"}`(표준 출력은 끝 결과 JSON 하나 그대로). 기다림 상한(30분)은
   진행이 `queued` 인 동안 멈춘다(그 밖 — 진행 모름 · `running` — 은 센다).
6. **받는 쪽 화면 · 설정 = `GET /api/runs`** `{cap, running: [..], waiting: [..]}`(항목: `conversation_id` · `message_id` · `peer_id` · 상대 이름 · 에이전트 ·
   앞부분 40자) · **`PUT /api/runs {cap}`**(1 ~ 8 밖 400, 설정 키 `max_runs` 에 저장 · 문에 바로). 실행 상세 패널 맨 위 "이 PC 실행 — 실행 중 N · 대기 M"
   과 목록(항목을 누르면 그 대화), 설정 → 알림 · 실행 칸에 "동시 실행 상한"(1 ~ 8). 화면은 지금처럼 5초 새로 고침.
7. **피어 포트를 잡은 프로세스(A6)** — `node::bind_peer_port_on(port) -> (listener, Option<Holder>)`: 그 포트를 못 열면 임의 포트로 열고, 잡은 쪽을
   찾는다 — Windows `netstat -ano -p TCP` 의 `LISTENING` 줄 PID → `tasklist /FI "PID eq n" /FO CSV /NH` 이름 / Unix `lsof -nP -iTCP:{port} -sTCP:LISTEN -Fpc`.
   못 찾으면 `None`(사유 "알 수 없음"). 운영은 `bind_peer_port()` = 47100. 트레이 툴팁 기본 문구 · 웹UI 내 정보에 "피어 포트 47100 사용 중 — {이름}(PID
   {n}) · 임의 포트 {p} 로 받는다".
8. **새 주소 통지(A7) = `Ping { port }`** — 보내는 쪽이 자기 수신 포트를 싣고, 받는 쪽은 페어링된 키(TLS 로 인증)면 (연결 출처 IP, `port`) 가 저장된 주소와
   다를 때 `peers::set_addr` 로 바꾼다(승인 · 세대 그대로). 시작 직후 `watch_presence` 가 모든 상대에 Ping 하므로 포트 · IP 가 바뀌어도 첫 주기에 퍼진다.
   페어링 안 된 키 · Hello 경로는 지금 그대로. D32 — 같은 빌드.
9. **열린 연결 수(A9)** — 피어 수신 중인 연결 수(`Node::open_connections()` = 전체 자리 32 중 쓰는 수). 웹UI 내 정보: "웹UI 포트 {n}" · "피어 포트 {p}
   (고정 47100 | 임의 — 47100 사용 중)" · "열린 피어 연결 {k}". `/api/me` 에 `peer_port_fixed` · `peer_port_holder` · `ui_port` · `open_connections`.
10. **포트 풀림(A8)** — (a) 자식에 소켓이 물려가지 않는지: 수신 소켓을 연 채 `run_process` 로 자식을 띄우고 소켓을 닫으면 자식이 살아 있는 동안에도 같은
    포트를 다시 연다(테스트). (b) 실물: `tools/port_check.py` — 실행 중 앱을 강제 종료하고 웹UI · 피어 포트가 5초 안에 풀리는지 · 남은 `instance.json`
    인 채로 다시 띄워 47100 을 새 프로세스가 여는지. `--occupy` 는 47100 을 먼저 잡고 띄워 `/api/me` 의 잡은 프로세스를 읽는다. 정상 종료(트레이 "종료")
    는 사람 손이라 19 TC 로.

## 설계

### `an-agent`
- `gate.rs` — `Gate { state: Mutex<{cap, running: Vec<i64>, waiting: VecDeque<(i64, oneshot::Sender<()>)>}> }` · `acquire(msg) -> (Ticket, Option<ahead>)`
  (기다리면 `Some(앞 사람 수)` 를 먼저 돌려줘 알림을 보낸 뒤 `Ticket::ready().await`) · `set_cap` · `snapshot() -> (cap, running, waiting)`.
  `Ticket` drop = 놓기(실행 실패 · panic 에도). 단위 테스트(같은 파일).
- `chat.rs` — `Chat.gate` · `Chat.progress` · `process` 가 실행 직전 문을 지나며 상대 요청이면 `Progress` 를 보냄 · `inbound_progress` · `Reply` 받으면
  진행 지움 · `runs()` · `set_max_runs` · 시작 때 설정 `max_runs`(기본 2) 읽기. 상수 `MAX_RUNS_KEY` · `DEFAULT_MAX_RUNS` 2 · `MAX_RUNS` 8.

### `an-net`
- `proto.rs` — `Request::Progress` · `Request::Ping { port }` · `ids_ok` 에 `Progress`.
- `node.rs` — `bind_peer_port_on(port)` · `Holder { name, pid }` · `port_holder(port)` · `Node.slots`(열린 연결 수) · Ping 받을 때 주소 갱신 · `ping` 이 내
  포트를 싣는다 · `Node::peer_port_note()`(시작 때 정한 문구).

### `an-core`
- `peers::set_addr(id, addr)`.

### `an-app`
- `chat_api.rs` — `GET/PUT /api/runs` · 메시지 JSON `progress`. `peers_api.rs` `/api/me` 포트 필드. `cli.rs` 진행 출력 · 기다림 멈춤. `main.rs` — 피어 포트
  잡은 쪽 · 트레이 툴팁 기본 문구 · `/api/me` 에 넘김. `tray.rs` — 기본 툴팁 문구를 인자로.
- 웹UI — `view.js` `progressText` · `runsSummary`, `app.js` 메타 줄 · 실행 상세 "이 PC 실행" · 설정 "동시 실행 상한" · 내 정보 포트 줄, `index.html` · `app.css`.

### 도구
- `tools/port_check.py` — 강제 종료 · 포트 풀림 · 다시 띄우기 · `--occupy`.

## Stage

| stage | 목표 | 산출 | gate |
|---|---|---|---|
| 1 | 실행 문 · 진행 프레임 · 보낸 쪽 진행 · 실행 목록 · 상한 설정 · CLI | 테스트(gate 단위 · capacity · runs_api · cli · view) | E1 ~ E6 |
| 2 | 포트 — 잡은 프로세스 · Ping 주소 갱신 · 열린 연결 수 · 내 정보 · 툴팁 · 자식 소켓 | 테스트(ports · port_release) | E1 · E2 · E7 · E8 |
| 3 | 실측(포트 풀림 · 잡은 프로세스 표시 · 실화면) · 매뉴얼 | `tools/port_check.py` · 매뉴얼 | E9 ~ E11 |

## 평가 항목

- [ ] E1: A10 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E2: A10 — 우리 코드 경고 0(clippy -D warnings)
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: pending
- [ ] E3: A3·A5·A10 — 화면 로직 node 테스트 37건 이상 · 실패 0(기존 35 + progressText · runsSummary)
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# pass (3[7-9]|[4-9][0-9])\s+# fail 0/
  EVIDENCE: pending
- [ ] E4: A1·A2 — 실행 문 단위 테스트 5건(상한 · FIFO · 앞 사람 수 · 상한 늘리면 바로 당김 · 티켓 drop 이 실패에도 놓음)
  CHECK: cargo test -p an-agent --lib gate
  EXPECT: test result: ok. 5 passed; 0 failed
  EVIDENCE: pending
- [ ] E5: A1·A2·A3 — 두 PC 통합 6건(상한 2 · 대화 5건 동시 실행 최대 2 · 도착 순서 / 30건 모두 답 · 거절 0 / 보낸 쪽 진행 queued(앞 사람 수) → running → 답 오면 지움 / 바로 실행이면 running 만 / 남의 대화 · 내 글 아님 진행은 unknown / 같은 대화 후속 요청도 queued · 실행 목록 — 개정 1)
  CHECK: cargo test -p an-agent --test capacity
  EXPECT: test result: ok. 6 passed; 0 failed
  EVIDENCE: pending
- [ ] E6: A3·A4·A5 — 로컬 API · CLI 5건(`/api/runs` 목록 · `PUT` 1 ~ 8 · 저장 · 메시지 JSON `progress` · CLI 평문 · `--json` 진행 줄 · 대기 중 기다림 멈춤)
  CHECK: cargo test -p coworkagent-app --test capacity_api
  EXPECT: test result: ok. 5 passed; 0 failed
  EVIDENCE: pending
- [ ] E7: A6·A7·A9 — 포트 4건(잡힌 포트 → 임의 포트 + 잡은 프로세스 이름 · PID / 포트를 바꿔 다시 뜬 상대가 Ping 으로 승인 그대로 새 주소 / 다른 출처 IP 의 같은 키 Ping 이 IP 를 바꿈 · 페어링 안 된 키는 못 바꿈 / 열린 연결 수)
  CHECK: cargo test -p an-net --test ports
  EXPECT: test result: ok. 4 passed; 0 failed
  EVIDENCE: pending
- [ ] E8: A8 — 자식(`run_process`)이 살아 있어도 닫은 수신 소켓의 포트를 바로 다시 연다
  CHECK: cargo test -p an-agent --test port_release
  EXPECT: test result: ok. 1 passed; 0 failed
  EVIDENCE: pending
- [ ] E9: A8 — 실물: 강제 종료 뒤 5초 안에 두 포트 풀림 · 남은 instance.json 인 채로 다시 띄워 47100 을 새 프로세스가 연다
  CHECK: python tools/port_check.py
  EXPECT: freed=1 relisten=1 instance_left=1
  EVIDENCE: pending
- [ ] E10: A6 — 실물: 47100 을 잡아 둔 채 띄우면 `/api/me` 에 잡은 프로세스(python) · 임의 포트, 끝나면 정상으로 다시 띄움
  CHECK: python tools/port_check.py --occupy
  EXPECT: fixed=0 holder=python
  EVIDENCE: pending
- [ ] E11: A3·A5·A6·A9 — 실화면(헤드리스 Edge · 실행 중 앱): 메타 줄 진행 문구 · 실행 상세 "이 PC 실행" · 설정 동시 실행 상한 · 내 정보 포트 줄 · 트레이 툴팁(사람 확인은 19 TC)
  EVIDENCE: pending

## 종결 시 할 일

- masterPlan D24 개정(PC 전체 동시 실행 상한 기본 2 · 1 ~ 8 · 거절 없는 대기열 · 진행 알림) · D7 보강(포트 · IP 가 바뀌면 Ping 으로 따라옴) · D32 적용 기록
  (`Progress` · `Ping{port}`).
- PRODUCT 도메인 규칙 "실행 한도" · "피어 주소", 매뉴얼 `an_agent` · `an_net` · `an_app` · user-guide(대기 안내 · 실행 목록 · 상한 · 포트) · developer-guide.
- 19 acceptance TC(1a 몫과 함께) — 실화면 · 두 PC 대기 안내 · 트레이 툴팁 · 정상 종료 뒤 포트.

## 개정 기록

- 개정 1(2026-10-13, 적대적 평가 라운드 1 high — 위임): 같은 대화의 앞 실행을 기다리는 후속 실행 요청도 닿자마자 `queued`(앞 사람 수 = 같은 대화의 앞 요청 + 문 줄)를 알리고 `GET /api/runs` 기다림에 넣는다. E5 5건 → 6건.
