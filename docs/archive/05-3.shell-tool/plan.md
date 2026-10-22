# plan — 05-3.shell-tool

> 근거: [intent](intent.md)(승인 2026-10-22 — 결정 1 ~ 9). **구현자는 이 문서만 보고 일한다.** D·S 는 [masterPlan](../../masterPlan.md)
> (D21 에이전트 프로필 · D22 헤드리스 권한 · D31 문의 응답 전용 — 종결 때 환류). 브랜치 `task/05-3-shell-tool`(작업 폴더 `D:\dev\git\coworkAgent-05-3` — 원 작업 폴더에
> 다른 세션의 미커밋 변경이 있어 따로). 기준 커밋: `7c9dbe2`(origin/main).
> plan 승인: 2026-10-22(질문 답으로 갈음 — intent 결정 9).

## 목표

에이전트별 "셸 명령 실행 허용"(기본 끔) — 켜면 `claude -p` 가 `Bash` · `PowerShell` 도구를 쓰고(셸용 가드 · 정리한 환경), 화면은 위험을 늘 안내하고 켤 때 한 번 확인받는다.

### 성공 기준(intent 사본)

- A1: 옵션을 켠 에이전트의 실행 인자 `--tools` · `--allowedTools` 에 `Bash` · `PowerShell` 이 더해지고 시스템 프롬프트가 셸용 가드로 바뀌며, 끄면 읽기 전용 3개(`LS` · `NotebookRead` 뺌) · `GUARD_PROMPT`(시험).
- A2: 실제 `claude` 를 제품 인자 · 제품 환경으로 돌려 — 켜면 셸 명령이 돌고 앱에 심은 환경 변수가 셸에 보이지 않으며, 끄면 셸 명령이 돌지 않는다(실측).
- A3: 등록 화면 · 에이전트 상세에 셸 허용 체크박스와 위험 도움말이 있고, 켜고 등록 · 저장하면 위험 확인 창이 뜨며 취소하면 보내지 않는다. 켠 에이전트 상세에 경고 줄이 보인다. 저장 · 다시 읽기에 값이 남는다(시험 · 실제 화면).
- A4: 있던 에이전트는 끔(스키마 25 기본값), 옵션을 바꾸면 그 에이전트의 세션을 비운다(시험).
- A5: 회귀 없음 — PROJECT.md 검증 명령 PASS, 매뉴얼 · PRODUCT · 원장 반영(검사).

## 설계

### core

- 스키마 25: `agents.shell INTEGER NOT NULL DEFAULT 0`(`has_column` 로 건너뜀 — 스키마 24 방식), `SCHEMA_VERSION` 25.
- `Agent.shell: bool` · `COLS` · `row`. `agents::SHELL_TOOLS` = Windows `["Bash", "PowerShell"]` · 그 밖 `["Bash"]`(`cfg` — macOS 에 없는 이름을 넣지 않는다). `add_with(…, parent_read, shell)` · `update(…, parent_read, shell)` —
  폴더 · `parent_read` · `shell` 중 하나라도 바뀌면 새 `uid` · `forget_sessions`. 시험: `tests/paths.rs` `schema12_focus_mention` 에 셸 토글 → 새 `uid` · 세션 비움 ·
  기본 끔 단언 추가, `db_schema_version_kept_on_reopen` 상수 25.

### agent

- `SHELL_GUARD_PROMPT` — `GUARD_PROMPT` 에서 "명령 · 스크립트 실행" 금지를 빼고, 셸 = 이 PC 에 설정된 workflow(스크립트 · 빌드 · 테스트 등) 수행 · 조회용, 파일 · 코드 삭제 · 편집 · 이동 · 저장소 · 설정 변경 금지(그런 명령을 직접 실행하지 않음)를 적은 글(나머지 문장 그대로).
- `claude_args`: `agent.shell` 이면 도구 = `READ_ONLY_TOOLS` + `SHELL_TOOLS`, 시스템 프롬프트 첫머리 = `SHELL_GUARD_PROMPT`. 끄면 지금과 같다.
  상위 읽기 `[읽기 범위]` 줄은 셸이면 "그 위는 읽을 수 없다" 를 뺀 "파일 도구는 … 아래를 읽는다" 로(셸은 범위 밖을 읽으므로 거짓 안내를 하지 않는다).
- 압축 요약 실행(`SUMMARY_PROMPT`, `chat.rs`)은 셸을 끈 에이전트 사본으로 돌린다 — 요약에 셸이 필요 없고, 세션에 남은 원격 주입이 요약 때 명령으로 이어지지 않게(구현자 결정 — 안전 쪽).
- `shell_env(vars) -> Vec<(String, String)>` — 기본 변수 목록(`SHELL_ENV_KEEP`)에 든 것만 남긴다. Windows 는 이름 대소문자 무시(`Path` = `PATH`).
  `run_process_lines(…, env: Option<&[(String, String)]>)` — `Some` 이면 `env_clear` + 그 목록. `run_claude` 는 `agent.shell` 이면 `Some(shell_env(std::env::vars()))`.
- 시험: `tests/runner.rs` `shell_adds_tools`(켬 = 3 + 셸 · 셸 가드 · 끔 = 3개 · `GUARD_PROMPT`) · `shell_env_keeps_base_only`(대소문자 · 비밀 변수 빠짐),
  `tests/shell_canary.rs`(새, `#[ignore]` 실제 claude — `run_claude` 로: 켬 = `../up.txt` 내용이 답에 · 심은 변수 값이 답에 없음, 끔 = 내용 없음).

### app

- `NewAgent.shell`(`#[serde(default)]`) → `add_with`, `AgentPatch.shell: Option<bool>` → `update`, `agent_json` 에 `shell`.
- 웹UI: 등록 칸 `index.html` `agent_form` 상위 읽기 다음 `<label for="agent_shell">셸 명령 실행 허용</label>` + 체크박스(title = `SHELL_NOTE`),
  상세(`renderAgentList`) 체크박스 + 켰을 때 경고 줄(`SHELL_ON`). 확인 = `view.js` 순수 함수 `shellConfirmNeeded(before, after)`(끔 → 켬일 때만 참 — 등록은 before = 끔)이
  참이면 `window.confirm(SHELL_WARN)`(`app.js` 의 지역 함수 `confirm` 과 겹치지 않게 `window.` 로), 취소면 보내지 않는다. 시험: `view.test.mjs`.
  글 `view.js`(등록 칸 title 은 `SHELL_NOTE` 와 같은 글):
  - `SHELL_NOTE` = "켜면 이 PC 에 설정된 workflow(스크립트 · 빌드 · 테스트 등)를 셸(Bash · PowerShell) 명령으로 수행해 답합니다. 삭제 · 편집 등 이 PC 를 바꾸는 행동은 허용하지 않습니다. 셸에는 폴더 제한이 적용되지 않아 켤 때 위험 안내를 확인합니다."
  - `SHELL_WARN`(확인 창) — 머리 2줄(셸 = 이 PC 에 설정된 workflow 수행 기능 · 삭제 · 편집 등 변경은 허용하지 않음) + 위험 항목 6줄 + "믿는 상대만 연결된 PC 에서 켜세요. 켜시겠습니까?":
    연결된 상대 누구나(멘션 포함) 사람 승인 없이 명령 실행 · 셸에는 폴더 · 상위 읽기 제한 없음 · coworkAgent 데이터 폴더(신원 키 · 대화 DB) · Claude 인증 정보(`~/.claude`) · `~/.ssh` 를
    읽어 답에 실을 수 있음 · "바꾸지 않는다" 가드는 안내일 뿐(원격 글에 숨긴 지시로 수정 · 삭제 · 외부 전송 가능) · 자동 왕복의 내 에이전트로 쓰면 상대 에이전트 답이 이 PC 셸을 움직임 ·
    앱 환경 변수는 넘기지 않지만 파일 · 저장소 속 비밀은 막지 못함.
  - `SHELL_ON`(상세 경고 줄) = "켜짐 — 연결된 상대가 이 PC 에서 셸 명령을 실행할 수 있습니다(폴더 제한 없음 · 사람 승인 없음)."

### 문서

- [an_core](../../manual/dev/an_core.md) · [an_agent](../../manual/dev/an_agent.md) · [an_app](../../manual/dev/an_app.md) · [개발자 가이드](../../manual/dev/developer-guide.md)(실행 인자) ·
  [사용자 가이드](../../manual/user/user-guide.md) 6장 · `PRODUCT.md`(Claude 실행). 원장 `#1a A-I3` 갱신(셸 실행은 `env_clear` — 닫힘) · 새 행 `#05-3`(셸 허용 위험 — `SHELL_WARN` 의 항목 그대로, 수용 · 공지).
  `docs/tech/security-posture.md` 는 원 작업 폴더의 다른 세션 미커밋 파일이라 손대지 않는다 — "셸 차단" 문장 갱신 필요를 report 에 남긴다. masterPlan(종결 환류): D21 · D22 · D31 개정 · 개정 기록.

## 평가 항목

- [ ] E1: A4·A5 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E2: A5 — clippy 경고 0
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: pending
- [ ] E3: A1·A4 — 실행 인자 · 환경 목록 · 세대 시험
  CHECK: cargo test -p an-agent --test runner shell && cargo test -p an-core --test paths schema12_focus_mention
  EXPECT: /test result: ok\. 2 passed; 0 failed/
  EVIDENCE: pending
- [ ] E4: A2 — 실제 claude(사용량): 켬 = 셸로 위 폴더 파일 읽음 · 심은 변수 안 보임, 끔 = 못 읽음
  CHECK: cargo test -p an-agent --test shell_canary -- --ignored
  EXPECT: /test result: ok\. 1 passed; 0 failed/
  EVIDENCE: pending
- [ ] E5: A3 — API 시험 · 웹UI 로직 시험 · 체크박스 라벨 literal · 확인 창 배선 · 스키마 25 실재
  CHECK: cargo test -p coworkagent-app --test chat_api agents_crud_api && node --test --test-reporter=tap "crates/app/web/*.test.mjs" && python -c "import pathlib as P;h=P.Path('crates/app/web/index.html').read_text(encoding='utf-8');j=P.Path('crates/app/web/app.js').read_text(encoding='utf-8');d=P.Path('crates/core/src/db.rs').read_text(encoding='utf-8');L='\uc178 \uba85\ub839 \uc2e4\ud589 \ud5c8\uc6a9';print('box=%d label=%d warn=%d schema=%d'%(h.count('<input type=\x22checkbox\x22 id=\x22agent_shell\x22'),h.count('<label for='+chr(34)+'agent_shell'+chr(34)+'>'+L+'</label>')+j.count('field('+chr(34)+L+chr(34)),j.count('window.confirm(SHELL_WARN)'),d.count('pub const SCHEMA_VERSION: i64 = 25;')))"
  EXPECT: box=1 label=2 warn=2 schema=1
  EVIDENCE: pending
- [ ] E6: A5 — 문서 규약 · 크레이트 경계 · 매뉴얼 · PRODUCT · 원장 반영
  CHECK: python tools/check_ddd_docs.py && python tools/check_crates.py && python -c "import pathlib as P;t=lambda f:P.Path(f).read_text(encoding='utf-8');L='\uc178 \uba85\ub839 \uc2e4\ud589 \ud5c8\uc6a9';print('core=%d agent=%d app=%d dev=%d user=%d product=%d ledger=%d'%(min(1,t('docs/manual/dev/an_core.md').count('SHELL_TOOLS')),min(1,t('docs/manual/dev/an_agent.md').count('shell_env')),min(1,t('docs/manual/dev/an_app.md').count('SHELL_WARN')),min(1,t('docs/manual/dev/developer-guide.md').count('SHELL_GUARD_PROMPT')),min(1,t('docs/manual/user/user-guide.md').count(L)),min(1,t('PRODUCT.md').count(L)),min(1,t('docs/overview/ledger.md').count('#05-3'))))"
  EXPECT: core=1 agent=1 app=1 dev=1 user=1 product=1 ledger=1
  EVIDENCE: pending
- [ ] E7: A3 — 실제 화면: 디버그 빌드(시험 데이터 폴더)로 등록 화면 셸 체크 → 등록 → 확인 창(취소 = 등록 안 됨 · 확인 = 등록) → 상세에 체크 · 경고 줄 → 끄고 저장(확인 창 없음) → 다시 읽기에 끔
  EVIDENCE: pending

## 개정 기록

- 2026-10-22 — 구현 중 지시 "U1. 삭제,편집 등의 사용자 PC 의 변경은 불가능함은 안내. 쉘 실행은 사용자 pc 에 설정된 workflow 를 수행하기 위함임." → `SHELL_GUARD_PROMPT` · `SHELL_NOTE` · `SHELL_WARN` 머리 · 사용자 가이드 글 개정(gate EXPECT 변경 없음).
- 2026-10-22 — 구현 중 지시 "U4. 권장안 수용. 사용가능한 툴 중 notebook 은 제거." → 셸용 가드 · 안내 문구에 workflow 산출물 예외, `READ_ONLY_TOOLS` 에서 `NotebookRead` 제거(4개, A1 문구 개정 — gate EXPECT 변경 없음).
- 2026-10-22 — E5 CHECK 보정(EXPECT 그대로): 라벨 literal 을 문서 전체 글자 수가 아니라 등록 칸 `<label for="agent_shell">` · 상세 `field("…")` 자리로 센다 — "내 에이전트" 도움말(`tip_agents`)이 라벨 이름을 인용해 label=3 이 된 것(라벨 자체는 2곳).
- 2026-10-22 — 구현 중 지시 "의견 수용. 효과 없는 도구 목록은 정리 병행." → `READ_ONLY_TOOLS` 에서 `LS` 도 뺌(3개), `crates/agent/tests/tools_canary.rs`(새, `#[ignore]` — 실제 init `tools` 대조). gate 추가 없음(EXPECT 변경 없음 — 증거는 stage · eval).
- 2026-10-22 — 적대적 평가 R1 반영: 상세 저장은 바꾼 셸 · 상위 읽기만 보냄(`view.js` `agentPatch`), 프록시 주소 계정 · 비밀번호 노출을 `SHELL_WARN` · 사용자 가이드 · 원장에 공지.
