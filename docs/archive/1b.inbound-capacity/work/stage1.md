# stage1 — 실행 문 · 진행 · 실행 목록 · CLI

> 근거: [plan](../plan.md) Stage 1. 사용자 검증은 D28 — 19.prototype-acceptance 에서 일괄.

## 목표

받는 PC 의 `claude -p` 동시 실행을 상한(기본 2)으로 묶고 넘는 요청은 거절 없이 닿은 순서로 기다리게 한다(A1 · A2). 보낸 쪽은 상대가 알려 준 차례를
글 메타 줄 · 실행 상세 · CLI 에 보이고(A3 · A4), 받는 쪽은 이 PC 실행 목록과 상한 설정을 본다(A5). 대상 gate: E1 ~ E6.

## 산출 파일

- `crates/agent/src/gate.rs`(새) — `Gate::new(cap)` · `enter(id) -> (Ticket, Option<앞 사람 수>)` · `Ticket::ready()` · drop = 놓고 다음 사람 ·
  `set_cap`(늘면 바로 당김, 1 미만은 1) · `snapshot()`. 단위 테스트 5. `lib.rs` 등록.
- `crates/agent/src/chat.rs` — `MAX_RUNS_KEY` · `DEFAULT_MAX_RUNS` 2 · `MAX_RUNS` 8 · `max_runs(conn)` · `Progress{state, ahead}` · `Chat.gate` ·
  `Chat.progress`(메모리) · `Chat.notices`(진행 알림 줄) · `run_queue` · `set_max_runs` · `progress_of` · `send_progress` · `inbound_progress` ·
  `process` 가 실행 직전 문을 지난다(상대 실행 요청이면 기다릴 때 `queued` → 차례가 오면 `running`, 바로면 `running` 만).
- `crates/net/src/proto.rs` — `Request::Progress{conv, msg, state, ahead}`(`ids_ok` 포함) · `Request::Ping{port}`(Stage 2 몫이지만 같은 프레임 파일).
  `crates/net/src/node.rs` — `Progress` 를 페어링된 상대의 대화 핸들러로. 같은 변경에 Stage 2 의 `bind_peer_port_on` · `Holder` · `port_holder` ·
  `slots` · `open_connections` · Ping 주소 갱신이 들어 있다(시험은 Stage 2 `ports`). `crates/core/src/peers.rs` `set_addr`.
- `crates/app/src/chat_api.rs` — `GET /api/runs` `{cap, running, waiting}`(항목 `message_id` · `conversation_id` · `peer_id` · `peer` · `agent` ·
  `text` 앞 40자) · `PUT /api/runs {cap}`(1 ~ 8 밖 400) · 대화 글 목록 JSON 에 `progress`.
- `crates/app/src/cli.rs` — `wait_reply` 가 진행이 바뀔 때마다 표준 오류에 한 줄(평문 · `--json`) · `queued` 동안 기다림 상한을 세지 않는다.
  `exec` · `ask` 에 `err` 를 넘긴다. `crates/app/src/main.rs` — `bind_peer_port()` 튜플(잡은 쪽은 Stage 2 에서 쓴다).
- 웹UI — `view.js` `progressText` · `runsSummary` · `runSteps` 가 진행을 본다 / `app.js` 메타 줄 · 실행 상세 "이 PC 실행"(열려 있을 때 5초마다) ·
  설정 "동시 실행 상한"(`PUT /api/runs`) / `index.html` 절 · 입력.
- 테스트 — `agent/tests/capacity.rs`(5) · `app/tests/capacity_api.rs`(5) · `view.test.mjs` 2 · `net/tests/pairing.rs` `Ping { port: 0 }`.

## 설계 결정

- **진행 알림은 한 줄(작업 하나)로 차례대로 보낸다** — plan 해석 2 는 "보내기 실패는 로그만" 이었고 처음엔 `process` 안에서 바로 보냈다. 30건
  시험에서 대기 알림 30개가 한꺼번에 상대에 붙어 받는 쪽 출처당 연결 상한(`MAX_CONNS_PER_IP` 4)을 채웠고, 그 사이 끝난 **답 전달이 거절되어
  `undelivered`** 로 남았다(3회 중 1회, 29/30). 알림을 `mpsc` 한 줄 + 보내는 작업 하나로 바꿔 동시에 붙는 알림을 1개로 묶었다 — 같은 글의
  `queued` → `running` 순서는 줄이 지키고, 실행은 알림을 기다리지 않는다. 바꾼 뒤 `capacity` 7회 · `thirty` 단독 3회 통과.
- **진행 지우기는 읽을 때** — plan 해석 3 "답이 오면 지운다" 를 `progress_of` 가 글이 `sent` 가 아니면 항목을 지우고 `None` 으로 맞췄다(답 받는
  경로 여러 곳을 건드리지 않음). 화면 · CLI 결과는 같다.
- **`progress` 는 대화 글 목록(`GET /api/conversations/{id}/messages`)에만** — 메타 줄 · 실행 상세 · CLI 가 모두 그 목록을 읽는다. 가로지른 글 ·
  검색 · 내 활동 JSON 에는 붙이지 않았다(그 화면은 상태 문구를 그리지 않음).
- **CLI "처리 중" 은 확인 간격(1초) 안에 끝나면 건너뛸 수 있다** — 진행은 확인할 때 본 값이다. 시험은 자기 실행도 3초 걸리게 해 두 줄을 본다.
- **기다림 멈춤 = 지난 확인 때 `queued` 였던 구간은 세지 않음** — 첫 확인 전 · 진행 모름 · `running` 은 센다(plan 해석 5).
- **상한 읽기 실패는 기본값 + warn** — `Chat::new` 가 시작을 막지 않는다. 범위 밖 저장값도 기본값.

## 테스트 결과

```text
cargo test -p an-agent --lib gate                    → test result: ok. 5 passed; 0 failed
cargo test -p an-agent --test capacity               → test result: ok. 5 passed; 0 failed
cargo test -p coworkagent-app --test capacity_api    → test result: ok. 5 passed; 0 failed
node --test --test-reporter=tap "crates/app/web/*.test.mjs" → # pass 37 / # fail 0
cargo test --workspace --no-fail-fast                → test result ok 51 줄 · FAILED 0
cargo clippy --workspace --all-targets -- -D warnings → Finished
```

- RED 확인: `capacity` `five_conversations…` 는 처음에 시작 순서 전체를 비교해 실패 — 바로 들어간 둘은 함께 시작해 순서가 섞인다(시험을 "앞 둘은
  집합 · 기다린 셋은 순서" 로 고침). `thirty…` 는 위 진행 알림 폭주로 29/30 실패 → 알림 줄로 고침. CLI 진행 시험은 첫 판에서 "처리 중" 을 못 봐
  실패 → 자기 실행도 느리게.

## 남은 위험(평가 때 확인)

- 상한을 8 까지 올리면 같은 상대로 가는 **답 8개가 한순간에** 겹칠 수 있다 — 받는 쪽 출처당 4 를 넘으면 답이 `undelivered`. 1b 이전에는 상한 자체가
  없어 더 넓었던 위험(새로 생긴 것 아님). 답 전달 재시도는 이 Task 밖.

## 단계 리뷰 (2026-10-13, 분리된 리뷰 에이전트 · diff `35f1b8f..d136d2f`)

High 0 · Medium 2 · Low 4. 티켓 반납(완료 · 실패 · panic · 대기 중 취소) · FIFO · 잠금 순서(db → progress) · `Progress` 위조(남의 대화 · 내 글 아님 ·
`sent` 아님) · 원격 글 `textContent` 는 결함 없음.

| 지적 | 판단 | 처리 |
|---|---|---|
| M · 대화 · 에이전트 · 자동 왕복 멈춤을 문 **앞**에서 읽는다 — 수 분 기다리는 사이 지운 에이전트 · 멈춘 왕복을 낡은 값으로 실행 | 사실(명세 밖 결함) | `Chat::admit` — `process` 첫 줄에서 문을 지나고 그 **뒤에** 읽는다. `capacity` `cap_bounds…` 에 "기다리는 동안 지운 에이전트는 실행하지 않음" 추가(고치기 전 RED 확인) |
| M · CLI 가 `queued` 를 본 뒤 상대가 꺼지면 시계가 멈춘 채 끝나지 않는다 | 사실 — 명세(해석 5 "queued 동안 멈춤")가 정한 동작 | **미조치 · 사용자 결정**(eval 에 올림). `--no-wait` · Ctrl+C 로 빠질 수 있다 |
| L~M · 진행 알림 한 줄 — 응답 없는 상대(10초씩)가 다른 상대 알림을 밀고 채널 상한 없음 | 추정(표시가 늦어지는 체감 문제 · 답은 따로 감) | 미조치 — 기록. 상대별 줄은 실측에서 늦으면 |
| L · `PUT /api/runs` `{"cap":-1}` · `"3"` · `2.5` 가 422 | 사실(명세 "1 ~ 8 밖 400") | `Json<Value>` 로 받아 정수 1 ~ 8 이 아니면 400. `capacity_api` `put_runs…` 에 6 가지 |
| L · 첫 대기자에 "앞에 0건" | 사실 — 명세 문구 · E4 와 일치 | 미조치(문구는 명세). eval 에 의견 |
| L · 진행이 `queued` → 없음이 되면 `shown` 이 남아 시계가 계속 멈춤 | 추정(지금은 도달 어려움) | 시계 판정을 `paused`(이번 확인의 진행)로 표시값과 분리 |
