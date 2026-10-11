# stage1 — 스키마 6 · `post` · `Chat::post` · 맥락 앞붙임

## 목표

[plan](../plan.md) Stage 1 — 양쪽 사람 발화의 저장 · 프로토콜 · 조율. 대상 gate: E3 · E4 · E5.

## 산출 파일

- `crates/core/src/{db.rs,chat.rs}` — 스키마 6(`mine` · `exec`, 이전 이력은 방향으로 채움 · 한 트랜잭션 · 사본), `MINE_BY_DIR`,
  `add_human` · `notes_between` · `last_exec_before`, `next_pending` · `pending_conversations` 는 실행 요청만
- `crates/net/src/{proto.rs,node.rs}` — `Request::Post`(페어링된 상대만)
- `crates/agent/src/chat.rs` — `Chat::post` · `inbound_post` · 내 실행 요청의 답은 `post`(agent) 미러 · `with_notes`
- 테스트: `paths`(`schema6_fills_mine_exec`) · `store`(`human_posts_and_notes`) · `two_sided`(신규 4건)

## 설계 결정

- 받은 쪽 실행 요청은 `sent` 로 저장 → 상대에 사람 글 미러 → `queued` 로 바꾸고 깨운다 — 답 미러가 사람 글보다 먼저 닿지 않게.
- `post` 받기 규칙: 대화는 그 상대 소유(`find`), 에이전트 답 · 실행 요청 글은 내 `out` 대화(에이전트가 상대 쪽)에만, 에이전트 답은 상대가 쓴
  `sent` 실행 요청에만(한 번만). 같은 msg uuid 는 `ok` 만.
- 맥락 앞붙임: `[대화 메모 — 실행 요청 아님]` · `- {발화자}: {본문}` · `[요청 — {발화자}]` — 발화자 = "이 PC 사용자" 또는 상대 이름 · PC.
  사람 글이 없으면 본문 그대로(기존 왕복 테스트 불변).

## 테스트 결과

- an-core · an-agent 전체 통과(two_sided 4 · roundtrip 11), clippy 통과.
- mutation: 맥락 앞붙임 끔 → `notes_prepended_to_run` FAIL · 에이전트 답 미러의 방향 확인 제거 → `post_refused_when_foreign` FAIL.

## 다음 단계

Stage 2 — API(`to`) · 웹UI(양쪽 입력창 · 대상 · 발화자).
