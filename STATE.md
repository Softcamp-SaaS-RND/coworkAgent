# STATE.md — 작업 상태

> **세션 시작 시 가장 먼저 읽는 문서.** 지금 어디서 재개하는지의 단일 좌표 — **매번 덮어쓴다.**
> Task 목록은 폴더가 SSOT(`python tools/check_ddd_docs.py --tasks`), 미조치는 [overview/ledger.md](docs/overview/ledger.md).

- **갱신**: 2026-10-21 / task/1z-chat-look — Task #1z gate 14/14 충족(E11 실 화면 DOM 측정), 종결 대기

## 현재 작업

| 항목 | 값 |
|------|-----|
| Task | #1z.chat-look — 사람 ↔ 에이전트 구분 · 입력창 여백 · 의견 · 레일 끌기 · 타일 순서 · 그룹설정 |
| 단계 | gate 전건 충족 → report · 종결 대기(사용자) |
| 재개 좌표 | [eval](docs/tasks/1z.chat-look/eval.md) · [stage2](docs/tasks/1z.chat-look/work/stage2.md) · [plan](docs/tasks/1z.chat-look/plan.md) |

- **직전에 한 일**: Design 캔버스 시안 3안 → 사용자 확정(말풍선 A · 목록 A · 에이전트 아이콘 · 의견 Shift+Enter 호환), intent · plan 작성.
- **중단 지점**: 종결 전 — 미조치 지적 2행(DOM 이벤트 경로 자동 시험 없음 — eval.md) 처리 방침(U1) 사용자 결정 대기.
  PRODUCT.md 의 다른 세션 수정 · docs/tech/security-posture.md 는 그대로(미커밋).

## 다음 행동

1. U1 결정 뒤 report.md · 매뉴얼 · 원장(ledger) · masterPlan 환류(S4 · S1 · D35 · D37 · 개정 기록) · archive · main merge.
2. v1.0.0 · v1.0.1 · v1.1.0 macOS zip 첨부(보유한 Mac). 19 acceptance TC(사용자).

- **블로커 / 미해소**: 19 acceptance TC(사용자). 구현자 해석(사용자 미확인 — 각 report): 1q 결정 7 · 15,
  1r plan 해석 1 · 3 · 결정 3 · 11, 1s 결정 1 · 2. D34 개정 문구 "편집 · 삭제는 CLI(· API)" 와 달리 CLI 에 편집 · 삭제 명령 없음(확인 필요).
