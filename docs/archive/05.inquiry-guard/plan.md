# plan — 05.inquiry-guard

> 근거: [intent](intent.md)(승인 2026-10-11 — 사용자 지시 + 확인 답). **구현자는 이 문서만 보고 일한다.** D·S 는 [masterPlan](../../masterPlan.md).

## 목표

원격 실행을 문의 응답 전용으로 — 시스템 프롬프트 항상 + 도구 읽기 전용 고정.

### 성공 기준(intent 사본)

- A1: 모든 실행 인자에 문의 응답 전용 시스템 프롬프트가 들어가고, 프로필 추가 지시는 그 뒤에 붙는다.
- A2: 실행 인자의 `--tools` · `--allowedTools` 는 읽기 전용 목록 안의 도구만 담는다 — 프로필에 Edit · Bash 등이 저장돼 있어도 빠진다.
  남는 것이 없으면 기본(Read · Grep · Glob).
- A3: 에이전트 등록 API 는 읽기 전용 목록 밖 도구를 400 으로 거부한다.
- A4: 루트 `PROJECT.md` 검증 명령이 통과한다.

### intent 결정 · 제외(사본)

- 프롬프트 + 도구 이중 · 목록 `Read,Grep,Glob,LS,NotebookRead` · 비밀 싣지 않기 지시. 제외: DLP · 웹 도구 · 실제 claude 확인(D28).

## 설계

- `an_core::agents::READ_ONLY_TOOLS`(`["Read","Grep","Glob","LS","NotebookRead"]`) · `agents::read_only(tools) -> String`(목록 안 도구만,
  쉼표 · 공백 정리, 비면 `DEFAULT_TOOLS`) · `agents::unknown_tools(tools) -> Vec<String>`(목록 밖).
- `an_agent::runner::GUARD_PROMPT`(문의 응답 전용 지시) · `claude_args` 가 `--tools` · `--allowedTools` 에 `read_only(&agent.tools)`,
  `--append-system-prompt` 에 `GUARD_PROMPT` + (프로필 지시가 있으면 `"\n\n[수신자 추가 지시]\n" + prompt`), `--system-prompt-snapshot off`.
- `chat_api::add_agent` — `unknown_tools` 가 비지 않으면 400 "읽기 전용 도구만: …".
- 웹UI 설정 안내 문구.

## Stage

| stage | 목표 | 산출 | gate |
|---|---|---|---|
| 1 | 가드 프롬프트 · 읽기 전용 도구 | 테스트(runner · store · chat_api) | E1~E5 |

## 평가 항목

- [ ] E1: A4 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E2: A4 — 우리 코드 경고 0(clippy -D warnings)
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: pending
- [ ] E3: A1·A2 — 실행 인자(args_follow_d22 · guard_prompt_always · tools_limited_to_read_only · claude_version_gate 포함 8건)
  CHECK: cargo test -p an-agent --test runner
  EXPECT: test result: ok. 8 passed; 0 failed
  EVIDENCE: pending
- [ ] E4: A2 — 읽기 전용 도구 걸러내기(read_only_tools_filter 포함 저장 4건)
  CHECK: cargo test -p an-core --test store
  EXPECT: test result: ok. 4 passed; 0 failed
  EVIDENCE: pending
- [ ] E5: A3 — 등록 API 쓰기 도구 거부(agents_crud_api)
  CHECK: cargo test -p agentnetwork-app --test chat_api agents_crud_api
  EXPECT: test result: ok. 1 passed; 0 failed
  EVIDENCE: pending

## 종결 시 할 일

- masterPlan D31 신규 · D22 개정 · `#05` · Phase 0 행 · 매뉴얼 `an_core` · `an_agent` · `an_app` · PRODUCT "Claude 실행".

## 개정 기록

- 개정 1(2026-10-11, 적대적 평가 라운드 1 반영): 모든 실행에 `--system-prompt-snapshot off` — 기본(on)은 대화 첫 요청의 시스템
  프롬프트를 기록해 재개 때 그대로 쓰므로, 가드 이전에 만든 세션을 재개해도 지금 가드가 들어가게(`guard_prompt_always` 에 재개 단정, E3 안).
- 개정 2(2026-10-11, 적대적 평가 라운드 2 반영): 실행 전 `claude --version` 을 읽어 `MIN_CLAUDE`(2.1.267) 미만이면 실행하지 않고 업데이트
  안내 오류(가드 없이 돌리지 않는다). 회귀 `claude_version_gate`(E3 7 → 8건).
