# stage 1 — 에이전트 셸 명령 실행 허용

## 목표

intent 결정 1 ~ 12 — 에이전트별 "셸 명령 실행 허용"(기본 끔), 켜면 `Bash` · `PowerShell` 도구 · 셸용 가드(이 PC 에 설정된 workflow 수행 · 조회, workflow 산출물 예외,
소스 · 문서 · 설정 직접 삭제 · 편집 · 저장소 변경 금지) · 앱 환경 변수 비움, 위험 공지(도움말 · 켤 때 확인 창 · 상세 경고 줄), 읽기 도구에서 `NotebookRead` 제거.

## 산출 파일

- `crates/core/src/db.rs`(스키마 25) · `agents.rs`(`shell` · `SHELL_TOOLS` · `add_with(…, shell)` · `update(…, shell)` · `READ_ONLY_TOOLS` 4개).
- `crates/agent/src/runner.rs`(`SHELL_GUARD_PROMPT` · `SHELL_ENV_KEEP` · `shell_env` · `claude_args` 셸 도구 · 가드 · `[읽기 범위]` 문구 · `run_process_lines(…, env, …)` ·
  `run_claude` 셸이면 `env_clear`), `chat.rs`(압축 요약은 셸 끈 사본).
- `crates/app/src/chat_api.rs`(등록 · 수정 · JSON `shell`), `crates/app/web/index.html`(체크박스 · `tip_agents` 문구) · `app.js`(상세 체크박스 · 경고 줄 · `window.confirm`) ·
  `view.js`(`SHELL_NOTE` · `SHELL_WARN` · `SHELL_ON` · `shellConfirmNeeded` · `READ_ONLY_TOOLS` 4개) · `app.css`(`.hint.shell-on`).
- 시험: `crates/agent/tests/runner.rs`(`shell_adds_tools` · `shell_env_keeps_base_only` · 도구 4개) · `shell_canary.rs`(새, `#[ignore]` 실제 claude) ·
  `crates/core/tests/paths.rs`(셸 세대 · 기본 끔 · 스키마 25) · `crates/app/tests/chat_api.rs`(등록 · PATCH `shell`) · `view.test.mjs`(`shell_confirm` · 도구 넷) · `Agent` 리터럴 · `update` 호출 맞춤.
- 문서 — an_core · an_agent · an_app · 개발자 가이드 · 사용자 가이드 6장(경고 상자) · PRODUCT · 원장(`#1a A-I3` · `#1a A-I8` · `#05-3 셸 위험`) · security-review(도구 4개).

## 설계 결정

- 환경은 셸을 켠 실행만 비운다(원장 `#1a A-I3` 조건 문구). 목록은 `vars_os` 로 읽어 유니코드가 아닌 변수에서도 멈추지 않고, Windows 는 `Path` 처럼 대소문자가 달라도 남긴다.
- `SHELL_TOOLS` 는 `cfg` 로 OS 마다 — macOS 에 없는 `PowerShell` 이름을 넣지 않는다(가드레일 2행 위험 제거).
- canary 는 제품 경로 `run_claude` 로 돈다 — `env_clear` 를 실제 경로에서 잰다. 1차는 비밀 변수가 빈 값이라 통과했지만 모델이 "비밀이라 싣지 않음" 이라 적어 값을 숨긴 것과
  가릴 수 없었다 → 넘기는 목록의 `NO_PROXY` 를 대조군으로 심고 이름을 `AN_CANARY_MARK` 로 바꿔 `[][canary-keep-7733.invalid]` 를 확인.
- `LS` · `NotebookRead` 는 claude 2.1.308 에 없는 도구 이름(`stream-json` init `tools` 실측 — `--tools Read,Grep,Glob,LS,NotebookRead` → `Glob · Grep · Read`).
  `NotebookRead` 만 뺐다(지시 범위), `LS` 는 남기고 확인을 받는다.
- 화면 확인 중 "내 에이전트" 도움말(`tip_agents`)이 "명령 실행 없음" 이라 셸을 켠 에이전트와 어긋나 고쳤다.
- 시험 환경: 이 PC 의 Visual Studio 자동 업데이트(15:13 ~ 15:26)가 MSVC `link.exe` 를 지운 동안 `cargo test` 가 링크에서 실패했다(`LNK1104 libucrt.lib` · Git 의 `link.exe`) — 끝난 뒤 다시 돌려 통과.

## 테스트 결과

```text
cargo test --workspace                                         → rc 0 (81 묶음 모두 ok, failed 0)
cargo clippy --workspace --all-targets -- -D warnings          → Finished
node --test "crates/app/web/*.test.mjs"                        → pass 165 · fail 0
cargo test -p an-agent --test shell_canary -- --ignored        → 켬: "CANARY-SHELL-UP-7731" · "[][canary-keep-7733.invalid]" / 끔: 셸 없음 · 폴더 밖 Read 거부 · 1 passed
cargo test -p an-agent --test restricted_canary -- --ignored   → 1 passed (회귀)
실제 화면(ego-browser, 디버그 빌드 · 시험 데이터 폴더)         → 등록 칸 라벨 · title(SHELL_NOTE) · 셸 체크 → 등록 → 확인 창(confirm, 머리 2줄 + 위험 6줄) 취소 = "등록된 에이전트 없음" ·
                                                                 확인 = 등록 · 상세 체크 + 빨간 경고 줄(rgb(182, 50, 45)) · 켠 채 저장 = 확인 창 없음 · 끄고 저장 = 확인 창 없음 →
                                                                 다시 읽기 끔 · 경고 줄 없음 · 끔 → 켬 저장 = 확인 창 → 취소 시 서버 값 shell=false
```

## 다음 단계

- `python tools/check_gates.py docs/archive/05-3.shell-tool` → `eval.md` 평가 원장 · E7 EVIDENCE.
- 적대적 평가(codex-peer-review, 라운드 상한 2).
