# stage4 — 처리경과(stream-json 단계 요약)

> 근거: [plan](../plan.md) Stage 4 · 해석 10. 사용자 검증은 D28 — 19.prototype-acceptance 에서 일괄.

## 목표

상대가 보낸 실행 요청이 도는 동안 보낸 쪽에 "처리 중 — 2턴 · Read 3 · Grep 1 · 41초" 를 알린다(A10). 대상 gate: E1 ~ E3 · E8 · E9.

## 산출 파일

- `crates/agent/src/runner.rs` — `claude_args` `--output-format stream-json --verbose` · `run_process_lines(…, on_line)`(줄마다 부르고 참인 줄만
  `stdout` 에 — `run_process` 는 모두 남김) · `StepSink`(비교 · 출력에서 무시) · `RunInput.steps` · `count_step`(`assistant` `tool_use` 이름 ·
  `mcp__*` → `MCP` · 도구 결과 `user` 마다 한 턴) · `run_claude` 는 결과 줄 · JSON 아닌 줄만 남기고 바뀔 때마다 `steps` 로.
- `crates/agent/src/chat.rs` — `step_every`(기본 10초) · `set_step_every`(시험용) · `execute` 가 상대 실행 요청이면 실행기 콜백은 마지막 값만 두고,
  tokio 작업이 `step_every` 마다 바뀐 것만 `send_step(running, detail)` — 실행이 끝나면 작업을 멈춘다.
- 테스트 — `agent/tests/steps.rs`(4) · `steps_live.rs`(1, `#[ignore]`) · `runner.rs` 인자 단정 `stream-json` + `--verbose`.

## 설계 결정

- **단계 요약 타입은 `an_net::proto::StepDetail` 하나** — plan 해석 10 의 `Step` 을 따로 두지 않았다(필드 같음 — 보내는 값을 그대로). 편차 기록.
- **합침은 받는 쪽 `Chat` 의 작업이** — 실행기 콜백은 출력 읽기 스레드(tokio 밖)에서 불리므로 거기서 보내지 않고 마지막 값만 둔다. 10초 안에 끝나는
  실행은 단계 요약을 보내지 않는다(답이 곧 온다).
- **실시간 이벤트는 쌓지 않는다** — stream-json 은 도구 결과(파일 내용)까지 실어 커질 수 있어 결과 줄만 남긴다. `parse_output` 은 그대로 마지막
  JSON 줄.
- **요약에 입력 · 경로 · 글 없음** — 이름 · 횟수 · 턴 · 초만 센다(`count_step` 은 `input` · `content` 글을 읽지 않음). 받은 쪽은 이름을 다시
  `clean_name` · 24자 · 16개로(Stage 1).

## 테스트 결과

```text
cargo test -p an-agent --test steps                              → test result: ok. 4 passed; 0 failed
cargo test -p an-agent --test steps_live -- --ignored --nocapture → test result: ok. 1 passed; 0 failed
  steps: [StepDetail { turns: 1, tools: {"Read": 1}, secs: 2 }, StepDetail { turns: 2, tools: {"Read": 1}, secs: 2 }]
  out: RunOutput { text: "노을-42", …, is_error: false, turns: Some(2), duration_ms: Some(1977) }   (claude 2.1.296 · haiku · low)
cargo test --workspace                                           → 실패 0
cargo clippy --workspace --all-targets -- -D warnings            → 경고 0
node --test --test-reporter=tap "crates/app/web/*.test.mjs"      → # pass 42 / # fail 0
```
