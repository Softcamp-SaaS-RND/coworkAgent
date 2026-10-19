# stage 1 — 요청 보드 · 에이전트 대화 · 피드백 말풍선 · 입력창 · 편집 모드 삭제

## 목표

plan stage 1 — view.js 순수 함수 + 시험 → app.js · index.html · app.css 그리기 · 편집 모드 삭제 → 문서(E1 ~ E13).

## 산출 파일

- `crates/app/web/view.js` — `BOARD_COLS` · `boardColumns` · `canDrop` · `reqFeedback` · `convListParts` · `agentConvSections`, `sideNav` 5번째 인자 `agentUnread` · 에이전트 머리 + 하위 둘,
  `route("#agents/convs")` = `agentConvs`, `groupLeaves` 에 `agentConvs`, `keepPicked` 삭제.
- `crates/app/web/app.js` — 요청 보드(`reqBoard` · `REQ_BOARD` · `boardEls` · 보기 단추 · `requestItem` 분리 · 탭 토글), `renderSideNav` 머리 · 하위 · 안 읽음 합,
  `renderConvList` → `convListParts`, `renderAgentConvs`(+ `VIEW_EL` · 라우터 · `loadConvs` · `moveItem` · 끌어 옮기기 상자), 등록된 에이전트 머리 이름,
  `feedbackEl`(`messageEls` 가 요청 뒤에), 말풍선 메타 줄의 메모 삭제, 편집 모드 전부 삭제(`setEditMode` · `renderEditBar` · `askScope` · `deletePicked` · `editBody` · 상태 셋),
  `renderMentionHint` → `renderAskHint`.
- `crates/app/web/index.html` — 요청 보기 세그먼트 · `#req_state_label` · `#req_board`, `#agent_convs` 화면, 편집 줄 · 편집 막대 · `#scope_dialog` · `#mention_hint` 삭제,
  질의 / 요청 라디오 뒤 `span`(입력창 · 새 대화).
- `crates/app/web/app.css` — `.composer` 최대 폭 · 그림자, `.ask-kind` 세그먼트, 피드백 말풍선, `.req-board` · `.board-col`, `.nav-head` · `.nav-sub`, 편집 모드 클래스 삭제.
- 시험: `view.test.mjs` 새 7(`req_board_columns` · `req_board_drop` · `conv_list_mine` · `side_nav_agents` · `agent_convs_sections` · `req_feedback` · `composer_a`) + 고침 4
  (`side_nav` · `menu_icons` · `side_nav_sections` · `keep_picked` → `messages_sig_edit`), `crates/app/tests/ux_literals.rs` 있음 · 없음 표.
- 문서: [web-ui](../../../manual/dev/web-ui.md) 사이드바 · 요청 · 대화 목록 · 대화, PRODUCT.md 이력 정리 · 요청 보드 줄.

## 설계 결정

- 보기 기억 = 접힘 목록 `ui_folded` 의 `reqboard:me`(plan 해석 2 — 열쇠 규칙 `{화면}:me` 통과, 서버 변경 없음, `ponytail:` 주석).
- 보드 끌기는 대화 줄 끌어 옮기기(`rail` · `conv_sections` 처리기)와 따로 — 카드 · 열에 직접 처리기(끄는 행 `boardDrag`), 놓으면 카드의 글자 단추 줄에 `noteForm`.
  다시 그리기는 `needsRender` 서명이 같으면 하지 않아 열린 메모 칸이 남는다(데이터가 바뀌면 다시 그림 — 목록 보기와 같은 한계).
- 메뉴로 '요청' 을 열 때 보드가 기억돼 있으면 받은 요청 탭으로(결정 12 "보드는 탭 한 쪽만" — 개요로 열면 기억이 소용없음). 보드에서 같은 탭을 다시 눌러도 개요로 가지 않는다.
- 세그먼트는 `:has()` 대신 형제 선택자(plan 개정 기록 ①), 새 대화 질의 / 요청도 같은 모양(②).
- 나 타일에서 에이전트 대화만 있는 상대 절은 빈 안내("대화 없음")로 남는다 — 상대별 절 목록은 지금 규칙(`convSections`) 그대로.

## 테스트 결과

- node: 새 시험 먼저 → 화면 고치기 전 `side_nav_sections` · `composer_a` FAIL(2) → 구현 뒤 `# pass 140 # fail 0`(기준 133).
- `cargo test -p coworkagent-app --test ux_literals` — `literals_present` 가 옛 문구 3(`askScope("고친 본문 저장")` · `splitByDir(s.rows)` · 탭 토글)로 FAIL → 표 갱신 뒤 `5 passed`.
