# stage1 — 대화 화면: 말풍선 · 날짜 구분 · 라벨 정리 · 머리 · 말풍선 아이콘 · 대화방 줄

> 근거: [plan](../plan.md) Stage 1. 단계 끝 mock(가상 데이터) 화면을 사용자에게 보인다(Q3).

## 목표

해석 1 ~ 5 — 이 PC 쪽 글은 오른쪽 말풍선 · 날짜 구분선 · 방향 라벨(사람 → 에이전트 · 보냄 · 받음) 없앰 · 머리 "대화 보기 / 원문 보기 · 북마크 n · 세부정보"(보낸 메시지
탭 없음) · 말풍선 우상단 북마크 · 복사 아이콘 · 대화방 줄(☆ · 아바타 · 마지막 글 미리보기). 대상 gate: E1 ~ E4.

## 산출 파일

- `crates/core/src/chat.rs` — `Summary.last: Option<LastMessage>`(`text` 앞 `PREVIEW_CHARS` = 120자 · `author` · `mine` · `agent`) — 요약 SQL 에 마지막 글 하위 질의 4개.
- `crates/app/src/chat_api.rs` — 대화 JSON `last`(`{text, author, mine, agent}` 또는 `null`).
- `crates/app/web/view.js` — `convLabel`(태그 없음) · `roomPreview` · `bubbleSide` · `dayKey` · `dayLabel` · `dayBreaks` · `inboxRows`(`kind` = 요청 칩, `room`) ·
  `activityRows`(방향 라벨 없음) · `convSections`(★ 절 없음 — 제목 "대화방") · `messageActions`(북마크 뺌). `kindText` 없앰.
- `crates/app/web/app.js` — `messageEl`(쪽 · 아바타는 상대 쪽만 · 말풍선 + 아이콘 줄) · `messageEls`(날짜 구분선) · `bookmarkButton` · `toggleBookmark` ·
  `starButton` · `toggleStar` · 대화방 줄 · 머리(`showRaw` 한 쌍 · `saved_n`) · `#s/{id}` → `#c/{id}`. 탭 · `PANELS` · `openSent` 없앰.
- `crates/app/web/index.html` — 머리 `head-tools`(`.seg` `view_btn` · `raw_btn` · `saved_btn` "북마크" + `saved_n` · `detail_btn`) · `star_btn`(`.star-btn`) ·
  `conv_tag` · 탭 · `panel_sent` 없앰.
- `crates/app/web/app.css` — `#1j` 블록(말풍선 · 쪽 · 아이콘 · `.day-break` · `.seg` · `.tool-btn` · `.star-btn` · 대화방 줄) · `.conv-head { flex: none; }` ·
  `.sr-only` 여백 0.
- 시험 — `view.test.mjs` 82 → 86(`message_actions_no_bookmark` · `bubble_side` · `day_label · day_breaks` · `room_preview`, 기존 `convLabel` · `inbox_rows` ·
  `direction_tag` · `conv_sections*` · `activity_rows` · `message_actions` · `ask_marks` · `group_sections` 고침) · `ux_literals.rs` `messenger_view_literals`(새) ·
  `workspace_labels` · `stabilization_literals` 고침 · `store.rs` `reply_meta_and_summaries`(미리보기 · 자르기 · 쪽) · `chat_api.rs`
  `conversation_summaries_and_message_meta`(`last`).

## 설계 결정 · plan 편차

- **`last` 한 객체** — plan 해석 5 · 설계는 `last_text` · `last_author` 두 칸이었으나 말풍선 쪽(`mine`) · 에이전트 이름(`agent`)도 미리보기 "말한 이" 에 필요해
  `last: {text, author, mine, agent}` 한 객체로 냈다. 화면 밖 내부 API 라 사용자 노출 식별자는 아니다.
- **미리보기 길이** — plan 해석 5 "본문 앞 60자" 대신 기존 `preview()`(120자 + "…")를 재사용하고 줄 폭은 CSS 말줄임으로 자른다. 서버도 120자(`PREVIEW_CHARS`).
- **보기 전환 = `role=group` + `aria-pressed`** — plan 해석 3 은 `role=tablist` 였으나 두 단추가 같은 패널의 표시 방식을 바꾸는 토글이라(탭 패널이 따로 없다)
  눌림 상태 단추 묶음으로 했다.
- **`.sr-only` 여백** — `<p id="order_live" class="sr-only">`(#1g)의 기본 `margin` 이 문서를 15px 넘치게 해 화면이 위로 밀리던 것(main 에서도 재현) — `.sr-only`
  에 `margin: -1px; padding: 0; border: 0`.
- **머리 높이 고정** — 말풍선 목록이 길면 머리(`flex` 자식)가 줄어 세 번째 줄(통신 시각)이 가려져 `.conv-head { flex: none; }`.

## 검증

- `node --test crates/app/web/view.test.mjs` — pass 86 · fail 0(E3 의 95건은 stage 2 · 3 에서 채운다).
- `cargo test --workspace` — 실패 0. `cargo test -p coworkagent-app --test ux_literals` — 15 passed. `cargo clippy --workspace --all-targets -- -D warnings` — Finished.
- mock(가상 데이터) 화면 — 상대 말풍선 왼쪽(아바타 · 이름 · 시각 · 턴 · 걸린 시간) · 내 글 오른쪽 · "어제" · "오늘" 구분선 · 말풍선 아이콘 북마크 → 머리
  "북마크 2 → 1" · 세부정보 북마크 절 · 원문 보기 · 대화방 줄 ☆ 켜고 끄기(`aria-pressed`) · `#s/1` → `#c/1` · 문서 높이 900 = 창 높이. JS 오류 없음
  (mock 의 favicon 404 만).
- 남은 것(stage 2) — 우상단 프로필의 머리글자 + 이름 두 번 표시("박 박지훈") · 상태 색 · 메뉴 즐겨찾기 · 레일 아이콘.
