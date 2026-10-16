# stage1 — 아이콘 · 말풍선 색 · 설정 '?' · 한 열 · 사용자 정보 팝업 · 내 에이전트 · 타일 순서 · 새로 만들기

> 근거: [plan](../plan.md) Stage 1.

## 목표

해석 1 ~ 8 전부 + 아이콘 크기 일관성(plan 승인 때 사용자 추가 지시). 대상 gate: E1 ~ E8.

## 산출 파일

- 서버 — `crates/core/src/run_opts.rs` `RAIL_ORDER_KEY` · `RAIL_ORDERS` · `rail_order()` · `crates/app/src/notify.rs` `SettingsPatch.ui_rail_order`(틀린 값 400) ·
  `Notifier::rail_order` · `settings_json`. 시험 `ux_api` `rail_order_setting_persists`(기본 · 바꿈 · 틀린 값 400 · 그대로).
- `crates/app/web/view.js` — `sideNav` · `settingsTree` · `newMenu` 항목 `icon` · `settingsTree` 에서 `me` 빼고 화면 이름 "화면 — 테마 · 타일 순서" · `newMenu` 에서
  "자동 왕복 대화" 없앰 · `profileMenu` `{key: "me", label: "사용자 정보"}` · `route` 의 `#me` 없앰.
- `crates/app/web/app.js` — `ico(name)` · `renderSideNav`(`.nav-label` = 아이콘 + 이름) · `renderTree` 아이콘 · `openTileMenu` · `peerGroupMenu` · `openConvMenu`
  항목 아이콘 · 새로 만들기 아이콘 · `messageEl` `data-tone` · `profileAct("me")` → `me_dialog.showModal()` · 에이전트 검색 없앰 · `showAgentNew(on)` ·
  `applyRailOrder(order)` · `PREFS` 에 `ui_rail_order`.
- `crates/app/web/index.html` — 화면 머리 제목 아이콘(동적 제목은 안쪽 `span` 이 id) · 대화 머리 북마크 · 세부정보 단추 · 페어링 단추 아이콘을 `.ico` 로 ·
  설정 한 열 · 카드 안내 → `.help-wrap` · `#me` 카드 → `<dialog id="me_dialog">` · `<div id="agent_new" hidden>` · "추가 지시" '?' · "타일 순서" 고르기.
- `crates/app/web/app.css` — `.ico` 한 벌(22개, 16px) · 말풍선 색(`data-tone` → `--tone`) · `.help` · `.tip` · `.me-dialog` · `.settings-cols` 없앰 · 설정 폭 1000px.
- 시험 — `view.test.mjs` 98 → 100(`menu_icons` · `icons_defined`, `settings_tree` · `new_menu` · `profile_menu` · `route` 고침) · `ux_literals.rs`
  `icons_settings_literals`(새) · 1g · 1h · 1j · 1k 문구 시험 고침(아이콘 붙은 제목 · 메뉴 열쇠 · 없어진 두 열 · 검색).
- 문서 — 사용자 가이드(4장 · 5장 지문 · 6장 등록 · 7.2 말풍선 색 · 10장 자동 왕복 진입 · 프로필 메뉴 · 문제 해결 표) · an_app(웹UI 1l 문단 · 트레이 내 정보) ·
  developer-guide(포트) · PRODUCT(다듬기 1l) · README(에이전트 등록 · 메신저형 대화) · 캡처 `home` · `workspace` · `pairing` · `agents`(가상 데이터 mock6).

## 설계 결정 · plan 편차

- **아이콘 크기 일관성**(사용자 추가 지시) — `.ico` 정의 하나(16px)를 메뉴 · 사이드바 · 트리 · 화면 제목 · 단추가 같이 쓴다. 대화 머리의 15px 인라인 SVG
  (북마크 · 세부정보)와 페어링 단추 SVG 도 `.ico` 로 바꿨다. 아이콘만 있는 단추(상단 알림 18px · 레일 설정 20px)와 레일 타일 그림은 손대지 않았다.
  `ux_literals` 가 `.ico {` 정의 1개 · index.html 에 15px SVG 없음을 확인한다.
- **아이콘 정의 방식** — data URI 안 SVG 속성은 홑따옴표(겹따옴표면 CSS `url("…")` 이 끊겨 모양 대신 네모가 칠해졌다 — mock 에서 발견해 고침).
- **'?' 띄우기** — `:focus-visible` 은 프로그램 초점에서 안 떠 `:focus` 로. Esc 로 닫기는 없음(초점을 옮기면 닫힘).
- **페어링 '?'** — 카드 안내와 "초대 생성" 안내(5분 · 1건 바로 등록 · 모르는 연결 삭제 — 1k 에서 보안상 남긴 것)를 한 '?' 에 합쳤다.
- **그룹 체크 항목** — 체크 표시(왼쪽 네모 · 오른쪽 토글)는 그대로 두고 이름 앞에 그룹 아이콘을 더했다.

## 검증

- `cargo test --workspace` 실패 0 · clippy Finished · node 100/100 · `ux_literals` 19 · check_ddd_docs · check_readme · check_crates PASS.
- mock(가상 데이터, scratchpad `l1/l1.mjs`, 다크 · 라이트) — 아이콘 전부 16×16(사이드바 7 · 화면 제목 · 트리 9 · 대화 머리 2 · ⋯ 메뉴) · 말풍선 내 글 강조색
  (다크 rgb(124,131,255) · 라이트 rgb(78,85,217)) · 상대 글 타일 색 연한 바탕 · 새로 만들기 4 항목(자동 왕복 없음) · "에이전트 등록" → 등록 칸 펼침 · 초점
  `agent_name` · 다시 누르면 숨김 · 설정 카드 7개 한 열(왼쪽 같음) · 늘 보이는 카드 안내 0 · '?' 올리면 · 초점이면 뜸 · 타일 순서 그룹 → 사용자 → 레일 DOM
  순서 바뀜 · 새로 고침 뒤 유지 · 되돌림 · 프로필 "사용자 정보" 팝업 열림 · Esc 닫힘 · JS 오류 0.
- 이 PC 실측(E8) — [eval](../eval.md) E8.
