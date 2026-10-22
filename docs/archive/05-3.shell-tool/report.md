# report — 05-3.shell-tool

종결: 2026-10-22 · 브랜치 `task/05-3-shell-tool` → main(squash)

## 결과

에이전트별 **"셸 명령 실행 허용"**(등록 · 상세 체크박스, 기본 끔 — C 안) — 켜면 `claude -p` 의 `--tools` · `--allowedTools` 에 `Bash` · `PowerShell`(macOS `Bash`),
가드는 `SHELL_GUARD_PROMPT`(이 PC 에 설정된 workflow 수행 · 조회용 — workflow 산출물은 예외, 소스 · 문서 · 설정 직접 삭제 · 편집 · 저장소 변경 금지), 실행 환경은 기본 변수 목록만(`env_clear`).
위험 공지 = 체크박스 도움말 · 끔 → 켬 저장 때 확인 창(취소 = 보내지 않음) · 켠 상세 빨간 경고 줄 · 사용자 가이드 경고 상자. 바꾸면 세대 · 세션 · 요약 비움, 압축 요약은 셸 없이. 스키마 25.
읽기 전용 도구는 claude 에 실재하는 `Read` · `Grep` · `Glob` 3개로 정리(`LS` · `NotebookRead` 는 2.1.308 에 없는 이름 — 동작 변화 없음).

| 성공 기준 | 판정 | 근거 |
|---|---|---|
| A1 켜면 셸 도구 · 셸 가드, 끄면 읽기 전용 · `GUARD_PROMPT` | 충족 | E3(`shell_adds_tools` · `shell_env_keeps_base_only`) |
| A2 실제 claude — 켜면 셸 · 심은 변수 안 보임, 끄면 셸 없음 | 충족 | E4(`shell_canary` — 대조군 `NO_PROXY` 보임 · 목록 밖 변수 빈 값 · 끔 = 폴더 밖 Read 거부) · `tools_canary` |
| A3 체크박스 · 도움말 · 확인 창 · 경고 줄 · 저장 | 충족 | E5 · E7(실제 화면 — 취소 = 등록 안 됨 · 켠 채 / 끄고 저장 = 확인 없음 · 끔 → 켬 = 확인) |
| A4 있던 에이전트 끔 · 바꾸면 세션 비움 | 충족 | E1 · E3(`schema12_focus_mention` 보강 · 스키마 25) |
| A5 회귀 없음 · 문서 · 원장 | 충족 | E1 · E2 · E6 |

- 적대적 평가 2 라운드(상한, Orca 가 라운드마다 새로 띄운 Codex) — R1 needs-attention(high 오래 열린 상세가 끈 셸을 확인 없이 되살림 → `agentPatch` 바꾼 칸만,
  medium 프록시 주소 계정 · 비밀번호가 셸에 보임 → 공지), R2 approve. 미조치 2행 원장(`#05-3 R1-1` 수용 · `#05-3 R2-1` 조건 대기 — [eval](eval.md)).
- 구현 중 지시: U1(삭제 · 편집 등 변경 불가 안내 · 셸 = 이 PC 에 설정된 workflow 수행), U4(권장안 — workflow 산출물 예외), notebook 도구 제거, 효과 없는 도구 정리(`LS`).
- 구현자 해석(사용자 미확인 — intent 결정): 옵션 변경 시 세션 비움(7) · 라벨 "셸 명령 실행 허용"(8) · 압축 요약은 셸 없이(9).
- plan 밖: "내 에이전트" 도움말(`tip_agents`)의 "명령 실행 없음" 을 셸 허용과 맞게 고침. E5 CHECK 보정 2회(라벨을 자리로 세기 · 셸 인용 — EXPECT 그대로).

## 산출

- core: 스키마 25 `agents.shell` · `Agent.shell` · `SHELL_TOOLS` · `add_with(…, shell)` · `update(…, shell)` · `READ_ONLY_TOOLS` 3개.
- agent: `SHELL_GUARD_PROMPT` · `SHELL_ENV_KEEP` · `shell_env` · `claude_args`(셸 도구 · 가드 · `[읽기 범위]` 문구) · `run_process_lines(…, env, …)` · `run_claude` 셸이면 `env_clear`,
  `chat` 압축 요약은 셸 끈 사본. 시험 `shell_canary` · `tools_canary`(새, `#[ignore]`) · `runner` · `paths` · `chat_api`.
- app: API `shell`, 웹UI 등록 · 상세 체크박스 · `SHELL_NOTE` · `SHELL_WARN` · `SHELL_ON` · `shellConfirmNeeded` · `agentPatch` · `.hint.shell-on`.
- 문서: an_core · an_agent · an_app · 개발자 가이드 · 사용자 가이드 6장 · PRODUCT · security-review · 원장(`#1a A-I3` · `#1a A-I8` 닫힘 · `#05-3 셸 위험` 수용) · masterPlan D21 · D22 · D31 · Phase 0 표 · 개정 기록.

## 남은 것

- 미조치 2행(원장). macOS 실기(셸 · `env_clear` 뒤 로그인 · `Bash` 만) 미검증.
- 원 작업 폴더의 다른 세션 미커밋 `docs/tech/security-posture.md` 2.2 절 "도구 = 읽기 전용 5개 · 셸 없음" 은 이 Task 뒤 사실과 다르다 — 그 문서를 커밋할 때 갱신 필요.
- 매 gate 재측정이 E4 로 실제 claude 사용량을 쓴다(약 15초).
