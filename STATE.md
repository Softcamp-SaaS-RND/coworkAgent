# STATE.md — 작업 상태

> **세션 시작 시 가장 먼저 읽는 문서.** 지금 어디서 재개하는지의 단일 좌표 — **매번 덮어쓴다.**
> Task 목록은 폴더가 SSOT(`python tools/check_ddd_docs.py --tasks`), 미조치는 [overview/ledger.md](docs/overview/ledger.md).

- **갱신**: 2026-10-23 / 브랜치 `task/1z-9-agent-engine`(작업 폴더 `../coworkAgent-1z-9`) — Task #1z-9 종결(archive) · main 반영 대기(선생님 승인)

## 현재 작업

#1z-9 에이전트 실행 도구 Claude Code / Codex — **종결**([report](docs/archive/1z-9.agent-engine/report.md) — 등록 시안 B 2단계 · 도구 택1 필수 · 등록 뒤 고정 · Codex = `codex exec`
(-C 폴더 · read-only · thread_id · Git 저장소 아닌 폴더 허용 기본 켬) · Claude 인자 D22 개정(로컬과 맞춤 · 편집 도구만 막음), 버전 1.3.5 · 스키마 27, gate 8/8, 적대적 평가 2/2 — R1 medium 1 반영 · R2 approve · 미조치 없음).

- **중단 지점**: main squash 반영 · push 는 선생님 승인 뒤. 시험 인스턴스(scratchpad `ui1z9\data`) 정지함. 앞서 v1.3.4 정식 릴리스(`0e5d88d`) 끝.
- **작업 메모**: 원 작업 폴더(main)의 다른 세션 미커밋(PRODUCT.md · docs/tech/security-posture.md)은 건드리지 않음 — main 반영 때 stash → 반영 → pop(이번 Task 도 PRODUCT.md 를 고침 — 겹치면 손으로 합침).
  `cargo fmt` 쓰지 않음, `cargo test` 는 PowerShell 로.

## 다음 행동

1. main squash 반영 · push(선생님 승인) → 브랜치 · 작업 폴더 정리. 1.3.5 서명 릴리스 여부.
2. 선생님 확인: #1z-9 구현자 해석(report — MCP 미리 허용 안 함 · CLI 없는 PC 등록 = 실행 때 실패 · Codex 환경 변수 그대로 · Codex 컨텍스트 표시 없음) · 보안 영향(폴더 밖 읽기 · 훅 적용).
3. 19 acceptance TC(사용자) · macOS zip 첨부 — 이전 STATE 그대로.

- **블로커 / 미해소**: 19 acceptance TC(사용자). 구현자 해석(사용자 미확인 — 각 report): 1q 결정 7 · 15, 1r plan 해석 1 · 3 · 결정 3 · 11, 1s 결정 1 · 2, 1z-2 결정 2,
  #1z-7 구현자 해석, #1z-8 결정 11 · plan 승인 갈음, #1z-9 report 구현자 해석. #30 미조치 2행 원장 수용. #1z-4 미조치 1행 원장 조건 대기.
