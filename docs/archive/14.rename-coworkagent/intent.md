# intent — 14.rename-coworkagent

승인: 2026-10-11 (사용자 지시 + 확인 답 2건)

## 왜

제품 · 저장소 이름을 agentNetwork 에서 coworkAgent 로 바꾼다. — 출처: 사용자 지시(2026-10-11) "프로젝트, 리포지토리 이름을 변경. coworkAgent 로
변경 검토." · 확인 답 "사용자 노출 전부 (권장)" · "코드·문서는 내가, 개명은 선생님 (권장)".

## 범위

- 사용자에게 보이는 이름 전부: 화면 · 트레이 · 실행 파일(`coworkAgent.exe`) · CLI literal(D14 `coworkAgent` · `coworkAgent chat`) · 데이터 폴더
  (D15 `%LOCALAPPDATA%\coworkAgent` · `~/Library/Application Support/coworkAgent`) · DB · 로그 · 시작 실패 파일 이름 · 초대 토큰
  (`coworkagent-invite:v1`) · 알림 태그 · 가드 프롬프트 · 패키지(`coworkagent-app`) · 살아 있는 문서(PRODUCT · PROJECT · masterPlan · 매뉴얼).
- GitHub 저장소 · 로컬 폴더 이름 변경 절차(명령)를 보고에 정리 — 실행은 사용자.

## 제외

- 내부 약어: 크레이트 `an-core` · `an-net` · `an-agent`, 쿠키 `an_token`, 매뉴얼 파일 이름 `an_*.md`(확인 답 — 사용자 노출만).
- 종결된 Task 문서(`docs/archive`) · 부트스트랩 인터뷰 기록 — 당시 기록이라 그대로.
- GitHub 저장소 · 로컬 폴더 이름 변경 실행(사용자).

## 성공 기준

- A1: 살아 있는 코드 · 문서에 옛 이름(`agentNetwork` · `agentnetwork`)이 없다(archive · 인터뷰 기록 제외).
- A2: 릴리스 빌드 결과가 `coworkAgent.exe` 다.
- A3: 루트 `PROJECT.md` 검증 명령이 통과한다.
- A4: 개명 전 데이터 폴더(`agentNetwork`)가 있으면 첫 실행 때 신원 · DB · 로그를 새 폴더로 이어받는다(개정 — 평가 라운드 1).

## 결정

- 표기: 화면 · 파일 · CLI `coworkAgent`, 패키지 · 토큰 · 태그 소문자 `coworkagent-` — 기존 대소문자 규칙 그대로.
- 범위 = 사용자 노출 전부 · 내부 약어 유지 — 확인 답. 기각안: 내부 약어까지 · 표시 이름만.
- 저장소 · 폴더 개명은 사용자 — 확인 답(실행 중인 세션의 작업 폴더라 안에서 옮기면 깨진다).
- 옛 데이터 이어받기 — 처음 권장안은 "옮기기 없음(배포 전)" 이었으나 적대적 평가 라운드 1 지적 · 이 PC 에 개명 전 데이터가 실제로 있어 개정(2026-10-11).

## 미해소

- 없음
