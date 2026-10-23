# STATE.md — 작업 상태

> **세션 시작 시 가장 먼저 읽는 문서.** 지금 어디서 재개하는지의 단일 좌표 — **매번 덮어쓴다.**
> Task 목록은 폴더가 SSOT(`python tools/check_ddd_docs.py --tasks`), 미조치는 [overview/ledger.md](docs/overview/ledger.md).

- **갱신**: 2026-10-23 / 브랜치 `task/1z-8-settings-search`(작업 폴더 `../coworkAgent-1z-8`) — Task #1z-8 종결(archive) · main 반영 대기(선생님 승인)

## 현재 작업

#1z-8 설정 화면 · 검색 · 미리보기 — **종결**([report](docs/archive/1z-8.settings-search/report.md) — 상단 바 버전 · 검색 "나간 대화 포함" 스위치 · 유형 칩 S1 · 미리보기 2줄 ·
대화 정보 제목 줄 삭제 · 설정 시안 B(메뉴 8개 · 고른 화면만 · 처음 연결 · 목록 + 상세 · 스위치 · 2열 타일 · 디버깅), 버전 1.3.4, gate 8/8, 적대적 평가 2/2 — R1 high 1 · R2 medium 1 반영 · 미조치 없음).

- **중단 지점**: main squash 반영 · origin push 는 선생님 승인 뒤. 시험 인스턴스(scratchpad `ui1z8\data`) 정지함.
- **작업 메모**: 원 작업 폴더(main)의 다른 세션 미커밋(PRODUCT.md · docs/tech/security-posture.md)은 건드리지 않음 — main 반영 때 stash → 반영 → pop. `cargo fmt` 쓰지 않음, `cargo test` 는 PowerShell 로.

## 다음 행동

1. main squash 반영 · push(선생님 승인) → 브랜치 · 작업 폴더 정리. 1.3.4 서명 릴리스 여부 결정(1.3.3 은 릴리스됨).
2. 선생님 확인: #1z-8 구현자 해석(report — 검색 스위치 탭 동안만 · 960px · 로그 200줄 / 64 KiB · plan 개정 ② ③ · 로그 줄 자기 지문 · 다른 탭 저장이면 초안 버림).
3. 19 acceptance TC(사용자) · macOS zip 첨부 — 이전 STATE 그대로.

- **블로커 / 미해소**: 19 acceptance TC(사용자). 구현자 해석(사용자 미확인 — 각 report): 1q 결정 7 · 15, 1r plan 해석 1 · 3 · 결정 3 · 11, 1s 결정 1 · 2, 1z-2 결정 2,
  #1z-7 구현자 해석, #1z-8 결정 11 · plan 승인 갈음. #30 미조치 2행 원장 수용. #1z-4 미조치 1행 원장 조건 대기.
