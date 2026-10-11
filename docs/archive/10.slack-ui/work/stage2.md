# stage2 — API 확장 · `/view.js` · 화면 로직 모듈

## 목표

[plan](../plan.md) Stage 2 — 대화 요약 · 메시지 실행 정보 API, 화면 로직 순수 함수 모듈과 `node --test`. 대상 gate: E6 · E7.

## 산출 파일

- `crates/app/src/chat_api.rs` — `GET /api/conversations` 가 요약(`title` · `last_state` · `last_at`), 메시지에 `created_at` · `reply_to` ·
  `turns` · `duration_ms`
- `crates/app/src/server.rs` — `/view.js`(guard 뒤)
- `crates/app/web/view.js`(신규) · `crates/app/web/view.test.mjs`(신규, 8건)
- `crates/app/tests/chat_api.rs` — `conversation_summaries_and_message_meta`

## 설계 결정

- `view.js` 는 DOM 없는 ES 모듈 — `stateText` · `peerState` · `convLabel` · `filterByPeer` · `formatDuration` · `formatTime` · `timeline` ·
  `route` · `Latest`. 브라우저는 `/view.js` 로, 테스트는 상대 경로로 같은 파일을 읽는다(빌드 없음 — D10).
- `node --test` 는 디렉터리 인자를 파일로 본다 — 글롭 `"crates/app/web/*.test.mjs"`(Node 22+)와 `--test-reporter=tap`(gate 출력 고정).
  `package.json` 없이 `.js` 를 ES 모듈로 읽는다(Node 의 모듈 문법 감지, 검증 환경 Node 25.8.0).
- 타임라인 답 찾기: `reply_to` 우선, 없으면(스키마 5 이전 이력) 다음 요청 전의 대응 없는 첫 답.

## 테스트 결과

- chat_api 3 · node 8 통과.
- mutation: `Latest.is` 항상 참 → "늦게 온 응답" FAIL · 이전 이력 답 찾기에서 다음 요청 경계 제거 → 처음엔 **생존**, "답 없는 앞 요청"
  사례를 넣어 FAIL 로.

## 다음 단계

Stage 3 — 화면 조립(A · 설정 · C) · 검증 명령 표.
