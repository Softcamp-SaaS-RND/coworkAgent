# stage 2 — 화면(web)

## 목표

plan stage 2 — 타일 편집 '아이콘 색' 단추(결정 1 · 3 · 4 · 6) · 색 붙임(결정 5) · 그룹 세 사람 그림(결정 7).

## 산출 파일

- `crates/app/web/js/logic/peer.js` — `tileTint(color)`(여섯 중 하나면 `tinted {색}`, 아니면 빈 글자).
- `crates/app/web/js/core/widgets.js` — `tileFace(icon, name, color)` · `rowTile(cls, icon, name, color)` · `railFace(icon, group, color)`, `GROUP_SVG` = 앞 사람 가운데 + 뒤 좌우 옅은 둘.
- `crates/app/web/js/tiles.js` — `tileNow` 색 · `tileEdit.color` · `renderTileColors`('나' 는 맨 앞 흑연 `""`) · 미리보기 색 · 저장 때 바뀐 색만(`color` · `me_color`).
- 색 붙이는 곳: `js/rail.js`(나 · 상대 · 그룹, 서명에 색) · `js/home.js` 개요 목록 머리 · `js/peers.js` 주고받은 메시지 머리 · 사람 아바타(`peerRowEls(rows, peerColor)`) ·
  `js/search.js` 사람 칩 · `js/settings/conn.js` 목록 · 대화 줄 · 상세 · `js/settings/groups.js` 목록 · 상세 · `js/conv.js` 말풍선 · 요청 의견 상대 아바타(`peerTint`, `messagesSig` 에 색) ·
  `js/shell.js` 프로필 얼굴 · `js/settings/prefs.js` `me_color` 바뀌면 레일 · 프로필 다시.
- `crates/app/web/index.html` — `#tile_colors`(아이콘 색). `crates/app/web/app.css` — `.swatch.rail-2` · `:is(.tile-shape, .avatar, .avatar-face, .pm-av).tinted` 흰 글자 + 여섯 바탕.
- 시험: `view.test.mjs` `tile_tint`, `ux_literals.rs` 바뀐 행 8(부르는 꼴에 색) · 새 PRESENT 10 · `token_contrast` 82 → 88(`--on-dark` × 여섯 — 두 테마 같은 값).

## 설계 결정

- 색 class 이름 = 에이전트와 같은 열쇠(`peer-1` …) + `tinted` — 이미 있던 `.peer-N` 바탕 규칙은 그대로 두고, 무채 네모 규칙(`.tile-shape` · `.rail-btn.me .tile-shape`)보다 뒤 · 같은 특이도로 이긴다.
- '나' 의 흑연 단추 값 = `""`(설정 지움) — 고른 색이 없으면 레일은 흑연, 그 밖 머리글자 네모는 무채 그대로.

## 테스트 결과

- RED — node `tile_tint` 실패(없는 export), `ux_literals` 바뀐 행 8 실패.
- GREEN — node `# pass 173` · `# fail 0`, `ux_literals` · `css_tokens` ok 5 · ok 3, `check_code_tidy.py` PASS.

## 다음 단계

stage 3 — 실제 화면 · 문서 · README 사진.
