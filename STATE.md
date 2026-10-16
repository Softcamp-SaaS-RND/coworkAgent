# STATE.md — 작업 상태

> **세션 시작 시 가장 먼저 읽는 문서.** 지금 어디서 재개하는지의 단일 좌표 — **매번 덮어쓴다.**
> 누적 기록은 여기 없다: Task 목록은 폴더가 SSOT(`python tools/check_ddd_docs.py --tasks`),
> 미조치는 [overview/ledger.md](docs/overview/ledger.md). 가드레일·검증 명령은 루트 `PROJECT.md`,
> 제품 구조는 루트 `PRODUCT.md`, 절차는 [etc/ddd-workflow.md](docs/etc/ddd-workflow.md).

- **갱신**: 2026-10-16 / main — 버전 0.5.0 · GitHub push(squash) · 릴리스 v0.5.0(사전 배포, 1m 까지)

## 현재 작업

| 항목 | 값 |
|------|-----|
| Task | 없음 — 다음 후보 = 통계 메뉴(별도 Task, 내용 미정) · 19 인수 |
| 단계 | 0.5.0 릴리스 직후 |
| 상태 | 1h ~ 1m 종결 · main 병합 · GitHub main push(squash, 1m 까지) · GitHub 릴리스 v0.5.0(사전 배포, 1m 까지) |
| 재개 좌표 | [1m report](docs/archive/1m.context-compact/report.md) "남은 것" · 아래 다음 행동 |

- **직전에 한 일**: 1m — 대화 머리 컨텍스트 표시("42k / 1M", 상한 1M 고정 · 60 · 80% 색) · 압축(요약 후 새 세션) · 비우기 · 자동 압축(답하는 PC, 800k, 설정 기본 켜짐) ·
  피어 프레임 `Context`(outbox, 보내기 전 `Pong.ctx` — 0.4.0 거절 안내) · 자동 왕복 두 세션 함께 · 스키마 19. masterPlan D19 보강. 적대적 평가 2/2 라운드 — 지적 6건
  고침, 미조치 4(권고 대안 3 · 두 PC 실제 확인 — 원장 `#1m R2-PC` 조건 대기). 그 전: 1l(아이콘 · 말풍선 색 · 설정 정리) · 디자인 캔버스 판 17 · v0.4.0 릴리스.
- **중단 지점**: 사용자 결정 대기 — `#1h R1-M1` A8 해석(수용 확인). 0.5.0 push · 릴리스 끝(README "알아 둘 점" 에 압축 사용량 · 0.5.0 상호 조건). 다음 화면 Task intent 에 "디자인 캔버스 판 17 기준".
- **이 PC 반영**: 0.5.0 릴리스 빌드로 실행(스키마 19 — 올리기 전 사본 `.v18.bak`).

## 다음 행동

1. 사용자 결정 — A8 해석(원장 `#1h R1-M1`).
2. 통계 메뉴 착수 인터뷰(보여줄 숫자).
3. 19 acceptance TC 일괄(사용자) — 1a ~ 1m TC + 1j · 1k · 1l 항목 + 1m(두 PC 압축 · 비우기 · 꺼진 상대 · 자동 왕복 연동 · 0.4.0 상대 거절 안내 · 자동 압축 800k).

- **블로커 / 미해소**: 1c 원장 미조치 4건(19 TC 뒤 판단). 1f 범위 밖 관찰(기존 outbox 종류의 "지운 뒤 정산" 순서). #1g R2-F1 · R1·R2-F2(1h 에서 뺌).
  캡처 일부(activity · search · relay · new-conversation · comm-*)는 1j 이전 레일 모습 · 주고받은 글 · 에이전트 화면 캡처 없음. 이전 대기 그대로 — UI 시안 차이 8건(원본
  목록 확인 필요) · 매뉴얼의 미구현 코드 제안 7건 후속 여부 · `agentNetwork-t19` 워크트리 prunable.
