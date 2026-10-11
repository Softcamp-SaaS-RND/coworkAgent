# STATE.md — 작업 상태

> **세션 시작 시 가장 먼저 읽는 문서.** 지금 어디서 재개하는지의 단일 좌표 — **매번 덮어쓴다.**
> 누적 기록은 여기 없다: Task 목록은 폴더가 SSOT(`python tools/check_ddd_docs.py --tasks`),
> 미조치는 [overview/ledger.md](docs/overview/ledger.md). 가드레일·검증 명령은 루트 `PROJECT.md`,
> 제품 구조는 루트 `PRODUCT.md`, 절차는 [etc/ddd-workflow.md](docs/etc/ddd-workflow.md).

- **갱신**: 2026-10-11 / main(00 ~ 05 · 10 ~ 14 종결 merge) · 진행 `task/19-prototype-acceptance`

## 현재 작업

| 항목 | 값 |
|------|-----|
| Task | 19.prototype-acceptance — 사용자 수동 테스트 대기(D28) |
| 단계 | 개발 완료(Phase 0 ~ 1) · 19 Stage 2 |
| 상태 | 수동 gate 는 전부 `19.prototype-acceptance`(D28)로 이관 — 사용자 지시 "개발완료. 사용자 테스트는 완료 후 진행." |
| 재개 좌표 | `task/19-prototype-acceptance` 브랜치의 `docs/tasks/19.prototype-acceptance/acceptance.md` |

- **직전에 한 일**: 14 rename-coworkagent — 제품 이름 coworkAgent(구 agentNetwork) · 개명 전 데이터 이어받기 · 적대적 평가 2/2 · 종결.
- **중단 지점**: 19 수동 TC 준비 완료 — 사용자 실행 대기.
- **개발 완료**(2026-10-11, Phase 0 ~ 1 — macOS 는 D25 Phase 4): 개발 Task 00 ~ 05 · 10 ~ 14 종결 · main merge. 검증 — 워크스페이스
  테스트 131 passed · 0 failed, clippy 0, node `# fail 0`, `check_crates` violations=0, `check_ddd_docs` PASS, 19 수집 `sources=25 missing=0` ·
  릴리스 빌드. 남은 것은 사용자 수동 테스트(19 E3 ~ E11 — 사용자 지시 "사용자 테스트는 완료 후 진행").

## 다음 행동

1. 사용자: acceptance.md TC-A ~ TC-I 실행 · 결과 기록.
2. 결과로 원장 D28 행 갱신 · 19 종결, 실패 TC 는 후속 Task.

- **블로커 / 미해소**: 없음. 위험 수용 — D24 개정(실행 한도 = 시간 제한만)으로 원격 반복 요청이 수신 PC Max 사용량을 제한 없이 쓸 수 있다.
