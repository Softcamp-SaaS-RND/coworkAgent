# eval.md — 00.skeleton 적대적 평가

## 평가 원장

<!-- tools/check_gates.py 산출 — 손으로 고치지 않는다. 단 CHECK 가 없는 수동 gate 의
     EVIDENCE 와 ABANDON 줄은 사람이 적고, 재실행해도 보존된다.
     WEAK 줄은 기계가 매 실행 다시 단다 — 약한 gate 를 셀 수 있게 하는 표시이며
     판정에는 영향이 없다. -->

측정 커밋: 87bd9989f51e1a77f94344ae1cba5dd20a86df05

- [x] E1: A7 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

- [x] E2: A7 — 우리 코드 경고 0(clippy -D warnings)
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.19s

- [x] E3: A1 — Windows 릴리스 빌드의 실행 파일이 agentNetwork.exe 하나
  CHECK: cargo build --release --workspace && python -c "import glob,os;fs=sorted(os.path.basename(f) for f in glob.glob('target/release/*.exe') if os.path.isfile(f));n=','.join(fs);print(f'exes={len(fs)} names={n}')"
  EXPECT: exes=1 names=agentNetwork.exe
  EVIDENCE: exes=1 names=agentNetwork.exe

- [x] E4: A4 — 로컬 웹UI 경계 6건(loopback_only · no_token_401 · wrong_token_401 · wrong_host_403 · cross_origin_403 · valid_200)
  CHECK: cargo test -p agentnetwork-app --test local_ui
  EXPECT: test result: ok. 6 passed; 0 failed
  EVIDENCE: test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

- [x] E5: A5 — 데이터 경로 2건(layout_created_under_root · db_user_version_1_kept_on_reopen)
  CHECK: cargo test -p an-core --test paths
  EXPECT: test result: ok. 2 passed; 0 failed
  EVIDENCE: test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

- [x] E6: A5 — 로그 회전 경계 4건(limit_minus_1_kept · limit_exact_kept · limit_plus_1_rotated · limit_constant_is_1048576)
  CHECK: cargo test -p an-core --test log_rotate
  EXPECT: test result: ok. 4 passed; 0 failed
  EVIDENCE: test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

- [x] E7: A6 — 인스턴스 잠금 3건(first_is_primary · second_while_held_is_secondary · reacquire_after_release)
  CHECK: cargo test -p an-core --test instance_lock
  EXPECT: test result: ok. 3 passed; 0 failed
  EVIDENCE: test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

- [x] E8: A6 — 핸드오프 2건(valid_token_opens_once · wrong_token_401_opens_none)
  CHECK: cargo test -p agentnetwork-app --test handoff
  EXPECT: test result: ok. 2 passed; 0 failed
  EVIDENCE: test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

- [ ] E9: A2 — 보유한 Mac 에서 universal 빌드 · 실행, 같은 Mac 에서 cargo test --workspace 통과(수동 — 누가·언제 / lipo -archs 출력 / 테스트 결과 줄 / macOS 버전·칩)
  EVIDENCE: pending

- [ ] E10: A3·A5·A6 — Windows 실동작(수동): 빈 폴더에 릴리스 exe 하나만 두고 실행 → 트레이 아이콘 · "열기" 로 상태 페이지(버전 · 데이터 경로 %LOCALAPPDATA%\agentNetwork · DB) · 두 번째 실행은 브라우저만 열고 agentNetwork 프로세스 1개 · exe 폴더 파일 1개 유지 · 데이터 폴더에 db · logs · "종료" 뒤 프로세스 0개
  EVIDENCE: pending

- [ ] E11: A3·A5·A6 — macOS 실동작(수동): E10 과 같은 순서 + Dock 아이콘 없음 + 데이터 경로 ~/Library/Application Support/agentNetwork
  EVIDENCE: pending

- [x] E12: 의존 경계 — 워크스페이스 크레이트 2개 · 내부 간선 1개(app → core) · 위반 0
  CHECK: python tools/check_crates.py
  EXPECT: crates=2 edges=1 violations=0
  EVIDENCE: crates=2 edges=1 violations=0

- [x] E13: A6 — 핸드오프 열기 실패 1건(browser_failure_is_500_not_204)
  CHECK: cargo test -p agentnetwork-app --test handoff_failure
  EXPECT: test result: ok. 1 passed; 0 failed
  EVIDENCE: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

ABANDON: E10 원격 세션이라 화면 확인 불가 — 프로토타입(Phase 1 끝) 인수 Task 에서 수동 테스트 일괄(masterPlan D28, 사용자 지시 2026-10-11)
ABANDON: E11 macOS 환경 없음 — 현재 Windows 에서만 개발·확인 가능. masterPlan D25 의 macOS TODO 로 이관(사용자 지시 2026-10-11)
ABANDON: E9 macOS 환경 없음 — 현재 Windows 에서만 개발·확인 가능. masterPlan D25 의 macOS TODO 로 이관(사용자 지시 2026-10-11)

## 적대적 평가 — 라운드 1/2

- 수단: codex-review 세션 `/codex:adversarial-review --base 46c9c8d --scope branch` · 대상 HEAD `0eaacfb` · 2026-10-11
- Verdict: needs-attention

### 원문

```text
- [high] 브라우저 실행 실패에도 핸드오프가 204 성공을 반환한다 (crates/app/src/server.rs:160-162)
  `api_open`은 `open_browser` 결과를 받을 수 없는 호출 뒤 무조건 204를 반환한다. 운영 callback인 `browser::open`은 `Command::spawn` 오류를 로그만 남기고 삼키며, 두 번째 프로세스는 204를 받으면 성공으로 종료한다. 따라서 `rundll32`/`open` 실행 실패 시 웹UI가 열리지 않아도 A6가 성공한 것처럼 보인다. 현재 테스트도 실패 불가능한 카운터 callback만 사용해 이 경로를 놓친다.
  Recommendation: `open_browser`를 `Result` 반환형으로 바꾸고 실패 시 5xx를 반환하라. 두 번째 프로세스는 확인된 성공에서만 0으로 종료하며, callback 실패를 주입해 비-204와 미성공 종료를 단정하는 테스트를 추가하라.
- [high] Windows 실동작 계약 E10이 아직 미충족이다 (docs/tasks/00.skeleton/eval.md:55-56)
  E9·E11만 사용자 지시로 ABANDON됐고 E10은 미체크 상태이며 `EVIDENCE: pending`이다. Stage 3 기록도 트레이 열기, 두 번째 실행 핸드오프, 종료 동작을 확인하지 않았다고 명시한다. 따라서 Windows의 A3·A5·A6 핵심 경로는 승인된 평가 계약상 아직 검증되지 않았다.
  Recommendation: E10 절차 전체를 실제 릴리스 exe로 수행해 트레이·브라우저·프로세스 수·배포 폴더·데이터 파일·종료를 증명하고 원장에 환경, 시각, 명령과 관측값을 기록한 뒤 ship 판정을 다시 수행하라.
- [medium] 종료 중 잠금을 먼저 풀어 단일 인스턴스가 겹칠 수 있다 (crates/app/src/main.rs:61-67)
  종료 callback은 `instance.json` 삭제와 `drop(lock)`을 먼저 수행한 뒤 최대 1초 동안 runtime을 종료한다. 그 사이 새 프로세스는 Primary가 되어 새 서버를 시작할 수 있지만 기존 axum 서버는 아직 살아 있다. 두 토큰 endpoint와 로그 writer가 겹쳐 A6의 단일 인스턴스 불변식이 깨진다.
  Recommendation: 서버 종료 완료 후 `instance.json`을 정리하고 마지막에 잠금을 해제하라. 종료를 의도적으로 지연한 상태에서 다른 인스턴스가 Primary가 되지 않음을 검증하는 프로세스 수준 경쟁 테스트를 추가하라.
- [medium] 200 응답의 빈·손상 결과가 상태 페이지에서 조용히 실패한다 (crates/app/web/app.js:2-5)
  `res.ok`이면 곧바로 JSON을 파싱하지만 fetch 실패, 빈 200, 손상 JSON을 처리하지 않는다. 파싱 예외에서는 초기 `…`가 그대로 남고, `{}` 같은 성공 응답에서는 필드가 `undefined`로 표시된다. 사용자는 로딩과 실패를 구분할 수 없어 A3의 DB·경로 상태 화면이 장애를 숨긴다.
  Recommendation: fetch·JSON 파싱을 `try/catch`로 감싸고 `version`, `data_dir`, `db`의 존재·타입·비어 있지 않음을 검증하라. 빈/손상 200과 네트워크 실패에서 명시적 오류를 표시하는 브라우저 또는 JS 테스트를 추가하라.
```

### 대조 · 조치

| 지적 | 판단 | 조치 |
|---|---|---|
| 브라우저 실패에도 204 | 사실 — `browser::open` 이 spawn 오류를 삼킴 | `open_browser: Fn() -> io::Result<()>`, 실패 시 500 · 두 번째 프로세스는 5xx 를 재시도 없이 실패 종료(exit 1). 회귀 `tests/handoff_failure.rs::browser_failure_is_500_not_204`(RED: 컴파일 실패 확인) · 반대 방향은 E8 `valid_token_opens_once`(204) |
| E10 미충족 | 사실 — 수동 gate, 사용자 화면 필요 | 코드 조치 없음. 사용자 수동 확인 대기(종결 전 필수) |
| 종료 중 잠금 선해제 | 사실 — 순서가 잠금 해제 → 런타임 종료 | 순서를 런타임 종료 → `instance.json` 삭제 → 잠금 해제로. 프로세스 수준 경쟁 테스트는 넣지 않음 — 트레이 이벤트 루프 안의 직선 3줄이라 테스트 비용 대비 이득이 작다 |
| 상태 UI 조용한 실패 | 사실 | `app.js` 에 try/catch + 세 필드 문자열·비어 있지 않음 검증, 실패 시 "오류: …" 표시. JS 테스트 기반이 없어 테스트 미추가(Phase 1 UI Task 에서 도입 검토) |

## 적대적 평가 — 라운드 2/2 (상한 — 재평가 없음)

- 수단: codex-review 세션 · 같은 base `46c9c8d` · 대상 HEAD `bc06451` · 2026-10-11
- Verdict: needs-attention

### 원문

```text
- [high] 브라우저 프로세스가 실패해도 핸드오프는 204 성공을 반환한다 (crates/app/src/browser.rs:4-6)
  `open()`은 `Command::spawn()` 성공 즉시 `Child`를 버립니다. 이는 프로세스 생성만 확인하며 이후 non-zero exit나 URL dispatch 실패를 관측하지 않습니다([Rust 문서](https://doc.rust-lang.org/std/process/struct.Command.html#method.spawn)). 따라서 `rundll32`/`open`이 실행된 뒤 실패하면 `api_open`은 204, 두 번째 인스턴스는 exit 0으로 끝납니다. 새 테스트도 인위적 `Err` callback만 검사해 실제 spawn-after-failure 경로를 놓칩니다.
  Recommendation: Windows는 `ShellExecuteW` 반환값을 판정하고, macOS는 `open` 종료 상태를 확인하십시오. spawn 후 non-zero로 끝나는 대역을 통해 500과 두 번째 프로세스 실패 종료·무재시도를 함께 검증하십시오.
- [high] bc06451 변경 후 자동 gate 증거가 갱신되지 않았다 (docs/tasks/00.skeleton/eval.md:10)
  평가 원장의 측정 SHA는 `28d5a79`지만 HEAD는 `bc06451`이며 그 사이 `main.rs`, `server.rs`, `browser.rs`, `tray.rs`, `app.js`와 테스트가 변경됐습니다. 따라서 E1/E2/E3/E4/E8의 PASS는 현재 산출물 근거가 아닙니다. 특히 새 `handoff_failure` 테스트는 E8의 정확히 2개 테스트 gate에도 포함되지 않습니다. 현재 재검증도 `python: The term 'python' is not recognized`로 실행하지 못했습니다.
  Recommendation: 사용 가능한 Python 호출형을 확정한 뒤 HEAD에서 `check_gates.py`와 문서 체커를 재실행해 측정 SHA를 `bc06451`로 갱신하십시오. 브라우저 실패와 5xx 무재시도는 E8 확장 또는 별도 gate로 고정하십시오.
- [medium] Windows 릴리스의 초기화 실패가 사용자에게 완전히 숨겨진다 (crates/app/src/main.rs:13-17)
  `run()`이 데이터 경로 생성이나 `FileLogger::install` 전에 실패하면 `log::error!`는 설치된 logger가 없어 버려집니다. Windows 릴리스는 GUI subsystem이라 Explorer 실행 시 `eprintln!`도 보이지 않습니다. ACL·디스크·로그 파일 오류에서 트레이와 로그 모두 없이 종료되어 A5 실패 원인을 복구할 수 없습니다.
  Recommendation: logger 설치 전 실패용 Windows `MessageBox` 또는 Event Log fallback을 추가하고, 쓰기 불가 데이터 경로에서 사용자-visible 오류와 non-zero 종료를 검증하십시오.
```

### 대조 · 조치

| 지적 | 판단 | 조치 |
|---|---|---|
| spawn 뒤 브라우저 실패 204 | 사실 — `rundll32` 종료 코드는 열기 결과가 아니고 `open` 종료 상태를 안 봄 | Windows `ShellExecuteW` 반환값(≤ 32 = 실패) · macOS `open`(그 밖 `xdg-open`) 종료 상태로 판정. 실 OS 실패 경로 테스트는 미추가(아래 미조치) |
| gate 증거 stale | 사실 | 반영 커밋에서 `check_gates.py` 재측정 — 측정 커밋 위 원장. E13(`handoff_failure`) 추가(plan 개정 3). `python` 은 이 PC 의 Git Bash 에서 실행된다(리뷰 세션 PowerShell 경로 문제) |
| 시작 실패 은폐 | 사실 | Windows 릴리스에서 `MessageBoxW`(MB_ICONERROR), 그 밖은 stderr. 자동 테스트는 미추가(아래 미조치) |

## 미조치 지적

| 지적 | 판단 | 사유 |
|---|---|---|
| E10 Windows 실동작 미충족(라운드 1) | 사실 | 수동 gate — 프로토타입(Phase 1 끝) 인수 Task 로 이관(D28). ABANDON 기록 |
| 종료 순서 프로세스 수준 경쟁 테스트(라운드 1) | 사실 | 순서는 고쳤다(런타임 종료 → instance.json → 잠금). 트레이 이벤트 루프 안 직선 3줄이라 프로세스 경쟁 테스트 비용 대비 이득이 작다 |
| 상태 UI JS 테스트(라운드 1) | 사실 | 오류 표시는 고쳤다. JS 테스트 기반이 없음 — Phase 1 UI Task 에서 도입 검토 |
| 실 OS 브라우저 실패 경로 테스트(라운드 2) | 사실 | 판정은 고쳤다. 실패를 일으키려면 테스트가 사용자 데스크톱에 OS 대화상자를 띄운다 — 자동 테스트로 두지 않는다 |
| 시작 실패 메시지 상자 테스트(라운드 2) | 사실 | 메시지 상자는 사람 입력까지 막는다 — 자동 테스트 불가. E10 수동 확인 때 함께 볼 후보 |
