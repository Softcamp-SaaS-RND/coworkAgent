# stage 1 — 에이전트 응답 · 개요 아이콘 · 레일 + · 대화 줄 한 줄 · 레일 놓는 자리 · 그룹설정 B · 주고받은 메시지

## 목표

intent 결정 1 ~ 8 — 사용자 지시 7항 + 주고받은 메시지 시안 적용을 한 stage 로.

## 산출 파일

- `crates/app/web/view.js` — `sideNav`(에이전트 응답 '나' 에만), `peerLeaves`, `tileUnread`, `railSlot`, `avatarKind`, `inboxRows` `mine`, `groupSettingsPick` 삭제.
- `crates/app/web/app.js` — `render`(상대 타일 + 에이전트 응답 → '나'), `renderRail`(`tileUnread`), `renderHome`(아이콘), `convItem`(한 줄), `tileGrab`(`railSlot`),
  그룹설정 B(`renderPeerList` · `gsDot` · `gsToggle` · `gsGroupsEl` · `gsUsersEl` · `settingsConvRow` · `userDetail` · `gsJoin`, 시안 A · HTML 끌기 처리 삭제),
  `convGrab`(`join` · `lostpointercapture` · 한 번만), `peerRowEls`(시안 Peer), `rowEls`(`tile` 갈래 삭제).
- `crates/app/web/index.html` — 레일 "+ 사용자" · "+ 그룹", 에이전트 응답 제목.
- `crates/app/web/app.css` — `--agent-soft` 토큰, `.add-text`, `#rail { user-select: none; }`, 대화 줄 · 그룹설정 B · `.pm-*`, 죽은 규칙(시안 A · 옛 연결 목록 · `row-with-tile`) 삭제.
- 시험 — `view.test.mjs` 새 3(`rail_slot` · `tile_unread` · `avatar_kind`) · 갱신 4(`side_nav` · `side_nav_sections` · `side_nav_agents` · `inbox_rows_author`) · 삭제 1(`group_settings_pick`),
  `ux_literals.rs` PRESENT · ABSENT.
- 문서 — 사용자 가이드(4 · 4.1 · 4.3 · 5) · 웹UI 매뉴얼 · README.

## 설계 결정

- 레일 끌기 원인(코드 사실 · 실측): 보통 크기에서 타일 아래 이름(`pointer-events: none`) · 틈에 놓으면 `elementFromPoint` 가 `rail_pcs` 라 놓을 자리가 없었다(놓은 뒤 순서 그대로).
  작게 크기 타일 위 놓기는 원래 됐다. → `railSlot`(가장 가까운 같은 묶음 타일 가운데) — 레일 밖(가로 벗어남)이면 놓을 자리 없음.
- 그룹설정 펼침 = `state.open`(이 창에만 — `manageItem` 과 같은 기억). 다시 그리기는 서명이 바뀔 때만이라 펼침 · 편집 중 모델 칸이 5초 새로 고침에 지워지지 않는다.
- 설정 대화 줄 끌기 = `convGrab` 재사용(`join`) — 카드 안 대화 줄 위에 놓아도 그 카드(`closest('[data-drag="group"]')`), 대화 순서는 바꾸지 않는다.
- 주고받은 메시지 줄 = `rowEls` 와 나눔(북마크 목록 줄 모양은 그대로 — 제외).
- 남긴 것: `view.roomPreview`(대화 줄에서 쓰지 않게 됐지만 다른 시험이 함께 단정 — 정리는 따로), 그룹 타일 묶음 머리 "에이전트 대화"(intent 제외).

## 테스트 결과

```text
node --test --test-reporter=tap "crates/app/web/*.test.mjs"            → # pass 160 / # fail 0
cargo test -p coworkagent-app --test css_tokens --test ux_literals     → ok 3 · ok 5
```

RED 확인: `ux_literals` `literals_present` 실패(옛 문구 10건 — 시안 A · `room-preview` · `draggable: "true"` · "에이전트 대화" 메뉴 등) → PRESENT · ABSENT 갱신.

실 화면(E8 — `tools/rebuild_run.py` 릴리스 빌드, ego-browser · 실제 마우스):
- 메뉴: '나' = … 등록된 에이전트 · 에이전트 응답, 상대 타일 = 요청 · 즐겨찾기 · 북마크 · 등록된 에이전트(에이전트 응답 화면에서 상대 타일 → 해시 `#` 개요).
- 개요: 절 머리 6/6 사용자 아이콘(`.pa`), 대화 줄 12/12 로봇. 레일 "+ 사용자" · "+ 그룹"(aria-label 없음). 대화 줄 4 = 로봇 4 · `.sub` 0 · 높이 40px.
- 레일 끌기: 보통 크기 — 둘째 타일 **아래 이름** 위에 놓기 → `2:after` 표시 · 순서 2,1,… → 되돌림. 작게 — 마지막 타일 **아래 밖** → `6:after` · 1 이 맨 뒤 →
  첫 타일 **위 틈** → `2:before` · 원래 순서. 매번 서버 `GET /api/peers` 순서 = 화면 순서, 끝 순서 = 시작 순서(1,2,3,5,4,6).
- 그룹설정: 카드 3 · 3열 격자, 사용자 6 줄 · 머리 사용자 · PC · 상태 · 모델 · effort · 대화 · 관리, `draggable="true"` 0. 사용자 줄 빈 칸 누름 → 펼침(대화 줄 3), 6초 뒤에도 펼침.
  대화 줄 손잡이를 그 대화가 이미 든 그룹 카드에 끌기 → 카드 `drop-into` · 사본 1 → 놓으면 "이미 그룹 AI Discovery 에 있습니다"(데이터 안 바뀜) · 사본 0 · 펼침 유지.
- 주고받은 메시지: 줄 9 = 에이전트 4 · 나 2 · 상대 3, 에이전트 줄 바탕 rgb(242, 249, 248) · "에이전트" 칩 4 · 북마크 9 · 옛 줄 타일 0.
- 시험 중 주의: 화면 밖 좌표로 끌다 멈춘 시험 실행(강제 종료) 뒤 서버 상대 순서가 1,2,6,3,5,4 로 남아 있었다 — 원인은 그 중단된 실행으로 추정(확인 필요 — 같은 끌기를 다시 재현하면 서버 = 화면).
  `PUT /api/order` 로 시작 순서 1,2,3,5,4,6 을 되돌리고 확인.

## 다음 단계

- gate 원장 → 적대적 리뷰 라운드 1/2(base `0bdca87`).
