# plan — 1e.autostart-watchdog

> 근거: [intent](intent.md)(승인 2026-10-13 — 라운드 2). **구현자는 이 문서만 보고 일한다.** D·S 는 [masterPlan](../../masterPlan.md).
> 기준 커밋: `974e5eb`(`task/1e-autostart-watchdog` — 분기점 `d58b291` main). plan 승인: 2026-10-13 — 사용자 원문 "의견 수용.  진행해"(mac 등록 이름 `com.coworkagent` 그대로 · E7 실측은 이 PC 에서 — 실행 중인 앱이 있으면 끄지 않고 19 TC 로 넘김).

## 목표

인자 없이 실행하면 작은 **감시 프로세스**가 지금의 앱(트레이 + 수신 · 실행 한 몸, 구조 그대로)을 자식으로 띄우고, 앱이 비정상 종료되면 다시
띄운다(1분 안에 3번이면 멈추고 오류 트레이). 앱은 시작할 때 **자동 시작**(Windows `HKCU\…\Run` · macOS LaunchAgent)을 등록 · 경로 고침하고,
웹UI 설정에서 끌 수 있다. 패닉은 메시지 · 위치를 로그에 남긴다.

### 성공 기준(intent 사본)

- A1: 인자 없이 실행하면 감시 프로세스가 앱을 띄우고, 앱을 강제 종료하면 다시 띄워 남은 대기열 · outbox 가 이어 처리된다.
- A2: 앱이 1분 안에 3번 비정상 종료되면 다시 띄우기를 멈추고 트레이 오류 모드(사유 · 로그 위치)가 보인다.
- A3: 트레이 "종료" 는 감시 프로세스까지 끝내고 다시 띄우지 않는다. 이미 실행 중일 때 실행하면 "열기" 를 넘기고 새 감시 프로세스도 남지 않는다.
- A4: 앱이 시작하면 자동 시작이 등록되고(Windows `HKCU\…\Run`), 실행 파일을 옮긴 뒤 실행하면 새 경로로 고쳐진다. 웹UI 설정에서 끄면 등록이
  지워지고 다음 실행에도 다시 등록하지 않는다. macOS LaunchAgent 는 Phase 4 Mac 수동 gate.
- A5: 패닉이 나면 메시지 · 위치가 로그에 남는다.
- A6: CLI 하위 명령은 감시 프로세스를 거치지 않는다(지금 그대로).
- A7: 루트 `PROJECT.md` 검증 명령이 통과한다.

## 코드 사실(실측)

- `main()` — 인자가 있으면 `cli::main` 으로 끝(`main.rs`), 없으면 `Tray::new` → `start()` → `tray.run` / `run_error`.
- tao `ControlFlow::Exit` = `ExitWithCode(0)` → 트레이 "종료" · 오류 트레이 "종료" · 핸드오프 끝(`Ok(None)`) 모두 **종료 코드 0**(tao 0.37.1
  `event_loop.rs:177` · `process::exit(exit_code)`). 메인 스레드 패닉은 unwind → 종료 코드 101. tokio 작업 안 패닉은 작업 단위로 잡혀 프로세스가
  살아 있다(`panic` 프로필 설정 없음).
- 강제 종료 뒤 다시 켜면 `Chat::recover` 가 `queued` · `running` 실행을 다시 깨우고 outbox 를 서두른다(1c) — 이 Task 는 그 경로를 바꾸지 않는다.
- 설정 저장 = `Notifier::update(&SettingsPatch)` 한 트랜잭션 · `GET/PUT /api/settings`(`notify.rs`). 웹UI 설정 입력은 `PREFS` 목록(입력 id = 설정
  이름, `app.js`), 설정 목차는 `settingsTree()`(`view.js`, node 시험 있음).
- `windows-sys` 기능에 레지스트리(`Win32_System_Registry`)가 없다 — 더한다.
- `FileLogger::install`(`an_core::log_file`)이 로거 설치의 단일 지점.

## intent 대비 해석 (plan 승인 대상)

1. **감시 · 앱 구분 = 환경 변수 `COWORKAGENT_SUPERVISED`** — 값 = 다시 띄운 횟수(처음 `0`). 순서: 인자 있음 → CLI(지금 그대로, A6) · 변수
   없음 → 감시 프로세스 · 변수 있음 → 앱. 감시 프로세스는 로거 · DB · 잠금을 쓰지 않는다(회전 로그를 두 프로세스가 쓰지 않게). 앱은 로거를 연 뒤
   값이 1 이상이면 `info` "감시 프로세스가 다시 띄움({n}번째)" — 다시 띄움 기록은 앱 로그 한 곳.
2. **감시 규칙(`an_app::watchdog`)** — 자식을 `current_exe()` 로 띄우고 끝날 때까지 기다린다. 종료 코드 0 → 감시도 끝(A3). 0 아님 → 1초 뒤 다시.
   최근 비정상 종료 3번이 60초 안이면 멈춤(상수 `MAX_CRASHES = 3` · `CRASH_WINDOW = 60초` · `RESTART_DELAY = 1초`). 자식을 띄우지 못해도 멈춤.
   멈추면 감시 프로세스가 직접 오류 트레이(`Tray::run_error`, 사유 파일 `write_start_error` — 로그 위치 포함): 사유 "앱이 1분 안에 3번 비정상
   종료 — 다시 띄우지 않습니다"(띄우기 실패면 그 오류). 자식은 Job 에 넣지 않는다(감시가 죽어도 앱은 산다 — 그 뒤 비정상 종료는 다시 띄우지 않음).
3. **이미 실행 중일 때** — 새 감시 → 새 앱이 보조 인스턴스로 "열기" 를 넘기고 0 으로 끝 → 새 감시도 끝(D14 그대로). 넘기기 실패는 지금처럼 오류
   트레이(사용자가 "종료" → 0).
4. **자동 시작(`an_app::autostart`)** — 등록 대상 = `current_exe()`(인자 없음 = 감시 프로세스).
   - Windows: `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` 값 이름 `coworkAgent` · 데이터 `"<exe 전체 경로>"`(따옴표). 같은 값이면 쓰지
     않는다. 함수는 키 경로를 인자로 받는다(시험은 `Software\coworkAgent-test\…` 를 쓰고 지운다 — 실제 Run 키를 건드리지 않는다).
   - macOS: `~/Library/LaunchAgents/com.coworkagent.plist` — `Label` `com.coworkagent` · `ProgramArguments` [exe] · `RunAtLoad` true(다시 띄우기는
     감시 프로세스가 — `KeepAlive` 없음, 두 OS 같은 동작 D30). 경로는 XML 이스케이프. 내용 생성 함수는 모든 OS 에서 시험한다.
   - 설정 `autostart`(`an_core::settings`, 없으면 켜짐 — D13 기본 ON, `0` = 끔). 앱 시작 때 켜져 있으면 등록(실패는 `warn` — 시작을 막지 않음),
     꺼져 있으면 아무것도 하지 않는다(끌 때 이미 지웠다).
   - `PUT /api/settings {autostart}` — `Notifier` 의 자동 시작 훅(기본 없음, 앱이 설치)을 먼저 부르고 성공해야 저장. 훅 실패 = 500 · 저장 안 함.
     `GET /api/settings` 에 `autostart`.
   - **디버그 빌드는 등록하지 않는다**(훅을 설치하지 않고 시작 때 등록도 건너뜀 — `cargo run` 이 개발 경로를 로그인 실행에 남기지 않게). 설정은
     저장된다.
5. **웹UI** — 설정 목차에 `시작 — 자동 실행`(id `startup`, '화면 — 테마' 다음), 카드 안 체크 `로그인할 때 자동 실행`(id `autostart`, `PREFS`
   에 더함) + 안내 "끄면 다음 로그인부터 자동으로 켜지지 않습니다. 비정상 종료되면 자동으로 다시 실행합니다."
6. **패닉 기록** — `FileLogger::install` 이 패닉 훅도 단다: `error` "패닉: {메시지} — {파일}:{줄}"(`PanicHookInfo` 표시) 후 기본 훅(stderr).

## 설계

### `an-core`
- `settings.rs` — `AUTOSTART_KEY` · `autostart(conn) -> bool`(없으면 true).
- `log_file.rs` — `FileLogger::install` 에 패닉 훅.

### `an-app`
- `watchdog.rs`(신규) — `ENV` · 상수 · `Crashes::record(at) -> bool`(멈출지) · `supervise(spawn: FnMut(u32) -> io::Result<bool>, delay) -> Outcome`
  (`Done` · `GaveUp(사유)`) · `spawn_app(n)`(실제 자식).
- `autostart.rs`(신규) — Windows `registered(key)` · `register(key, exe) -> 바뀜` · `unregister(key)` / `launch_agent_plist(exe) -> String` ·
  mac 파일 쓰기 · 지우기 / `sync(on)`(이 OS 의 실제 자리).
- `notify.rs` — `SettingsPatch.autostart` · 훅 `set_autostart_hook` · 설정 JSON.
- `main.rs` — 구분(해석 1) · 감시 실행 · 멈춤 오류 트레이 · 다시 띄움 로그 · 시작 때 등록 · 훅 설치(릴리스만).
- `Cargo.toml` — `windows-sys` 기능 `Win32_System_Registry`.
- 웹UI — `index.html`(카드) · `view.js`(`settingsTree`) · `app.js`(`PREFS`).

### 문서
- 매뉴얼: user-guide(시작하기 · 설정 '시작' · 비정상 종료) · developer-guide(실행 구조 — 감시 · 앱 · CLI) · an_app · an_core.

## Stage

| stage | 목표 | 산출 | gate |
|---|---|---|---|
| 1 | 감시 프로세스 · 패닉 기록 | `watchdog` 시험 · `panic_log` 시험 | E1 · E2 · E4 · E5 |
| 2 | 자동 시작 · 설정 · 웹UI | `autostart` 시험 · node `settingsTree` 시험 갱신 | E1 ~ E3 · E6 |
| 3 | 매뉴얼 · 실측 | 실측 기록 | E1 ~ E3 · E7 |

## 평가 항목

- [ ] E1: A7 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E2: A7 — 우리 코드 경고 0(clippy -D warnings)
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: pending
- [ ] E3: A4·A7 — 화면 로직 node 테스트 47건 이상 · 실패 0(설정 목차에 `startup`)
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# pass (4[7-9]|[5-9][0-9])\s+# fail 0/
  EVIDENCE: pending
- [ ] E4: A1·A2·A3·A6 — 감시 6건(정상 종료면 한 번 띄우고 끝 / 비정상이면 다시 띄우고 횟수가 늘어 자식에 전달 / 60초 안 3번이면 멈춤 · 사유 / 간격이 60초를 넘는 비정상 종료는 계속 / 자식을 띄우지 못하면 멈춤 / 실제 실행 파일: 인자가 있으면 감시 없이 CLI 사용법 오류(2)로 바로 끝나고, 그 실행 파일을 자식으로 감시하면 비정상 3번에 멈춤)
  CHECK: cargo test -p coworkagent-app --test watchdog
  EXPECT: test result: ok. 6 passed; 0 failed
  EVIDENCE: pending
- [ ] E5: A5 — 패닉 메시지 · 파일 · 줄이 로그 파일에 남는다(1건)
  CHECK: cargo test -p an-core --test panic_log
  EXPECT: test result: ok. 1 passed; 0 failed
  EVIDENCE: pending
- [ ] E6: A4 — 자동 시작 8건(시험 키에 등록 · 같은 경로면 쓰지 않음 · 다른 경로면 고침 · 해제 / LaunchAgent plist 내용 · 경로 XML 이스케이프 / 설정 API 기본 켜짐 · 끄면 훅(false) 뒤 저장 · 다시 읽어도 꺼짐 / 훅 실패면 500 · 저장 안 함 / 켜기 뒤 저장 실패면 등록을 되돌림 · 끄기는 저장 실패면 OS 그대로 / 순서 — 켜기 훅 때 저장 꺼짐 · 끄기 훅 때 저장 이미 꺼짐(개정 2) / 동시 요청 뒤 마지막 등록 = 저장 값 / plist 통째 교체 — 실패하면 원래 파일 그대로 — 개정 1)
  CHECK: cargo test -p coworkagent-app --test autostart
  EXPECT: test result: ok. 8 passed; 0 failed
  EVIDENCE: pending
- [ ] E7: A1·A2·A3·A4 — Windows 릴리스 실측: 실행 → 감시 · 앱 두 프로세스 · `HKCU\…\Run` 에 그 경로 / 앱 강제 종료(`taskkill /F`) → 다시 뜸 · 로그 "다시 띄움(1번째)" · 남은 outbox 이어 보냄 / 1분 안 3번 강제 종료 → 오류 트레이 / 트레이 "종료" → 두 프로세스 모두 없음 / 실행 중 다시 실행 → 브라우저 열림 · 감시 하나 / 설정에서 끄기 → Run 값 없음 · 다시 실행해도 없음(사람 확인은 19 TC)
  EVIDENCE: pending
- [ ] E8: A4 — macOS 실측: LaunchAgent plist 생성 · 로그인 시 실행 · 끄기 → 파일 없음
  EVIDENCE: pending

## 종결 시 할 일

- masterPlan: D13 보강(감시 프로세스 · 켜기 · 끄기 = 웹UI 설정 · 끈 선택 저장 · 디버그 빌드 등록 없음 · "시작 시 웹UI 열기" 제외) · S7 실현 · S2
  보강(인자 없는 실행 = 감시 프로세스 + 앱).
- PRODUCT 도메인 규칙(실행 구조 · 자동 시작) · 기술 스택(`windows-sys` 레지스트리), 매뉴얼.
- 19 acceptance TC — 로그인 자동 실행 · 강제 종료 후 다시 뜸 · 거듭 종료 오류 트레이 · 설정 끄기.
- E8 은 Mac 환경 없음 → `ABANDON`(D25).
- GitHub 반영(push)은 사용자 확인 뒤.

## 개정 기록

- 개정 1(2026-10-14, 적대적 평가 라운드 1/2 — high 1 · medium 1 사실): ① 해석 4 의 "꺼져 있으면 아무것도 하지 않는다" → **시작마다 설정 값으로
  맞춘다**(켜짐 = 등록 · 경로 고침, 꺼짐 = 지움 — 지난 실행에서 어긋난 OS 상태를 고친다). `PUT {autostart}` 는 훅 잠금을 저장이 끝날 때까지 쥐어
  동시 요청의 등록 순서 = 저장 순서, 저장이 실패하면 등록을 저장된 값으로 되돌린다. ② macOS plist 는 같은 폴더 `{이름}.tmp` 에 다 쓰고
  `sync_all` 뒤 이름 바꾸기(`autostart::write_atomic`) — 실패하면 원래 파일 그대로. E6 4 → 7건.
- 개정 2(2026-10-14, 적대적 평가 라운드 2/2 — high 1 사실): 설정 바꾸기 순서 — **켜기는 OS 등록 → 저장, 끄기는 저장 → OS 지우기**. 어느 지점에서
  중단돼도(강제 종료 · 전원) 남는 어긋남은 "OS 켜짐 · 저장 꺼짐" 뿐 — 다음 로그인에 실행되고 시작 맞추기가 지운다("저장 켜짐 · OS 꺼짐" 은 생기지
  않는다). 끄기 훅이 실패하면 저장을 이전 값으로 되돌리고 500. E6 7 → 8건(순서 시험).
