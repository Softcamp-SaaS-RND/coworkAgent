# report — 1n.cli-start-stop

종결: 2026-10-17 · 브랜치 `task/1n-cli-start-stop` → main

## 결과

부모 도구가 하위 메뉴로 부르는 CLI `coworkAgent start [--parent <PID>]` · `open [--parent <PID>]` · `stop` 을 더했다. 꺼져 있어도 동작한다. `start` · `open` 은
실행 파일을 **트레이 없는 모드**(트레이 · 이벤트 루프 없음, 브라우저 안 엶)로 떼어 띄우고 받을 준비를 기다린다 — 부모(기본 = 부른 프로세스)가 끝나면 앱 ·
감시가 정상 종료, 트레이 모드로 떠 있으면 끄고 다시 띄운다. `stop` 은 로컬 API `POST /api/quit` → 감시 · 앱 잠금 풀림까지(10초), 응답이 없으면 감시와 그 자식을
강제 종료한다. 감시도 잠금(`supervisor.lock`)을 쥐어 어떤 실행 경로로도 앱 · 감시는 하나, 트레이(오류 트레이 포함)는 많아야 하나. 시작 실패는 종료 코드 8.

| 성공 기준 | 판정 | 근거 |
|---|---|---|
| A1 꺼진 상태 `start` — 트레이 없이 · 준비 뒤 0 · 브라우저 안 엶 | 충족 | E1 · E3 `start_again_stop_and_stop_when_off` · E8(트레이 UIA 버튼 없음) — 브라우저 안 엶은 코드 확인(자동 시험 없음) |
| A2 이미 트레이 없는 모드 = 0 · 트레이 모드 = 바꿈 | 충족 | E3 `start_again…`(같은 PID) · `start_switches_tray_to_headless` · E8 |
| A3 부모 끝 → 앱 · 감시 정상 종료 | 충족 | E3 `parent_exit_ends_headless_app` · E8(부모 셸 끝 → 프로세스 0) |
| A4 `open` — 떠 있으면 열기 · 꺼져 있으면 띄운 뒤 열기 | 충족 | E3 `open_when_running_opens_once` · E8 |
| A5 `stop` — 두 모드 · 오류 트레이 · 10초 · 꺼져 있으면 0 | 충족(상한 최악 약 12초 — 원장 `#1n R2-F4`) | E3 `start_again…` · `error_tray_is_single_and_stoppable` · E8 |
| A6 동시 · 거듭 실행 — 앱 · 감시 각 하나 · 트레이 ≤ 1 | 충족 | E3 `concurrent_supervisors_and_starts_leave_one` · `error_tray_is_single…` |
| A7 트레이 없는 모드 시작 실패 = 0 아닌 코드 + 사유 | 충족 | E3 `start_failure_exits_8_with_reason` |
| A8 인자 없는 실행 · 두 번째 실행 · 기존 CLI 회귀 없음 | 충족 | E7 cli 16 · E1 `watchdog` · E8(두 번째 실행 = 화면 열기) |
| A9 검증 명령 | 충족 | `check_gates` 8/8 |

- 적대적 평가 2 라운드(상한, codex-review 세션) — 라운드 1 high 2 · medium 2(강제 종료 PID · 남는 앱 · 코드 3 ↔ `abort()` · stop 상한 · 이전 버전) 모두 고침,
  라운드 2 high 3 · medium 1 중 사유 파일 세대(자식 PID)는 고침 · 상한은 일부. 미조치 5(사용자 수용 2026-10-17 — [eval](eval.md), 원장 `#1n R2-*`).
- plan 편차: 개정 4건(사용자 승인 2026-10-17 — E1 EXPECT · Windows 띄우기 플래그 · `router_control` · `start` 30초 되풀이). 리뷰 반영으로 plan 이 정하지 않은 것을
  더함 — `kill_tree`(감시 + 자식, Windows 이름 확인) · 사유 파일 `PID:` 줄로 시작 실패 판정 · `/api/quit` 404 → 4.

## 산출

- `an_core` — `paths` `instance_pid_file` · `supervisor_lock_file` · `supervisor_pid_file` · 디버그 `DATA_DIR_ENV`(`COWORKAGENT_DATA_DIR`), `instance`
  `is_held` · `write_pid` · `read_pid` · `acquire` 다시 잡기.
- `an_app` — 새 `control`(`start` · `open` · `stop`) · `proc`(부모 PID · 살아 있음 · 부모 지켜보기 · `kill` · `kill_tree` · 떼어 띄우기), `watchdog`(`PARENT_ENV` ·
  `START_FAILED` · `Outcome::StartFailed` · `acquire`), `server::router_control`(`mode` · `POST /api/quit`), `tray`(`Quit` · 사유 파일 함수), `cli`
  (`EXIT_START_FAILED` 8 · 사용법 3줄), `main`(감시 잠금 · 모드 가르기 · 트레이 없는 대기 · `instance.pid`).
- 시험 — 새 `tests/lifecycle.rs` 9건(디버그 실행 파일 + 임시 데이터 경로) · `watchdog` 1 · `ux_literals` 1 · 단위 `proc` 3 · `control` 2 · `an_core` 1.
- 문서 — 사용자 가이드 3 · 14.1 · 16장 · 개발자 가이드 3.1 · 4 · 4.1 · 7.1 · 8.10 · 9 · 매뉴얼 `an_app` · `an_core` · PRODUCT · README · masterPlan D13 · D14 보강 ·
  원장 6행.

## 남은 것

- macOS 실기 확인(Unix 길 컴파일 포함 — 이 PC 는 Windows 대상만, 원장 `#1n MAC`) · Unix 강제 종료 이름 확인(`#1n R2-F1u`).
- 상한 넘침(`#1n R2-F4`) — 상태 확인 · 강제 종료 대기를 남은 시간으로 자르기.
- 이 PC 반영(릴리스 빌드) · GitHub 반영 · 릴리스는 사용자 확인 뒤.
