# report — 00.skeleton

종결: 2026-10-11 · 브랜치 `task/00-skeleton` → main

## 결과

단일 실행 파일 `agentNetwork` 가 트레이에 상주하며 127.0.0.1 전용 웹UI(상태 페이지)를 띄우고, 사용자 데이터 경로에 SQLite ·
회전 로그를 둔다. 두 번째 실행은 기존 인스턴스에 "열기" 를 넘기고 끝난다.

| 성공 기준 | 판정 | 근거 |
|---|---|---|
| A1 Windows exe 1개 | 충족 | E3 `exes=1 names=agentNetwork.exe` |
| A2 macOS universal | ABANDON | E9 — Mac 환경 없음, masterPlan D25(Phase 4) |
| A3 트레이 · 열기 · 종료 | 자동 근거만 | E10(수동) ABANDON — 프로토타입 인수 Task 로 이관(D28). 서버 · 핸드오프 경로는 E4 · E8 · E13 |
| A4 로컬 웹UI 경계 | 충족 | E4 6건 + guard 추가 테스트 |
| A5 데이터 경로 · 로그 | 충족 | E5 · E6 |
| A6 단일 인스턴스 | 충족 | E7 · E8 · E13 |
| A7 테스트 · 경고 0 | 충족 | E1 · E2 |

- 적대적 평가 2 라운드(상한) — 라운드 1·2 지적 7건 중 코드 지적 5건을 반영(핸드오프 500 · `ShellExecuteW` · 종료 순서 ·
  시작 실패 메시지 상자 · 상태 UI 오류 표시), 미조치 5건은 [eval.md](eval.md) `## 미조치 지적` → 원장.
- plan 개정 3건(쿠키 비격리 수용 · 진입 순서 · E13).

## 산출

- 크레이트 `an-core`(경로 · DB · 로그 회전 · 인스턴스 잠금) · `agentnetwork-app`(웹UI 서버 · 핸드오프 · 트레이 · 브라우저).
- `tools/check_crates.py` — 의존 경계 검사.
- 매뉴얼 [manual/dev/an_core.md](../../manual/dev/an_core.md) · [manual/dev/an_app.md](../../manual/dev/an_app.md).

## 남은 것

- 수동 확인(A3 · E10)은 Phase 1 끝 프로토타입 인수 Task 에서 일괄(D28). macOS 는 Phase 4(D25).
- 가드레일 4행("트레이 메뉴 외에 네이티브 창을 두지 않는다")과 시작 실패 메시지 상자의 관계는 사용자 확인 필요 — STATE 에 기록.
