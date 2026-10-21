# STATE.md — 작업 상태

> **세션 시작 시 가장 먼저 읽는 문서.** 지금 어디서 재개하는지의 단일 좌표 — **매번 덮어쓴다.**
> Task 목록은 폴더가 SSOT(`python tools/check_ddd_docs.py --tasks`), 미조치는 [overview/ledger.md](docs/overview/ledger.md).

- **갱신**: 2026-10-22 / main — Task #1z-4 종결(개요 미리보기 시안 B)

## 현재 작업

없음 — 열린 Task 없음. #1z-4 개요 미리보기 종결(기본 보기 = 상대별 작은 대화 창 · 탭 3개 + "+n개" · 열 수 = 브라우저 폭 · 보기 기억).
이 PC 앱 = 서명된 1.3.0 `D:\dev\git\coworkAgent\target\release\coworkAgent.exe` — #1z-4 는 다음 서명 릴리스에 들어간다.

- **중단 지점**: 없음.
  PRODUCT.md 의 다른 세션 수정(security-posture 링크 2줄) · docs/tech/security-posture.md 는 `D:\dev\git\coworkAgent` 작업 폴더에 그대로(미커밋).
- **다음 릴리스 절차**: 개발자 가이드 2.4 — 버전 올림 · push → 앱 끄고 `cargo build --release -p coworkagent-app` → 선생님 서명(같은 경로) →
  Authenticode Valid · CN · 파일 버전 확인 → `gh release create v{X.Y.Z} … --target <origin/main SHA> --latest`. 1.2.0 이상 사용자는 자동으로 바뀐다.
- **작업 메모**: `cargo fmt` 쓰지 않음(저장소가 기본 폭으로 맞춰져 있지 않다). Bash 셸 `cargo test` 는 링커 LNK1104/1201 이 잦다 — PowerShell 로.

## 다음 행동

1. 19 acceptance TC(사용자) — 19 문서는 `task/19-prototype-acceptance` 브랜치에(main 병합 — 이후 main 은 일반 merge). 19 원장 재측정 · `tools/check_acceptance.py` 번호 형식(1a ~ 1z) 수정 필요.
2. v1.0.0 · v1.0.1 · v1.1.0 · v1.3.0 macOS zip 첨부(보유한 Mac).

- **블로커 / 미해소**: 19 acceptance TC(사용자). 그룹 타일 묶음 머리 "에이전트 대화" vs 메뉴 "에이전트 응답"(#1z-2 report — 사용자 확인 필요). 구현자 해석(사용자 미확인 — 각 report): 1q 결정 7 · 15,
  1r plan 해석 1 · 3 · 결정 3 · 11, 1s 결정 1 · 2, 1z-2 결정 2. D34 개정 문구 "편집 · 삭제는 CLI(· API)" 와 달리 CLI 에 편집 · 삭제 명령 없음(확인 필요).
  #30 미조치 2행 원장 수용(사용자 "수용" — `#30 R2-1` · `#30 (#1y 이어받음)`). #1z-4 미조치 1행 원장 조건 대기(`#1z-4 R1-1`).
