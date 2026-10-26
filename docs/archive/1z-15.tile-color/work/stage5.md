# stage 5 — 개정 ③: 사람 · 그룹 얼굴 = 레일 얼굴

## 목표

plan stage 5 — 머리글자 네모 전부를 레일과 같은 얼굴(사람 · 세 사람 모양 + 타일 색, 그림이 있으면 그림)로, '나' 는 어디서나 흑연 기본.

## 산출 파일

- web: `js/core/widgets.js`(`tileFace` 삭제 · `rowTile` · `railFace` `me`) · `js/logic/peer.js`(`meTint` · `initialOf` 삭제) · `js/conv.js`(`peerFace` · `messagesSig`) ·
  `js/peers.js`(`peerRowEls(rows, peer)` · 서명 `me_icon`) · `js/shell.js`(프로필) · `js/rail.js` · `js/tiles.js`('나' 흑연) · `js/home.js` · `js/search.js` ·
  `js/settings/{conn,groups}.js` · `js/logic/message.js` 주석 · `app.css`(`.tile-shape.me-tile` · 얼굴 그림 크기 · `.tinted` 는 `.tile-shape` 에만).
- 시험: `view.test.mjs`(`me_tint`) · `crates/app/tests/ux_literals.rs`.
- 문서: PRODUCT · masterPlan · web-ui · 사용자 가이드, README 사진 `docs/images/{workspace,pairing,agents}.png`.

## 측정(디버그 빌드 `target\t15` · ego-browser 1440 × 900 · 시험 데이터 `ui1z15b`)

- 얼굴 판독(라이트 · 다크 같음 — 모양/색 class/폭): 프로필 `person/me-tile/24`, 레일 나 `person/me-tile` · 상대 `person/tinted peer-3` · 그룹 `group/tinted peer-4`,
  대화 11 말풍선 · 요청 의견 `person/tinted peer-3/24` × 5, 주고받은 메시지 머리 `person/tinted peer-3/24` · 줄 나 `person/me-tile/28` · 상대 `person/tinted peer-3/28`,
  연결 줄 `person/tinted …/26`, 그룹설정 줄 `group/tinted peer-4/26` · `group/tinted peer-5/26`. 글자만 든 얼굴 네모 0.
- 그림: 상대 2 에 PNG 그림 → 레일 · 주고받은 메시지 머리 · 연결 줄 = `img`. 상대 1 에 그림 → 대화 11 말풍선 아바타 5 개 `img/24`. 측정 뒤 그림 지움.
- 얼굴 그림 크기: 26px 줄 · 24px 프로필 안 사람 모양 17px(70 %).
- 공유 대화 21: 사람 글이 이 PC 쪽(오른쪽)이라 아바타 없음 — 원 상대 무채 분기는 화면에 안 나옴(코드만).
- 콘솔 · 페이지 오류 0. README 사진 셋(다크 @2x) `readme pop=true errs=[]` — 말풍선 · 요청 의견 · 연결 줄 · 상세 머리 얼굴이 레일과 같다.
- 스크린샷 판독은 README 사진만(측정 스크립트의 창 최소화로 측정 화면 찍기 실패 — 창을 `Browser.setWindowBounds` 로 되돌린 뒤 README 사진은 찍힘).
- 측정 스크립트 · 화면은 scratchpad(`ego/1z15b/e17.mjs` · `e17b.mjs` · `readme.mjs`) — 저장소에 넣지 않음.
