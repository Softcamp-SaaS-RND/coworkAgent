# eval.md — 1a.security-hardening 적대적 평가

## 평가 원장

<!-- tools/check_gates.py 산출 — 손으로 고치지 않는다. 단 CHECK 가 없는 수동 gate 의
     EVIDENCE 와 ABANDON 줄은 사람이 적고, 재실행해도 보존된다.
     WEAK 줄은 기계가 매 실행 다시 단다 — 약한 gate 를 셀 수 있게 하는 표시이며
     판정에는 영향이 없다. -->

측정 커밋: 53eecf3bfe18e2f025e3bed8fe0edc912000f61a

- [x] E1: A12 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

- [x] E2: A12 — 우리 코드 경고 0(clippy -D warnings)
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.26s

- [x] E3: A2·A12 — 화면 로직 node 테스트 35건 이상 · 실패 0(기존 34 + entryToken)
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# pass (3[5-9]|[4-9][0-9])\s+# fail 0/
  EVIDENCE: # skipped 0 / # todo 0 / # duration_ms 80.5863

- [x] E4: A2·A3 — 웹 인증 · 보안 헤더 테스트 11건(공개 경로 · Bearer 만 · 쿠키 401 · events 쿼리 · 다른 API 쿼리 401 · 틀린 진입 401 · Host 403 · Origin 403 · 헤더 3종 + CSP · 401/403 에도 헤더 · 진입에 Set-Cookie 없음)
  CHECK: cargo test -p coworkagent-app --test web_auth
  EXPECT: test result: ok. 11 passed; 0 failed
  EVIDENCE: test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

- [x] E5: A7·A14 — 에이전트 폴더(데이터 루트 · 그 조상 · 그 안 거부, 다른 폴더 허용) · CLI 제어 문자 제거 테스트 5건
  CHECK: cargo test -p coworkagent-app --test hardening
  EXPECT: test result: ok. 5 passed; 0 failed
  EVIDENCE: test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s

- [x] E6: A4·A8·A10 — 실패 공개 문구(시간 초과 · 버전 · 그 밖 · 기록 오류 · 오류 결과(`is_error`) — 상대 쪽 글에 상세 없음 · 내 쪽에 상세) · 메모 상한(개수 · 바이트) · 프롬프트 틀(메모 없음 · 있음 · 구분자 지움) 테스트 10건
  CHECK: cargo test -p an-agent --test hardening
  EXPECT: test result: ok. 10 passed; 0 failed
  EVIDENCE: test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.58s

- [x] E7: A5·A8·A9 — 초대 실패 상한(10 회째 거절 · 다른 IP 정상 수락 · 새 창이면 셈 초기화) · 원격 id 형식(`invalid` · 기록 없음) · Cf 제거 · 거절 사유 정리 테스트 7건
  CHECK: cargo test -p an-net --test remote_input
  EXPECT: test result: ok. 7 passed; 0 failed
  EVIDENCE: test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s

- [x] E8: A1·A14 — 보호 DACL 읽어 보기(항목 2 · 상속 없음 · 하위 파일 상속) · 오염된 잠금 계속 쓰기 테스트 3건 + 원시 잠금 호출 0(대상 13곳 이상이 `sync::lock`)
  CHECK: cargo test -p an-core --test hardening && python -c "import glob,re;fs=glob.glob('crates/*/src/**/*.rs',recursive=True);t=''.join(open(f,encoding='utf-8').read() for f in fs);r=len(re.findall(r'\.lock\(\)\s*\.(?:expect|unwrap)\(',t));h=t.count('sync::lock(');print(f'files={len(fs)} raw={r} helper={h}')"
  EXPECT: /raw=0 helper=(1[3-9]|[2-9][0-9])/
  EVIDENCE: files=31 raw=0 helper=17

- [x] E9: A13 — 릴리스 exe 가 VCRUNTIME 을 가져오지 않고 DependentLoadFlags 0x800(앱을 끄고 다시 빌드 · 실행)
  CHECK: python tools/rebuild_run.py && python tools/pe_imports.py target/release/coworkAgent.exe
  EXPECT: vcruntime=0 dependent_load_flag=0x800
  EVIDENCE: imports=15 vcruntime=0 dependent_load_flag=0x800

- [x] E10: A1 — 이 PC 실데이터(E9 로 새 빌드가 시작한 뒤): `identity.key` · `instance.json` · DB 의 ACL 이 2 항목(사용자 · SYSTEM), 그 밖 0
  CHECK: python -c "import subprocess,os,re;r=os.path.expandvars(r'%LOCALAPPDATA%\coworkAgent');fs=['identity.key','instance.json','coworkAgent.db'];o=[subprocess.run(['icacls',os.path.join(r,f)],capture_output=True,text=True,encoding='mbcs').stdout for f in fs];a=[len(re.findall(r':\(',x)) for x in o];u=os.environ['USERNAME'].lower();x=sum(1 for t in o for l in t.splitlines() for p in [l.split(':(')[0].strip().split(os.sep)[-1].lower()] if ':(' in l and p not in (u,'system'));print(f'files={len(fs)} aces={a} others={x}')"
  EXPECT: files=3 aces=[2, 2, 2] others=0
  EVIDENCE: files=3 aces=[2, 2, 2] others=0

- [x] E11: A11 — `--restricted` canary(실제 claude · 개발 PC): 밖 파일 노출 0 · 거부 기록 있음 · 훅 안 돎 · 대조 실행은 훅 돎
  CHECK: cargo test -p an-agent --test restricted_canary -- --ignored --nocapture
  EXPECT: canary leak=0 denied=1 hook=0 control_hook=1
  EVIDENCE: canary leak=0 denied=1 hook=0 control_hook=1

- [x] E12: A3 — 실화면(헤드리스 Edge · 실행 중 앱): 작업 화면 · HTML 본문 보기 · 설정에서 CSP 위반 · 콘솔 오류 0, 새 탭(토큰 없음)은 다시 열기 안내
  EVIDENCE: 2026-10-13 07:57 · 구현 세션(Claude) · 이 PC 의 E9 릴리스 빌드(444b2f9, 실행 중) · 헤드리스 Edge(playwright-core · msedge 채널) — 진입 `/?t=` 뒤 주소 `/`(토큰 지워짐) · 내 활동 · 저장 · 설정 3 · 실제 대화 3 을 돌며 securitypolicyviolation 0건 · 콘솔 오류는 HTML 본문 시험 iframe 의 `<script>` sandbox 차단 1건(의도) · HTML 본문(sandbox="" + htmlSrcdoc) 인라인 스타일 · data: PNG 그려짐 · 새 문맥(토큰 없음) `/` → `#disconnected` "연결이 끊겼습니다 — 트레이 아이콘 메뉴 '열기' 로 다시 여세요". 상세 [stage3](work/stage3.md). 라운드 2 반영 뒤 `b0e1e13` 빌드(E9 재빌드)에서 같은 스크립트로 다시 — 주소 `/` · 위반 0 · 콘솔 오류 1(같은 sandbox 차단) · 안내 같음


WEAK: E12 CHECK 없음 — 수동 gate(사람이 적은 EVIDENCE 로 판정)

## 적대적 평가 — 라운드 1/2

- 평가자: codex 리뷰 세션 `codex-review`(구현 세션과 분리) · `/codex:adversarial-review --base 35d5758 --scope branch` · 측정 HEAD `f193ae0` · job `review-mv5usu9y-hhhitg`.
- 판정(원문): `needs-attention` — 출하 보류. A4·A1 핵심 보안 계약에 우회 분기가 있고, E11 canary도 부분 회귀를 놓칠 수 있다.

### 지적 원문

- **[high] A4 우회: JSON 실행 오류가 공개 문구를 거치지 않는다** (`crates/agent/src/chat.rs:1377-1394`) — `execute()`가 `Ok(Ran)`을 반환하면 `ran.out.is_error == true`여도 `public=None`으로 둔다. 따라서 Claude가 정상 JSON 형식의 실행 오류를 반환하면 `result` 원문이 그대로 상대에게 전송된다. `runner::parse_output()`은 `is_error=true`도 `Ok(RunOutput)`으로 반환하므로, 경로·인증 상태 같은 오류 상세가 포함되면 A4의 일반화 경계를 우회한다. 권고: `is_error=true`를 `Failure`로 변환해 원문은 로컬 기록에만 남기고 상대에게는 고정 공개 문구를 전송한다. `Ok(RunOutput { is_error: true, text: 상세 오류 })` 회귀 테스트를 E6에 추가한다.
- **[medium] 빈 `ok=true` 응답이 성공으로 확정되고 자동 왕복까지 진행된다** (`crates/agent/src/chat.rs:1212-1266`) — 수신 `Reply` 검사는 최대 길이만 확인해 빈 `text`를 허용한다. 페어링된 상대가 `ok=true, text=""`를 보내면 요청과 답이 `Done`으로 저장되고, 자동 왕복 중이면 빈 답을 다음 실행 입력으로 큐잉한다. 사용자에게는 조용한 빈 성공으로 보이며 왕복 횟수와 Claude 사용량도 소모할 수 있다. 동일한 누락이 `inbound_post`의 일반 agent 답 분기에도 있다. 권고: 성공 응답은 `text.trim().is_empty()`이면 `bad_request` 또는 명시적 실패로 처리한다. `Reply`·`Post` 각각 빈 성공, 빈 실패, 자동 왕복 경계 테스트를 추가한다.
- **[medium] A1 권한 경계가 fail-open이며 Unix 파일 0600도 구현되지 않았다** (`crates/app/src/main.rs:83-89`) — `paths.restrict()` 실패를 경고만 남기고 DB·신원 키·로컬 API 초기화를 계속한다. 원래 취약한 상속 ACL이 그대로인 상황에서도 앱이 정상 서비스되어 키·`instance.json`·DB가 노출될 수 있다. 또한 Unix 구현은 루트에 0700만 적용하고 기존·신규 `instance.json`과 SQLite 파일을 0600으로 맞추지 않아 A1의 macOS 파일 계약도 충족하지 않는다. 권고: 권한 강화 실패 시 민감 파일을 열거나 서버를 시작하지 말고 시작 오류로 종료한다. Unix에서는 기존 민감 파일을 0600으로 교정하고 신규 파일도 생성 시 0600을 보장하며, 파일별 mode 테스트를 E8에 추가한다.
- **[medium] E11 canary가 네 가지 접근 중 한 건의 거절만 요구한다** (`crates/agent/tests/restricted_canary.rs:92-122`) — Read·Grep·Glob·junction Read 네 시도를 한 프롬프트에 넣지만 단정은 `permission_denials >= 1`뿐이다. 일부 도구 또는 junction 경계만 풀려도 다른 한 시도가 거절되면 통과할 수 있다. `CANARY` 미출력 단정도 모델이 GUARD_PROMPT의 비밀 비공개 지시를 따라 읽은 내용을 답하지 않으면 헛통과할 수 있어, A11의 작업 폴더 밖 Read 거부를 결정적으로 증명하지 못한다. 권고: 각 접근을 별도 실행하고 해당 도구·대상 경로의 denial을 각각 단정한다. unrestricted 대조에서도 각 canary가 실제로 읽히는지 확인해 검사 민감도를 입증한다.

### 대조 · 처리

| 지적 | plan 항목 | 판단 | 처리 |
|---|---|---|---|
| high · `is_error` 결과 원문이 상대로 | A4 · 해석 5 · E6 | 사실 — 해석 5 가 그대로 보내기로 정했으나 claude 오류 결과에 인증 · 사용량 · 경로가 실릴 수 있다 | plan 개정 1: 상대에게 정해진 문구, 원문은 내 쪽 · `error_result_hides_detail`(E6 10건) |
| medium · 빈 `ok=true` 답이 조용한 성공 · 자동 왕복 빈 입력 | (A 밖 — 정확성) | 사실 | 보내는 쪽은 빈 답을 실패 문구로, 받는 쪽은 빈 에이전트 답 `Reply` · `Post` 를 `bad_request` · `roundtrip` `empty_answer_is_failure_and_refused` |
| medium · A1 fail-open · Unix 파일 0600 없음 | A1 · 해석 11 | Unix 0600 은 사실 · fail-open 은 해석 11 의 결정 | Unix: 루트 · 하위 폴더 0700 + 파일 0600, `instance.json` 0600 으로 쓰기(이 PC 에서 컴파일 못 함 — D25 Phase 4 확인). fail-open 은 유지 — 반박 근거: 실패 상태 = 1a 이전과 같은 상속 권한이고, 리디렉션된 프로필 · 권한을 못 바꾸는 파일 시스템에서 앱 전체가 멈추는 것이 더 나쁘다(한 PC 한 사용자 전제). 실패는 로그 경고 |
| medium · canary 가 네 시도 중 거부 1건만 요구 · 헛통과 가능 | A11 · E11 | 사실 | plan 개정 1: 시도마다 따로 실행 · 그 도구의 거부 단정 · 대조 실행이 밖 파일을 실제로 읽어야(`control_read=1`). 파일 이름 `secret.txt` 는 가드 때문에 대조에서도 안 읽혀(실측) `notes.txt` 로 |

라운드 1 반영 뒤 `check_gates` 재측정(`733ef68`) — 12건 충족.

## 적대적 평가 — 라운드 2/2

- 평가자: `codex-review`(분리) · 같은 명령 `--base 35d5758` · 측정 HEAD `733ef68` · job `review-mv5vi723-8mjjgq`. 라운드 상한(2/2) — 반영 뒤 재리뷰하지 않는다.
- 판정(원문): `needs-attention` — 출하 보류. `is_error` 공개 문구와 빈 답 거절은 닫혔지만, canary와 Unix 0600 수정은 헛통과/불완전하다. A14도 gate가 놓친 원시 잠금이 남았다.

### 지적 원문

- **[medium] junction canary가 링크를 만들지 않아 항상 헛통과한다** (`crates/agent/tests/restricted_canary.rs:75-76`) — `linked` 경로만 계산하고 junction/symlink를 생성하지 않는다. 따라서 해당 Read는 존재하지 않는 파일을 대상으로 실행된다. 더구나 `via_link`이면 실제 `permission_denials` 없이도 `denied=true`로 강제한다. E11은 junction 경계가 완전히 풀려도 통과하며, 라운드 1 지적이 만든 새 회귀다. 권고: Windows `mklink /J`, Unix `symlink` 생성을 복구하고 생성 실패 시 테스트를 실패시킨다. linked 경로의 존재/대상도 단정하고, unrestricted 대조에서 linked 경로를 실제로 읽어 `CANARY`가 노출되는지 확인한다.
- **[medium] 신규 Unix DB 파일은 0600 교정 이후 생성된다** (`crates/app/src/main.rs:83-89`) — 권한 교정은 먼저 실행되지만 DB는 그 뒤 `db::open`으로 생성된다. SQLite 생성 경로에는 0600 지정이나 후속 chmod가 없어 일반적인 umask 022에서는 DB가 0644가 될 수 있다. 부모 0700이 현재 접근을 막더라도 A1·개정 1의 파일별 0600 계약은 충족하지 않으며, 파일 보호가 부모 권한에만 의존한다. 회전 후 새 로그 등 이후 생성 파일도 같은 문제를 가진다. 권고: DB·마이그레이션 사본·회전 로그 등 민감 파일 생성 시 Unix 0600을 명시하거나 생성 직후 교정한다. 실제 앱 생성 순서와 로그 회전을 거친 뒤 파일 mode를 검사하는 Unix E8 테스트를 추가한다.
- **[medium] A14가 요구한 원시 poisoned Mutex 경로가 남아 있다** (`crates/agent/src/chat.rs:974-981`) — `outbound`는 `std::sync::Mutex`인데 여전히 여러 줄의 `.lock().expect(...)`를 사용한다. 잠금이 한 번 poison되면 이후 보내기 경로가 다시 panic한다. E8의 문자열 검사는 `.lock().expect(`처럼 붙은 형태만 세어 이 코드를 놓치면서 `raw=0`으로 잘못 판정한다. 권고: `an_core::sync::lock(&self.outbound)`로 교체한다. E8 검사를 공백·줄바꿈을 허용하는 정규식 또는 구문 기반 검사로 바꾸고, 해당 잠금의 poison 복구 회귀 테스트를 추가한다.

### 대조 · 처리

| 지적 | plan 항목 | 판단 | 처리 |
|---|---|---|---|
| medium · junction canary 가 링크를 만들지 않아 헛통과(라운드 1 수정이 만든 회귀) | A11 · E11 · 개정 1 | 사실 — 라운드 1 재작성 때 `mklink` 블록이 잘려 나갔고 `via_link` 가 거부를 강제했다 | 링크 생성 복구 · 만들지 못하면 실패 · 링크로 밖 파일이 보이는지 단정 · 링크 시도도 그 도구의 거부 단정 · 대조가 직접 · 링크 둘 다 읽어야 `control_read=1`. 실측: 네 시도 모두 `denied=1 leak=0`, 대조 직접 · 링크 모두 읽힘 |
| medium · Unix 새 DB · 사본 · 회전 로그가 0600 교정 뒤에 생긴다 | A1 · 개정 1 | 사실 | `main` 첫머리 Unix `umask(0o077)` — 이 프로세스가 만드는 파일 0600 · 폴더 0700(`libc` 0.2.190, lock 에 있던 판). 이 PC(Windows)에서 컴파일 · 실행 못 함 — 미조치 표 |
| medium · `outbound` 잠금이 여러 줄 `.lock()` `.expect(` 로 남아 E8 검사를 빠져나감 | A14 · E8 | 사실 | `sync::lock` 으로 · E8 CHECK 를 공백 · 줄바꿈을 건너는 정규식으로 보정(EXPECT 그대로 — 고치기 전 커밋 `733ef68` 의 `chat.rs` 에서 1건을 잡는 것을 확인) · 이 잠금 전용 오염 회귀 테스트는 두지 않았다 — 미조치 표 |

## 미조치 지적

| 지적 | 판단 | 사유 |
|---|---|---|
| R1 · A1 권한 맞추기 실패 시 시작을 계속한다(fail-open) | 사실 · 설계 선택(plan 해석 11) | 실패 상태 = 1a 이전 상속 권한과 같다. 리디렉션된 프로필 · 권한을 못 바꾸는 파일 시스템에서 앱 전체가 멈추는 쪽이 더 나쁘다(한 PC 한 사용자 전제) — 로그 경고 |
| R1 · R2 · Unix 0700/0600 · `umask(0o077)` · `instance.json` 0600 을 컴파일 · 실행으로 확인하지 못함 | 사실 | 이 PC 는 Windows 만 빌드한다(Linux · macOS 대상 std 없음) — Phase 4 macOS(D25) 실동작 확인 |
| R2 · `outbound` 잠금 전용 오염 회귀 테스트 없음 | 사실 | `sync::lock` 단위 테스트(오염 뒤 이어 쓰기) + E8 정적 검사(원시 호출 0 · 도우미 17)로 덮는다 — 잠금마다 panic 주입 고리는 비용 대비 이득 작음 |
| Stage 1 리뷰 L · 주소창 토큰 지우기가 클라이언트 `replaceState` 에 기댄다(스크립트가 안 돌면 남음) | 사실 | 스크립트 없이는 웹UI 자체가 돌지 않는다 · `no-referrer` · `no-store` 로 노출 제한 |
| Stage 1 리뷰 L · `RUSTFLAGS` 환경 변수가 `.cargo/config.toml` rustflags 를 통째로 무시시킨다 | 사실 | E9 가 릴리스 산출물의 import 표를 직접 잰다 · 개발자 매뉴얼에 적었다 |
| Stage 2 리뷰 L · 연결이 끊긴 매핑 네트워크 드라이브(`Z:`)는 UNC 판정을 빠져나간다 | 사실 | 그 폴더는 실행도 실패한다 — 영향 작음 |
| Stage 2 리뷰 L · Cf 밖 화면 위장 문자(U+2028/2029 · 한글 채움 문자)는 이름에 남는다 | 사실 · 명세(Cf) 밖 | 웹UI 는 `textContent` 로 그려 줄바꿈 효과가 없다 · 표시 위장만 — 필요해지면 목록을 넓힌다 |
| 점검 S12 · 받는 쪽이 건 자동 왕복 멈춤을 연 쪽이 다시 켤 수 있다 | 사실 · 제외(intent 결정 6) | D24 동작 설계 — 확실한 차단은 집중 · 연결 삭제 |
| 점검 I1 · 피어 포트 `0.0.0.0` 수신 · 방화벽 규칙은 매뉴얼 안내뿐 | 사실 · 제외(intent) | 관리자 권한이 필요해 사용자 권한 단일 실행 파일과 맞지 않는다 |
| 점검 I3 · `claude -p` 가 앱 환경 변수를 그대로 물려받는다 | 사실 | 탐색기로 띄운 앱은 깨끗한 환경(`rebuild_run.py`) · `--restricted` 로 셸 도구가 없다 — 셸에서 띄우면 그 셸 비밀이 섞인다 |
| 점검 I4 · 원격 문의 세션 기록을 사용자가 그 폴더에서 `--resume` 하면 가드 없이 이어받을 수 있다 | 확인 필요 | Claude Code 동작 — 실측 전 |
| 점검 I5 · 마이그레이션 `.vN.bak` 영구 보존(지운 대화가 사본에 남는다) | 사실 | 데이터 루트 소유자 전용 권한 안 · 보존 정책은 사용자 결정 대상 |
| 점검 I7 · `cargo audit`(RUSTSEC 대조) 미실행 | 사실 | 이 PC 에 도구 없음 — 배포(S6) 전 |
| 점검 I8 · 허용 도구 이름 `LS` · `NotebookRead` 가 현행 CLI 에 있는지 | 확인 필요 | 실측 전 |
