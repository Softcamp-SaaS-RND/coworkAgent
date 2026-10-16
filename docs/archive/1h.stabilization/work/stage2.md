# stage2 — 결함 3건: 트레이 다시 시도 · 사라진 세션 · 상대 번호(스키마 17)

> 근거: [plan](../plan.md) Stage 2. 사용자 검증은 D28 — 19.prototype-acceptance 에서 일괄.

## 목표

해석 7 · 8 · 9 — 트레이 만들기 실패 시 다시 시도(1e 남은 것) · `--resume` 세션이 사라졌으면 새 세션으로 한 번 다시(원장 #15 R2-F2r) · 상대 번호를 다시
쓰지 않음(원장 #17 S1-P). 대상 gate: E1 · E2 · E5 ~ E7.

## 산출 파일

- `crates/core/src/db.rs` — 스키마 17: `peers` 를 지금 정의 그대로 `AUTOINCREMENT` 새 표로 → 번호째 복사 → 옛 표 지움 → 이름 바꿈(한 트랜잭션) · 순번 =
  `max(peers.id, conversations.peer_id, outbox.peer_id)`.
- `crates/agent/src/runner.rs` — `SessionMissing` · `session_missing` · `session_gone(stdout, stderr)` · `run_claude` 가 사라진 세션이면 `SessionMissing`.
- `crates/agent/src/chat.rs` — `Session::Resume` 이 `SessionMissing` 이면 새 세션(`RunInput { session: New, ..input }`)으로 한 번 다시.
- `crates/app/src/tray.rs` — `retry_after(n)`(1 · 2 · 4 · 8 · 15 · 30초) · `Tray::run` 이 실패하면 `ControlFlow::WaitUntil` 로 기다렸다 다시, 끝내 실패하면
  지금처럼 사유 표시 · 정리 · 종료.
- 시험 — `crates/core/tests/store.rs` `schema17_peer_ids_never_reused` · `schema17_fresh_db_no_reuse`, `paths.rs` 스키마 번호(16 → 17, "더 새 스키마"
  18) · `crates/agent/tests/resume_missing.rs`(새, 5건) · `tray.rs` `retry_after_schedule`.

## 설계 결정

- **이름 바꾼 표의 정의는 `CREATE TABLE "peers"`** — 시험에서 실측(이름 바꾸기 뒤 SQLite 가 따옴표로 적는다). 스키마 17 은 두 꼴을 모두 받는다 — 알 수
  없는 정의면 고치지 않고 오류(DB 를 열지 않음 · 사본은 남음).
- **upsert 도 순번을 쓴다** — 있는 상대의 `INSERT … ON CONFLICT DO UPDATE` 도 AUTOINCREMENT 순번을 하나 올린다(실측). 빈 번호가 생길 뿐 다시 쓰지는
  않는다 — 시험은 "이전보다 큼" 으로 단정.
- **판별은 결과 줄 먼저** — 결과 줄이 있으면 그 `errors` 만 본다(답 본문 · 다른 이벤트에 같은 문장이 있어도 오판하지 않게). 결과 줄이 없을 때만 stderr.
- **루프 첫 줄의 `ControlFlow`** — 지금 코드는 이벤트마다 `Wait` 로 되돌린다. 기다리는 동안 다른 이벤트가 와도 `WaitUntil` 이 풀리지 않게 다음 시도
  시각(`retry_at`)을 두고 첫 줄에서 그것으로 정한다.

## 검증

- `cargo test -p an-core --test store` — 10 passed(`schema17_*` 2건 포함). `cargo test -p an-agent --test resume_missing` — 5 passed.
  `cargo test -p coworkagent-app --lib tray::` — 2 passed. `cargo test --workspace` 실패 0 · `cargo clippy --workspace --all-targets -- -D warnings`
  Finished.
- 실제 트레이 실패 · 실제 claude 세션 파일 지움은 이 PC 에서 재현하지 않는다(로그인 직후 상태 · 사용자 세션 파일) — 19 TC.
