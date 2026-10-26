# stage 4 — 개정 ②: 받은 대화 직접 쓰기 · 자동 왕복 표시 · 요청 접기

## 목표

plan stage 4 — 받은 대화에 쓴 글은 내 에이전트를 거치지 않고 양쪽 창에만, 받은 쪽 사람 글로 자동 왕복을 멈추지 않음, 머리글 '자동 왕복 거절 중' · 세부정보
'자동 왕복' 절 없앰, 세부정보 '요청' 최근 3건 + 더보기 · 접기.

## 산출 파일

- `crates/app/src/chat_api/tx.rs` — `continue_conversation` 이 대화 `dir` 로 `to_agent`(받은 대화 = `false`), `NewMessage.to` 설명.
  `chat_api/mod.rs` `to_of` 오류 글 "모든 글은 대화 에이전트에게 간다" → "사람에게 보내기는 없다"(받은 대화에는 맞지 않게 된 설명).
- `crates/agent/src/chat/send.rs` — `post_with` 받은 쪽 거절 분기 삭제(연 쪽 끔만).
- web: `js/logic/message.js` `relayText`(`relay_blocked` → `null`) · `js/logic/request.js` `REQ_FOLD` · `reqFold` · `js/detail.js`(자동 왕복 절 코드 삭제 · `reqView` · `toggleReqs`) ·
  `js/conv.js`(머리글 단추만) · `js/composer.js` 주석 · `index.html`(`#detail_relay_sec` 삭제) · `app.css`(`.relay-box` · `.relay-line` · `.bar` 삭제, `.req-more`).
- 시험: `crates/app/tests/chat_api.rs` · `crates/agent/tests/relay.rs` · `crates/app/tests/ux_literals.rs` · `crates/app/web/view.test.mjs`.
- 문서: PRODUCT(양쪽 발화 · 자동 왕복 줄) · masterPlan(D23 · D24 · 기록) · an_agent · developer-guide 8.3 · web-ui · 사용자 가이드.

## 측정(디버그 빌드 `target\t15` · ego-browser 1440 × 900)

- 시험 데이터 = scratchpad `ui1z12\data` 사본(`ui1z15b`) — 받은 대화 11 에 요청 4 건을 더해 5 건(더한 글은 `lc` 없음 — 목록 순서가 uuid 순, 시험 데이터 탓) · `relay_blocked = 1`.
- 라이트: 대화 11 머리글 `#conv_relay` 숨김 · 문구 없음, 페이지에 "자동 왕복 거절 중" 없음, `#detail_relay_sec` 없음. 요청 "5" · 3 줄 + "더보기" → 5 줄 + "접기" → 3 줄.
  펼친 채 대화 2 로 옮겼다 돌아오면 3 줄 + "더보기".
- 직접 쓰기: 입력창에 쓰고 보내기 → 새 글 1 개 `author human · mine · exec false · state sent`(상대 꺼짐 — 보관 `pending`, "보내는 중"), 실행 문 `running [] · waiting []`, 상태 줄 "보냈습니다".
- 다크: 같은 화면 · 요청 펼침. 콘솔 · 페이지 오류 0.
- 측정 스크립트 · 화면은 scratchpad(`ego/1z15b/e9.mjs` · `out/*.png`) — 저장소에 넣지 않음.
- 결정 16(report-4 답 — 뒤에 더함): '대화 정보' 의 '자동 왕복' 줄 — 측정은 아래 '결정 16 측정'.

## 결정 16 측정(같은 시험 데이터 · 다시 빌드)

- 라이트 · 다크: 대화 11(`relay_blocked = 1`) '대화 정보' = 기간 · 답하는 도구 · 읽는 폴더 · 컨텍스트 · **자동 왕복 = "멈춤 — 이 PC 가 멈춤(재개는 연 쪽)"** · 실행 명령, 머리글 `#conv_relay` 숨김.
  대화 12(멈추지 않음)에는 그 줄 없음. 페이지 오류 0. 화면 캡처는 ego 창이 최소화돼 찍지 못함(판독은 DOM).

## 설계 결정

- 판단 자리 = 로컬 API(`continue_conversation`) — 코어 `Chat::post(…, true)` 의 받은 쪽 실행 경로는 남는다(시험 `cli_origin` 이 쓰는 코어 동작 · 받기 쪽 호환). 웹 · API 는 더는 그 길을 쓰지 않는다.
- 받은 대화의 "질의 | 요청" 고르기는 그대로 — 질의 = 실행 없는 사람 글, 요청 = 기존 요청(상태 · 메모).
- 머리글 멈춤 · 재개 단추는 남긴다(결정 12) — 세부정보 절이 사라져도 멈출 길이 있게.

## 테스트 결과

- `node --test` 174 통과 · `cargo test -p an-agent --test relay` 13 · `-p coworkagent-app --test chat_api --test ux_literals` 12 · 5 통과,
  `cargo test --workspace --no-fail-fast` exit 0(첫 실행은 `capacity` 의 페어링 간헐 실패 — 단독 재실행 통과), clippy 경고 0.

## 다음 단계

평가(E1 ~ E12 다시 — codex 리뷰 연장 없음, 결정 15) → main → 1.4.0 릴리스 빌드 다시.
