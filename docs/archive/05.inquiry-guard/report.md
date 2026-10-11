# report — 05.inquiry-guard

종결: 2026-10-11 · 브랜치 `task/05-inquiry-guard` → main

## 결과

D31(기본 기능 = 진행 중 프로젝트 · 제품 문의 응답)을 실현했다 — 모든 원격 실행에 문의 응답 전용 시스템 프롬프트를 항상 붙이고(재개 세션에도,
`--system-prompt-snapshot off`), 허용 도구를 읽기 전용 목록으로 고정했다(등록 거부 + 실행 시 걸러내기). Claude Code 가 낮으면 실행하지 않는다.

| 성공 기준 | 판정 | 근거 |
|---|---|---|
| A1 가드 프롬프트 항상 · 프로필 지시는 뒤에 | 충족 | E3 `guard_prompt_always`(새 · 재개 세션) |
| A2 실행 도구 = 읽기 전용 목록 안 | 충족 | E3 `tools_limited_to_read_only` · E4 `read_only_tools_filter` |
| A3 등록 API 쓰기 도구 400 | 충족 | E5 `agents_crud_api` |
| A4 검증 명령 | 충족 | E1 · E2 |

- 적대적 평가 2 라운드(상한) — high 2 전부 반영: 재개 세션 가드 미적용(`--system-prompt-snapshot off`), 구형 CLI 전면 실패(버전 확인 ·
  업데이트 안내). 미조치 2건(실제 claude 확인 · 도입 버전 근거)은 [eval.md](eval.md) → 원장.

## 산출

- `an_core::agents::{READ_ONLY_TOOLS, read_only, unknown_tools}` · `an_agent::runner::{GUARD_PROMPT, MIN_CLAUDE, parse_version, version_ok}` ·
  `claude_args`(읽기 전용 도구 · 가드 · 스냅샷 끔) · `run_claude`(버전 확인) · `chat_api` 등록 거부 · 웹UI 안내.
- masterPlan D31 신규 · D22 개정.

## 남은 것

- 실제 `claude -p` 로 거부 동작 · 재개 가드 확인 — `19.prototype-acceptance`(D28).
