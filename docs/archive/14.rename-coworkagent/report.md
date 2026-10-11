# report — 14.rename-coworkagent

종결: 2026-10-11 · 브랜치 `task/14-rename-coworkagent` → main

## 결과

제품 이름을 agentNetwork 에서 coworkAgent 로 바꿨다(사용자 노출 전부). 개명 전 설치는 첫 실행 때 신원 · DB · 로그를 이어받는다.

| 성공 기준 | 판정 | 근거 |
|---|---|---|
| A1 살아 있는 코드 · 문서에 옛 이름 없음 | 충족 | E3 `residue=none`(이어받기 상수 · 테스트 · PROJECT/masterPlan 의 "구 이름" 표기는 의도) |
| A2 `coworkAgent.exe` | 충족 | E4 |
| A3 검증 명령 | 충족 | E1 · E2 · E5 |
| A4 개명 전 데이터 이어받기 | 충족 · macOS 확인 D25 | E6(paths 9건) |

- 적대적 평가 2 라운드(상한) — high 3 전부 코드 반영: 개명 전 데이터 이어받기(라운드 1), 옛 잠금을 잡고 옮기기 · 두 루트 충돌 거부
  (라운드 2). 미조치 2건은 [eval.md](eval.md) → 원장.

## 산출

- 이름: 실행 파일 · CLI(D14) · 데이터 폴더(D15) · DB · 로그 · 시작 실패 파일 · 초대 토큰 `coworkagent-invite:v1` · 알림 태그 · 가드 프롬프트 ·
  패키지 `coworkagent-app` · 웹UI · 문서.
- `an-core`: `Paths::legacy_root` · `adopt_legacy`. `agentnetwork` 표기는 archive · 인터뷰 기록에만.

## 남은 것

- GitHub 저장소 이름 변경 — 사용자 완료(`tskimS/coworkAgent`). 로컬 폴더 이동은 세션 밖에서 사용자(`git worktree repair`).
- 19 acceptance 의 실행 파일 · 경로를 새 이름으로(이 종결과 함께).
