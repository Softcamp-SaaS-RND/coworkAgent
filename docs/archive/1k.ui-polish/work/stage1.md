# stage1 — 에이전트 연필 · 주고받은 글 타일 · 칩 · 다시 보내기 없앰 · 레일 + → 페어링 · 설정 두 열

> 근거: [plan](../plan.md) Stage 1.

## 목표

해석 1 ~ 6 전부. 대상 gate: E1 ~ E8.

## 산출 파일

- `crates/app/web/view.js` — `inboxRows` 행 `agent`(보낸 대화 = `conv.agent`, 받은 대화 = `내 {conv.agent}`) · `messageActions` 의 `resend` 없앰 · `refusedText`
  "(입력창에서 다시 보낼 수 있다)".
- `crates/app/web/app.js` — `rowEls(rows, empty, {tile})`(주고받은 글만 상대 타일 · 에이전트 칩) · `renderAgentsView` 내 에이전트 연필 · `editAgent` · `messageAct`
  다시 보내기 갈래 없앰 · `rail_add` → `showAccept` + `goSettings("conn", "invite_text")` · `showAccept` · `accept_btn` · `manageItem`(머리 + "관리" 로 여닫는 상세,
  `state.open`) · `rowTile` · `svgButton` · `TRASH_SVG` · `renderPeerList` · `renderAgentList` 를 관리 줄로 · `agent_new_btn`. 안 쓰게 된 `remembered` 없앰.
- `crates/app/web/index.html` — 두 안내문 없앰 · `settings-cols`(왼쪽 `#conn` · `#peers`, 오른쪽 `#me` · `#agents`) · 페어링 카드(큰 단추 둘 · `invite_make` 안내 ·
  `invite_accept` 접힘 · 제출 "추가 · 승인") · 내 에이전트 머리 "에이전트 등록" · `#agent_new`.
- `crates/app/web/app.css` — `.settings-cols`(1100px 아래 한 열) · `.pair-btns` · `.manage-*` · `.pa` · `.agent-tile` · `.row-with-tile` · `.agent-chip` · 에이전트 연필.
- 시험 — `view.test.mjs` 96 → 98(`inbox_rows_agent` · `message_actions_no_resend`, `message_actions` · 요청 표시 · `refusedText` 고침) · `ux_literals.rs`
  `polish_literals`(새) · `workspace_labels` · `invite_labels_without_messenger` · `peer_group_settings_literals` 고침.

## 설계 결정 · plan 편차

- **관리 단추 = `aria-expanded` 토글** — 1j 의 `<details>` 대신 머리 줄(끌기 대상) + "관리"(`aria-controls` 상세). 펼침은 `state.open` 이라 5초 새로 고침 · 다시 그리기에도
  남는다. 상대 줄 "그룹 ▾" 는 더 이상 `summary` 안이 아니라 `preventDefault` 없앰.
- **연필 초점** — `editAgent` 가 `state.open` 에 넣고 목록을 다시 그린 뒤 `goSettings("agents", "agent{id}_manage")` — 설정 절로 스크롤 · "관리" 에 초점.
- **'+' 초점** — `goSettings("conn", "invite_text")` · 수락 칸을 먼저 펼친다. 트리 · "+ 새로 만들기" 의 "초대 수락"(`#settings/invite_accept`)도 칸을 펼친다.
- **"초대 생성" 안내** — 캔버스엔 없지만 "5분 · 1건 바로 등록 · 모르는 연결은 삭제" 안내는 보안상 남겼다(단추 아래 `invite_make`).
- **에이전트 화면 상대 줄 부제** "{상대} 의 에이전트 — 누르면 새 대화" 는 줄 부제라 그대로(없앤 것은 머리 안내문).

## 검증

- `cargo test --workspace` 실패 0 · clippy Finished · node 98/98 · `ux_literals` 18.
- mock(가상 데이터, `j1/k1.mjs`) — 주고받은 글 타일 15 · 칩(guide · 내 review …) · 안내문 없음 · 연필 1 → `#settings/agents` · 초점 `agent1_manage` · 펼침 ·
  대화 다시 보내기 0 · + → `#settings/conn` · 페어링 카드 위 48px(상단 바 바로 아래) · 수락 칸 펼침 · 초점 `invite_text` · 관리 열고 닫기 · 초대 수락 단추로 접기 ·
  1000px 창 한 열 · 문서 높이 900 · JS 오류 0.
- 이 PC 실측(E8) — [eval](../eval.md) E8.
