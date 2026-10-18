# stage3 — 주고받은 메시지(상대별 접힘 요약 · 서버 페이지 · 행 북마크)

## 목표

plan Stage 3 — 해석 8 · 9: 상대별 절 기본 접힘 + 머리 요약, 펼치면 서버에서 20건씩 · 더 보기, 행마다 북마크.

## 산출 파일

- `crates/core/src/chat.rs` — `messages_page(scope, before, limit)`(양쪽 · context 제외 · 글 id 순 · 커서) · `peer_message_counts(scope)`.
- `crates/app/src/chat_api.rs` — `GET /api/messages?side=both[&before=]`(거르기 kind · ask · req · q 와 함께면 400, `before` 는 both 만) · `GET /api/messages/peers`
  (`[{peer, count, last}]`, `last` = 메시지 JSON + 대화 제목).
- `crates/app/web/view.js` — `PEER_PAGE` · `peerSummaryRows` · `pageMore` · `inboxRows` 에 `saved` · `reqBoxes`(요청 절 데이터 — Stage 1 의 화면 안 계산을 옮김) ·
  `peerMessages` 지움(쓰는 곳 없음).
- `crates/app/web/app.js` — `loadPeerMessages`(요약 + 펼친 절 다시) · `loadPeerPage` · `renderPeerMessages` · `moreRow` · `peerRowEls` · `markRow` · `state.peerHeads` ·
  `state.peerPages` · `loadRequests` 는 `reqBoxes`.
- `crates/app/web/app.css` — `.peer-msgs` 머리 · `.rows li.has-mark` · `.row-mark` · `.more-row`.
- 시험: app `ux_api.rs` `peer_pages_summary_and_cursor` · node `peer_msgs_heads` · `peer_msgs_more` · `peer_msgs_saved` · `req_overview_filter` · `ux_literals` 1q 행
  (1k 행은 새 자리 `rowEls(rows, …)` 로).
- `design/mock.py` — `/api/messages/peers` · `side=both` · `before`.

## 설계 결정

- 해석 8 의 "`messages_across` 에 `before`" 대신 새 `messages_page` — 기존 함수의 호출 6곳 · 시그니처를 건드리지 않고, 정렬을 글 id 순으로 커서와 맞춘다
  (공유 사본은 상대 시각이라 시각 순과 조금 어긋날 수 있다 — `ponytail:` 주석).
- 상대 절 접힘 열쇠 `peer:{상대}` + 처음 접힘(`open:` 기억, Stage 1 `foldEl` `shut`).
- 행 북마크는 받은 페이지 사본 · 열린 대화 글(`state.msgs`)을 함께 고쳐 말풍선과 어긋나지 않게.
- 시험 꼬리표 충돌(`pp-a` — 기존 `peer_patch_group_model_effort` 와 같은 임시 폴더 → `database is locked`)을 `pg-*` 로 바꿈.

## 테스트 결과

- RED: `cargo test -p coworkagent-app --test ux_api peer_pages` → `left: 405`(경로 없음). node `peer_msgs` → `# pass 0 # fail 1`.
- GREEN: `cargo test --workspace --no-fail-fast` → FAILED 0(꼬리표 고친 뒤 `ux_api` 3회 `32 passed`). clippy → `Finished`. node → `# pass 119` `# fail 0`.
- mock 눈 확인(다크): 주고받은 메시지 상대 절 머리(타일 · 이준호 · 14건 · 마지막 메시지 · 시각) · 펼침 행 · 오른쪽 북마크(켜진 것 강조) · JS 오류 없음.

## 다음 단계

stage4 — 문서(web-ui · an_app · 사용자 가이드 · 개발자 가이드) · eval. plan 개정: E3 EXPECT 119(사용자 미승인 — 승인 받기).
