# report — 1e.autostart-watchdog

종결: 2026-10-14 · 브랜치 `task/1e-autostart-watchdog` → main

## 결과

재부팅 뒤에도 앱이 켜져 있고, 비정상 종료돼도 저절로 다시 켜진다. 인자 없이 실행하면 작은 **감시 프로세스**가 지금의 앱(트레이 + 수신 · 실행 한 몸,
구조 그대로)을 자식으로 띄운다. 자식이 0 으로 끝나면(트레이 "종료" · 이미 실행 중이라 "열기" 를 넘김) 감시도 끝나고, 비정상 종료면 1초 뒤 다시
띄운다 — 남은 대기열 · outbox 는 1c `recover` 가 이어 간다. 60초 안 3번이면 멈추고 오류 트레이. CLI 는 감시를 거치지 않는다. 앱은 시작마다 **로그인
자동 실행**(Windows `HKCU\…\Run` · macOS LaunchAgent)을 설정 값으로 맞추고(기본 켜짐 · 경로 고침 · 꺼짐이면 지움), 웹UI 설정 "시작 — 자동 실행" 에서
끈다. 바꾸기 순서는 중단돼도 "저장 켜짐 · OS 꺼짐" 이 남지 않게. 패닉은 메시지 · 위치를 로그에. 처음 검토한 엔진 · 트레이 이원화는 실효성 검토 뒤
폐기했다(intent 라운드 2).

| 성공 기준 | 판정 | 근거 |
|---|---|---|
| A1 감시가 앱을 띄우고 강제 종료면 다시 · 남은 일 이어감 | 충족(자동 시험) · 실측 19 TC | E4 `crash_respawns_with_count` · `real_exe_cli_bypasses_and_crashes_stop` · E7 ABANDON |
| A2 60초 안 3번이면 멈춤 · 오류 트레이 | 충족(규칙) · 화면 19 TC | E4 `three_crashes_in_window_give_up` · `spaced_crashes_continue` · `spawn_failure_gives_up` · E7 ABANDON |
| A3 "종료" 면 감시도 끝 · 실행 중 다시 실행하면 감시 남지 않음 | 충족(규칙) · 실측 19 TC | E4 `normal_exit_runs_once`(tao `Exit` = 종료 코드 0 실측) · E7 ABANDON |
| A4 자동 시작 등록 · 경로 고침 · 끄면 지우고 다시 등록 안 함 | 충족(Windows API · 설정) · mac 19 TC 밖(D25) | E6(8) · E3 `settings_tree` · E7 · E8 ABANDON |
| A5 패닉 메시지 · 위치가 로그에 | 충족 | E5 |
| A6 CLI 는 감시를 거치지 않음 | 충족 | E4 `real_exe_cli_bypasses_and_crashes_stop` · 기존 `cli` `unknown_command_exe_exits_usage` |
| A7 검증 명령 | 충족 | E1(331) · E2 · E3(47) · `check_readme` · `check_crates` · `check_ddd_docs` |

- 적대적 평가 2 라운드(상한, codex-review 세션) — 라운드 1: high 1(OS 등록과 DB 저장이 따로 — 저장 실패 · 시작 때 꺼짐 미동기화) · medium 1(plist
  덮어쓰기) — 사실 · 고침(plan 개정 1). 라운드 2: high 1(중단 때 "저장 켜짐 · OS 꺼짐" 이 남음) — 사실 · 순서로 고침(plan 개정 2). 미조치 0.
- E7(Windows 릴리스 실측)은 이 PC 에서 사용자 앱이 실행 중이라 격리 불가 → 19 TC, E8(macOS)은 D25 → 둘 다 `ABANDON`.
- masterPlan S7 실현 · D13 보강 · S2 보강 — D 추가 없음.

## 산출

- `coworkagent-app`: `watchdog`(감시 규칙 · `spawn_app` · `restarts`) · `autostart`(Run 값 · LaunchAgent plist · `write_atomic` · `sync`) · `main`(감시
  분기 · 멈춤 오류 트레이 · 다시 띄움 로그 · 시작 맞추기 · 훅 설치 — 릴리스만) · `notify`(설정 `autostart` · 훅 · 바꾸기 순서) · `windows-sys`
  `Win32_System_Registry` · 웹UI 설정 카드 "시작 — 자동 실행".
- `an-core`: `settings::AUTOSTART_KEY` · `autostart` · `FileLogger::install` 패닉 훅.
- 시험: `app/tests/watchdog.rs`(6) · `app/tests/autostart.rs`(8) · `core/tests/panic_log.rs`(1) · `view.test.mjs` `settings_tree` 갱신.
- 문서: README(시작하기 1) · user-guide(3 · 15 · 16) · developer-guide(3.1 · 4 · 4.1) · an_app · an_core · PRODUCT(기술 스택 · 도메인 규칙) · masterPlan.

## 남은 것

- 19 인수 TC — 로그인 자동 실행 · 강제 종료 뒤 다시 켜짐(로그 "다시 띄움(n번째)") · 60초 안 3번 오류 트레이 · 트레이 "종료" 뒤 두 프로세스 없음 ·
  실행 중 다시 실행 · 설정 끄기 → Run 값 없음 · 다시 실행해도 없음(E7 이관).
- 확인 필요(종결 뒤 검토): 로그인 직후 알림 영역이 아직 아이콘을 받지 못하면 트레이 생성 실패 → `show_failure` → 종료 코드 0 → 감시가 정상 종료로
  보고 멈춘다(앱도 아이콘도 없음). 19 TC 에서 "로그인 직후 아이콘이 뜨는가 · 안 뜨면 로그에 '트레이 아이콘을 만들 수 없다'" 를 본다. 재현되면 트레이
  생성 실패를 0 아닌 종료로 바꾸고 첫 재시도 간격을 늘린다(1초 × 3 은 느린 셸 시작을 못 넘길 수 있음) — 새 Task.
- 관찰 항목: Windows 로그오프 · 종료 때 앱이 0 아닌 코드로 끝나면 감시가 1초 뒤 다시 띄우려 할 수 있다 — 19 TC 에서 "종료 때 '앱이 종료를 막고
  있습니다' 가 뜨지 않는가".
- macOS LaunchAgent 실측 — Phase 4(D25, E8 이관).
- 이 PC 실측(종결 뒤 · 2026-10-14, 1f 반영 빌드): 인자 없는 실행 → 감시 · 앱 두 프로세스(앱의 부모 = 감시) · `HKCU\…\Run` 값 `coworkAgent` =
  그 실행 파일 경로 · 앱 강제 종료(`taskkill /F`) → 감시가 다시 띄움 · 로그 "감시 프로세스가 다시 띄움(1번째)" · 피어 포트 다시 받음 · 상대 접속 그대로
  — 확인. 남은 E7 항목(60초 안 3번 오류 트레이 · 트레이 "종료" · 로그인 직후 아이콘 · 설정 끄기)은 19 TC.
- GitHub 반영(push)은 사용자 확인 뒤.
