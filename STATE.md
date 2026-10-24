# STATE.md — 작업 상태

> **세션 시작 시 가장 먼저 읽는 문서.** 지금 어디서 재개하는지의 단일 좌표 — **매번 덮어쓴다.**
> Task 목록은 폴더가 SSOT(`python tools/check_ddd_docs.py --tasks`), 미조치는 [overview/ledger.md](docs/overview/ledger.md).

- **갱신**: 2026-10-24 / main — **#1z-12 origin push · 브랜치 정리 · v1.3.6 첨부 파일 다시 올림 뒤**(squash `1ffea62` · STATE `7a1b7a8` 뒤)

## 현재 작업

없음 — #1z-12 코드 정리 종결(재개 뒤 — [report](docs/archive/1z-12.code-tidy/report.md)) · main 반영 `1ffea62` · origin push · 브랜치 `task/1z-12-code-tidy` 지움.
v1.3.6 릴리스 첨부 `coworkAgent.exe` 를 코드 정리 빌드 서명본(`dist` — SOFTCAMP 서명 Valid)으로 다시 올림: SHA-256 `71ffdc6f1b35ea915f1dc8914f7debd1b17b94d2916e9c3bd8668aff62468e87` ·
12,908,432 바이트, 본문 소스 `1ffea62`, 처음 파일(`ef31b5b1…` · 소스 `785358a`)은 본문 한 줄로 남김. 태그 `v1.3.6` 은 그대로 `785358a`. 받은 파일 대조(해시 · 서명) 같음.

- **직전에 한 일**: 브랜치 정리 → `gh release upload v1.3.6 --clobber` → 본문 SHA-256 · 소스 줄 고침 → 받아서 대조 → 이 STATE → push.
- **중단 지점**: 없음. 개발 PC 앱은 `target/release` 새 빌드로 떠 있음(19:26). 시험 인스턴스(scratchpad `ui1z12\data`)는 꺼져 있음.
- **작업 메모**: 원 작업 폴더의 다른 세션 미커밋(PRODUCT.md 2줄 · 미추적 docs/tech/security-posture.md)은 건드리지 않음 — 이 Task 의 PRODUCT.md 줄만 부분 스테이징.
  main 반영 때 stash → 반영 → pop(PRODUCT.md 가 겹치면 손으로 합침). Task 는 worktree 없이 원 작업 폴더에서 브랜치만(2026-10-24 사용자 지시). `cargo fmt` 쓰지 않음, `cargo test` 는 PowerShell 로.

## 다음 행동

1. 선생님 결정: #1z-12 report "사용자 결정 필요" — report-6(A2b 결정 8 밖 쓰기 일곱 — 예외에 넣을지 · 쪼갤지) · stage3-1(`#p` 화면) · stage3-2(시험만 쓰는 logic 넷) ·
   stage4-1 ~ stage4-9(이상한 로직) · report-2(an_app 매뉴얼 없어진 이름) · report-3(an_agent 매뉴얼 Codex 공백) · report-4(다른 세션 security-posture 낡은 근거).
2. 다음 Task #1z-13 디자인(목적 1 — AI 디자인 느낌 제거): `stitch-design-taste` 는 모델이 부를 수 없는 스킬 — 선생님이 `/stitch-design-taste` 를 직접 실행.
3. 19 acceptance TC(사용자) · macOS zip 첨부 — 이전 STATE 그대로.

- **블로커 / 미해소**: 19 acceptance TC(사용자). 구현자 해석(사용자 미확인 — 각 report): 1q 결정 7 · 15, 1r plan 해석 1 · 3 · 결정 3 · 11, 1s 결정 1 · 2, 1z-2 결정 2,
  #1z-7 구현자 해석, #1z-8 결정 11 · plan 승인 갈음, #1z-9 report 구현자 해석, #1z-10 report 구현자 해석(편차는 2026-10-24 결정 — 확인 줄 뺌), #1z-11 report 구현자 해석,
  #1z-12 report 구현자 해석. #30 미조치 2행 원장 수용. #1z-12 R2-1 원장 열림(report-6). #1z-4 미조치 1행 원장 조건 대기 · #1z-10 1행 열림.
