# stage2 — 실행기

## 목표

[plan](../plan.md) Stage 2 — 신규 `an-agent`(domain) `runner`: `claude_args` · `parse_output` · `run_process` · `find_claude` ·
`run_claude`. 대상 gate: E4.

## 산출 파일

- `crates/agent/{Cargo.toml,src/lib.rs,src/runner.rs,tests/runner.rs}` · `tools/check_crates.py`(`"an-agent": "domain"`)

## 설계 결정

- 인자: `-p --output-format json --restricted --strict-mcp-config --permission-prompts none --permission-mode dontAsk --tools <t>
  --allowedTools <t> [--append-system-prompt <p>] (--session-id <uuid> | --resume <id>)`. `--bare` · `bypassPermissions` 없음(D20 · D22).
- 원격 텍스트는 표준 입력으로만 — 인자 주입(따옴표 · `$(...)`)이 생길 자리가 없다(테스트가 따옴표 · 개행 · `$(whoami)` 를 그대로 단정).
- 출력은 마지막 JSON 줄을 읽는다(앞선 경고 줄 허용), `session_id` 없으면 오류.
- 쓰기 · 읽기(stdout · stderr)를 각각 스레드로 — 파이프가 가득 차 교착하지 않게. 시간 초과면 kill 후 `TimedOut`.
- Windows 는 `CREATE_NO_WINDOW` — GUI 서브시스템 앱이 콘솔 자식을 띄울 때 창이 뜨지 않게(std `CommandExt`, OS 전용 UI 아님).
- `--max-turns` 는 2.1.296 `--help` 에 없고 D24 개정으로 쓰지 않는다.

## 테스트 결과

- runner 4 통과(시간 제한: Windows `ping -n 30` · 그 밖 `sleep 30` 을 0.8초에 죽임). mutation: `--restricted` 제거 → args_follow_d22 FAIL.
- 처음 쓴 표준 입력 확인(`sort`)은 Windows `sort.exe` 출력 인코딩 차이로 실패 — `findstr "^"` / `cat` 으로 바꿨다.
- 실제 `claude -p` 실행은 사용자 구독 사용량을 쓰므로 자동 테스트에서 부르지 않는다(E8 수동, D28).

## 다음 단계

Stage 3 — 프로토콜(`agents` · `message` · `reply`) · `Chat` 조율 · 왕복 테스트.
