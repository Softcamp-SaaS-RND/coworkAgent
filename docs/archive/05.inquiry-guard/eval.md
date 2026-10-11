# eval.md — 05.inquiry-guard 적대적 평가

## 평가 원장

<!-- tools/check_gates.py 산출 — 손으로 고치지 않는다. 단 CHECK 가 없는 수동 gate 의
     EVIDENCE 와 ABANDON 줄은 사람이 적고, 재실행해도 보존된다.
     WEAK 줄은 기계가 매 실행 다시 단다 — 약한 gate 를 셀 수 있게 하는 표시이며
     판정에는 영향이 없다. -->

측정 커밋: 071b895cf03444b033bfaf5b5ab38c0dc0d1de82

- [x] E1: A4 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

- [x] E2: A4 — 우리 코드 경고 0(clippy -D warnings)
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.91s

- [x] E3: A1·A2 — 실행 인자(args_follow_d22 · guard_prompt_always · tools_limited_to_read_only · claude_version_gate 포함 8건)
  CHECK: cargo test -p an-agent --test runner
  EXPECT: test result: ok. 8 passed; 0 failed
  EVIDENCE: test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.89s

- [x] E4: A2 — 읽기 전용 도구 걸러내기(read_only_tools_filter 포함 저장 4건)
  CHECK: cargo test -p an-core --test store
  EXPECT: test result: ok. 4 passed; 0 failed
  EVIDENCE: test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s

- [x] E5: A3 — 등록 API 쓰기 도구 거부(agents_crud_api)
  CHECK: cargo test -p agentnetwork-app --test chat_api agents_crud_api
  EXPECT: test result: ok. 1 passed; 0 failed
  EVIDENCE: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.06s

## 적대적 평가 — 라운드 1/2

- 수단: codex-review 세션 `/codex:adversarial-review --base fe91755 --scope branch` · 대상 HEAD `3ec9d1c` · 2026-10-11
- Verdict: needs-attention

### 원문

```text
- [high] 기존 세션 resume 시 새 guard prompt가 무시됨 (crates/agent/src/runner.rs:68-72)
  `claude_args`는 모든 호출에 `--append-system-prompt`를 넣지만, 확인된 Claude Code 2.1.296의 기본값은 `--system-prompt-snapshot on`입니다. 이 모드에서는 최초 요청의 system prompt가 기록되고, 이후 `--resume`에 새 prompt를 전달해도 기존 기록을 그대로 사용합니다. `chat.rs:460-464`는 SQLite에 저장된 세션을 자동으로 `Session::Resume`하므로 업그레이드 전 생성된 대화에는 `GUARD_PROMPT`가 적용되지 않습니다. 읽기 도구는 여전히 허용되므로 원격 요청이 프로젝트의 키·토큰·개인정보를 읽어 답으로 반환할 수 있습니다.
  Recommendation: 모든 실행에 `--system-prompt-snapshot off`를 명시하고 현재 Claude CLI에서 resume 동작을 회귀 테스트하십시오. 또는 guard 정책 버전을 저장해 기존 session ID를 무효화하고 `Session::New`로 전환하십시오.
```

### 대조 · 조치

| 지적 | 판단 | 조치 |
|---|---|---|
| 재개 세션에 가드 미적용 | 사실(`claude --help` 2.1.296 `--system-prompt-snapshot <on\|off>` 기본 on 확인) | 모든 실행 인자에 `--system-prompt-snapshot off` — 매 요청 지금의 가드로 그린다. 회귀: `guard_prompt_always` 에 새 · 재개 세션 모두 `off` 단정 — "스냅샷 끔 제거" mutation FAIL. 실제 `claude` 재개 동작 확인은 19(D28) |

## 적대적 평가 — 라운드 2/2 (상한 — 재평가 없음)

- 수단: codex-review 세션 `/codex:adversarial-review --base fe91755 --scope branch` · 대상 HEAD `9a4eed0` · 2026-10-11
- Verdict: needs-attention

### 원문

```text
- [high] 구형 Claude Code에서 모든 원격 실행이 실패함 (crates/agent/src/runner.rs:72)
  `claude_args`가 `--system-prompt-snapshot off`를 무조건 전달합니다. 이 옵션은 [Claude Code v2.1.267](https://github.com/anthropics/claude-code/releases/tag/v2.1.267)에서 추가됐지만, 저장소에는 최소 버전 선언이나 실행 전 capability/version 검사가 없습니다. 따라서 v2.1.266 이하에서는 CLI가 옵션 파싱 중 종료하고 JSON을 출력하지 않아 모든 신규·재개 원격 문의가 실패합니다.
  Recommendation: 시작 시 Claude CLI 버전/capability를 검사하십시오. 지원 시에만 이 플래그를 전달하고, 미지원 버전은 안전한 호환 경로를 사용하거나 최소 v2.1.267 요구사항과 명확한 업그레이드 오류를 제공하십시오. 양쪽 분기의 회귀 테스트도 추가하십시오.
```

### 대조 · 조치

| 지적 | 판단 | 조치 |
|---|---|---|
| 구형 CLI 에서 실행 전면 실패 | 사실(실패 방식) · 도입 버전 2.1.267 은 확인 필요(이 PC 는 2.1.296) | 실행 전 `claude --version` → `MIN_CLAUDE`(2.1.267) 미만이면 실행하지 않고 "`claude update`" 안내 오류(가드 없이 도는 호환 경로는 두지 않는다 — D31). 회귀 `claude_version_gate`(경계 2.1.266/2.1.267) — "최소 버전 낮춤" mutation FAIL |

## 미조치 지적

| 지적 | 판단 | 사유 |
|---|---|---|
| 실제 `claude` 재개 · 버전 경계 통합 테스트(라운드 1 · 2) | 사실 | 자동 테스트에서 실제 `claude` 를 부르지 않는다(사용자 구독 사용량) — 19 수동 확인(D28) |
| `--system-prompt-snapshot` 도입 버전(2.1.267) 근거(라운드 2) | 확인 필요 | 지적이 든 릴리스 노트 링크 기준 — 이 PC 에서는 2.1.296 `--help` 만 확인. 틀리면 `MIN_CLAUDE` 한 줄 |
