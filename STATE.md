# STATE.md — 작업 상태

> **세션 시작 시 가장 먼저 읽는 문서.** 지금 어디서 재개하는지의 단일 좌표 — **매번 덮어쓴다.**
> 누적 기록은 여기 없다: Task 목록은 폴더가 SSOT(`python tools/check_ddd_docs.py --tasks`),
> 미조치는 [overview/ledger.md](docs/overview/ledger.md). 가드레일·검증 명령은 루트 `PROJECT.md`,
> 제품 구조는 루트 `PRODUCT.md`, 절차는 [etc/ddd-workflow.md](docs/etc/ddd-workflow.md).

- **갱신**: 2026-10-16 / main — 1l 종결 뒤 디자인 캔버스 갱신 · README · GitHub main squash push(1h ~ 1l)

## 현재 작업

| 항목 | 값 |
|------|-----|
| Task | 없음 — 다음 후보 = 통계 메뉴(별도 Task, 내용 미정) · 19 인수 |
| 단계 | 1l 종결 · GitHub push 뒤 |
| 상태 | 1h · 1j · 1k · 1l 종결 · main 병합 · GitHub main push(squash, 1h ~ 1l) · GitHub 릴리스 v0.3.0(사전 배포, 1i 까지 — 1h ~ 1l 릴리스 전) |
| 재개 좌표 | [1l report](docs/archive/1l.ui-icons/report.md) "남은 것" · 아래 다음 행동 |

- **직전에 한 일**: 1l — 메뉴 · 화면 제목 · 설정 트리 · ⋯ · "+ 새로 만들기" 아이콘(16px 한 벌) · 말풍선 색(내 글 강조색 · 상대 글 타일 색 층) · 설정 한 열 · '?'
  도움말 · 내 정보 → 프로필 "사용자 정보" 팝업 · 내 에이전트 검색 없앰 · 등록 칸 보이기 · 숨기기 · 타일 순서 설정(서버 열쇠 `ui_rail_order`) · 새로 만들기 자동 왕복
  없앰. masterPlan D35 개정 · S4 보강. 적대적 평가 2/2 라운드 — 지적 2건 고침, 미조치 1(Safari 15 실기 — 원장 `#1l R2-SAF` 조건 대기). 그 전: 1k(1j 다듬기) ·
  1j(메신저형 화면 · 스키마 18) · 1h(안정화 · 스키마 17).
- **중단 지점**: 사용자 결정 대기 — 릴리스(1h ~ 1l, 버전 올림) · `#1h R1-M1` A8 해석(수용 확인). 디자인 캔버스(1j 캔버스 판 17)를 1k · 1l 화면으로 갱신 —
  ⑦ "+ 새로 만들기 · 사용자 정보 팝업" 추가.
- **이 PC 반영**: main 릴리스 빌드로 실행 중(스키마 18 그대로 — 1l 은 웹UI + 설정 열쇠 하나). 화면 확인 [1l eval](docs/archive/1l.ui-icons/eval.md) E8.

## 다음 행동

1. 사용자 결정 — 릴리스(1h ~ 1l) · A8 해석(원장 `#1h R1-M1`).
2. 통계 메뉴 착수 인터뷰(보여줄 숫자).
3. 19 acceptance TC 일괄(사용자) — 1a ~ 1l TC + 1j(여러 상대 Alt + ↑ / ↓ · 앱과 연결 끊김 · 옛 그룹 이름표가 있는 DB 올리기 · 연결 목록 끌어 옮기기) ·
   1k(여러 상대 주고받은 글 타일 색 · 한 상대가 여러 그룹일 때 연결 목록 관리 줄) · 1l(macOS 12 Safari 15 상대 말풍선 색 · 타일 순서 그룹 → 사용자에서 끌어
   옮기기 · '?' 도움말 키보드).

- **블로커 / 미해소**: 1c 원장 미조치 4건(19 TC 뒤 판단). 1f 범위 밖 관찰(기존 outbox 종류의 "지운 뒤 정산" 순서). #1g R2-F1 · R1·R2-F2(1h 에서 뺌).
  캡처 일부(activity · search · relay · new-conversation · comm-*)는 1j 이전 레일 모습 · 주고받은 글 · 에이전트 화면 캡처 없음. 이전 대기 그대로 — UI 시안 차이 8건(원본 목록 확인 필요) · 매뉴얼의 미구현 코드 제안 7건 후속 여부 · `agentNetwork-t19` 워크트리 prunable.
