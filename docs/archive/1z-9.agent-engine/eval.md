# eval.md — 1z-9.agent-engine 적대적 평가

## 평가 원장

<!-- tools/check_gates.py 산출 — 손으로 고치지 않는다. 단 CHECK 가 없는 수동 gate 의
     EVIDENCE 와 ABANDON 줄은 사람이 적고, 재실행해도 보존된다.
     WEAK 줄은 기계가 매 실행 다시 단다 — 약한 gate 를 셀 수 있게 하는 표시이며
     판정에는 영향이 없다. -->

측정 커밋: 4444bd7424c31a92cc2dd96cb5f5ef2c756cd3b1

- [x] E1: A7 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s

- [x] E2: A7 — clippy 경고 0
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.26s

- [x] E3: A1·A3 — 웹UI 로직 시험(도구 라벨 · 블록 비활성)
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# fail 0/
  EVIDENCE: # fail 0

- [x] E4: A4·A5·A8 — 실행기 시험(Claude 인자 결정 9 · Codex 인자 · 출력 해석 · 단계 · 없는 세션)
  CHECK: cargo test -p an-agent --test runner
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.88s

- [x] E5: A1·A2·A3·A5·A6 — 에이전트 API(도구 필수 · 못 바꿈 · Codex + Claude 옵션 거부 · Git 스위치 · 기존 행 claude)
  CHECK: cargo test -p coworkagent-app --test chat_api agents_
  EXPECT: /test result: ok\. [2-9] passed; 0 failed/
  EVIDENCE: test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 8 filtered out; finished in 0.14s

- [x] E6: A1·A3 — 화면 원본 literal(1단계 타일 · 2단계 블록 · 칩 · 문구)
  CHECK: cargo test -p coworkagent-app --test ux_literals --test css_tokens
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s

- [x] E7: A7 — 문서 규약 · 크레이트 경계 · 버전 · 매뉴얼 반영
  CHECK: python tools/check_ddd_docs.py && python tools/check_crates.py && python -c "import pathlib as P;r=lambda f:P.Path(f).read_text(encoding='utf-8');print('ver=%d app=%d core=%d dev=%d guide=%d'%(sum(r('crates/%s/Cargo.toml'%c).count('version = \x221.3.5\x22') for c in ('agent','app','core','net')),r('docs/manual/dev/an_app.md').count('engine'),r('docs/manual/dev/an_core.md').count('skip_git'),r('docs/manual/dev/developer-guide.md').count('codex exec'),r('docs/manual/user/user-guide.md').count('Codex')))"
  EXPECT: /ver=4 app=[1-9][0-9]* core=[1-9][0-9]* dev=[1-9][0-9]* guide=[1-9][0-9]*/
  EVIDENCE: ver=4 app=2 core=2 dev=2 guide=10

- [x] E8: A1 ~ A6 · A8 — 실제 화면 · 실제 실행: 디버그 빌드에서 등록 1단계(도구 안 고르면 등록 없음) → Codex 고름 → 2단계 Claude 블록 비활성 · Git 스위치 켬 → 등록 → 상세 칩 · 블록 ·
  EVIDENCE: Claude(구현자) 2026-10-23 — 디버그 빌드 `start --parent <대기 프로세스>` + `COWORKAGENT_DATA_DIR`=scratchpad `ui1z9\data`(빈 데이터), ego-browser 1440 × 900 · 900 × 900 DOM 판독 · 스크린샷.


WEAK: E8 CHECK 없음 — 수동 gate(사람이 적은 EVIDENCE 로 판정)

## 적대적 평가 — 라운드 1/2

평가자: Orca 가 새로 띄운 Codex(`worker-start --agent codex`, 작업 폴더 `C:/Users/tskim/.claude/skills`), handle `term_c1b4be41-c80b-4e33-ade8-ad3ae5c8842e`.
Run `run_c74a1fd939dd` · Task `task_bc6135b73668` · Dispatch `ctx_aa42e09bd0bd`, worker_done(`msg_cb19c3518bf7`) 발신 handle 일치 · outcome succeeded. 범위 `38c2c73..1282fcd`(self-collect),
메타 `요청 1z-9-agent-engine-r1 · revision 1282fcd · 라운드 1/2`. 리뷰 동안 대상 작업 폴더 HEAD 1282fcd · 변경 없음(`git status` 빈 줄). 리뷰어 Dispatch `worker-release` → `released`(원문 transcript `captured`). 원문(render):

```text
# Codex Adversarial Review

Target: branch diff against main
Verdict: needs-attention

38c2c73..1282fcd를 읽기 전용으로 검토했다. Codex 비정상 종료를 성공 답으로 기록할 수 있어 출하 보류 의견이다. 완료 이벤트·종료 상태 검증과 회귀 시험이 남았으며 런타임 재현은 수행하지 않았다.

Findings:
- [medium] Codex 중간 메시지를 비정상 종료 뒤에도 성공 답으로 처리 (crates/agent/src/runner.rs:781-792)
  정적 코드 추론: thread.started와 중간 agent_message를 출력한 뒤 CLI가 크래시·외부 종료로 끝나면 turn.failed/error 없이도 parse_codex는 is_error=false를 반환한다(669행). 기존 runner 테스트 342행도 '먼저 볼게요'를 중간 agent_message로 모델링한다. run_codex는 out.status를 오류 문구에만 쓰고 성공 여부는 검사하지 않으며, 782~783행 필터가 turn.completed까지 버린다. 따라서 process는 미완료 안내문을 State::Done·ok=true로 상대에게 보내고 자동 왕복도 계속하며, 압축 실행이면 이 안내문을 요약으로 채택해 기존 세션을 비울 수 있다. 실제 CLI 크래시 재현은 미실행이다.
  Recommendation: turn.completed를 보존·검증하고 out.status.success()와 정상 완료 이벤트를 모두 만족할 때만 성공 처리하라. thread.started → 중간 agent_message → 비정상 종료 및 완료 이벤트 없는 EOF가 실패로 기록되고 자동 왕복·압축 성공으로 진행되지 않는 회귀 시험을 추가하라.

Next steps:
- Codex 실행의 정상 종료·턴 완료 조건을 함께 검증하고 중도 종료 회귀 시험을 수행한다.
- 수정 후 라운드 2/2에서 지정 revision 범위로 재검토한다.
```

판정 · 조치:

| 지적 | 판정 | 조치 |
|---|---|---|
| R1-1 medium Codex 중간 메시지를 비정상 종료 뒤에도 성공 답으로 처리 | 사실 — `parse_codex` 는 `turn.completed` 를 보지 않았고 `run_codex` 의 줄 거르기가 그 줄을 버렸으며 종료 상태를 성공 판정에 쓰지 않았다(중간 `agent_message` → 끊김 = 성공 답 · 자동 왕복 계속 · 압축 요약 채택 가능) | `turn.completed` 줄을 남기고 없으면 실패("Codex 실행이 끝나지 않았다(turn.completed 없음)"), 종료 코드 ≠ 0 이면 완료 이벤트가 있어도 실패("Codex 비정상 종료(…)"). 시험 `parse_codex_json`(끊김 = 실패) · 새 `codex_abnormal_exit_is_error`(가짜 codex — 정상 · 종료 3 · 완료 없음 3갈래, 실제 프로세스 경로) → runner 17 passed. 실제 `codex_live` 다시 → ok(정상 답 · resume 그대로) · clippy 깨끗 |

## 적대적 평가 — 라운드 2/2

평가자: Orca 가 새로 띄운 Codex(`worker-start --agent codex`, 작업 폴더 `C:/Users/tskim/.claude/skills`), handle `term_664db89d-4c09-4a8a-b0a3-4e4693d88a37`.
Run `run_c74a1fd939dd` · Task `task_412edb09ca83` · Dispatch `ctx_0efa77ebbe6d`, worker_done(`msg_244c8e759655`) 발신 handle 일치 · outcome succeeded. 범위 `38c2c73..336eb81`(self-collect),
메타 `요청 1z-9-agent-engine-r2 · revision 336eb81 · 라운드 2/2`. 리뷰 동안 대상 작업 폴더 HEAD 336eb81 · 변경 없음. 리뷰어 Dispatch `worker-release` → `released`(원문 transcript `captured`). 원문(render):

```text
# Codex Adversarial Review

Target: branch diff against main
Verdict: approve

38c2c73..336eb81을 읽기 전용으로 검토했다. R1-1의 완료 이벤트 보존·누락 시 실패·비정상 종료 시 실패 처리가 저장·자동 왕복·압축 경로까지 연결되며, 추가 출하 차단 결함은 입증하지 못했다. 판단은 정적 검토이며 시험 재실행·macOS·실제 두 PC 왕복 검증은 수행하지 않았다.

No material findings.

Next steps:
- 라운드 2/2로 리뷰 종료한다.
- 미실행 macOS와 실제 두 PC의 Codex 왕복은 별도 인수 검증으로 남긴다.
```

판정 · 조치: 지적 없음(R1-1 수정이 저장 · 자동 왕복 · 압축 경로까지 이어짐을 리뷰어가 확인). 상한 2/2 — 재리뷰 없음. next step 의 "macOS · 실제 두 PC Codex 왕복" 은 기존 조건 대기(19 acceptance · Mac 실기 — D28).

R1-1 반영 · R2 지적 없음.

## 미조치 지적

없음
