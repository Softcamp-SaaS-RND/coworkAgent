# stage 1 — 보안(지적 1 안내문 · 3 · 6)

## 목표

[plan](../plan.md) stage 1 — 결정 6(도구 · 탐색 · MCP 최대 허용)에 맞춰 안내문을 사실대로 고치고, 받는 쪽 일시 DB 오류를 영구 거절로 굳히지 않으며, 한 상대가 실행 자리를 독차지하지 않게 한다.

## 산출 파일

- `crates/agent/src/runner.rs` — 상위 읽기 안내 = 셸 여부와 무관하게 "파일 도구는 이 폴더와 바로 위 폴더 {up} 아래를 읽는다."(셸 끔의 "그 위는 읽을 수 없다" 삭제 — 막는 장치 없음).
- `crates/core/src/chat/conversation.rs` — `open_refused(&rusqlite::Error)`: 다른 상대 uuid(UNIQUE 위반) · 묘비(행 없음)만 참.
- `crates/agent/src/chat/inbound.rs` — 받은 대화 열기 두 곳(`inbound_message` · `inbound_ask`): `open_refused` 면 `bad_request`, 그 밖은 `db_err` → `retry`.
  집중 판정 자리에 상대별 상한 — `focused || 상한` 이고 재전송이 아니면 집중 = `FOCUS`, 상한 = `RETRY`.
- `crates/core/src/chat/message.rs` — `peer_pending_runs(conn, peer_id)`(받은 대화 · exec · queued/running 수).
- `crates/agent/src/chat/mod.rs` — `PEER_RUN_CAP = 10`.
- 시험: `crates/agent/tests/runner.rs`(`parent_read_adds_dir` — 셸 끔도 "[읽기 범위] 파일 도구는" · "읽을 수 없다" 없음), `crates/core/tests/store.rs`(`open_refused_only_for_refusals` —
  다른 상대 uuid · 묘비 = 거절, 다른 연결이 쓰기 잠금을 쥔 채 = 거절 아님), `crates/agent/tests/capacity.rs`(`peer_run_cap_defers_overflow` — 13건 보내 받는 쪽 최대 ≤ 10 · ≥ 9, 모두 답 · 실패 0).

## 설계 결정

- 거절 판정은 an-core 에 — rusqlite 오류 종류를 아는 곳(an-agent 는 rusqlite 직접 의존 없음 · 크레이트 경계 그대로). plan E5 CHECK 보정.
- 상한 초과 = `retry`(새 응답 코드 없음) — 보낸 쪽 outbox 의 기존 재시도(5초 → 두 배 · 상대에게서 무엇이든 받으면 즉시)가 그대로 처리, 24시간 규칙 그대로.
  상한은 처음에 연성(판정 · 기록 분리)이었으나 리뷰 R1 지적 2 로 같은 잠금 안 판정으로 고침(eval.md).
- 재전송(같은 uuid)은 상한과 무관하게 받는다 — 집중과 같은 규칙(`resent`).
- MCP · 경로 금지 · Codex 환경 변수는 손대지 않음(결정 6). `tools_canary` · 편집 도구 금지 그대로.

## 테스트 결과

- RED: `parent_read_adds_dir` FAILED(`runner.rs:259`) → 고친 뒤 runner 19 ok.
- `cargo test -p an-core --test store open_refused` → 1 passed. `cargo test -p an-agent --test capacity` → 7 passed(기존 `thirty_requests_all_answered_without_refusal` 포함 — 넘친 것도 재전송으로 모두 답).
- `cargo test --workspace` rc 0 · `cargo clippy --workspace --all-targets -- -D warnings` Finished · `check_code_tidy` PASS.

## 다음 단계

stage 2 — 버그(지적 2 · 7 ~ 19).
