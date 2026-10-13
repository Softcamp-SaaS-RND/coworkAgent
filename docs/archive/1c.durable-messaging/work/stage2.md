# stage2 — CLI 트랜잭션

> 근거: [plan](../plan.md) Stage 2. 사용자 검증은 D28 — 19.prototype-acceptance 에서 일괄.

## 목표

CLI `ask` 하나를 앱 DB 의 트랜잭션 하나로 — 중복 키(`--id`)는 새로 보내지 않고, CLI 가 죽어도 `wait <tx>` 로 이어 받고, 상대가 대기 · 실행 중으로
확인되는 동안 기다리다 정해진 시간 닿지 않으면 종료 코드 7(A6 · A7). 대상 gate: E1 · E2 · E6.

## 산출 파일

- `crates/core/src/cli_tx.rs` — `key_ok` · `create` · `by_id` · `by_key` · `due` · `asked` · `reached` · `unreachable`(멈춤 판정) · `resume`.
- `crates/agent/src/chat.rs` — `TxView` · `set_watch` · `tx_create` · `tx_by_key` · `tx_resume` · `tx_view`(상태 계산 + 감시 작업) · `watch_tx`(보관 중이면 그
  상대 줄이 닿지 않는지, 아니면 `ask_status` — 닿으면 `reached`, 아니면 `unreachable`).
- `crates/app/src/chat_api.rs` — 새 글 요청 `tx_key`(via `cli` 만 · 규칙 밖 400) · 같은 키면 보내지 않고 200 + 트랜잭션 · 보냄 201 에 `tx` ·
  `GET /api/tx/{id}` · `POST /api/tx/{id}/resume`.
- `crates/app/src/cli.rs` — `ask --id` · `wait <tx>` · `status <tx>` · `EXIT_STALLED` 7 · `WAIT_LIMIT` 24시간(안전판) · `wait_tx`(1b 의 `wait_reply` 대체 —
  진행 줄: 보내는 중 · 대기 · 처리 중 + 단계) · `--json` 결과에 `tx`.
- 테스트 — `app/tests/cli_tx.rs`(5). 1b 시험 `capacity_api` `cli_wait_clock_pauses_while_queued` → `cli_waits_while_queued`(CLI 자체 시계가 아니라
  앱 판정 — 안전판 20초).

## 설계 결정

- **감시는 조회가 띄운다** — `GET /api/tx/{id}` 가 상태를 계산하고, 끝나지 않았고 물을 때(마지막으로 물은 지 60초)면 상대에 묻는 작업을 띄우고 바로
  돌아온다(다음 조회가 결과를 본다). 같은 트랜잭션은 겹쳐 묻지 않는다. CLI 가 없으면 감시도 없다 — 앱이 상대를 계속 두드리지 않는다(주기 확인 삭제와
  같은 축).
- **보관 중(`sending`)의 닿음 = 그 상대 줄이 닿지 않고 있지 않음** — 상태 질의는 보내지 않는다(글이 아직 상대에 없다).
- **멈춘 뒤 답이 오면 답이 이긴다** — 상태 순서: 답(성공 · 실패) → 글 실패 → 멈춤 → 보관 → 진행. `wait` 은 멈춤을 풀고 다시 판정.
- **같은 키 동시 두 번** — 둘째 트랜잭션 기록이 UNIQUE 로 실패한다(보내기는 됐다) — 드문 경합, `ponytail` 주석 · 트랜잭션 없이 돌려준다(CLI 는 "앱이
  트랜잭션을 만들지 못했다 — 다시 보내지 말고 history" 로 끝).
- **`--no-wait`** — 평문은 지금처럼 대화 ID, `--json` 에 `tx`.

## 테스트 결과

```text
cargo test -p coworkagent-app --test cli_tx        → test result: ok. 5 passed; 0 failed
cargo test -p coworkagent-app --test cli           → test result: ok. 16 passed; 0 failed
cargo test -p coworkagent-app --test capacity_api  → test result: ok. 5 passed; 0 failed
```

- 같은 묶음에서 고친 1b 시험 하나: `relay` `stop_during_run_then_resume` 이 8번 중 6번 실패 — B 대역이 바로 답해 마지막 멈춤과 B 의 답이 경주했다(B 답이
  먼저 닿으면 A 에이전트가 한 번 더 돈다). 시험의 전제("멈춘 뒤 온 답은 넘기지 않음")를 지키게 B 도 0.4초로. 반영 뒤 8/8.
