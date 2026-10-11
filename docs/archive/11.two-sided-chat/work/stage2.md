# stage2 — API · 웹UI

## 목표

[plan](../plan.md) Stage 2 — 이어 쓰기 API 의 대상 · 양쪽 입력창 · 발화자 표시. 대상 gate: E1 · E2 · E6 · E7(E8 수동은 D28).

## 산출 파일

- `crates/app/src/chat_api.rs` — `POST /api/conversations/{id}/messages {text, to: agent|human}`(양쪽 대화, 기본 agent, 그 밖 400) ·
  메시지 JSON 에 `mine` · `exec`
- `crates/app/web/{index.html,app.js,app.css,view.js,view.test.mjs}` — 입력창 대상 라디오(에이전트에게 · 사람에게), 받은 대화 입력창 열림,
  `authorLabel`, 타임라인 · 답 연결은 실행 요청만
- `crates/app/tests/chat_api.rs` — `conversation_post_targets`

## 설계 결정

- 에이전트 라벨은 대화 방향으로 — 보낸 대화 "{상대}/{에이전트} 가 실행", 받은 대화 "내 {에이전트} 가 실행".
- 메시지 종류 표시: 에이전트 · "사람 → 에이전트"(실행 요청) · "사람"(실행 없음).
- 이전 이력 답 연결(`linkReplies`)은 실행 없는 사람 글을 기다리는 요청으로 세지 않는다.

## 테스트 결과

- chat_api 4 · node 11 통과, clippy 통과.

## 다음 단계

평가 — 적대적 평가(BASE `30f0ac9`).
