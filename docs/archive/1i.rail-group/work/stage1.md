# stage1 — 스키마 16 · 그룹 · 순서 API · 그룹 타일 · 그룹 사이드바 · 넣고 빼기

> 근거: [plan](../plan.md) Stage 1. 사용자 검증은 D28 — 19.prototype-acceptance 에서 일괄.

## 목표

그룹 저장 · 소속 · 손으로 놓는 자리(해석 1 ~ 6)와 레일 그룹 타일 · "그룹 +" · 그룹 사이드바(메뉴 4 · 요약 · ★ + 상대별 절 · "대화 추가") · 대화 ⋯
"그룹" 체크 · "그룹에서 빼기" · 그룹 화면에서 만든 새 대화 소속(해석 7 · 8 · 10 · 12 · 13). 대상 gate: E1 ~ E5(E5 는 stage 3 끝 건수).

## 산출 파일

- `crates/core/src/db.rs` — 스키마 16: `peers.pos` · `conversations.pos`(첫 자리 = id 순 · 최근 활동순, 창 함수) · `groups`(AUTOINCREMENT) ·
  `group_convs` · 색인.
- `crates/core/src/groups.rs`(새) — `Group` · `Order` · `clean_name` · `list` · `by_id` · `name_taken` · `add` · `rename` · `set_icon` · `remove`(소속
  함께) · `unknown_conversations` · `change` · `join` · `reorder`.
- `crates/core/src/chat.rs` — 대화 INSERT `pos = min - 1` · `Summary.pos` · `Scope`(전부 · 상대 · 그룹) · `messages_across` · `saved` 가 `Scope` 를 받는다.
- `crates/core/src/peers.rs` — 새 상대 `pos = max + 1` · `list` = `ORDER BY pos, id`.
- `crates/agent/src/chat.rs` — `Compose.group`(새 대화 · 자동 왕복 시작 트랜잭션에서 `groups::join`).
- `crates/app/src/groups_api.rs`(새) — `GET · POST /api/groups` · `PATCH · DELETE /api/groups/{id}` · `PUT /api/groups/{id}/conversations` ·
  `PUT /api/order`. `chat_api.rs` — `group` 거르기(`/api/messages` · `/api/saved`, `peer` 와 함께면 400) · 새 대화 `group`(없는 그룹 404, 보내지
  않음) · 대화 JSON `pos` · 그룹 라우트 붙이기.
- `crates/app/web/view.js` — `sideNav("group")` · `sortByPos` · `inGroup` · `groupPeers` · `scopeQuery` · `groupNameError` · `membershipDiff` ·
  `peerSummary` 대화 순서 = `pos`.
- `crates/app/web/app.js` — `state.groups` · `state.railGroup` · `loadGroups` · 그룹 타일 · `pickGroup` · `scopedConvs` · 그룹 사이드 머리 · 목록
  거르기(요청 · 저장 · 주고받은 글 · 대기 수) · 그룹 요약 · 떠 있는 상자(`openFloat` · `closeFloat` — 화면 기준 위치, Esc · 바깥 누름으로 닫힘) ·
  "그룹 +" · 대화 ⋯ 메뉴(행 · 대화 머리) · "대화 추가" · 새 대화 `group`. `index.html` — 레일 구분선 · `#rail_groups` · "그룹 +" · `#conv_more_btn` ·
  `#home_title` · `#group_empty` · `#float`. `app.css` — 레일 그룹 · 떠 있는 상자 · 행 ⋯.
- 시험 — `crates/core/tests/store.rs` `schema16_groups_and_order` · `crates/app/tests/groups_api.rs`(12건) · `view.test.mjs`(6건 추가, `peer_summary`
  고침) · `ux_literals.rs` `rail_group_literals` · `paths.rs` 스키마 번호(15 → 16, "더 새 스키마" 17).

## 설계 결정

- **`reorder` 는 `groups.rs`** — plan 설계는 `chat::reorder` 라 적었으나 상대 · 그룹 · 대화 셋의 자리를 한 함수로 다뤄 그룹 모듈에 두었다(공개 계약은
  `PUT /api/order` 그대로).
- **`Scope` enum** — `messages_across` 에 그룹 인자를 더하면 인자 8개(clippy `too_many_arguments`) — 상대 · 그룹을 한 값으로 묶었다. 부르는 곳 5군데
  고침(시험 포함).
- **새 대화의 그룹 확인은 보내기 전** — 없는 그룹이면 404 · 대화를 만들지 않는다. 소속은 대화와 같은 트랜잭션(`groups::join` — 없는 그룹이면 아무것도
  안 함).
- **떠 있는 상자 하나(`#float`)** — 레일은 `overflow-y: auto` 라 그 안의 `position: absolute` 팝오버가 잘린다. 화면 기준(`fixed`) 하나를 메뉴 · 이름 ·
  대화 추가가 같이 쓴다(stage 2 타일 메뉴도).
- **그룹 화면 접힘 열쇠** = '나' 와 같은 `convs:{상대}` · `home:{상대}`(해석 8).
- 새 함수는 모듈 export · `pub fn`(파일 전용 자유 함수 추가 없음 — 단 `chat_api.rs` 의 `scope_of` 는 주변 `bad` · `fail` 과 같은 파일 내 도우미).

## 테스트 결과

```text
$ cargo test -p an-core --test store schema16           → test result: ok. 1 passed; 0 failed
$ cargo test -p coworkagent-app --test groups_api        → test result: ok. 12 passed; 0 failed
$ cargo test -p coworkagent-app --test ux_literals       → test result: ok. 13 passed; 0 failed
$ node --test --test-reporter=tap "crates/app/web/*.test.mjs" → # pass 63 · # fail 0
```

mock 화면 확인(scratchpad `mock3.py` — 가상 데이터, 실제 앱 · 이력 미사용, Edge headless) 14/14 PASS: 빈 이름 · 같은 이름 거절 문구 · 그룹 타일 ·
메뉴 4 · 빈 그룹 안내 · Esc 닫힘 · "대화 추가" 검색 + 체크 2 · 그룹 요약 = 대화 있는 상대만 · 대화 ⋯ 체크 토글 · "그룹에서 빼기" · 그룹인 채 `#activity`
→ '나'. 찾아 고친 것 1: 그룹을 골라도 '나' 타일이 눌린 표시(`render` 가 `railPeer` 만 봄).

TDD 기록: store · API 시험은 구현 직후 썼다(먼저 실패를 보지 못함). API 시험 `order_partial_ties_by_id` 는 기대값을 잘못 잡아 1회 실패 → 시험을
고침(새 대화 `pos` 는 -1, -2, … — 동점이 생기도록 전체 순서 뒤 일부만 보냄).

## 다음 단계

stage 2 — 타일 메뉴(우클릭 · ⋯ · 메뉴 키) · 이미지 지우기 · 그룹 이름 · 그림 · 그룹 삭제 · 타일 색 id 기준 · 사이드 머리 ✎ 를 고른 타일(그룹 포함)에.
