# STATE.md — 작업 상태

> **세션 시작 시 가장 먼저 읽는 문서.** 지금 어디서 재개하는지의 단일 좌표 — **매번 덮어쓴다.**
> Task 목록은 폴더가 SSOT(`python tools/check_ddd_docs.py --tasks`), 미조치는 [overview/ledger.md](docs/overview/ledger.md).

- **갱신**: 2026-10-24 / `main`(`2bbcc5a` = #1z-11 squash, `e65f617` 위) — main 반영 · 개발 PC 다시 빌드 · 미리보기 바탕 더 연하게(`fc556c5`, 사용자 지시) · origin push 완료, 브랜치 · 작업 폴더 정리는 선생님 승인 뒤

## 현재 작업

#1z-11 개요 미리보기 시안 C "편안함 · 종이 톤" — **종결**([report](docs/archive/1z-11.overview-calm/report.md) — 미리보기만: 종이 바탕 · 절 사이 선, 상대 이름 14px · 500 · 타일 없음,
15% 확대 뺌, 카드 한 색 테두리 · 유형 아이콘 색 · "새 글 n" · 바닥 상태 점, 카드 안 말풍선 12px · 1.5, "+n개 더 보기", 토큰 `--pv-*` · 다크 `--dark-pv-*`, 버전 1.3.5 그대로,
gate 6/6, 적대적 평가 R1 approve(지적 0 — R2 생략), 미조치 없음).

- **중단 지점**: local main 반영(`2bbcc5a`) · `python tools/rebuild_run.py --yes` 성공(릴리스 빌드 · 새 앱 시작 — 선생님 로컬 시험). 미리보기 바탕 `--pv-bg` #F9F7F3(사용자 지시 — Task 밖) · 다시 빌드 · origin push(`PS_MIN_GUARD` 제거 포함). 브랜치 `task/1z-11-overview-calm` · 작업 폴더 `coworkAgent-1z-11` 정리는 선생님 승인 뒤. 시험 인스턴스(디버그 · scratchpad `ui1z11\data`)는 꺼져 있음.
- **작업 메모**: 원 작업 폴더(main)의 다른 세션 미커밋(PRODUCT.md · docs/tech/security-posture.md)은 건드리지 않음 — main 반영 때 stash → 반영 → pop(이번 Task 도 PRODUCT.md 를 고침 — 겹치면 손으로 합침).
  main = origin/main(push 완료). `cargo fmt` 쓰지 않음, `cargo test` 는 PowerShell 로.

## 다음 행동

1. 버전 1.3.6 으로 올림 · 개발 PC 다시 빌드(사용자 지시, local 커밋). 선생님 결정: origin push · #1z-11 브랜치 · 작업 폴더 정리 · 1.3.6 서명 릴리스.
   이후 Task 는 worktree 없이 원 작업 폴더에서 브랜치만(2026-10-24 사용자 지시).
2. 선생님 확인: #1z-11 구현자 해석(report — 글자 "새 글 n" · "+n개 더 보기" · 말풍선 사이 12px 등) · #1z-10 구현자 해석 · 원장 `#1z-10 R2-2`.
3. 19 acceptance TC(사용자) · macOS zip 첨부 — 이전 STATE 그대로.

- **블로커 / 미해소**: 19 acceptance TC(사용자). 구현자 해석(사용자 미확인 — 각 report): 1q 결정 7 · 15, 1r plan 해석 1 · 3 · 결정 3 · 11, 1s 결정 1 · 2, 1z-2 결정 2,
  #1z-7 구현자 해석, #1z-8 결정 11 · plan 승인 갈음, #1z-9 report 구현자 해석, #1z-10 report 구현자 해석(편차는 2026-10-24 결정 — 확인 줄 뺌), #1z-11 report 구현자 해석.
  #30 미조치 2행 원장 수용. #1z-4 미조치 1행 원장 조건 대기 · #1z-10 1행 열림.
