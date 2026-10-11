# stage1 — 스키마 7 · 프로토콜 · 연 쪽 구동 · 멈춤 · 재개

## 목표

[plan](../plan.md) Stage 1 — 자동 왕복의 저장 · 프로토콜 · 조율. 대상 gate: E3 · E4.

## 산출 파일

- `crates/core/src/{db.rs,chat.rs}` — 스키마 7(`relay_agent` · `relay_cap` · `relay_count` · `relay_on` · `relay_blocked`, 이전은 10 규칙),
  `Relay` · `set_relay` · `relay_on` · `relay_step` · `set_blocked` · `add_agent_request` · `set_flags`, 대기열 · 맥락은 작성자 무관 `exec = 1`
- `crates/net/src/{proto.rs,node.rs}` — `Message.by_agent` · `Request::Relay`(페어링된 상대만)
- `crates/agent/src/chat.rs` — `start_relay`(상한 1 ~ `MAX_RELAY_CAP` 50) · `relay`(멈춤 · 재개) · `inbound_relay` · `block_relay` ·
  `relay_forward`, `inbound_reply` 자동 전달 · `process` 의 내 에이전트 실행, 사람이 쓰면 멈춤(`send` · `post`)
- 테스트: `paths`(`schema7_relay_columns`) · `relay`(신규 4건) · `roundtrip`(미페어링 거부에 `by_agent` · `Relay`)

## 설계 결정

- 시작은 `Chat::send` 인자 대신 `Chat::start_relay` — 새 대화일 때만이라 따로 둔다(plan 개정 1).
- 셈: 내 에이전트 실행 직전 `relay_step`. 상한 n 이면 양쪽 에이전트 n 번씩 — n 번째 내 답은 상대 이력에 기록만 하고 멈춘다
  (평가 라운드 2 — plan 개정 3).
- 멈춤 경합: 대기열에 든 상대 답은 실행 직전 다시 보고 꺼져 있으면 넘기지 않은 상대 답으로 되돌린다. 실행 중 멈추면 내 답은 남기되
  넘기지 않는다. 전달 거절(`stopped`) · 오류면 끄고 그 글을 `error` · `undelivered`.
- 받은 쪽 사람 글: 거절은 바로, 연 쪽 알림(`relay{on:false}`)은 기다리지 않는다(상대가 꺼져 있어도 글쓰기가 늦지 않게).
- 재개: 받은 쪽에 `relay{on:true}`(거절 해제)가 성공해야 켠다. 마지막 글이 넘기지 않은 상대 답이면 바로 넘긴다.
- 실행 입력: 에이전트발 글은 `[요청 — 상대 에이전트({이름 · PC})]` 로 표시(양쪽). 가드(D31)는 실행기 그대로.

## 테스트 결과

- 워크스페이스 전체 통과 · clippy 통과. relay 4건.
- mutation 6종 모두 FAIL: 상한 무시 · 연 쪽 사람 글 멈춤 제거 · 받은 쪽 거절 제거 · 재개 카운트 유지 · 받은 쪽 알림 제거 · 내 답 `mine` 제거.

## 다음 단계

Stage 2 — API(`relay_agent` · `relay_cap` · `/relay`) · 웹UI(새 대화 자동 왕복 · 머리글 · 멈춤 · 재개).
