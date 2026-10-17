# stage1 — 경로 · 잠금 · 프로세스 · 감시 · 모드 · API · `start` · `open` · `stop`

> 근거: [plan](../plan.md) Stage 1.

## 목표

해석 1 ~ 12 의 코드 전부(문서 · 이 PC 실제 확인은 stage 2). 대상 gate: E1 · E2 · E3 · E7.

## 산출 파일

- `crates/core` — `paths.rs` `DATA_DIR_ENV`(`COWORKAGENT_DATA_DIR`, 디버그 빌드만 — 있으면 `legacy_root` 는 없음) · `instance_pid_file` ·
  `supervisor_lock_file` · `supervisor_pid_file`. `instance.rs` `InstanceLock::is_held`(잠깐 잡아 보고 놓기) · `write_pid` · `read_pid`.
- `crates/app/src/proc.rs`(새) — `parent_pid`(Windows Toolhelp · Unix `getppid`) · `alive`(권한 거부 = 살아 있음) · `watch_parent`(Windows
  `OpenProcess(SYNCHRONIZE)` 대기, 못 열면 1초 폴링 · Unix 1초 폴링) · `kill`(`TerminateProcess` · `SIGKILL`) · `spawn_detached`(표준 입출력 비움 ·
  `COWORKAGENT_SUPERVISED` 지움 · Windows `CREATE_NO_WINDOW | CREATE_NEW_PROCESS_GROUP` + 자기 표준 핸들 상속 끔 · Unix `process_group(0)`).
- `crates/app/src/control.rs`(새) — `start` · `open` · `stop`(`run` · `probe` · `is_command`). `cli.rs` — `main` 이 세 명령을 handshake 없이 `control` 로,
  `EXIT_START_FAILED = 8`, `USAGE` 세 줄 + 설명 한 줄.
- `crates/app/src/watchdog.rs` — `PARENT_ENV` · `START_FAILED = 3` · `Outcome::StartFailed` · `supervise` 가 종료 코드(`io::Result<i32>`)를 받음 ·
  `acquire`(감시 잠금 + `supervisor.pid`) · `parent()`.
- `crates/app/src/server.rs` — `router_control(…, mode, quit, extra)`(`router_with` 는 `"tray"` · 종료 없음으로 부른다), `/api/status` 에 `"mode"`,
  `POST /api/quit`(204 → 100ms 뒤 종료 호출).
- `crates/app/src/tray.rs` — `TrayEvent::Quit` · `Tray::quitter` · `Attention::none` · `start_error_file` · `read_start_error`, `run_error` "오류 보기" 는
  열기 전에 사유 파일을 다시 쓴다.
- `crates/app/src/main.rs` — 감시(`supervise`): 감시 잠금 → 보조면 트레이 없는 모드 0 · 인자 없는 실행 화면 넘기기(실패 = 사유 파일 열기 · 1), 시작 실패 ·
  멈춤 = 트레이 모드 오류 트레이(잠금 쥔 채) · 트레이 없는 모드 3. 앱: 부모가 없으면 0, 트레이 없는 모드는 트레이 · 이벤트 루프 없이 `mpsc` 대기 + 부모
  감시, `instance.pid` 쓰기 · 종료 때 지움, 시작 실패 = 사유 파일 + 코드 3(앱은 오류 트레이를 띄우지 않는다).
- `Cargo.toml` — `windows-sys` 기능 `Win32_Foundation` · `Win32_System_Console` · `Win32_System_Diagnostics_ToolHelp` · `Win32_System_Threading`.
- 시험 — 새 `tests/lifecycle.rs` 8건(`#![cfg(debug_assertions)]`, 시험마다 임시 `COWORKAGENT_DATA_DIR` + 대역 부모): start · 다시 start 같은 PID · stop ·
  꺼진 stop / 부모 끝 → 앱 · 감시 끝 / 끝난 PID `--parent` → 2 / 감시 4개 동시 → 하나만(나머지 0) · 동시 start 4개 → 앱 하나(떠 있을 때 · 꺼져 있을 때) /
  DB 자리를 폴더로 막음 → start 8 + 사유 / 트레이 모드 → start 가 트레이 없는 모드로 바꿈 / 오류 트레이 → stop 0 / open → 가짜 열기 1회.
  `tests/watchdog.rs` 종료 코드로 바꿈 + `start_failed_stops_without_respawn`. `ux_literals.rs` `control_usage_literals`. 단위 — `proc`(살아 있음 · 부모 ·
  끝난 자식) · `control`(인자 · `--parent` · 끝난 부모) · `an_core` `is_held_tracks_holder_and_pid_round_trip`.

## 설계 결정 · plan 편차

- plan 개정 4건(사용자 미승인 — [plan](../plan.md) 개정 기록): E1 EXPECT · Windows 띄우기 플래그 · `router_control` · `start` 30초 되풀이.
- **오류 트레이 시험** — 거듭 실행(인자 없는 두 번째 실행)의 화면 넘기기 실패는 사유 파일을 OS 기본 앱으로 연다 — 시험이 창을 띄우지 않게 부르지 않는다.
  오류 트레이 하나는 감시 잠금이 보장하고, 감시 단일은 동시 실행 시험이 본다.
- **사유 파일 공유** — 사유 파일은 사용자 임시 폴더의 한 파일(데이터 경로가 원인일 수 있다)이라 시험 · 실제 인스턴스가 같은 파일을 쓴다. 시작 실패 시험은
  사유 문구의 공통 부분("시작 실패")만 본다.
- **강제 종료 PID** — PID 파일은 그 잠금이 잡혀 있을 때만 믿는다(남은 파일이 다른 프로세스를 가리킬 수 있다). 감시를 먼저 끊는다.
- **rustfmt** — 손댄 파일(`cli.rs` 포함)에 `rustfmt --edition 2024` — `cli.rs` 의 기존 긴 줄 몇 곳이 함께 줄바꿈됐다(동작 변화 없음).

## 테스트 결과

```text
cargo test --workspace                                → rc=0, test result: ok 73건, Running tests\lifecycle.rs 포함
cargo test -p coworkagent-app --test lifecycle (3회)  → test result: ok. 8 passed; 0 failed (5.5 ~ 6.0s)
cargo clippy --workspace --all-targets -- -D warnings → Finished
node --test --test-reporter=tap "crates/app/web/*.test.mjs" → # pass 104 / # fail 0
python tools/check_ddd_docs.py                        → PASS
python tools/check_crates.py                          → crates=4 edges=6 violations=0
```

- 시험 뒤 남은 `target\debug\coworkAgent.exe` 프로세스 · 임시 폴더 없음(확인).
- **macOS · Unix 길은 컴파일 확인 못 함** — 이 PC 대상은 `x86_64-pc-windows-msvc` 뿐. `#[cfg(unix)]` 코드(`getppid` · `kill` · `process_group`)는 원장(19 인수)
  으로.

## stage 리뷰(별도 에이전트, 2026-10-17)

고침:
- **Unix 컴파일 오류** — `proc.rs` `alive` 의 `unsafe { … } == 0` 뒤 `||` 가 문장 끝 뒤 클로저로 파싱됐다 → `let` 으로 받음. pid 0 · 음수(프로세스 그룹 신호)는
  살아 있지 않음, `--parent 0` 은 사용법 오류.
- **`is_held` 가 잠깐 잡는 사이 진짜 획득자가 보조로 판정** — `InstanceLock::acquire` 가 `WouldBlock` 을 50ms 간격 4번까지 다시 잡아 본다(두 잠금 공통),
  `is_held` 는 잡았으면 직접 `unlock`.
- **잠금 없이의 실패가 오류 트레이를 띄움**(A6) — 데이터 경로 없음 · 감시 잠금 IO 오류 · 화면 넘기기 실패는 오류 트레이 없이 사유 파일 열기 + 1(트레이 없는 모드 3).
- **`start` 30초 상한을 트레이 분기가 건너뜀** — 상한 확인을 되풀이 맨 앞으로.
- **사유 파일 공유** — 디버그 빌드 + `COWORKAGENT_DATA_DIR` 면 그 폴더 안(시험이 실제 인스턴스 사유 파일을 덮지 않는다). 시작 실패 시험이 사유 파일의 사유와
  표준 오류가 같은지 본다.
- **시험** — `open` 시험은 없는 실행 파일을 넘김(잘못 띄워도 실제 데이터 루트로 뜨지 않게) · `Env` 정리가 `stop` 실패 때 잡힌 잠금의 PID 를 끊는다.

미조치(사유):
- 이미 떠 있는 트레이 없는 앱에 **다른 부모**로 `start` — 앱은 처음 부모만 지켜본다(intent 범위 "부모는 처음 것 그대로" 와 같음). 부모 툴이 여럿인 경우의 정책은
  사용자 결정.
- `stop` 이 오류 트레이를 강제 종료하면 Windows 알림 영역에 아이콘 잔상(마우스를 올리면 사라짐) — OS 동작.
- `START_FAILED = 3` 이 MSVC `abort()` 종료 코드와 같다 — 네이티브 `abort` 를 시작 실패로 읽어 다시 띄우지 않을 수 있다(plan 해석 3 의 값). 바꾸려면 plan 개정.
- 데이터 경로가 없을 때 `start` · `open` · `stop` 은 기존 CLI 와 같은 3(실행 중 아님).
- "브라우저 안 엶"(A1) 자동 시험 없음 — `start` 경로에 열기 호출이 없다(코드 확인). E8 실제 확인에서 본다.

재검증: `cargo test --workspace` rc=0(ok 73건) · lifecycle 3회 `8 passed; 0 failed` · clippy Finished · `check_ddd_docs` PASS.

## 다음 단계

- stage 리뷰 → plan 개정 4건 사용자 승인 → stage 2(사용자 가이드 §14 · 개발자 가이드 · 매뉴얼 · PRODUCT · README · 이 PC 실제 확인 E8).
