# stage1 — 감시 프로세스 · 패닉 기록

> 근거: [plan](../plan.md) Stage 1. 사용자 검증은 D28 — 19.prototype-acceptance 에서 일괄.

## 목표

인자 없는 실행을 감시 프로세스로 — 앱을 자식으로 띄우고 비정상 종료면 다시 띄우며, 60초 안 3번이면 멈추고 오류 트레이(A1 · A2 · A3).
CLI 는 감시를 거치지 않는다(A6). 패닉 메시지 · 위치를 로그에(A5). 대상 gate: E1 · E2 · E4 · E5.

## 산출 파일

- `crates/app/src/watchdog.rs`(새) — `ENV`(`COWORKAGENT_SUPERVISED`) · `MAX_CRASHES` · `CRASH_WINDOW` · `RESTART_DELAY` · `Outcome` · `Crashes::record` ·
  `supervise` · `spawn_app` · `restarts`.
- `crates/app/src/main.rs` — 인자 → CLI(그대로) · 변수 없음 → `supervise(spawn_app)`, 멈추면 `write_start_error` + `Tray::run_error` · 변수 있음 →
  앱(`start(attention, restarts)` — 로거 뒤 "감시 프로세스가 다시 띄움(n번째)").
- `crates/app/src/lib.rs` — `pub mod watchdog`.
- `crates/core/src/log_file.rs` — `FileLogger::install` 이 패닉 훅(`error` "패닉: {메시지} — {파일}:{줄}" 뒤 기본 훅).
- 테스트 — `app/tests/watchdog.rs`(새, 6) · `core/tests/panic_log.rs`(새, 1).

## 설계 결정

- **감시는 스레드 대기 `Command::status`** — 자식이 끝날 때까지 막혀 있는 단순 루프. 비동기 · 프로세스 핸들 감시 API 없음.
- **멈춘 뒤 오류 트레이는 감시 프로세스가 직접** — 앱을 "오류 모드로" 다시 띄우는 신호(변수 · 인자)를 만들지 않는다. 같은 실행 파일이라
  `Tray::run_error` 를 그대로 쓴다.
- **시험의 실제 자식 = 같은 실행 파일 + 모르는 명령**(사용법 오류 2) — 트레이 · 데이터 폴더를 건드리지 않고 비정상 종료를 만든다.
- 패닉 줄은 한 줄 — `PanicHookInfo` 의 기본 표시(`panicked at …:\n메시지`)는 줄이 갈려 `payload_as_str` · `location` 으로 직접 만든다.
- 서식은 손댄 파일만 `rustfmt`(저장소 일괄 적용 상태 아님 — 1d 와 같음).

## 테스트 결과

```text
$ cargo test -p coworkagent-app --test watchdog → test result: ok. 6 passed; 0 failed
$ cargo test -p an-core --test panic_log → test result: ok. 1 passed; 0 failed
$ cargo test --workspace → passed 323 failed 0 (합)
$ cargo clippy --workspace --all-targets -- -D warnings → Finished
```

RED 확인: 구현 전 `watchdog` 시험은 `unresolved import an_app::watchdog`, `panic_log` 는 `FAILED`(로그에 패닉 없음). 웹UI 는 이 stage 에서 바꾸지
않아 node 시험은 Stage 2 에서.

## 다음 단계

`work/stage2.md` — 자동 시작(`autostart.rs` · 설정 `autostart` · `PUT /api/settings` 훅 · 웹UI '시작 — 자동 실행').
