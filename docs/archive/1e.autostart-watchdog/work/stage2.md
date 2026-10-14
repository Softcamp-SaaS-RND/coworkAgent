# stage2 — 자동 시작 · 설정 · 웹UI

> 근거: [plan](../plan.md) Stage 2. 사용자 검증은 D28 — 19.prototype-acceptance 에서 일괄.

## 목표

앱이 시작할 때 자동 시작을 등록 · 경로 고침하고(Windows `HKCU\…\Run` · macOS LaunchAgent), 웹UI 설정에서 끄면 지운 뒤 끈 선택을 저장해 다음
실행에도 등록하지 않는다(A4). 대상 gate: E1 · E2 · E3 · E6.

## 산출 파일

- `crates/app/src/autostart.rs`(새) — `RUN_KEY` · `VALUE_NAME`(`coworkAgent`) · `LABEL`(`com.coworkagent`) · `sync(on)` · `launch_agent_plist` ·
  Windows `registered` · `register`(같은 값이면 쓰지 않음) · `unregister`(없어도 성공).
- `crates/core/src/settings.rs` — `AUTOSTART_KEY` · `autostart`(없으면 켜짐 — D13).
- `crates/app/src/notify.rs` — `SettingsPatch.autostart` · `set_autostart_hook` · `autostart()` · 설정 JSON `autostart` · `update` 가 훅 먼저(실패면
  저장 없음).
- `crates/app/src/main.rs` — 릴리스 빌드만: 훅 설치 · 켜져 있으면 시작 때 `sync(true)`(실패는 `warn`).
- `crates/app/Cargo.toml` — `windows-sys` `Win32_System_Registry`.
- 웹UI — `index.html` 카드 `startup`("시작 — 자동 실행" · 체크 "로그인할 때 자동 실행" · 안내) · `view.js` `settingsTree` · `app.js` `PREFS`.
- 테스트 — `app/tests/autostart.rs`(새, 4) · `view.test.mjs` `settings_tree` 목록에 "시작 — 자동 실행".

## 설계 결정

- **`RegSetKeyValueW`** — 키가 없으면 만든다(시험 키로 실측). 열기 · 닫기 핸들 관리 없음.
- **시험은 실제 Run 키를 쓰지 않는다** — 키 경로를 인자로 받는 함수를 `Software\coworkAgent-test-{pid}\Run` 으로 시험하고 `reg delete` 로 통째로
  지운다. 시험 뒤 실측: 실제 Run 키에 `coworkAgent` 값 없음 · 시험 키 남지 않음.
- **macOS 파일은 내용이 같으면 쓰지 않는다**(경로 고침 = 내용 비교). `launchctl` 은 부르지 않는다 — 다음 로그인 때 launchd 가 읽는다.
- **값 읽기 버퍼 1024 칸**(ponytail 주석) — 더 긴 경로는 오류로 드러난다.
- 시작 때 등록 실패 · 설정 읽기 실패는 시작을 막지 않는다(`warn`) — 자동 시작은 부가 기능.

## 테스트 결과

```text
$ cargo test -p coworkagent-app --test autostart → test result: ok. 4 passed; 0 failed
$ cargo test --workspace → passed 327 failed 0 (합)
$ cargo clippy --workspace --all-targets -- -D warnings → Finished
$ cargo clippy -p coworkagent-app --release -- -D warnings → Finished (릴리스만의 배선)
$ node --test --test-reporter=tap "crates/app/web/*.test.mjs" → # pass 47 # fail 0
```

RED 확인: 구현 전 `unresolved import an_app::autostart` · `settings::autostart` 없음 · `set_autostart_hook` 없음.

## 다음 단계

`work/stage3.md` — 매뉴얼 · PRODUCT · 실측(E7). 이 PC 에 coworkAgent 프로세스가 실행 중(2026-10-13 12:33 시작) — plan 승인 조건대로 끄지 않고
E7 은 19 TC 로 넘긴다.
