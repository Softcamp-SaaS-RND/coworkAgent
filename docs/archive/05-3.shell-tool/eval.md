# eval.md — 05-3.shell-tool 적대적 평가

## 평가 원장

<!-- tools/check_gates.py 산출 — 손으로 고치지 않는다. 단 CHECK 가 없는 수동 gate 의
     EVIDENCE 와 ABANDON 줄은 사람이 적고, 재실행해도 보존된다.
     WEAK 줄은 기계가 매 실행 다시 단다 — 약한 gate 를 셀 수 있게 하는 표시이며
     판정에는 영향이 없다. -->

측정 커밋: 673b80d7fd5eeab9d25d515b34d40abcba9865be

- [x] E1: A4·A5 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

- [x] E2: A5 — clippy 경고 0
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.26s

- [x] E3: A1·A4 — 실행 인자 · 환경 목록 · 세대 시험
  CHECK: cargo test -p an-agent --test runner shell && cargo test -p an-core --test paths schema12_focus_mention
  EXPECT: /test result: ok\. 2 passed; 0 failed/
  EVIDENCE: test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 10 filtered out; finished in 0.00s

- [x] E4: A2 — 실제 claude(사용량): 켬 = 셸로 위 폴더 파일 읽음 · 심은 변수 안 보임, 끔 = 못 읽음
  CHECK: cargo test -p an-agent --test shell_canary -- --ignored
  EXPECT: /test result: ok\. 1 passed; 0 failed/
  EVIDENCE: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 12.79s

- [x] E5: A3 — API 시험 · 웹UI 로직 시험 · 체크박스 라벨 literal · 확인 창 배선 · 스키마 25 실재
  CHECK: cargo test -p coworkagent-app --test chat_api agents_crud_api && node --test --test-reporter=tap "crates/app/web/*.test.mjs" && python -c "import pathlib as P;h=P.Path('crates/app/web/index.html').read_text(encoding='utf-8');j=P.Path('crates/app/web/app.js').read_text(encoding='utf-8');d=P.Path('crates/core/src/db.rs').read_text(encoding='utf-8');L='\uc178 \uba85\ub839 \uc2e4\ud589 \ud5c8\uc6a9';print('box=%d label=%d warn=%d schema=%d'%(h.count('<input type=\x22checkbox\x22 id=\x22agent_shell\x22'),h.count('<label for='+chr(34)+'agent_shell'+chr(34)+'>'+L+'</label>')+j.count('field('+chr(34)+L+chr(34)),j.count('window.confirm(SHELL_WARN)'),d.count('pub const SCHEMA_VERSION: i64 = 25;')))"
  EXPECT: box=1 label=2 warn=2 schema=1
  EVIDENCE: box=1 label=2 warn=2 schema=1

- [x] E6: A5 — 문서 규약 · 크레이트 경계 · 매뉴얼 · PRODUCT · 원장 반영
  CHECK: python tools/check_ddd_docs.py && python tools/check_crates.py && python -c "import pathlib as P;t=lambda f:P.Path(f).read_text(encoding='utf-8');L='\uc178 \uba85\ub839 \uc2e4\ud589 \ud5c8\uc6a9';print('core=%d agent=%d app=%d dev=%d user=%d product=%d ledger=%d'%(min(1,t('docs/manual/dev/an_core.md').count('SHELL_TOOLS')),min(1,t('docs/manual/dev/an_agent.md').count('shell_env')),min(1,t('docs/manual/dev/an_app.md').count('SHELL_WARN')),min(1,t('docs/manual/dev/developer-guide.md').count('SHELL_GUARD_PROMPT')),min(1,t('docs/manual/user/user-guide.md').count(L)),min(1,t('PRODUCT.md').count(L)),min(1,t('docs/overview/ledger.md').count('#05-3'))))"
  EXPECT: core=1 agent=1 app=1 dev=1 user=1 product=1 ledger=1
  EVIDENCE: core=1 agent=1 app=1 dev=1 user=1 product=1 ledger=1

- [x] E7: A3 — 실제 화면: 디버그 빌드(시험 데이터 폴더)로 등록 화면 셸 체크 → 등록 → 확인 창(취소 = 등록 안 됨 · 확인 = 등록) → 상세에 체크 · 경고 줄 → 끄고 저장(확인 창 없음) → 다시 읽기에 끔
  EVIDENCE: Claude(구현자) 2026-10-22, 커밋 95dcaea 내용의 디버그 빌드 — `COWORKAGENT_DATA_DIR`=scratchpad `uidata` + `coworkAgent start`(사용자 앱 그대로), ego-browser(space 23)로 `#settings/agents` 조작. 관측: 등록 칸 `label[for=agent_shell]` = "셸 명령 실행 허용" · title = `SHELL_NOTE` 전문 → 셸 체크 후 "등록" = `confirm`(머리 2줄 + 위험 6줄 + "켜시겠습니까?") → 취소 = 목록 "등록된 에이전트 없음" → 다시 "등록" · 확인 = 목록에 shelly → 상세 셸 체크 = true · 경고 줄 "켜짐 — 연결된 상대가 …" 빨강(rgb(182, 50, 45), 스크린샷 scratchpad `ego-05-3/detail-on.png`) → 켠 채 "저장" = 확인 창 없음 → 끄고 "저장" = 확인 창 없음 → 새로 고침 뒤 체크 = false · 경고 줄 없음 → 다시 켜고 "저장" = `confirm` → 취소 뒤 `GET /api/agents` = shell false. 끝에 `coworkAgent stop` = "종료함".


WEAK: E7 CHECK 없음 — 수동 gate(사람이 적은 EVIDENCE 로 판정)

## 적대적 평가 — 라운드 1/2

평가자: Orca 가 새로 띄운 Codex(`worker-start --agent codex`, 작업 폴더 `C:/Users/tskim/.claude/skills`), handle `term_ac6b7ff9-5f31-4a45-a462-4bf257bdc5bf`.
Run `run_59ee7c2034b5` · Task `task_71db342b60fe` · Dispatch `ctx_42787336d857`, worker_done(`msg_16c7c33a837a`, outcome succeeded) 발신 handle 일치. 범위 `7c9dbe2..95dcaea`(self-collect),
메타 `요청 05-3-shell-tool · revision 95dcaea41684b98181c3b51141053013f3dfa0dc · 라운드 1/2`(기대값과 같음). 리뷰 중 HEAD 95dcaea 그대로 — 그동안 고친 것은 커밋 밖 문서(plan E5 CHECK · eval E7 EVIDENCE · 도구 정리 착수)뿐이라
고정 범위에 들지 않는다. 리뷰어 Dispatch `worker-release` → `released`(`closed_agent_terminal`). 원문(render — raw JSON 은 scratchpad `review_r1_raw.json`):

```text
# Codex Adversarial Review

Target: branch diff against 7c9dbe2
Verdict: needs-attention

출하 보류: 오래 열린 상세 화면이 셸 권한을 확인 없이 복구하며, 환경 허용 목록은 인증 프록시 비밀번호를 그대로 노출한다. 지정 범위만 정적 검토했으며 런타임 재현·테스트 재실행은 하지 않았다.

Findings:
- [high] 오래 열린 상세 화면에서 저장하면 끈 셸이 확인 없이 다시 켜진다 (crates/app/web/app.js:1429-1430)
  창 A·B가 shell=true인 상세를 연 뒤 A에서 셸을 끄고 저장한다. B에서 추가 지시만 수정해 저장하면 캡처된 a.shell과 체크박스가 모두 true여서 confirm을 생략하고 shell:true를 PATCH한다. update_agent는 이를 그대로 적용하므로 사용자가 해제한 원격 명령 실행 권한이 일반 편집으로 복구된다. loadAgents는 초기 로드와 이 창의 CRUD 뒤에만 호출되며 다른 창의 변경을 반영하지 않는다. 코드 경로로 확인한 시나리오이며 브라우저 런타임 재현은 미실행이다.
  Recommendation: 체크박스를 변경하지 않은 저장에서는 shell을 PATCH에서 생략한다. 명시적 토글에는 조회 당시 버전/uid 조건을 붙여 오래된 편집을 거절하고 최신 상태로 다시 확인하게 한다. 두 창에서 해제 후 다른 필드 저장 시 false가 유지되는 회귀 검증을 추가한다.
- [medium] 인증 프록시 URL의 비밀번호가 셸 환경으로 그대로 전달된다 (crates/agent/src/runner.rs:127-129)
  SHELL_ENV_KEEP는 HTTP_PROXY·HTTPS_PROXY와 소문자 변형을 허용하고 shell_env는 값 검사 없이 보존한다. 따라서 앱 환경이 HTTPS_PROXY=http://user:password@proxy:8080 형태인 경우 env_clear 뒤에도 password가 셸에 남는다. 이는 파일·저장소의 비밀과 별개인 환경 변수 비밀이며, SHELL_WARN의 '앱 환경 변수는 넘기지 않지만(기본 변수만) 파일·저장소 속 비밀은 막지 못합니다' 및 키·토큰을 넘기지 않는다는 코드 설명이 이 예외를 알리지 않는다. 인증 URL을 사용하는 배포에서 발생하는 코드상 노출이며 실제 자격 증명으로 재현하지 않았다.
  Recommendation: 프록시 URL의 userinfo를 검사하고 자격 증명이 든 값을 셸에 그대로 넘기는 정책을 명시적으로 처리한다. 전달이 필요한 경우 확인 창·매뉴얼·위험 원장에 환경 변수 자격 증명도 셸에 노출된다는 예외를 공지하고, 합성 비밀번호가 든 프록시 URL로 검증한다.

Next steps:
- 두 창 편집으로 셸 권한이 되살아나는 경로를 막고 회귀 검증한다.
- 인증 프록시 환경 변수의 처리와 위험 공지를 일치시킨다.
- 수정 후 라운드 2/2에서 고정 revision 범위로 재검토한다.
```

판정 · 조치:

| 지적 | 판정 | 조치 |
|---|---|---|
| R1-1 high 오래 열린 상세에서 저장하면 끈 셸이 확인 없이 다시 켜진다 | 사실 — 상세 저장이 늘 `shell: shell.checked` 를 보내고, 확인 여부는 읽어 온 값(`a.shell`) 기준이라 다른 창에서 끈 값을 덮는다(상위 읽기도 같은 경로) | 상세 저장 본문을 `view.js` `agentPatch(a, form)` 로 — 셸 · 상위 읽기는 읽어 온 값에서 바꿨을 때만 싣는다(서버는 빠진 칸 그대로). 시험 `view.test.mjs` `agent_patch`. 명시 토글의 버전 조건(낙관적 잠금)은 넣지 않음 — 끔 → 켬은 늘 확인 창을 거치고, 켬 → 끔은 위험을 줄이는 쪽 |
| R1-2 medium 인증 프록시 URL 의 비밀번호가 셸 환경으로 전달 | 사실 — `HTTP(S)_PROXY` 는 claude 가 API 에 닿는 데 필요하고 셸은 claude 의 환경을 물려받아 claude 에만 주고 셸에서 뺄 수 없다 | 빼지 않고 공지 — `SHELL_WARN` 항목("프록시 주소에 계정 · 비밀번호가 들어 있으면 셸에 보입니다") · 사용자 가이드 경고 · 원장 `#05-3 셸 위험` · `SHELL_ENV_KEEP` 주석. 시험 `shell_confirm` 에 "프록시" |

같은 커밋에 구현 중 지시("효과 없는 도구 목록은 정리 병행") — `READ_ONLY_TOOLS` 에서 `LS` 도 뺌(3개), `tools_canary`(실제 init `tools` 대조) 추가.
시험: `cargo test --workspace` rc 0(82 묶음 · failed 0) · clippy Finished · `node --test` pass 166 · `tools_canary -- --ignored` → 끔 `["Glob", "Grep", "Read"]` · 켬 `["Bash", "Glob", "Grep", "PowerShell", "Read"]` 1 passed.

## 적대적 평가 — 라운드 2/2

평가자: Orca 가 새로 띄운 Codex(`worker-start --agent codex`, 작업 폴더 `C:/Users/tskim/.claude/skills`), handle `term_f0820c3e-d336-44b1-9d4e-8132b054609d`.
Run `run_59ee7c2034b5` · Task `task_b89a9a02bfb9` · Dispatch `ctx_8504edf1e54b`, worker_done(`msg_9d3880681aef`, outcome succeeded) 발신 handle 일치. 범위 `7c9dbe2..2af3699`(self-collect),
메타 `요청 05-3-shell-tool · revision 2af36991fa2a62e8b73f443e1da79c9a9f416b81 · 라운드 2/2`(기대값과 같음). 리뷰 중 작업 폴더 깨끗 · HEAD 2af3699 그대로. 리뷰어 Dispatch `worker-release` → `released`
(`closed_agent_terminal`). 원문(render — raw JSON 은 scratchpad `review_r2_raw.json`):

```text
# Codex Adversarial Review

Target: branch diff against 7c9dbe2
Verdict: approve

고정 범위의 diff와 결정 1~13, R1 조치를 읽기 전용으로 대조했다. 출하를 막을 실질 결함은 확인하지 못했고, agentPatch와 프록시 위험 공지는 R1 지적을 해소한다. 테스트는 재실행하지 않았으며 macOS 실제 실행 검증은 남아 있다.

No material findings.

Next steps:
- macOS에서 셸 실행·로그인·환경 정리·셸 없는 압축 요약을 실측한다.
- 라운드 2/2로 리뷰를 종료한다.
```

판정: 지적 없음(approve). Next steps 의 macOS 실측은 범위 밖(Phase 4 — 보유한 Mac) → 원장 `#05-3 R2-1` 조건 대기. 라운드 상한 2 도달 — 재리뷰 없이 종료.

## 미조치 지적

| 출처 | 지적 원문 요지 | 판정 | 미조치 사유 |
|---|---|---|---|
| R1-1 권고 일부 | 명시 토글에 조회 당시 버전/uid 조건(낙관적 잠금) | 사실(권고) | 바꾼 칸만 보내기로 지적한 결함(되살림)은 닫힘 — 같은 칸 동시 토글은 마지막 저장이 이김, 끔 → 켬은 늘 확인 창. 원장 `#05-3 R1-1` 수용 |
| R2 next step | macOS 에서 셸 실행 · 로그인 · 환경 정리 · 셸 없는 압축 요약 실측 | 확인 필요 | 이 PC 는 Windows — 원장 `#05-3 R2-1` 조건 대기(Phase 4 Mac 검증) |
