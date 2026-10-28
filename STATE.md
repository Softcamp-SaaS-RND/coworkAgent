# STATE.md — 작업 상태

> **세션 시작 시 가장 먼저 읽는 문서.** 지금 어디서 재개하는지의 단일 좌표 — **매번 덮어쓴다.**
> Task 목록은 폴더가 SSOT(`python tools/check_ddd_docs.py --tasks`), 미조치는 [overview/ledger.md](docs/overview/ledger.md).

- **갱신**: 2026-10-28 / 브랜치 `task/1z-17-time-date` — #1z-17 gate 전건 PASS(측정 `bc200c9`) · v1.4.2 릴리스 · origin push(사용자 지시) · 적대적 평가 · 종결 전

## 현재 작업

#1z-17 time-date — gate E1 ~ E9 PASS, v1.4.2 릴리스(사용자 서명 `dist\coworkAgent.exe`) · origin push. 남음: codex 적대적 평가 · report · 종결.
재개 좌표: [eval](docs/tasks/1z-17.time-date/eval.md) · [plan](docs/tasks/1z-17.time-date/plan.md).

- **직전에 한 일**: stage 1 · 2(날짜 표시방식 · 설정 1열 900px · 드롭다운 잘림 · 레일 상태 점 · 에이전트 저장 · 삭제 창 · 추가 지시 맨 앞 · 1.4.2) → 커밋 `bc200c9` →
  `python tools/check_gates.py docs/tasks/1z-17.time-date` PASS → 서명 exe 대조(서명 전 `target\release` 와 PE 머리 7 바이트 + 서명 10,640 바이트만 다름, `cargo build --release` 할 일 없음).
- **중단 지점**: 적대적 평가 전(사용자 지시로 릴리스 먼저).
- **작업 메모**: 원 작업 폴더의 다른 세션 미커밋(PRODUCT.md 끝 2줄 security-posture 링크 · 미추적 docs/tech/security-posture.md)은 커밋에서 뺐다.
  `cargo fmt` 쓰지 않음, `cargo test` 는 PowerShell 로(`CARGO_TARGET_DIR=target\t15`).

## 다음 행동

1. codex-peer-review(라운드 상한 2) → eval.md → report.md(결정 필요 intent-12 · stage2-1) → masterPlan D31 · S4 환류 · 개정 기록 → archive 이동.
2. 이전 후보 그대로: stage2-1 접힘 저장 실패 · `os error 10055` 간헐 실패 조사 · 19 acceptance TC(사용자) · macOS zip 첨부.

- **블로커 / 미해소**: 사용자 결정 intent-12(추가 지시 위치 해석) · stage2-1(사이드바 제목 칸). 19 acceptance TC(사용자).
