# stage1 — 가드 프롬프트 · 읽기 전용 도구

## 목표

[plan](../plan.md) Stage 1 — 모든 원격 실행을 문의 응답 전용으로(D31). 대상 gate: E1~E5.

## 산출 파일

- `crates/core/src/agents.rs` — `READ_ONLY_TOOLS` · `read_only` · `unknown_tools`
- `crates/agent/src/runner.rs` — `GUARD_PROMPT` · `claude_args`(도구 걸러내기 · 시스템 프롬프트 항상)
- `crates/app/src/chat_api.rs` — 등록 시 목록 밖 도구 400 · `crates/app/web/index.html` 안내 문구
- 테스트: `runner`(`args_follow_d22` 수정 · `guard_prompt_always` · `tools_limited_to_read_only`) · `store`(`read_only_tools_filter`) ·
  `chat_api`(`agents_crud_api` 에 쓰기 도구 거부 2건)

## 설계 결정

- 도구 이름은 대소문자까지 정확히 대조(`read` 는 거부) — `claude --tools` 가 받는 이름 그대로.
- 실행 시 걸러내기는 이전에 저장된 프로필(쓰기 도구 포함)도 덮는다 — 남는 것이 없으면 기본(Read · Grep · Glob).
- 시스템 프롬프트 = `GUARD_PROMPT` + `\n\n[수신자 추가 지시]\n{프로필 지시}` — 가드가 항상 앞.

## 테스트 결과

- workspace 전체 · clippy 통과, gate E1~E5 PASS.
- mutation: 등록 시 도구 확인 제거 → `agents_crud_api` FAIL · 실행 시 걸러내기 제거 → `tools_limited_to_read_only` FAIL.

## 다음 단계

적대적 평가(BASE `fe91755`).
