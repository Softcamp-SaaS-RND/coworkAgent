# STATE.md — 작업 상태

> **세션 시작 시 가장 먼저 읽는 문서.** 지금 어디서 재개하는지의 단일 좌표 — **매번 덮어쓴다.**
> Task 목록은 폴더가 SSOT(`python tools/check_ddd_docs.py --tasks`), 미조치는 [overview/ledger.md](docs/overview/ledger.md).

- **갱신**: 2026-10-28 / 브랜치 `task/1z-18-ui-fixes` — #1z-18 종결(리뷰 2 라운드 · 추가 요청 결정 10 ~ 14 · gate 8/8 · archive) · main squash push `b110c16` · v1.4.3 릴리스

## 현재 작업

없음 — #1z-18 종결 · v1.4.3 릴리스 끝(main `b110c16`, 사용자 서명 · SHA-256 `742a7f2c` · 받은 파일 대조 · `real_` 시험 latest = v1.4.3). [report](docs/archive/1z-18.ui-fixes/report.md).

- **직전에 한 일**: #1z-18 — 요청 의견 칸 · archive 토글 · 즐겨찾기 줄 · 에이전트 응답 줄 · 북마크 목록 해제(리뷰 지적 3 고침) · 트레이 "확인할 일"(받은 요청 · 연결 요청 ·
  연결 승인 · 내 요청 상태 변경, 알림 설정과 무관) · 설정 "브라우저 알림". 사용자 "U2: 종결 승인" · "U3: squash push 까지만".
- **중단 지점**: 없음.
- **작업 메모**: 원 작업 폴더의 다른 세션 미커밋(PRODUCT.md 끝 2줄 security-posture 링크 · 미추적 docs/tech/security-posture.md)은 커밋에서 뺀다.
  `cargo fmt` 쓰지 않음, `cargo test` 는 PowerShell 로(`CARGO_TARGET_DIR=target\t15`).

## 다음 행동

1. 실기기 트레이 "확인할 일" 확인(Windows · macOS — 사용자). macOS 빌드 첨부는 이전 후보와 함께.
2. 이전 후보: stage2-1 접힘 저장 실패 · `os error 10055` 간헐 실패 조사 · 19 acceptance TC(사용자) · macOS zip 첨부.

- **블로커 / 미해소**: 없음. 19 acceptance TC(사용자).
