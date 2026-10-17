# plan — 1n.cli-start-stop

> 근거: [intent](intent.md)(승인 2026-10-16 — 라운드 1 · 2 · 3, "mac 에서도 동일하게"). **구현자는 이 문서만 보고 일한다.** D·S 는 [masterPlan](../../masterPlan.md).
> 기준 커밋: `00dfc77`(task/1n-cli-start-stop — main `a3aca60` 위 intent). plan 승인: 2026-10-17(사용자 "승인.").

## 목표

부모 툴이 하위 메뉴로 부를 CLI `start` · `open` · `stop` 을 준다. `start` · `open` 은 트레이 아이콘 없는 모드로 띄우고 부모 프로세스가 끝나면 함께 정상
종료한다. `stop` 은 두 모드 모두 끝날 때까지 기다린다. 어떤 실행 경로에서도 앱 · 감시 프로세스는 하나, 트레이는 많아야 하나. Windows · macOS 같은 동작.

### 성공 기준(intent 사본)

- A1: 꺼진 상태에서 `start` 를 부르면 트레이 아이콘 없이 앱이 뜨고, 받을 준비가 된 뒤 종료 코드 0 으로 끝난다. 브라우저는 열리지 않는다.
- A2: 트레이 없는 모드로 떠 있을 때 `start` 를 부르면 새 프로세스 · 화면 없이 0 으로 끝난다. 트레이 모드로 떠 있을 때 부르면 트레이가 사라지고 트레이 없는
  모드로 다시 뜬 뒤 0 으로 끝난다.
- A3: 부모 프로세스(기본 = 부른 프로세스, `--parent <PID>` 지정)가 끝나면 트레이 없는 모드의 앱 · 감시가 정상 종료한다.
- A4: `open` 은 떠 있으면 화면을 열고, 꺼져 있으면 트레이 없이 띄운 뒤 연다.
- A5: `stop` 을 부르면 (두 모드 모두) 앱 · 감시가 모두 끝난 뒤 0 으로 끝난다(꺼져 있으면 바로 0). 10초 안에 끝나지 않으면 0 이 아닌 코드와 사유.
  오류 트레이도 `stop` 으로 꺼진다.
- A6: 인자 없는 실행 · `start` · `open` 을 동시에 · 거듭 불러도 앱 · 감시 프로세스는 각각 하나, 트레이는 많아야 하나다. 오류 트레이도 거듭 실행으로 늘지 않는다.
- A7: 트레이 없는 모드에서 시작에 실패하면 `start` · `open` 이 0 이 아닌 코드와 사유를 낸다(트레이 없음).
- A8: 인자 없는 실행 · 자동 시작은 트레이 모드, 두 번째 실행은 화면 열기(회귀 없음). 기존 CLI(`peers` · `ask` · `wait` · `status` · `history`) 회귀 없음.
- A9: 검증 명령(`cargo test --workspace` · clippy · `check_gates` · `check_ddd_docs` · `check_readme`) 통과.

## 코드 사실(실측, 기준 `00dfc77`)

- `app/src/main.rs` — 인자가 있으면 `cli::main`(트레이 · 잠금 · 로거 없음). 인자 없고 감시 변수(`COWORKAGENT_SUPERVISED`) 없으면 **감시 프로세스**: `watchdog::supervise
  (spawn_app)` — 자식이 0 으로 끝나면 같이 끝, 비정상이면 1초 뒤 다시(60초에 3번이면 멈춤 → 감시가 오류 트레이 `run_error`). 감시는 잠금을 쓰지 않는다 → **동시에 여러
  번 실행하면 감시가 그 수만큼 뜬다**(두 번째 이후는 자식이 보조 인스턴스로 핸드오프하고 끝나며 곧 사라진다).
- 앱(`start`) — `InstanceLock::acquire(instance.lock)`(`File::try_lock`, 프로세스가 죽으면 OS 가 푼다) → 보조면 `hand_off`(`instance.json` 읽어 `POST /api/open`,
  300ms × 10번) 실패 시 `Err` → **앱이 오류 트레이**(잠금 없음 — 거듭 실행하면 는다). 시작 오류(경로 · DB · 포트 등)도 앱이 오류 트레이. 정상이면 `instance.json`
  ({port, token}) 쓰고 `tray.run(url, note, on_seen, on_quit)`. `on_quit` = 런타임 1초 정지 → `instance.json` 지움 → 잠금 풂.
- 트레이(`tray.rs`) — tao 이벤트 루프 + tray-icon. 메뉴 "열기"(`browser::open` + `on_seen`) · "종료"(`on_quit`). 새 요청 수는 `Attention`(프록시)로. macOS 는
  Dock 없이 메뉴 막대(`ActivationPolicy::Accessory`).
- 로컬 API(`server.rs`) — 토큰 guard 뒤 `GET /api/status`({version, data_dir, db, cli_api}) · `POST /api/open`(204 · 실패 500). 종료 API 없음.
- CLI(`cli.rs`) — `is_command`(peers · ask · wait · status · history), `run(args, instance_file, out, err, wait_limit)` 가 `Api::open` + `handshake`(`cli_api`)
  를 먼저 한다(꺼져 있으면 3). 종료 코드 0 · 2 사용법 · 3 실행 중 아님 · 4 API · 5 답 실패 · 6 시간 초과 · 7 멈춤. 요청은 `handoff::local_request`.
- 데이터 경로 `Paths::default_root()` = `dirs::data_local_dir()/coworkAgent`(Windows 는 알려진 폴더 API — 환경 변수로 바꿀 수 없다). 시험이 실제 실행 파일로
  띄우면 **이 PC 의 실제 인스턴스 · 자동 시작 등록을 건드린다**(자동 시작 맞추기는 릴리스 빌드만).
- 옛 데이터 이어받기(`adopt_legacy`)는 새 루트가 이미 있어도 데이터 파일이 없으면 옮긴다 — 감시가 루트에 잠금 파일만 먼저 만들어도 된다.
- 의존 — `windows-sys` 기능 LibraryLoader · Registry · Shell · WindowsAndMessaging, Unix `libc`. 설치된 대상 `x86_64-pc-windows-msvc` 뿐.

## intent 대비 해석 (plan 승인 대상)

1. **트레이 없는 모드 진입** — `start` · `open` 이 실행 파일 자신을 **인자 없이** 띄우고 내부 환경 변수 `COWORKAGENT_PARENT=<부모 PID>` 를 붙인다(사용자 literal
   아님). 감시가 그 값을 앱에 물려준다. 트레이 없는 앱은 tao 이벤트 루프 · 트레이를 만들지 않고 종료 신호를 기다린다(macOS 도 메뉴 막대 · Dock 없음). 띄울 때 표준
   입출력은 비우고(부모 툴의 파이프가 끝나지 않는 일 방지) 새 프로세스 그룹으로(Windows `DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP`, Unix
   `process_group(0)`) — 부모 콘솔의 Ctrl+C 가 닿지 않는다.
2. **감시 단일화** — 감시 프로세스도 잠금을 쥔다: 데이터 폴더 `supervisor.lock`(+ PID 를 `supervisor.pid` 에). 못 잡으면 이미 떠 있는 것:
   - 트레이 없는 모드 요청(`COWORKAGENT_PARENT` 있음) → 조용히 0 으로 끝(부르는 `start` · `open` 이 준비를 확인한다).
   - 인자 없는 실행 → 화면 넘기기(지금 `hand_off`). 실패하면 **트레이 없이** 사유 파일을 OS 기본 앱으로 연다(조용히 사라지지 않는다) · 종료 코드 1.
   앱도 `instance.lock` 을 잡은 뒤 PID 를 `instance.pid` 에 쓴다(`stop` 의 강제 종료용).
3. **오류 표시는 감시가** — 앱의 시작 실패(잠금 뒤 DB · 포트 등)는 앱이 사유 파일을 쓰고 종료 코드 `START_FAILED`(= 3)로 끝난다 — 감시는 다시 띄우지 않고, 트레이
   모드면 **감시가** 오류 트레이(하나 — 감시 잠금 안), 트레이 없는 모드면 사유 파일만 남기고 같은 코드로 끝난다. 감시 멈춤(60초에 3번)도 같다. 앱은 더 이상 오류
   트레이를 띄우지 않는다.
4. **상태 · 종료 API** — `/api/status` 에 `"mode":"tray"|"headless"` 를 더한다. 새 `POST /api/quit`(같은 토큰 guard, 204) — 트레이 모드는 트레이 "종료" 와 같은 길
   (이벤트 루프에 종료 이벤트), 트레이 없는 모드는 대기 중인 메인 스레드를 깨워 같은 `on_quit`.
5. **부모** — 기본 = CLI 프로세스의 부모 PID(Windows Toolhelp `th32ParentProcessID`, Unix `getppid`). `--parent <PID>`(숫자)로 지정. 부모가 이미 없으면
   띄우지 않고 사용법 오류(2) "부모 프로세스 <PID> 가 없다". 앱은 시작 직후 부모를 지켜본다 — Windows `OpenProcess(SYNCHRONIZE)` + 대기 스레드, Unix 1초마다
   `kill(pid, 0)`. 부모가 끝나면 `POST /api/quit` 과 같은 정상 종료 → 감시도 끝. 앱이 다시 뜰 때(비정상 종료 뒤) 부모가 없으면 바로 0 으로 끝(감시도 끝).
6. **`start [--parent <PID>]`** — ① `instance.json` + `/api/status`(1초) 응답: `headless` 면 0("이미 실행 중"), `tray` 면 `stop` 과 같이 끈 뒤 ③.
   ② 응답 없음인데 감시 · 앱 잠금이 잡혀 있으면(시작 중) 최대 20초 응답을 기다려 ①; 끝내 없으면 실패 6 "실행 중이지만 응답이 없다 — stop 뒤 다시".
   ③ 띄우고 최대 30초 `/api/status` 가 `headless` 로 답할 때까지 기다려 0("시작함"). 띄운 감시가 0 이 아닌 코드로 끝나고 응답이 없으면 사유 파일의 사유로 실패
   8(새 — 시작 실패).
7. **`open [--parent <PID>]`** — 응답 있으면 `POST /api/open`(모드 무관) → 204 면 0, 아니면 4. 없으면 `start` 와 같이 띄운 뒤 `POST /api/open`.
8. **`stop`** — ① 응답 있으면 `POST /api/quit` → 감시 · 앱 잠금이 모두 풀릴 때까지(최대 10초) → 0("종료함"), 넘으면 6 "10초 안에 끝나지 않았다".
   ② 응답 없음 + 잠금 잡힘(오류 트레이 · 시작 중 · 응답 없는 앱) → 5초 응답을 기다려 오면 ①, 아니면 `supervisor.pid` → `instance.pid` 순으로 강제 종료(그 잠금이
   잡혀 있을 때만 — 감시를 먼저 끊어 다시 띄우지 않게) → 잠금 풀림 대기 → 0. ③ 둘 다 풀려 있으면 바로 0("실행 중이 아님").
   잠금 풀림 판정 = `stop` 이 두 잠금을 잠깐 잡아 보고 바로 놓는다.
9. **종료 코드 · 출력** — 기존 규약: 0 · 2 · 4 · 6, 새 8 = 시작 실패. 성공은 표준 출력 한 줄("시작함" · "이미 실행 중" · "열었음" · "종료함" · "실행 중이
   아님"), 실패는 표준 오류에 사유. `start` · `open` · `stop` 은 `handshake`(`cli_api`) 없이 동작(꺼져 있어도 된다). 사용법(`USAGE`)에 세 줄 더한다.
10. **시험용 데이터 경로** — **디버그 빌드에서만** 환경 변수 `COWORKAGENT_DATA_DIR` 가 데이터 루트를 바꾼다(시험이 실제 인스턴스 · 데이터를 건드리지 않게).
    릴리스 빌드는 읽지 않는다.
11. **macOS** — 같은 코드 길(부모 `getppid` · `kill(pid,0)` 폴링 · 강제 종료 `SIGKILL` · `process_group(0)`). 트레이 없는 모드는 이벤트 루프를 만들지 않아
    메뉴 막대 · Dock 이 없다. 이 PC 에서 macOS 실기 확인 불가 — Unix 길은 `#[cfg(unix)]` 로 두 OS 공통(Linux 포함 동작), 실기 확인은 원장(19 인수).
12. 바뀌지 않는 것 — 인자 없는 실행 · 자동 시작(트레이 모드) · 두 번째 인자 없는 실행 = 화면 열기(모드 무관) · 기존 CLI 명령 · 트레이 만들기 다시 시도(#1h).

## 설계

- `an_core` — `paths.rs` `supervisor_lock_file` · `supervisor_pid_file` · `instance_pid_file`, `default_root` 디버그 빌드 `COWORKAGENT_DATA_DIR`.
  `instance.rs` `InstanceLock::acquire` 그대로 재사용 · `is_held(path)`(잠깐 잡아 보고 놓기) · `write_pid` / `read_pid`.
- `app/src/proc.rs`(새) — 프로세스 다루기 두 OS: `parent_pid()` · `alive(pid)` · `watch_parent(pid, on_gone)` · `kill(pid)` · `spawn_detached(exe, envs)`.
  `windows-sys` 기능 `Win32_System_Diagnostics_ToolHelp` · `Win32_System_Threading` · `Win32_Foundation` 추가.
- `app/src/control.rs`(새) — `start` · `open` · `stop`: `run(cmd, args, paths, exe, out, err) -> i32`(시험이 경로 · 실행 파일을 준다). `cli.rs` — `is_command` 에
  세 명령, `main` 이 세 명령은 `control` 로(handshake 없음), `USAGE` · `EXIT_START_FAILED = 8`.
- `app/src/watchdog.rs` — 감시 잠금 · 보조 처리, `supervise` 결과에 `StartFailed(사유)`(자식 코드 3 → 다시 띄우지 않음), 자식에 `COWORKAGENT_PARENT` 전달.
- `app/src/server.rs` — `Status.mode` · `POST /api/quit`(`router_with` 인자 `quit`). `tray.rs` — `TrayEvent::Quit` · 종료 핸들(프록시). 오류 트레이는 감시만 부른다.
- `app/src/main.rs` — 모드 가르기(트레이 / 트레이 없음), 트레이 없음이면 `mpsc` 대기 + 부모 감시, 시작 실패 → 사유 파일 · 코드 3, `instance.pid` 쓰기.
- 시험 — `watchdog.rs`(StartFailed 는 다시 띄우지 않음 · 코드 3), `control.rs` 단위(인자 · `--parent` 숫자 · 모르는 옵션), 새 `tests/lifecycle.rs`(디버그 실행
  파일 + 임시 `COWORKAGENT_DATA_DIR` — 실제 인스턴스 비파괴): start → headless 응답 · 브라우저 안 엶 · 다시 start 같은 PID · 보조 감시 즉시 끝 · stop → 잠금 풂 ·
  부모(대역 프로세스) 끝 → 앱 · 감시 끝 · `--parent` 없는 PID → 2 · 동시 start 4개 → 앱 PID 하나 · DB 자리를 폴더로 막아 시작 실패 → start 8 + 사유 · 트레이 모드
  (인자 없는 실행) → start 가 headless 로 바꿈 · 오류 트레이(트레이 모드 시작 실패) → stop 0 · 꺼진 상태 stop 0), `open` 은 `cli.rs` 꼴 로컬 API + 가짜 열기로
  (떠 있음 → 열기 1회). `ux_literals` 사용법 문구.
- 문서 — 사용자 가이드 §14(부모 툴 메뉴 · `--parent` 예시 · 종료 코드) · 개발자 가이드(감시 잠금 · 모드 · `/api/quit`) · an_app · an_core 매뉴얼 · PRODUCT ·
  README 기능 줄.

## Stage

| stage | 목표 | 산출 | gate |
|---|---|---|---|
| 1 | 경로 · 잠금 · 프로세스 · 감시 · 모드 · API · `start` · `open` · `stop` | Rust 시험(lifecycle 포함) | E1 · E2 · E3 · E7 |
| 2 | 문서 · 이 PC 실제 확인(디버그 실행 파일 · 임시 데이터 경로 — 실제 인스턴스 비파괴) | 문서 · 확인 기록 | E4 ~ E6 · E8 |

## 평가 항목

- [ ] E1: A1·A2·A3·A5·A6·A7·A8·A9 — 워크스페이스 테스트 전부 통과(lifecycle: start · 다시 start · 모드 바꿈 · 부모 끝 · 동시 start · 시작 실패 8 · 오류 트레이 stop · 꺼진 stop · 보조 감시 즉시 끝 / watchdog StartFailed / 기존 CLI · 핸드오프)
  CHECK: cargo test --workspace
  EXPECT: /Running tests.lifecycle\.rs/
  EVIDENCE: pending
- [ ] E2: A9 — 우리 코드 경고 0(clippy -D warnings)
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: pending
- [ ] E3: A1·A4·A5 — 실제 실행 파일 수명 시험만 따로(start · open · stop · 부모 · 동시 · 시작 실패)
  CHECK: cargo test -p coworkagent-app --test lifecycle
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E4: A9 — 문서 규약
  CHECK: python tools/check_ddd_docs.py
  EXPECT: PASS
  EVIDENCE: pending
- [ ] E5: A9 — README 사용자 관점
  CHECK: python tools/check_readme.py
  EXPECT: readme ok
  EVIDENCE: pending
- [ ] E6: A9 — 의존 경계
  CHECK: python tools/check_crates.py
  EXPECT: /crates=4 edges=[0-9]+ violations=0/
  EVIDENCE: pending
- [ ] E7: A8 — 기존 CLI · 사용법 문구 회귀 없음
  CHECK: cargo test -p coworkagent-app --test cli
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E8: A1·A2·A4·A6·A8 — 이 PC 실제 확인(디버그 실행 파일 · 임시 `COWORKAGENT_DATA_DIR`): 인자 없는 실행 = 트레이 아이콘 보임 → `start` 로 트레이 사라짐 ·
  `open` 이 브라우저로 화면 · 인자 없는 두 번째 실행 = 화면 열기 · 부모(셸) 닫으면 끝 · `stop` 뒤 작업 관리자에 남은 프로세스 없음 · 실제 인스턴스 그대로.
  EVIDENCE: pending

## 종결 시 할 일

- masterPlan: D14 보강(`start` · `open` · `stop` · `--parent` · 트레이 없는 모드 · 감시 잠금 · 종료 코드 8) · S8 일부 실현 · 개정 기록. 원장: macOS 실기 확인(19 인수).
- 이 PC 반영(릴리스 빌드). GitHub 반영 · 릴리스는 사용자 확인 뒤.

## 개정 기록

- 2026-10-17 · E1 EXPECT `/test result: ok\. [1-9][0-9]* passed; 0 failed/` → `/Running tests.lifecycle\.rs/` — 사유: 앞 패턴은 어느 크레이트의
  결과 한 줄과도 맞아 lifecycle 시험이 없어도 통과했다(측정 커밋 `32341e3` 에서 거짓 통과). 성공 = 종료 코드 0 이므로 lifecycle 을 포함한 전부 통과를 본다 · 승인 2026-10-17(사용자 "승인. 개발 진행.")
- 2026-10-17 · 해석 1 Windows 띄우기 플래그 `DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP` → `CREATE_NO_WINDOW | CREATE_NEW_PROCESS_GROUP` — 사유: 콘솔
  없는 감시가 디버그(콘솔) 빌드 앱을 띄우면 새 콘솔 창이 생긴다. 숨은 콘솔을 물려받게 한다(릴리스는 창 없는 서브시스템이라 차이 없음) · 승인 2026-10-17(사용자 "승인. 개발 진행.")
- 2026-10-17 · 설계 `server.rs` — `router_with` 시그니처는 그대로 두고(시험 12개 파일이 직접 쓴다) 새 `router_control(…, mode, quit, extra)` 를 두며
  `router_with` 가 `mode = "tray"`, `quit` 없음으로 부른다 — 사유: 기존 시험 무변경 · 승인 2026-10-17(사용자 "승인. 개발 진행.")
- 2026-10-17 · 해석 6 `start` ② "응답 없음 + 잠금 잡힘 → 최대 20초" 와 ③ "띄운 뒤 최대 30초" 를 **한 30초 되풀이**로 합침(응답 확인 → 트레이면 끄기 →
  띄운 감시의 종료 코드 확인 → 잠금이 모두 풀려 있으면 띄우기). 끝에 잠금이 잡혀 있으면 6, 아니면 8 — 사유: 겹친 `start` 의 경쟁(다른 `start` 가 띄운 감시가
  잠금을 쥐는 동안)도 같은 되풀이로 풀린다 · 승인 2026-10-17(사용자 "승인. 개발 진행.")
