# stage 2 — 웹UI(대화 메뉴 나가기 · 삭제 · 읽기 전용 · 그룹 = 대화만 · 설정 연결 목록) · 문서

## 목표

plan stage 2 — view.js 순수 함수 + 시험 → app.js · app.css 조립 → 문서(E3 · E8 ~ E11, E12 는 19 인수로 이월 — D28).

## 산출 파일

- `crates/app/web/view.js` — 새 `convMenuItems` · `convClosed` · `USER_LIST` · `settingsGroupSections` · `convCopy`, `refusedText` `left`, `reqMoves(m, closed)` ·
  `reqResend(m, closed)` · `messageActions(m, closed)`(`requestRows` 는 `m.conversation` 으로 닫힘을 본다 — 보드 `canDrop` 도 따라 막힘), `convMeta.tag` = `convClosed`,
  `inGroup` = `convs` 만. 삭제: `NO_PEER_GROUP` · `peerSections` · `peerMove` · `peerGroupItems` · `peerGroupText`.
- `crates/app/web/app.js` — `openConvMenu` 끝 구분선 + 나가기 · 삭제(`convEndForm` 확인 — 지운 대화가 열려 있었으면 '개요'), `showConversation` 읽기 전용 = 공유 사본 또는
  닫힌 대화(입력창 · 컨텍스트), 대화 머리 부제에 닫힌 문구, 자동 왕복 단추 · 관람자 넣기 숨김, `messageActions(m, closed)`. `renderPeerList` 다시: `groupSection`(그룹 대화 줄 ·
  "그룹에서 빼기" · 이름 바꾸기 · 삭제) · `userListSection`(관리 줄 — "그룹 ▾" · 끌기 손잡이 없음 + 접이식 "대화 n" `settingsconvs:{상대}` 처음 접힘) · `settingsConvRow`,
  끌기 = 대화 줄 → 그룹 절 → `convCopy` → `PUT /api/groups/{id}/conversations`. `loadConvs` 가 연결 목록도 다시 그린다(서명에 대화). 삭제: `peerGroupMenu` · `changePeerGroup` ·
  타일 메뉴 상대 '그룹' 체크 · 상대 줄 끌기 처리기.
- `crates/app/web/app.css` — `.ico-exit` · `.settings-convs` · `.settings-conv` · `.settings-tree`, 없앤 `.peer-group-btn` 규칙 삭제. `index.html` 은 바꿀 곳 없음.
- 시험: `view.test.mjs` 새 6(`conv_menu_items` · `conv_closed` · `refused_left` · `req_moves_closed` · `settings_group_sections` · `conv_copy`), 고침 2
  (`group_summary_peer_member` → `group_summary_conv_member` · `in_group_peers` → `in_group_convs_only`), 지움 5(`peer_sections` · `peer_move` · `peer_move_round_trip` ·
  `peer_group_items` · `peer_group_text`). `crates/app/tests/ux_literals.rs` — 상대 그룹 문구 있음 표 6 + 관리 줄 열쇠 → 없음 표 6, 새 문구 있음 표 13.
- 문서: [web-ui](../../../manual/dev/web-ui.md) 레일 그룹 · 설정 연결 목록 · "대화 나가기 · 삭제" 절, [an_app](../../../manual/dev/an_app.md) 경로 · JSON · 웹 함수(#1j 상대
  소속 경로 · 함수는 "없앰"), [an_net](../../../manual/dev/an_net.md) `Leave` · `Pong.leave` · `left` · `knows_leave`, [an_agent](../../../manual/dev/an_agent.md) 나가기 · 삭제,
  [an_core](../../../manual/dev/an_core.md) 스키마 21 · chat 함수 · groups, [개발자 가이드](../../../manual/dev/developer-guide.md) 7.2 표(스키마 21 · `left_convs` ·
  `group_peers` 없앰) · 7.5 v21 · 프로토콜 목록 · 8.12, [사용자 가이드](../../../manual/user/user-guide.md) 4.1 · 4.2 · 5장 연결 목록 · 7.8, PRODUCT 대화 나가기 · 레일 그룹 줄.

## 설계 결정

- 설정 연결 목록 검색 — 사용자 목록은 연결(이름 · PC · 지문)을, 그룹 절은 맞는 연결의 대화 줄만 거른다(지금 검색 칸 하나 그대로).
- 대화 줄을 사용자 목록에 놓으면 할 일 없음(놓을 자리 표시도 그룹 절만). 이미 있는 그룹에 놓으면 상태 줄 "이미 그룹 … 에 있습니다".
- 나가기 확인 단추는 보통 단추, 삭제만 `danger`(plan "삭제 … danger").
- 상대 타일의 그룹 메뉴를 없애며 타일 메뉴 항목은 편집 · 이미지 지우기 · 위로 · 아래로 · 그룹 삭제만 남는다. 레일에서 상대 타일을 그룹 타일에 놓는 경로는 원래 없었다
  (`dropKind` — 대화 → 그룹만).

### plan 밖 변경

- 문서 — plan 은 web-ui · an_app · PRODUCT 만 적었으나, 상대 단위 그룹 · 스키마 20 · 프레임 목록을 사실로 적은 an_core · an_net · an_agent · 개발자 가이드 · 사용자 가이드도
  고쳤다(그대로 두면 틀린 문서). 프레임 · 오류 코드는 그것을 정본으로 적는 an_net 에 적고 an_app 은 링크.

## 테스트 결과

- node: 새 시험 먼저 → `conv_menu_items` FAIL(`.ico-exit` 없음) → CSS 뒤 `# pass 142 # fail 0`(바닥 141 — 141 − 5 + 6).
- `cargo test -p coworkagent-app --test ux_literals` — 화면을 고친 뒤 `literals_present` 가 옛 상대 그룹 문구 7 로 FAIL → 표 갱신 뒤 `5 passed`.
- gate(이 stage):

```text
E3  node --test --test-reporter=tap "crates/app/web/*.test.mjs"   → # pass 142 / # fail 0
E8  … --test-name-pattern="conv_menu_items|conv_closed|refused_left"  → # pass 3 / # fail 0
E9  … --test-name-pattern="settings_group_sections|conv_copy"         → # pass 2 / # fail 0
E10 python -c …                                                     → group_peers_live=0 peer_api=0 web=0
E11 python tools/check_ddd_docs.py && python tools/check_crates.py  → PASS · crates=4 edges=6 violations=0
E1  cargo test --workspace                                         → test result 줄 모두 0 failed
```

- E12(실화면 두 PC) — 프로토타입 인수 Task(D28)로 이월.

## 다음 단계

eval — `tools/check_gates.py` 로 원장 → 적대적 평가(라운드 상한 2) → report · 종결(masterPlan D37 개정 · D40 환류).
