# STATE.md — 작업 상태

> **세션 시작 시 가장 먼저 읽는 문서.** 지금 어디서 재개하는지의 단일 좌표 — **매번 덮어쓴다.**
> 누적 기록은 여기 없다: Task 목록은 폴더가 SSOT(`python tools/check_ddd_docs.py --tasks`),
> 미조치는 [overview/ledger.md](docs/overview/ledger.md). 가드레일·검증 명령은 루트 `PROJECT.md`,
> 제품 구조는 루트 `PRODUCT.md`, 절차는 [etc/ddd-workflow.md](docs/etc/ddd-workflow.md).

- **갱신**: 2026-10-12 / main `60d14b9`(Merge docs/manuals) — 15 · 17 · 18 종결 · 매뉴얼 · README · 원격 squash push · 대기 `task/19-prototype-acceptance` `8c23672`(사용자 일괄 수동 검증)

## 현재 작업

| 항목 | 값 |
|------|-----|
| Task | 18.focus-mention — 집중(들어오는 요청 거부) · 접속 상태 · 질의 · 요청 · 완료 · @멘션(상대 PC 에이전트) |
| 단계 | 종결 · main merge 끝 — 사용자 일괄 수동 검증(19.prototype-acceptance) 대기 |
| 상태 | 진행 목표 "Goal set: task 15,17,18 까지 완료. 사용자 테스트는 task 18 완료 후 일괄 검증." 의 개발 몫 끝 — 15 · 17 · 18 종결. plan 개정 1 ~ 6 승인 — 위임 |
| 재개 좌표 | [archive/18.focus-mention/report.md](docs/archive/18.focus-mention/report.md) — 종결 보고 · 19 acceptance(`task/19-prototype-acceptance` 브랜치 `docs/tasks/19.prototype-acceptance/acceptance.md`, TC-A ~ TC-M) |

- **직전에 한 일**: Stage 1 ~ 3 · 단계 리뷰 3회 반영 · 적대적 평가 2 라운드(상한 — 라운드 1 high 2 · 라운드 2 high 1 고침, `4b90f8f` · `70b2c49`) ·
  미조치 8건 → 원장 · report · 매뉴얼 · PRODUCT · masterPlan(D17 개정 · D19 · S1 보강) 환류 · archive 이동 · 19 TC-M(`9f8580c`) → 19 브랜치에 main merge ·
  개정 7(검사 도구가 못 세던 원장 19 행 셋 · TC-M 10 스크린리더 · 스키마 이전 문구) · 재측정 `sources=37 missing=0` · 릴리스 빌드(`8c23672`).
- **그 뒤**: 사용자 · 개발자 매뉴얼([user-guide](docs/manual/user/user-guide.md) · [developer-guide](docs/manual/dev/developer-guide.md), codex 리뷰 2/2) ·
  README(화면 캡처 7장 · 통신 메커니즘) · `tools/rebuild_run.py` → main merge(`60d14b9`) → 원격 `origin/main` 에 squash 1커밋 push(로컬 이력과
  별개 줄기 — 앞 squash `766b237` 위에 쌓는다).
- **중단 지점**: 없음. 다음은 사용자 검증.

## 다음 행동

1. 사용자: 19 acceptance TC 일괄 실행(TC-A ~ TC-M — 사용자 지시 "사용자 테스트는 task 18 완료 후 일괄 검증."). 결과로 원장 "조건 대기" 행 · D28 갱신.
2. 사용자 결정: UI 시안과의 차이 8건(문서 없는 7 · 빠진 섹션 접기 1) 중 맞출 것 · 매뉴얼에 적은 미구현 코드 제안 7건(LAN 필터 · `claude -p`
   환경 변수 고정 · resume 실패 시 새 세션 등)을 후속 Task 로 열지.

- **블로커 / 미해소**: 없음. 위임으로 정한 것 중 사용자가 뒤집기 쉬운 것 — 입력창 글 종류 = 라디오 두 값(질의 | 요청, 기본 질의) · 멘션은 내가 연
  대화(자동 왕복 아님)에서만 · 요청 다시 보내기 = 요청 · 맞지 않는 멘션 꼴은 한 번 막고 다시 보내면 멘션 없이. `agentNetwork-t19` 워크트리 등록은
  디렉터리가 없어 prunable(사용자 결정 대기).
