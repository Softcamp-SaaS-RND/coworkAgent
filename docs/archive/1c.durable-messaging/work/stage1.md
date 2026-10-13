# stage1 — outbox · 상대 줄 · 상태 질의

> 근거: [plan](../plan.md) Stage 1. 사용자 검증은 D28 — 19.prototype-acceptance 에서 일괄.

## 목표

상대에게 가는 대화 프레임을 SQLite outbox 에 먼저 적고 상대별 줄이 ack 까지 다시 보낸다(A1 ~ A5 — D7 실현). 보낸 쪽이 자기 글 상태를 묻는
`Status` 프레임. 화면 "보내는 중 · 보내지 못함". 대상 gate: E1 ~ E5.

## 산출 파일

- `crates/core/src/db.rs` — 스키마 13: `outbox`(peer · conv · msg · kind · frame · key · state · tries · waited · next_at · last_error) · `cli_tx`(Stage 2 가 쓴다).
- `crates/core/src/outbox.rs`(새) — `add`(키 합침) · `head` · `has_before`(같은 대화 · 앞질러도 되는 종류 제외) · `failing` · `get` · `set_state` · `retry` ·
  `hurry` · `fail` · `remove` · `pending_peers` · `for_message` · `clear_failed`. `crates/core/src/cli_tx.rs`(새, Stage 2). `peers::touch`.
- `crates/net/src/proto.rs` — `Request::Status` · `Response::Status` · `Progress.detail`(`StepDetail` — Stage 4 가 채운다) · `ids_ok` 에 `Status`.
  `crates/net/src/node.rs` — `Status` 를 처리기로 · 보내기 성공 · 페어링된 상대 요청 · `Ping` 받기마다 `touch` · `on_seen`(페어링된 상대 `Ping` →
  바깥 층) · `watch_presence` · `PRESENCE_EVERY` 삭제 → `announce`(Stage 3 몫이지만 같은 파일).
- `crates/agent/src/chat.rs` — `ship` · `enqueue` · `send_to` · `backoff` · `kick` · `lane` · `deliver` · `settle` · `give_up` · `peer_back` · `outbox_of` ·
  `ask_status` · `inbound_status` · `set_retry` · `UNREACHABLE`. 7곳 `node.send` → `ship`(사람 실행 요청 · 미러 · 답 · 자동 왕복 넘김 · 상한 기록 · 완료 ·
  멈춤 알림) / `enqueue`(진행 — 1b 의 상대별 알림 줄을 대체). 재개 `Relay{on:true}` · `Agents` 는 그대로.
- `crates/app/src/chat_api.rs` — 대화 글 목록 JSON `outbox{state, tries, error}` · `progress.detail`. `main.rs` — `announce`.
- 웹UI — `view.js` `outboxText` · `stepText` · `refusedText`(`unreachable` → "보내지 못함 …") · 서명에 보관 상태, `app.js` 메타 줄.
- 테스트 — `core/tests/outbox.rs`(5) · `agent/tests/durable.rs`(6) · `view.test.mjs` 2. 바뀐 동작에 맞춘 기존 시험 5: `roundtrip`
  `unconfirmed_delivery…`(오류 → 보관) · `ask_kind_and_done…`(완료 경고 → 없음) · `relay` `late_reply…`(전송 오류로 멈춤 → 이어 감) · `two_sided`
  `own_run_not_failed_by_mirror`(경고 → 없음 · 미러 보관) · `net/tests/pairing.rs`(`watch_presence` → `announce`) · `core/tests/paths.rs`(스키마 13).

## 설계 결정

- **순서는 대화 안에서만** — plan 해석 2 는 "그 상대 줄에 앞선 행이 없으면 바로" 였다. 상대 단위로 두니 받은 대화에서 사람이 글을 쓸 때 함께 나가는
  자동 왕복 멈춤 알림(`block_relay`)이 미러를 줄 뒤로 밀어 미러가 늦게 닿았다(`cli_origin` · `two_sided` 실패). 바로 보낼지는 **같은 대화**에 앞선 행이
  있는지로 가르고(`outbox.conv_id`), 진행 · 멈춤 알림은 앞질러도 된다(답보다 늦게 닿은 진행은 받는 쪽이 `unknown` 으로 버린다). 상대 줄(다시 보내기)은
  상대 단위 id 순 그대로.
- **닿지 않고 있는 상대면 바로 보내지 않는다** — 전송 오류로 기다리는 행(시도 1번 이상)이 있으면 새 글도 줄로. 화면이 연결 시간 초과(최대 10초)를
  기다리지 않는다. 상대가 돌아오면(`Ping` · 요청 받기 — `on_seen` · `peer_back`) 줄을 지금 보낸다.
- **보내는 중에 돌아온 증거** — 바로 보내는 시도가 아직 실패하기 전에 상대가 돌아오면 `hurry` 가 그 행에 표시(`next_at` 0)만 하고, 그 시도가 실패하면
  `retry` 가 바로 다시 잡는다(Windows 의 닫힌 포트 연결 거부는 2초쯤 걸려 시험에서 드러났다). 시작(`recover`)도 `hurry`.
- **`touch` 는 10초에 한 번** — 프레임마다 쓰니 30건 몰림 시험에서 다른 연결의 트랜잭션이 "database is locked" 로 실패했다. 읽기로 먼저 거른다.
- **글 목록 조회의 DB 잠금** — `list_messages` 가 잠금을 쥔 채 `outbox_of` 를 불러 같은 잠금을 다시 잡아 멈췄다(`capacity_api` 정지) — 읽기만 잠그고 놓는다.
- **바뀐 사용자 동작(plan 해석 5)** — 실행 요청 · 미러 · 완료 표시의 전송 오류는 오류 · 경고가 아니다(보관). 자동 왕복 넘김의 전송 오류도 멈추지 않는다.
- 줄 작업은 `Chat` 이 `Arc::new_cyclic` 의 자기 참조(`me`)로 띄운다(호출자가 `&self` 인 자리에서도).

## 테스트 결과

```text
cargo test -p an-core --test outbox          → test result: ok. 5 passed; 0 failed
cargo test -p an-agent --test durable         → test result: ok. 6 passed; 0 failed
cargo test -p coworkagent-app --test capacity_api → test result: ok. 5 passed; 0 failed
node --test --test-reporter=tap "crates/app/web/*.test.mjs" → # pass 41 / # fail 0
```

- RED: `durable` `reply_reaches…` 는 처음 상태 질의 뒤에도 답이 오지 않았다(보내는 중 행은 `hurry` 가 건드리지 않음) → 표시 · 재시도 규칙 추가. `restart…`
  은 지난 실행의 다음 시각(1시간)을 그대로 기다렸다 → 시작 때 `hurry`.

## 단계 리뷰 (2026-10-13, 분리된 리뷰 에이전트 · diff `da671b2..3724519`)

High 1 · Medium 3 · Low 3. 받는 쪽 중복 실행(`message_by_uuid` · 승격 조건) · `Status` 권한(그 상대 대화 · 상대 실행 요청만) · DB 잠금 재진입 ·
알림 유실(`Notify` permit) · 백오프 넘침 · XSS 는 결함 없음.

| 지적 | 판단 | 처리 |
|---|---|---|
| H · `ship` 을 기다리던 HTTP 처리기가 취소되면 행이 `inflight` 로 남아 그 상대 줄이 재시작까지 멈춘다 | 사실(흐름) | 바로 보내기 · 뒷정리를 `tokio::spawn` 작업으로 — 부른 쪽이 취소돼도 지우기 · 재시도 기록 · 줄 깨우기가 끝난다 |
| M · `waited` 에 예정 간격을 더해, 앞당긴 재시도(상대 프레임마다)가 24시간 한도를 몇 분에 채운다 | 사실(경로) | `retry` 가 지난 시도부터 **실제로 흐른 시간**을 직전 간격까지만 더한다(`last_try`) — 앱을 꺼 둔 시간도 빠진다. `outbox` 단위 시험 갱신 |
| M · 포기는 줄 맨 앞 하나씩 — 죽은 상대에 N 건이면 N × 24시간 | 사실 | 맨 앞이 한도에 닿으면 그 상대 줄에 남은 행도 함께 포기(`queued_for`) |
| M · 받는 쪽 처리 오류(DB 잠금 등)가 `bad_request` → 보낸 쪽이 거절로 확정 | 사실 | 처리 오류는 `retry` 코드 — `send_to` 가 전송 오류로 바꿔 다시 보낸다 |
| L · 줄 작업이 panic 하면 `kick` 이 죽은 작업만 깨운다 | 추정 | 줄 항목에 `JoinHandle` — 끝났으면 다시 띄운다 |
| L · 지운 · 페어링 해제한 상대에도 24시간 다시 보낸다 | 사실 | 미조치 — 기록(다시 페어링하면 이어 가는 쪽이 낫다, 한도가 끊는다) |
| L · 실패 보관 행이 쌓인다(`clear_failed` 부르는 곳 없음) | 사실 | 시작 때 7일 지난 실패 보관 지움(`prune_failed`) |
