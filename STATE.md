# STATE.md — 작업 상태

> **세션 시작 시 가장 먼저 읽는 문서.** 지금 어디서 재개하는지의 단일 좌표 — **매번 덮어쓴다.**
> Task 목록은 폴더가 SSOT(`python tools/check_ddd_docs.py --tasks`), 미조치는 [overview/ledger.md](docs/overview/ledger.md).

- **갱신**: 2026-11-03 / 브랜치 `task/1z-19-tray-guide` — #1z-19 stage 3 완료 · 메뉴 글자 `매뉴얼` 개정 · 커밋 · 릴리스 빌드(rebuild_run) · gate 11/11

## 현재 작업

#1z-19.tray-guide — 트레이 메뉴 `매뉴얼`(GitHub user-guide.md 열기) + 사용자 가이드 간결 재작성 · 캡처. 단계 = stage 3 완료 — 구현 끝, 적대적 평가 전.
재개 좌표: [plan](docs/tasks/1z-19.tray-guide/plan.md)· [eval](docs/tasks/1z-19.tray-guide/eval.md) 원장(11/11).

- **직전에 한 일**: stage 3 — 사용자 가이드 579 → 약 300줄(10장 · 캡처 11), README `§n` 새 장 · 1.4.4 줄, an_app · developer-guide · web-ui · manual README · PRODUCT 줄 — [stage3](docs/tasks/1z-19.tray-guide/work/stage3.md).
- **중단 지점**: gate 11/11 — 적대적 평가 전. 메뉴 글자 `사용자 매뉴얼` → `매뉴얼`(사용자 지시 — intent 결정 1 라운드 2).
- **작업 메모**: 원 작업 폴더의 다른 세션 미커밋(PRODUCT.md 끝 2줄 security-posture 링크 · 미추적 docs/tech/security-posture.md)은 커밋에서 뺀다.
  `cargo fmt` 쓰지 않음, `cargo test` 는 PowerShell 로(`CARGO_TARGET_DIR=target	15`).

## 다음 행동

1. `codex-peer-review` 라운드 1/2 → eval · report · 종결.
2. 이전 후보: 실기기 트레이 "확인할 일"(Windows · macOS) · stage2-1 접힘 저장 실패 · `os error 10055` 간헐 실패 · 19 acceptance TC(사용자) · macOS zip 첨부.

- **블로커 / 미해소**: 없음.
