# STATE.md — 작업 상태

> **세션 시작 시 가장 먼저 읽는 문서.** 지금 어디서 재개하는지의 단일 좌표 — **매번 덮어쓴다.**
> Task 목록은 폴더가 SSOT(`python tools/check_ddd_docs.py --tasks`), 미조치는 [overview/ledger.md](docs/overview/ledger.md).

- **갱신**: 2026-11-03 / main — #1z-19 종결(v1.4.4 main push · 리뷰 1/2 수정 · archive) · v1.4.4 릴리스

## 현재 작업

없음 — #1z-19.tray-guide 종결([report](docs/archive/1z-19.tray-guide/report.md)). 트레이 메뉴 "매뉴얼"(GitHub 사용자 가이드) · 사용자 가이드 10장 · 캡처 11장, 버전 1.4.4.

- **직전에 한 일**: 적대적 리뷰 라운드 1/2(지적 2 — 가이드 재작성 때 빠진 보안 안내: 초기화 뒤 남는 Claude Code 세션 기록 · 원격 삭제 지연) → 가이드 4.8 · 10.2 복원,
  report · masterPlan S4 · 개정 기록 · archive 이동 · 종결 squash push.
- **중단 지점**: v1.4.4 릴리스 끝(main `b1332dc`, 사용자 서명 · SHA-256 `7c8e50f7` · 받은 파일 대조 · `real_` 시험 2 ok). macOS zip 없음 — 1.4.3 zip 그대로.
- **작업 메모**: 원 작업 폴더의 다른 세션 미커밋(PRODUCT.md 끝 2줄 security-posture 링크 · 미추적 docs/tech/security-posture.md)은 커밋에서 뺀다.
  `cargo fmt` 쓰지 않음, `cargo test` 는 PowerShell 로(`CARGO_TARGET_DIR=target/t15`). 종결 중 `cargo test --workspace` 1회 rc=101 → 재실행 2회 rc 0(간헐).

## 다음 행동

1. macOS 1.4.4 zip 첨부(Mac 빌드 — 사용자) · 트레이 "매뉴얼" macOS 실기 확인.
2. 이전 후보: 실기기 트레이 "확인할 일"(Windows · macOS) · stage2-1 접힘 저장 실패 · `os error 10055` 간헐 실패 · 19 acceptance TC(사용자) · macOS zip 첨부.

- **블로커 / 미해소**: 없음.
