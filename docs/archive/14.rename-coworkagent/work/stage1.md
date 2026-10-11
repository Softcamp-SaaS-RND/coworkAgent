# stage1 — 이름 바꾸기 · 문서

## 목표

[plan](../plan.md) Stage 1. 대상 gate: E1 ~ E5.

## 산출 파일

- 코드: `crates/app/Cargo.toml`(패키지 `coworkagent-app` · bin `coworkAgent`) · `crates/core/src/paths.rs`(`APP_DIR` · DB · 로그) · `crates/app/src/tray.rs`
  (툴팁 · 시작 실패 파일) · `crates/net/src/invite.rs`(토큰 · 안내문) · `crates/agent/src/runner.rs`(가드 프롬프트) · 웹UI · 테스트 · `Cargo.lock`.
- 문서: PRODUCT · PROJECT · masterPlan(D14 · D15 개정 표시 · 개정 기록) · 매뉴얼 · `tools/check_crates.py`.

## 설계 결정

- 일괄 치환 4종(`agentNetwork` · `agentnetwork-app` · `agentnetwork-invite` · 알림 태그) — 대소문자 규칙 유지. archive · 인터뷰 기록은 당시 기록이라 제외.

## 테스트 결과

- 워크스페이스 131 passed · 0 failed, clippy 0, node `# fail 0`, `check_crates` violations=0, 릴리스 `coworkAgent.exe`.

## 다음 단계

평가 · 종결.
