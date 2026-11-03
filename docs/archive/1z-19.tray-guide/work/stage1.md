# stage 1 — 트레이 메뉴 · 버전

## 목표

[plan](../plan.md) stage 1 — 정상 모드 트레이 메뉴 `열기` · `사용자 매뉴얼` · `종료`, `사용자 매뉴얼` = GitHub main 사용자 가이드 열기. 버전 1.4.4.

## 산출 파일

- `crates/app/src/tray.rs` — `pub const USER_GUIDE`, `run` 메뉴 셋(`build(&[open, guide, quit], …)`), `guide_id` 클릭 = `browser::open(USER_GUIDE)`(실패는 `log::error!`), 머리 주석 · `run` 문서 주석, 시험 `user_guide_url`.
- `crates/app/tests/ux_literals.rs` — PRESENT `MenuItem::new("사용자 매뉴얼"`, COUNTS `MenuItem::new(` 4 → 5(plan "바꾸는 시험" — 의도된 변경).
- `crates/{core,net,agent,app}/Cargo.toml` · `Cargo.lock` — 1.4.3 → 1.4.4.

## 설계 결정

- `사용자 매뉴얼` 은 `on_seen` 을 부르지 않는다 — 확인할 일 표시는 웹UI 를 연 때만 지운다(plan stage 1).
- 오류 모드 `run_error` 는 그대로(intent 결정 1).
- 주소는 상수 하나 — 메뉴 · 시험 · gate E2 가 같은 값을 본다.

## 테스트 결과

- RED(구현 전): `cargo test -p coworkagent-app --test ux_literals` → `test result: FAILED. 3 passed; 2 failed`(literal_counts · literals_present), `--lib tray` → `error[E0425]: cannot find value USER_GUIDE`.
- GREEN(2026-11-03, `CARGO_TARGET_DIR=target\t15`):
  - `cargo test -p coworkagent-app --lib tray` → `test result: ok. 3 passed; 0 failed`
  - `cargo test -p coworkagent-app --test ux_literals` → `test result: ok. 5 passed; 0 failed`
  - `cargo test --workspace` → `test result:` 줄 전부 `0 failed`
  - `cargo clippy --workspace --all-targets -- -D warnings` → `Finished`
  - `node --test --test-reporter=tap "crates/app/web/*.test.mjs"` → `# pass 181` · `# fail 0`
  - `python tools/check_code_tidy.py` → `PASS — 제품 원본 116개 · 최대 968줄 · 이력 표식 0`, `python tools/check_crates.py` → `violations=0`, `python tools/check_ddd_docs.py` → `PASS`
- 실제 앱 트레이 클릭(E3)은 stage 2 의 트레이 캡처와 함께.

## 다음 단계

stage 2 — 캡처 도구(`design/mock.py` · `design/shots.mjs`) · 웹 10장 · 트레이 1장 · E3 실제 앱 확인.
