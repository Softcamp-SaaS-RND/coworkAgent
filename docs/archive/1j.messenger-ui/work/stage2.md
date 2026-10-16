# stage2 — 메뉴(즐겨찾기 · 상대 개요) · 상태 색 · 프로필 · 레일 아이콘 A · 초점 유지 · 타일 메뉴 옮기기

> 근거: [plan](../plan.md) Stage 2. 단계 끝 mock(가상 데이터) 화면을 사용자에게 보인다(Q3).

## 목표

해석 6 ~ 9 — 사이드바 메뉴 세 범위(즐겨찾기 `#starred` 새 화면 · 상대 타일 첫 화면 = 그 상대 개요) · 프로필 아이콘 상태 색(받기 가능 초록 · 집중 주황 · 앱과 연결
끊김 붉음) · 프로필 메뉴 상태 고르기 · `focus_chip` 없앰 · 레일 아이콘 A안(사람 · 사람 + 그림자 + 아래 이름, 72px) · 다시 그려도 초점 유지 · 타일 메뉴
"위로 옮기기 · 아래로 옮기기". 사용자 지시(stage 1 검토)로 말풍선 최대 폭 90%. 대상 gate: E1 ~ E4.

## 산출 파일

- `crates/app/web/view.js` — `sideNav(tile, unseen, waiting, starred)`('나' 개요 | 내 활동 · 요청 · 주고받은 글 | 즐겨찾기 · 북마크 항목 · 에이전트, 상대 요청 |
  즐겨찾기 · 북마크 항목 · 에이전트, 그룹 개요 | 요청 | 즐겨찾기 · 북마크 항목) · `route("#starred")` · `profileMenu(paused, focus, offline)` · `presenceOf` ·
  `starredSections` · `groupLeaves` 에 `peer`. `tileText` 없앰.
- `crates/app/web/app.js` — `tileFace(icon, group)`(`PERSON_SVG` · `GROUP_SVG`) · `tileName` · `drawRailTiles` · `keepFocus`(레일 · 대화방 목록) · `dragSel` ·
  `stepMove`(Alt + ↑ / ↓ · 타일 메뉴 공용) · `renderAvatar`(아이콘 또는 사람 모양 + 이름 한 번, `st-{상태}`) · `netResult` · `appOffline`(`OFFLINE_AFTER` = 2) ·
  `renderProfileMenu`(머리 · `menuitemradio` · 끊김 표시) · `renderStarred` · `renderHome`(상대 하나) · `pickTile` → `#` · `#p/{id}` → 그 상대 고르고 `#`.
  `renderFocusChip` 없앰.
- `crates/app/web/index.html` — `#focus_chip` 없앰 · `#starred` 절.
- `crates/app/web/app.css` — 레일 72px · `.tile-name` · `.st-online` · `.st-focus` · `.st-offline` · 프로필 알약 · 메뉴 머리 · 라디오 표시 · `.starred-list` ·
  `.msg` 최대 폭 90%. `.status-chip` · `.tile-text*` · `.rail-btn.group` 겹친 카드 없앰.
- 시험 — `view.test.mjs` 86 → 87(`tile_text` 다섯 없앰, `display_name` · `side_nav_exchanged_me_only` · `side_nav_starred` · `profile_menu_radio` ·
  `presence_of` · `starred_sections` 새, `side_nav` · `side_nav_group` · `profile_menu` · `group_leaves` · 위치 고침) · `ux_literals.rs`
  `menu_status_rail_literals`(새) · `focus_mention_labels` · `rail_scope_literals` · `stabilization_literals` 고침.

## 설계 결정 · plan 편차

- **말풍선 최대 폭 90%** — 사용자 지시(2026-10-15, stage 1 검토): "에이전트의 긴 답을 말풍선으로 표시하며 창의 최대 90% 까지 차지해도 됨 — 왼쪽, 오른쪽에
  붙어있는 모양으로 구분 가능". stage 1 의 `min(760px, 92%)` → `90%`. plan 개정 기록에 남긴다.
- **끊김 판정 = 모든 요청** — plan 해석 7 은 "5초 새로 고침이 연달아 2번 실패" 였으나 각 목록 읽기가 오류를 안에서 삼켜 새로 고침이 실패를 모른다. `api()` 의
  `fetch` 자체 실패(닿지 못함)를 세고 닿으면 0 으로 — 새로 고침이 5초마다 묻으므로 같은 결과. HTTP 오류 · 401 은 닿은 것(401 은 기존 다시 열기 안내).
- **프로필 메뉴 문구** — 시안(Exchanged 캔버스) 그대로: 머리 "상태 — 아이콘 테두리 색", "받기 가능 · 새 요청을 받는다", "집중 · 새 요청을 거절", "앱과 연결
  끊김 · 자동 표시". 옛 "집중 — 요청 받지 않음" 은 없앴다.
- **즐겨찾기 수** — 시안의 메뉴 "즐겨찾기 n"(그 범위 ★ 대화방 수)을 넣었다. 시안의 "북마크 항목 n" 은 범위별 수를 따로 읽어야 해 넣지 않았다(확인 필요 시 stage 3).
- **옛 `#p/{id}`** — 그 상대를 고르고 `#`(그 상대 개요)로 바꾼다. `#p`(전체 주고받은 글)는 '나' 에만 — 상대 · 그룹이면 '나' 로.
- **이미지 지우기 안내** — "이름 머리글자로 보입니다" → "사람 모양으로 보입니다"(A안이라 앞 문구가 틀려짐, 사용자 노출 문구 변경).
- **초점 유지** — 다시 그리기 전 초점 항목(`data-drag` · `data-id` · `data-section`)과 그 안의 자리(링크 · ⋯ · ☆)를 기억해 새 요소로 되돌린다. 타일을 누른 뒤
  Alt + ↓ 가 듣지 않던 원인(레일이 다시 그려져 초점이 body 로)이 이것으로 풀렸다.

## 검증

- `node --test --test-reporter=tap "crates/app/web/*.test.mjs"` — `# pass 87` · `# fail 0`(E3 95건은 stage 3 에서).
- `cargo test --workspace` — 실패 0. `ux_literals` — 16 passed. `cargo clippy --workspace --all-targets -- -D warnings` — Finished. `check_ddd_docs` — PASS.
- mock(가상 데이터) — 상대 타일 → "개요 · 이준호" · 메뉴 요청 / 즐겨찾기 · 북마크 항목 · 에이전트 · 누른 뒤 초점 = 그 타일 · Alt + ↓ 2,3,4 → 3,2,4(초점 유지 ·
  "이준호 — 2번째로 옮김") · Alt + ↑ 되돌림 · 타일 메뉴 위로 → 3,2,4 · 아래로 되돌림 · `#p/3` → "개요 · 최유나" · '나' 즐겨찾기 = 상대별 절 2건 · 집중 →
  `st-focus` · 요청을 막자 11초 뒤 `st-offline` · 풀자 `st-focus` · 받기 가능 → `st-online` · 문서 높이 900 = 창. JS 오류 없음(막은 요청 · mock favicon 만).
