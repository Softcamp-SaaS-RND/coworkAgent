# STATE.md — 작업 상태

> **세션 시작 시 가장 먼저 읽는 문서.** 지금 어디서 재개하는지의 단일 좌표 — **매번 덮어쓴다.**
> 누적 기록은 여기 없다: Task 목록은 폴더가 SSOT(`python tools/check_ddd_docs.py --tasks`),
> 미조치는 [overview/ledger.md](docs/overview/ledger.md). 가드레일·검증 명령은 루트 `PROJECT.md`,
> 제품 구조는 루트 `PRODUCT.md`, 절차는 [etc/ddd-workflow.md](docs/etc/ddd-workflow.md).

- **갱신**: 2026-10-14 / main — 1g 종결 · README · 사용자 가이드 화면 캡처 갱신 · GitHub squash push(1d ~ 1g)

## 현재 작업

| 항목 | 값 |
|------|-----|
| Task | 없음 — 1g.rail-scope 종결([report](docs/archive/1g.rail-scope/report.md)) |
| 단계 | 종결 · 병합 |
| 상태 | gate E1 ~ E6 PASS(E5 = 이 PC 화면 27/27) · 적대적 평가 2/2(R1 high 1 · medium 1 고침, R2 medium 1 · low 1 미조치) |
| 재개 좌표 | [1g report](docs/archive/1g.rail-scope/report.md) `## 남은 것` |

- **직전에 한 일**: 1g — 레일 '나' = 메뉴 6(대화 요약 · 내 활동 · 요청 · 저장된 항목 · 주고받은 글 · 에이전트)을 상대별 접이식 절로(접힘 = 설정
  `ui_folded`), 상대 타일 = 메뉴 4 · 첫 화면 주고받은 글 · 그 상대만, 사이드바 에이전트 절 제거. masterPlan D35.
  그 전: 1f(복사 아이콘 · 편집 모드) · 1e(감시 프로세스 · 자동 시작) · 1d.
- **중단 지점**: 미조치 2건 사용자 결정 대기 — 상대별 대기 요청 정확한 수(서버 집계 API, intent 제외 개정 필요) · 여러 창 접힘 덮어쓰기. GitHub 는 1d ·
  간헐 실패 조치 · 1e · 1f · 1g 를 squash 1커밋으로 push 했다(사용자 지시 "readme 및 사용자 매뉴얼 업데이트 후 스퀴즈 푸시까지 진행").
- **이 PC 반영**: `task/1g-rail-scope` `daed561` 릴리스 빌드로 실행 중(감시 + 앱, 자동 시작 등록 = `target\release` 경로). 실행 전 DB 사본
  `coworkAgent.db.pre-1g.bak`. 1g 화면 27/27([1g eval](docs/archive/1g.rail-scope/eval.md) E5).

## 다음 행동

1. 미조치 2건 결정(고치기 → 새 Task / 수용).
2. 19 acceptance TC 일괄(사용자) — 1a ~ 1g TC 추가분(1g: 상대 둘 이상에서 상대별 절 순서 · 요청 · 저장 묶음 · 앱 다시 실행 뒤 접힘).

- **블로커 / 미해소**: 1c 원장 미조치 4건(19 TC 뒤 판단). 1e 남은 것(로그인 직후 트레이 생성 실패 위험 — 19 TC). 1f 범위 밖 관찰(기존 outbox 종류의
  "지운 뒤 정산" 순서). 이전 대기 그대로 — UI 시안 차이 8건 · 매뉴얼의 미구현 코드 제안 7건 후속 여부 · `agentNetwork-t19` 워크트리 prunable.
