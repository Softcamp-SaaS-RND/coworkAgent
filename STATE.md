# STATE.md — 작업 상태

> **세션 시작 시 가장 먼저 읽는 문서.** 지금 어디서 재개하는지의 단일 좌표 — **매번 덮어쓴다.**
> 누적 기록은 여기 없다: Task 목록은 폴더가 SSOT(`python tools/check_ddd_docs.py --tasks`),
> 미조치는 [overview/ledger.md](docs/overview/ledger.md). 가드레일·검증 명령은 루트 `PROJECT.md`,
> 제품 구조는 루트 `PRODUCT.md`, 절차는 [etc/ddd-workflow.md](docs/etc/ddd-workflow.md).

- **갱신**: 2026-10-17 / main — 버전 0.6.0 · 1o 워크트리 · 브랜치 정리, 진행 중 Task 없음

## 현재 작업

| 항목 | 값 |
|------|-----|
| Task | 없음(1n · 1o 종결 — archive) |
| 단계 | 다음 Task 선정 대기 |
| 상태 | 1o — 대화방 목록 내 대화 · agent 대화 · 공유 대화 · 말풍선 음영 · 관람자 초대(스키마 20 · `Share*` 프레임) · 레일 우클릭 · 설명 지우기 |
| 재개 좌표 | [1o report](docs/archive/1o.chat-list-split/report.md) · [1n report](docs/archive/1n.cli-start-stop/report.md) |

- **직전에 한 일**: 1o 구현(stage 1 ~ 4) · eval(gate 14 · ABANDON 2 · 적대적 리뷰 2 라운드) · A9 관람자 24시간 한정 사용자 승인 · main(1n) 합침 · report · masterPlan D38 · 원장 5행 · archive.
- **중단 지점**: 없음. 버전 0.6.0(통신 형식 · 스키마 20). GitHub 반영 · 릴리스 · 이 PC 반영은 사용자 확인 뒤.

## 다음 행동

1. 19 acceptance TC 일괄(사용자) — 1n macOS 실기 · 1o 세 PC 관람자 · 레일 우클릭 포함.
2. 다음 Task 후보 — 공유 글 따로 보내는 줄(원장 `#1o R1-F3`) · 통계 메뉴.

- **블로커 / 미해소**: 1c 원장 미조치 4건(19 TC 뒤 판단). 1f 범위 밖 관찰(기존 outbox 종류의 "지운 뒤 정산" 순서). #1g R2-F1 · R1·R2-F2(1h 에서 뺌).
  캡처 일부(activity · search · relay · new-conversation · comm-*)는 1j 이전 레일 모습 · 주고받은 글 · 에이전트 화면 캡처 없음. 이전 대기 그대로 — UI 시안 차이 8건(원본
  목록 확인 필요) · 매뉴얼의 미구현 코드 제안 7건 후속 여부 · `agentNetwork-t19` 워크트리 prunable.
