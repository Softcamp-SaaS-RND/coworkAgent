# report — 05-2.parent-read

종결: 2026-10-21 · 브랜치 `task/05-2-parent-read` → main(squash, #1z-3 뒤 — origin/main 병합)

## 결과

에이전트별 **"상위 한단계 탐색 허용"**(등록 · 상세 체크박스, 기본 끔) — 켜면 `claude -p` 에 `--add-dir <에이전트 폴더 바로 위>` + 시스템 프롬프트 `[읽기 범위]` 한 줄.
위가 드라이브 루트 · 사용자 홈 · 데이터 루트를 품는 곳 · 네트워크 경로면 켜도 폴더만(실행마다 판단, 상세 "열지 않음"). 바꾸면 세대 · 세션 · 요약 비움. 스키마 24.

| 성공 기준 | 판정 | 근거 |
|---|---|---|
| A1 켜면 `--add-dir` 위 · 끄면 · 막힌 위면 없음 | 충족 | E3(`parent_read_adds_dir` · `parent_read_dir_guards`) |
| A2 실제 claude — 위 읽음 · 끄면 거부 · 두 단계 위 거부 | 충족 | E4(`parent_read_canary` — on_up=(true, false) off_up=(false, true) on_far=(false, true)) |
| A3 등록 · 상세 체크박스 · 저장 · "읽는 위 폴더" | 충족 | E5 · E7(실제 화면) |
| A4 있던 에이전트 끔 · 바꾸면 세션 비움 | 충족 | E1(`schema12_focus_mention` 보강 · 스키마 24) · R1 · R2 반영 |
| A5 회귀 없음 · 문서 | 충족 | E1 · E2 · E6 |

- 적대적 평가 2 라운드(상한, Orca 가 라운드마다 새로 띄운 Codex) — R1 high(압축 대기 중 옵션 변경 → 옛 세션을 새 세대로 요약), R2 high(자동 왕복이 왕복 에이전트를
  대화와 다른 DB 잠금으로 다시 읽음) 모두 반영. 미조치 2행 — 사용자 "push 병합"(2026-10-21)으로 수용, 원장 `#05-2 R2-D1` · `#05-2 R1·R2-D2`([eval](eval.md)).
- 구현자 해석(사용자 미확인 — intent 결정): 기본 끔(2) · 데이터 루트를 품는 위 · 네트워크 경로 제외(3) · 옵션 변경 시 세션 비움(4) · `[읽기 범위]` 한 줄(5) · Task 번호 `05-2`(6).
- plan 밖: 이 브랜치 코드 변경으로 열린 #30 원장이 낡아(`check_ddd_docs`) #30 gate 를 이 브랜치에서 재측정.

## 산출

- core: 스키마 24 `agents.parent_read` · `Agent.parent_read` · `add_with` · `update(…, parent_read)` · `paths::parent_read_dir`.
- agent: `claude_args` `--add-dir` · `[읽기 범위]`, `chat::compact` 세션 · 세대 같은 시점 재확인(R1), `process` 실행 에이전트 같은 잠금(R2). 시험 `parent_read_canary`(새, `#[ignore]`) ·
  `runner` · `context`(`compact_skips_session_cleared_while_waiting`) · `chat_api` · `paths`.
- app: API `parent_read` · `parent_dir`, 웹UI 등록 · 상세 체크박스 · `PARENT_NOTE`.
- 문서: an_core · an_agent · an_app · 개발자 가이드 · 사용자 가이드 6장 · PRODUCT · masterPlan D21 · D22 · Phase 0 표 · 개정 기록.

## 남은 것

- 미조치 2행(원장). 매 gate 재측정이 E4 로 실제 claude 사용량을 쓴다(약 20초).
