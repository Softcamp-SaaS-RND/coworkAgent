# stage2 — API · 웹UI

## 목표

[plan](../plan.md) Stage 2 — 자동 왕복을 웹UI 에서 열고 · 보고 · 멈추고 · 재개한다. 대상 gate: E1 · E2 · E5 · E6 · E7.

## 산출 파일

- `crates/app/src/chat_api.rs` — `POST /api/conversations {…, relay_agent?, relay_cap?}`(상한 기본 `DEFAULT_RELAY_CAP` 10),
  `POST /api/conversations/{id}/relay {on}`, 대화 JSON 에 `relay_agent` · `relay_cap` · `relay_count` · `relay_on` · `relay_blocked`
- `crates/app/web/{index.html,app.js,view.js,view.test.mjs}` — 새 대화 "내 에이전트로 자동 왕복" · 상한, 머리글 자동 왕복 표시 ·
  멈춤 · 재개 버튼, `relayText`, `authorLabel` 의 자동 왕복 에이전트 이름
- 테스트: `chat_api`(`relay_api`) · node(자동 왕복 문구)

## 설계 결정

- 받은 쪽은 자동 왕복 여부를 저장하지 않는다 — 상대 에이전트 글(`agent` · `mine = false`)이 있으면 "받는 중" + 멈춤, 거절 중이면 문구만.
- 받은 쪽에서 상대 에이전트 이름은 모른다(프로토콜에 이름만 실리고 저장하지 않음) — "{상대}/에이전트".
- 값은 textContent 로만(D10 · 10 의 규칙).

## 테스트 결과

- 워크스페이스 전체 · clippy · node(`# fail 0`) 통과. chat_api 5건.

## 다음 단계

평가(gate · 적대적 평가 2 라운드) · 종결.
