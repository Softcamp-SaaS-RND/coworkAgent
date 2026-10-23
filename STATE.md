# STATE.md — 작업 상태

> **세션 시작 시 가장 먼저 읽는 문서.** 지금 어디서 재개하는지의 단일 좌표 — **매번 덮어쓴다.**
> Task 목록은 폴더가 SSOT(`python tools/check_ddd_docs.py --tasks`), 미조치는 [overview/ledger.md](docs/overview/ledger.md).

- **갱신**: 2026-10-23 / 브랜치 `task/1z-10-conv-info`(작업 폴더 `D:\dev\git\coworkAgent-1z-10`, 기준 `6594e46`) — Task #1z-10 종결(archive) · main 반영 대기(선생님 승인)

## 현재 작업

#1z-10 세부정보 대화 정보 개편 · 앱 전체 아이콘 · 설정 상세 단추 줄 — **종결**([report](docs/archive/1z-10.conv-info/report.md) — 대화 정보 = 기간 · 컨텍스트 + 받은 대화 답하는 도구 ·
읽는 폴더 · 실행 명령 복사(스키마 28 · `runner::launch_line` — PowerShell 7.3+ / POSIX), 글자 앞 아이콘 30 · 설정 제목 강조색, 설정 상세 상자 = 내용 높이 · 단추 줄 내용 아래, 버전 1.3.5 그대로,
gate 8/8, 적대적 평가 2/2 — R1 high 1 · medium 2 · R2 medium 2 반영, 미조치 1 = 원장 `#1z-10 R2-2` 열림).

- **중단 지점**: 브랜치에서 종결 커밋. main squash 반영(#1z-9 `8c6e205` · `6594e46` 위) · origin push · 1.3.5 서명 릴리스는 선생님 승인 뒤. 시험 인스턴스(디버그 · scratchpad `ui1z8\data`)는 꺼져 있음.
- **작업 메모**: 원 작업 폴더(main)의 다른 세션 미커밋(PRODUCT.md · docs/tech/security-posture.md)은 건드리지 않음 — main 반영 때 stash → 반영 → pop(이번 Task 도 PRODUCT.md 를 고침 — 겹치면 손으로 합침).
  `cargo fmt` 쓰지 않음, `cargo test` 는 PowerShell 로.

## 다음 행동

1. 선생님 결정: #1z-10 main 반영(squash) → 개발 PC 다시 빌드(`python tools/rebuild_run.py`) · origin push(#1z-9 와 함께) · 브랜치 · 작업 폴더 정리 · 1.3.5 서명 릴리스.
2. 선생님 확인: #1z-10 편차(복사 명령 앞 PowerShell 확인 줄 — 결정 2 "그대로" 와 문자 그대로는 다름) · 구현자 해석(report) · 원장 `#1z-10 R2-2`(회귀 시험 — 구조 변경 필요).
3. 19 acceptance TC(사용자) · macOS zip 첨부 — 이전 STATE 그대로.

- **블로커 / 미해소**: 19 acceptance TC(사용자). 구현자 해석(사용자 미확인 — 각 report): 1q 결정 7 · 15, 1r plan 해석 1 · 3 · 결정 3 · 11, 1s 결정 1 · 2, 1z-2 결정 2,
  #1z-7 구현자 해석, #1z-8 결정 11 · plan 승인 갈음, #1z-9 report 구현자 해석, #1z-10 report 편차 · 구현자 해석. #30 미조치 2행 원장 수용. #1z-4 미조치 1행 원장 조건 대기 · #1z-10 1행 열림.
