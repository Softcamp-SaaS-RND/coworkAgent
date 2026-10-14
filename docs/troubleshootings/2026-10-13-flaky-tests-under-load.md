# 2026-10-13 — 부하에서 간헐 실패하는 시험(`cargo test --workspace` rc 101)

## 증상

#1d 평가 중 gate 체커의 E1(`cargo test --workspace`)이 약 12회 중 3회 rc 101 로 실패했다. 같은 시각 다른 cargo 작업(리뷰 세션 · 반복 실행)이
돌던 때였고, 단독 실행 20회 남짓은 모두 통과해 실패한 시험 이름을 잡지 못했다.

## 재현

시험 바이너리 59개를 미리 빌드하고(`cargo test --workspace --no-run --message-format=json`) **모두 한꺼번에** 각자 크레이트 폴더에서 돌리는 부하
실행기로 실패 이름 · 출력을 모았다(바이너리를 동시에 돌리지 않는 `cargo test` 보다 훨씬 무거운 부하 — 실패를 드러내려는 조건). 5 라운드에 19건,
7개 시험(전부 `an-agent` 통합 시험):

| 시험 | 실패 꼴 | 원인 |
|---|---|---|
| `roundtrip::folder_change_during_run_drops_stale_session` | `database is locked`(SQLITE_BUSY) | **제품 결함** — 읽고 나서 쓰는 지연(deferred) 트랜잭션은 다른 연결이 쓰는 중이면 잠금을 올리지 못해 `busy_timeout` 을 기다리지 않고 바로 실패한다(SQLite 교착 회피). 시험은 그 경쟁을 드러냈을 뿐 앱에서도 노드 · 알림 연결이 같은 DB 에 쓴다 |
| `relay::receiver_stop_refuses_agent_requests` | 내 답이 `Error` | **제품 동작** — 받은 쪽 멈춤 알림보다 넘김이 먼저 닿으면 받은 쪽이 `stopped` 로 거절하고, 연 쪽은 그것을 "상대가 거절" 오류로 표시했다(알림이 먼저 닿으면 오류 아님 — 도착 순서에 따라 결과가 달랐다) |
| `relay::relay_runs_until_cap` · `resume_restarts_count` | 셀 수 3 ≠ 5 · 2 ≠ 3 | 시험 — "0.3초 동안 그대로면 멈췄다" 판정이 다음 차례가 늦게 오면 이르게 끝난다 |
| `roundtrip::inbound_does_not_wait_for_run` | 보내기 두 번 0.5초 넘음 | 시험 — 실행 시간(0.7초)과 판정 기준이 가까워 부하에서 정상도 넘는다 |
| `capacity::cap_bounds_and_queue_snapshot` · `waiting_request…` · `same_conversation…` | 15초 안에 조건 안 맞음 · 순간 상태 놓침 | 시험 — 실행 시간(0.6 ~ 0.8초) 창 안에 보내기 · 상태 확인이 들어와야 하는데 부하에서 창을 놓친다. 대화마다 따로 오는 진행 알림의 도착 순서를 가정 |
| `capacity::five_conversations_respect_cap_in_arrival_order` | 시작 기록 순서가 섞임 | 시험 — 자리가 동시에 비면 들어간 둘의 시작 기록(실행 스레드가 뜬 뒤) 순서가 섞인다 |
| `atomic::relay_step_not_marked_sent_without_outbox_row` | 자동 왕복이 아직 켜짐 | 시험 — 고정 0.5초 뒤 판정 |

## 조치

- **쓰기 트랜잭션 = `BEGIN IMMEDIATE`**(`an_core::db::write_tx`) — `chat`(14곳) · `chat_api` · `notify` · `agents::update` · `remove` 의 트랜잭션을
  바꿨다. 처음부터 쓰기 잠금을 잡아 다른 연결이 쓰는 동안 `busy_timeout`(5초)을 기다린다. 마이그레이션(시작 때 한 연결)은 그대로.
- **넘김이 `stopped` 로 거절되면 멈춤 알림이 먼저 온 것과 같게**(`Chat::relay_stopped`) — 내 답은 넘기지 않은 답(`done`)으로 되돌리고 자동 왕복을
  끄고 "상대가 멈춤"(바로 보낼 때 · 줄이 보낼 때 모두). 알림이 뒤늦게 와도 이미 꺼져 있어 다시 알리지 않는다.
- 시험: `common::eventually` 5초 → 15초(맞으면 바로 끝난다) · `settled` 0.3초 1회 → 0.3초씩 4회 그대로 · 시간 창 시험의 실행 시간을 2 ~ 4초로 · 고정
  대기를 조건 대기로 · 진행 알림 도착 순서를 가정하지 않음 · 5대화 순서 시험은 실행 시간을 엇갈려(3 · 4 · 2초) 자리가 하나씩 빈다 · 멈춤 시험은 B 사람
  글과의 순서 대신 내 답만 본다.

## 확인

- 부하 실행기(바이너리 59개 동시) 1배 — 고치기 전 5 라운드 19건(7개 시험) → 마지막 코드에서 6 라운드 0건(중간 코드 포함 1배 누적 22 라운드 중
  고친 시험의 재발 0).
- 2배(같은 바이너리 둘씩, 118개 동시 — `cargo test` 에서는 생기지 않는 극단 부하) 4 라운드 — `start_error_file_has_reason` 1(고정 temp 파일 이름이
  같은 바이너리 둘 사이에서 겹침) · `capacity` 시간 창 시험 2(`five_conversations…` · `waiting_request…`).
- `cargo test --workspace` 316 passed · clippy 경고 0 · gate 체커(1d) 5회 연속 PASS.

## 남은 것

- `capacity` 시간 창 시험은 실제 시계에 기댄다 — 극단 부하에서는 아직 창을 놓친다. 더 줄이려면 실행기 대역을 "풀어 줄 때까지 기다림"
  (채널로 제어)으로 바꿔 시간 창 대신 순서로 판정한다(ponytail: 지금은 실행 시간을 늘린 것으로 충분 — `cargo test` 부하에서 재현 없음).
- 부하 실행기는 스크래치 도구(저장소 밖)다 — 같은 꼴이 다시 보이면 이 노트의 방법(빌드 → 바이너리 동시 실행 → 실패 이름 수집)으로 재현한다.
