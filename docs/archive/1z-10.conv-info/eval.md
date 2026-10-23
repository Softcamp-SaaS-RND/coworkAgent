# eval.md — 1z-10.conv-info 적대적 평가

## 평가 원장

<!-- tools/check_gates.py 산출 — 손으로 고치지 않는다. 단 CHECK 가 없는 수동 gate 의
     EVIDENCE 와 ABANDON 줄은 사람이 적고, 재실행해도 보존된다.
     WEAK 줄은 기계가 매 실행 다시 단다 — 약한 gate 를 셀 수 있게 하는 표시이며
     판정에는 영향이 없다. -->

측정 커밋: 34320922a878e155ccc9b1104044130a449bf3b7

- [x] E1: A5 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

- [x] E2: A5 — clippy 경고 0
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.28s

- [x] E3: A1 — 웹UI 로직 시험(대화 정보 줄 · 받은 / 보낸 / 공유)
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# fail 0/
  EVIDENCE: # fail 0

- [x] E4: A2 — 실행 명령(따옴표 · 한 곳에서 만든 인자 · 대화 실행이 남김 · 다시 실행 · 실패)
  CHECK: cargo test -p an-agent --test runner --test conv_cmd
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.15s

- [x] E5: A1·A3 — 대화 정보 API(in · out · shared · 404 · 목록에 명령 없음)
  CHECK: cargo test -p coworkagent-app --test chat_api conv_info
  EXPECT: /test result: ok\. [1-9] passed; 0 failed/
  EVIDENCE: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 10 filtered out; finished in 0.31s

- [x] E6: A1·A4·A6 — 화면 원본 literal(대화 정보 · 아이콘 자리 · 설정 제목 강조 · 단추 줄 규칙) · CSS 토큰
  CHECK: cargo test -p coworkagent-app --test ux_literals --test css_tokens
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s

- [x] E7: A5 — 문서 규약 · 크레이트 경계 · 스키마 · 매뉴얼 반영
  CHECK: python tools/check_ddd_docs.py && python tools/check_crates.py && python -c "import pathlib as P;r=lambda f:P.Path(f).read_text(encoding='utf-8');print('schema=%d app=%d core=%d web=%d guide=%d'%(r('crates/core/src/db.rs').count('SCHEMA_VERSION: i64 = 28'),r('docs/manual/dev/an_app.md').count('/info'),r('docs/manual/dev/an_core.md').count('cmd_at'),r('docs/manual/dev/web-ui.md').count('info_rows'),r('docs/manual/user/user-guide.md').count('실행 명령')))"
  EXPECT: /schema=1 app=[1-9][0-9]* core=[1-9][0-9]* web=[1-9][0-9]* guide=[1-9][0-9]*/
  EVIDENCE: schema=1 app=1 core=2 web=1 guide=2

- [x] E8: A1 · A4 · A6 — 실제 화면: 디버그 빌드 + 빈 데이터 · ego-browser 1440 × 900 — 받은 대화 세부정보 대화 정보 줄 · 실행 명령 복사(클립보드 = 저장값), 보낸 대화 = 기간 · 컨텍스트,
  EVIDENCE: Claude(구현자) 2026-10-23 — 디버그 빌드 + scratchpad `ui1z8\data`(스키마 28), ego-browser DOM 판독(창 최소화 — 스크린샷 없음). 받은 대화 = 기간 · 답하는 도구 · 읽는 폴더 ·


WEAK: E8 CHECK 없음 — 수동 gate(사람이 적은 EVIDENCE 로 판정)

## 적대적 평가 — 라운드 1/2

평가자: Orca 가 새로 띄운 Codex(`worker-start --agent codex`, 작업 폴더 `C:/Users/tskim/.claude/skills`), handle `term_fd02aefe-747c-43c8-b243-61c7e27fc37f`.
Run `run_2a4e25e76499` · Task `task_4ada85803a54` · Dispatch `ctx_2048bf4f6913`(시작 receipt `outcome_unknown` · `turn_start_unobserved` — 다시 띄우지 않고 기다림), worker_done(`msg_158e9a8cc470`) 발신 handle 일치 · outcome succeeded.
범위 `6594e46..d5ca5da`(self-collect), 메타 `요청 1z-10-conv-info-r1 · revision d5ca5da · 라운드 1/2`. 리뷰 동안 대상 작업 폴더 HEAD d5ca5da · 변경 없음(`git status` 빈 줄). 리뷰어 Dispatch `worker-release` → `released`(원문 transcript `captured`). 원문(render):

```text
# Codex Adversarial Review

Target: branch diff against main
Verdict: needs-attention

6594e46..d5ca5da의 명세·diff를 읽기 전용으로 검토했다. PowerShell 명령 주입·인자 손실과 /info 실패 후 재시도 누락을 재현해 출하 보류를 권고한다. 세 결함 수정과 셸별 argv 회귀 검증이 남았으며 실제 CLI·화면·두 PC 연동은 미실행이다.

Findings:
- [high] PowerShell 스마트 작은따옴표가 인용을 탈출한다 (crates/agent/src/runner.rs:648-656)
  ASCII '만 두 배로 바꾸지만 PowerShell은 U+2018/U+2019도 작은따옴표 구분자로 처리한다. 따라서 추가 지시(agent.prompt)나 경로에 don’t 같은 문자가 있으면 복사 명령이 깨지며, `prefix’; Write-Output QUOTE_INJECTION; #`를 인자로 넣으면 인용 밖 명령이 실행된다. shell_line과 같은 인용 결과를 PowerShell 7.6.1에서 실행해 node가 ["prefix"]만 받고 QUOTE_INJECTION이 별도 출력됨을 확인했다. 원격 본문 유입을 주장하는 지적은 아니며, 로컬 설정 문자열을 명령으로 복사하는 과정의 주입 결함이다. 실행 파일 경로 인용도 같은 문제가 있다.
  Recommendation: PowerShell이 구분자로 인식하는 ASCII·스마트 작은따옴표를 모두 이스케이프하고, U+2018/U+2019와 세미콜론을 포함한 인자·실행 경로의 실제 파싱 및 argv 보존 시험을 추가한다.
- [medium] Windows PowerShell 5.1에서 Codex 설정 인자의 큰따옴표가 사라진다 (crates/agent/src/runner.rs:648-649)
  작은따옴표로 감싸는 것만으로는 Legacy 네이티브 인자 전달의 큰따옴표 제거를 막지 못한다. Windows PowerShell 5.1.26100.9711에서 출력 형식과 같은 호출로 `'sandbox_mode="read-only"'`, `'developer_instructions="hello world"'`, `''`를 node에 전달했더니 각각 `sandbox_mode=read-only`, `developer_instructions=hello world`로 바뀌고 빈 인자는 사라졌다. codex_args는 실제로 이 TOML 문자열 인자를 사용하므로 기본 Windows PowerShell에 붙여 넣으면 원래 실행과 달라져 Codex 설정 파싱 실패로 이어진다(실제 Codex 실행은 미검증). 기존 시험은 완성 문자열만 비교해 이 손실을 잡지 못한다.
  Recommendation: PowerShell 5.1/Legacy에서도 argv를 보존하는 실행 문자열을 만들거나 보장하는 셸 버전·전달 모드를 명확히 지정하고 강제한다. 실제 네이티브 argv 캡처로 Codex -c 인자·내부 큰따옴표·빈 인자를 왕복 검증한다.
- [medium] /info 조회 실패가 같은 대화의 후속 재시도를 차단한다 (crates/app/web/app.js:2965-2975)
  GET 성공 전에 infoView.key를 확정하고 catch에서 해제하지 않는다. 일시적인 네트워크·서버 실패가 한 번 나면 글 수/마지막 id/상태가 그대로인 대화에서는 5초 갱신이나 상세 재열기로도 다시 조회하지 않아 오류 또는 이전 실행 명령이 계속 남는다. 해당 revision의 renderInfo를 Node VM으로 추출해 첫 GET을 실패시키고 같은 대화를 세 번 렌더링했을 때 infoRequests=1을 확인했다. 새 글이나 다른 대화 이동 없이 대화 정보와 복사 기능을 복구할 수 없다.
  Recommendation: 진행 중 요청 키와 마지막 성공 키를 분리하거나 현재 요청 실패 시 키를 무효화하여 다음 갱신에서 재시도한다. 같은 대화에서 실패→성공, 이전 응답 역전 시나리오를 시험한다.

Next steps:
- PowerShell 스마트 따옴표 주입 차단 후 5.1·7.x에서 실제 argv를 원본과 비교한다.
- /info 일시 실패 후 같은 대화의 자동 복구를 검증한다.
```

판정 · 조치:

| 지적 | 판정 | 조치 |
|---|---|---|
| R1-1 high PowerShell 둥근 작은따옴표 인용 탈출 | 사실 — `shell_line` 이 ASCII `'` 만 두 번으로 바꿈(PowerShell 은 ‘ ’ ‚ ‛ 도 작은따옴표) | `PS_QUOTES`(` ' ‘ ’ ‚ ‛ `) 를 각각 두 번 — 인자 · 실행 파일 경로 모두. 시험 `launch_line_matches_spawn_args` 에 둥근 따옴표 · `;` 주입 문자열, 새 `powershell_line_roundtrip`(실제 pwsh 7 이 node 에 넘긴 argv = 원래 인자 · `INJECTED` 따로 출력 없음) |
| R1-2 medium Windows PowerShell 5.1 에서 큰따옴표 · 빈 인자 손실 | 사실 — 옛 네이티브 인자 전달(5.1)은 작은따옴표로 감싸도 안 `"` 를 지우고 빈 인자를 버린다. 5.1 과 7.3+ 를 함께 만족하는 한 줄은 없다(5.1 용 `\"` 는 7.3+ 에서 글자 그대로, `--%` 는 줄바꿈 든 Claude 가드 인자를 못 실음) | 대상 셸을 **PowerShell 7.3 이상**으로 정하고 강제 — 명령 앞 `PS_MIN_GUARD`(7.3 아래면 `throw` — 실행 파일을 띄우지 않음). `powershell_line_roundtrip` 이 pwsh 7 = 큰따옴표 · 빈 인자 · 줄바꿈 · `$` `@` `%` 그대로, Windows PowerShell 5.1 = 실패 · node 출력 없음 · 오류에 "7.3" 을 잰다. 안내 문구 `INFO_CMD_HINT` · 사용자 가이드 · 개발자 가이드에 "Windows = PowerShell 7.3 이상(pwsh)" |
| R1-3 medium /info 실패 뒤 같은 대화 다시 읽기 없음 | 사실 — 실패해도 `infoView.key` 가 남아 같은 키에선 다시 부르지 않음 | 실패면 `infoView.key = ""`(다음 그리기 = 5초 새로 고침 · 다시 열기에서 다시 읽음) · 오류 줄은 `dataset.sig` 를 비워 다음 응답이 덮음. 늦은 응답 역전은 기존 키 비교가 버림. `ux_literals` 에 행 |

검증(R1 조치 뒤): `cargo test -p an-agent --test runner` → `ok. 19 passed`(실제 pwsh 7 · Windows PowerShell 5.1 · node 왕복 포함) · `cargo test --workspace --no-fail-fast` → FAILED 0줄 · clippy Finished · `node --test` 172 pass 0 fail.

## 적대적 평가 — 라운드 2/2

평가자: Orca 가 새로 띄운 Codex(`worker-start --agent codex`, 작업 폴더 `C:/Users/tskim/.claude/skills`), handle `term_563f767c-225f-48e6-9a57-dd4e8f957b8a`.
Run `run_2a4e25e76499` · Task `task_8d978c6e153c` · Dispatch `ctx_174a85fc20bd`(시작 `input_accepted`), heartbeat(`phase: reviewing`) 1건 ack, worker_done 발신 handle 일치 · outcome succeeded.
범위 `6594e46..3261f40`(self-collect, R1 조치 먼저 검증), 메타 `요청 1z-10-conv-info-r2 · revision 3261f40 · 라운드 2/2`. 리뷰 동안 대상 작업 폴더 HEAD 3261f40 · 변경 없음. 리뷰어 Dispatch `worker-release` → `released`(원문 transcript `captured`). 원문(render):

```text
# Codex Adversarial Review

Target: branch diff against main
Verdict: needs-attention

6594e46..3261f40를 읽기 전용 검토하고 R1 조치를 재검증했다. 따옴표 처리와 실패 후 재시도는 개선됐지만 PowerShell Legacy 인자 손실과 /info 응답 역전을 재현해 출하 보류를 권고한다. 두 결함 조치가 남았으며 실제 터미널 붙여 넣기·CLI 실행·macOS·두 PC 연동은 검증하지 않았다.

Findings:
- [medium] PowerShell 7에서도 Legacy 모드는 guard를 통과해 인자를 손실한다 (crates/agent/src/runner.rs:642-644)
  PS_MIN_GUARD는 버전만 검사하므로 프로필 등에서 $PSNativeCommandArgumentPassing='Legacy'로 설정한 PowerShell 7.3+를 허용한다. 동일 guard와 인용 결과를 실제 pwsh 7.6.1에서 node.exe로 전달했을 때 Windows/Standard 모드는 보존됐지만 Legacy에서는 sandbox_mode="read-only"가 sandbox_mode=read-only로 바뀌고 빈 인자가 사라졌다. codex_args가 실제 사용하는 TOML 문자열도 손상되므로 복사 명령이 원래 실행을 재현하지 못한다. powershell_line_roundtrip은 -NoProfile -File의 기본 모드만 검사해 이 경로를 놓친다(실제 Codex 실행은 미검증).
  Recommendation: 버전과 함께 실제 인자 전달 모드도 검사해 Legacy이면 실행 전에 중단하거나, 국소 범위에서 보존 모드를 강제한다. Windows/Standard/Legacy 각각에서 .exe 대상 argv 왕복을 검증한다.
- [medium] A→B→A 이동 시 오래된 /info 응답이 최신 값을 덮는다 (crates/app/web/app.js:2964-2975)
  응답 유효성을 요청 세대 대신 [대화 id, 글 수, 마지막 id, 상태] 키로만 판별한다. A의 첫 조회가 대기 중일 때 B를 거쳐 같은 메시지 상태의 A로 돌아오면 새 A 조회도 동일 키를 갖는다. revision의 renderInfo를 Node VM으로 추출해 새 A 응답 NEW를 먼저 완료하고 옛 A 응답 OLD를 나중 완료하자 최종 infoView.data가 OLD로 역전됐다. 실행 중 세션 재시도로 cmd가 바뀌거나 에이전트 설정이 변경되면 오래된 명령·폴더를 표시하고 복사할 수 있으며, 키가 그대로라 후속 갱신에서도 복구되지 않는다. 이는 함수 수준 재현이며 실제 브라우저 재현은 미실행이다.
  Recommendation: 조회마다 증가하는 generation 또는 고유 요청 토큰을 발급하고 then/catch 모두 현재 토큰과 비교한다. 실패→성공 외에 A→B→A 뒤 성공/실패 응답 역순 완료 회귀 시험을 추가한다.

Next steps:
- Legacy 모드에서도 argv를 보존하거나 실행 전 차단하도록 보완한다.
- /info 요청 세대를 분리해 ABA 응답 역전을 차단한다.
- 라운드 2/2에서 종료하고 잔여 지적을 coordinator가 기록한다.
```

판정 · 조치(상한 2/2 — 고친 뒤 재리뷰 없음):

| 지적 | 판정 | 조치 |
|---|---|---|
| R2-1 medium 7.3+ `Legacy` 인자 전달이 guard 를 통과 | 사실 — guard 가 버전만 봄 | `PS_MIN_GUARD` 에 `-or $PSNativeCommandArgumentPassing -eq 'Legacy'`. `powershell_line_roundtrip` 에 실제 pwsh 7 `Legacy`(멈춤 · node 출력 없음 · 오류에 "Legacy") · `Standard`(argv 그대로) 추가 — 기본(`Windows`) · 5.1 은 R1 대로 |
| R2-2 medium A → B → A 이동 때 늦은 /info 응답 역전 | 사실 — 응답 유효성을 키로만 봄(같은 키 재요청) | 읽기마다 `infoView.gen` 세대 · then / catch 모두 마지막 세대만 반영. `ux_literals` 행. 브라우저 회귀 시험(늦은 응답 순서 조작)은 없음 — 아래 미조치 |

검증(R2 조치 뒤): `cargo test -p an-agent --test runner` → `ok. 19 passed`(pwsh 7 기본 · Standard · Legacy · Windows PowerShell 5.1 · node 왕복) · `cargo test --workspace --no-fail-fast` → FAILED 0줄 · clippy Finished · `node --test` 172 pass 0 fail.

## 미조치 지적

| 지적 원문 | 판단 | 미조치 사유 |
|---|---|---|
| R2-2 Recommendation 중 "A→B→A 뒤 성공/실패 응답 역순 완료 회귀 시험을 추가한다." | 사실(시험 공백) — 수정(세대 토큰)은 반영, 그 동작을 재는 자동 시험은 문자열 확인(`ux_literals`)뿐 | `renderInfo` 는 DOM · `api` 에 묶인 `app.js` 안 함수라 node 시험 틀(`view.test.mjs` = DOM 없는 `view.js`)로 재려면 함수를 `view.js` 로 빼는 구조 변경이 필요 — 라운드 상한 2/2 에 닿아 재리뷰 없이 고칠 범위가 아님. 사용자 결정 대기 |

