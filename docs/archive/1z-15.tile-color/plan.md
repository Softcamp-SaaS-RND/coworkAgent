# plan — 1z-15.tile-color

> 근거: [intent](intent.md)(승인 2026-10-26 — 결정 1 ~ 9). **구현자는 이 문서만 보고 일한다.**
> 브랜치 `task/1z-15-tile-color`(원 작업 폴더 — worktree 없음). 기준 커밋: `bea90b3`(main — #1z-14 종결 뒤).
> 승인: 2026-10-26 — 사용자 지시 + 질문 답 U1 ~ U6 이 설계를 정함, plan 승인 질문 없이 구현(구현자 해석 — report 에 적음).

## 목표

레일 타일 편집(나 · 상대 · 그룹)에 '아이콘 색'(에이전트 여섯 색), 상대 · 그룹 기본 = 처음 무작위 저장 · '나' 기본 = 흑연, 색은 레일 + 그 사람 · 그룹 머리글자 네모 전부,
그룹 그림 = 세 사람, 스키마 29, 버전 1.4.0 그대로.

### 성공 기준(intent 사본)

- A1: 타일 편집에서 나 · 상대 · 그룹 색을 여섯 중 골라 저장 · 다시 열어도 그대로, '나' 는 흑연으로 되돌림.
- A2: 새 · 있던 상대 · 그룹은 여섯 중 한 색(저장 — 다시 열어도 같음).
- A3: 레일 · 머리글자 네모 전부에 그 색, 글자 대비 4.5:1 이상(두 테마).
- A4: 그룹 그림 = 세 사람(앞 가운데 + 뒤 좌우).
- A5: 기존 동작 · 문구 그대로(새 '아이콘 색' 문구만 더함), 검증 명령 PASS, 매뉴얼 · README 사진 반영, 버전 1.4.0.

## 설계

### 공통 규칙

- 색 값 = `an_core::agents::ICON_COLORS` 열쇠(웹 `AGENT_COLORS` · `app.css` `--agent-tile` · `--peer-1` ~ `--peer-5` 토큰). 새 함수는 둘레 규약대로(모듈 `pub fn` · `export`) —
  파일 전용 static 함수를 새로 만들지 않는다. 제품 원본 주석에 Task 번호 · 결정 번호 · 날짜 없음. TDD — 시험 먼저. 같은 개념 주석 · 매뉴얼은 함께 고친다.

### stage 1 — 저장(an_core · an_net · an_app)

1. `run_opts`: `TILE_COLOR_RANDOM`(SQL 식 — `ICON_COLORS` 여섯 중 `abs(random()) % 6`) · `clean_tile_color(s) -> Result<String, String>`(여섯 밖 = `Err`) ·
   `ME_COLOR_KEY = "me_color"` · `me_tile` 이 (표시 이름, 아이콘, 색 — 빈 · 틀린 값 `None` = 흑연) 셋을 낸다.
2. 스키마 29(한 트랜잭션): `peers.color` · `groups.color`(TEXT NOT NULL DEFAULT `'agent-tile'`) 더한 뒤 있던 행 = `TILE_COLOR_RANDOM`.
   새 상대(`peers::upsert` INSERT — 다시 페어링 갱신은 색 그대로) · 새 그룹(`groups::add`) = `TILE_COLOR_RANDOM`.
3. `Peer.color` · `Group.color`(String), `peers::set_tile(…, color)` · `groups::set_color`. `Node::set_peer_tile(…, color)`.
4. API: `PATCH /api/peers/{id}` · `PATCH /api/groups/{id}` 에 `color`(여섯 밖 = 400, 준 쪽만 씀), 보기에 `color`. `PUT /api/settings` `me_color`(빈 값 = 흑연 · 여섯 밖 = 400), `GET` 에 `me_color`.

### stage 2 — 화면(web)

1. `logic/peer.js` `tileTint(color)` — 여섯 중 하나면 `"tinted {color}"`, 아니면 `""`(흑연 · 무채). `core/widgets.js` `tileFace(icon, name, color)` · `rowTile(cls, icon, name, color)` ·
   `railFace(icon, group, color)` 가 `.tile-shape` 에 붙인다. `GROUP_SVG` = 앞 사람 가운데 + 뒤 좌우 옅은 사람 둘.
2. 색을 붙이는 곳: 레일(나 = `me_color` · 상대 · 그룹) · 타일 편집 미리보기 · 개요 목록 그룹 줄 · 연결 목록 · 상세 · 그룹설정 목록 · 상세 · 검색 사람 칩 · 주고받은 메시지 줄
   (`pm-av` — 사람: 내 글 = `me_color` · 상대 글 = 그 상대) · 말풍선 상대 아바타(`.avatar` — 그 대화 상대) · 요청 의견 말풍선 · 프로필 얼굴(`.avatar-face` = `me_color`).
3. 타일 편집 폼(`index.html` `#tile_form`): '아이콘' 아래 '아이콘 색' 단추 줄(`#tile_colors` — 에이전트 상세와 같은 `.swatches` · `.swatch`), '나' 는 맨 앞 '흑연'(기본) 하나 더.
   고르면 미리보기만, 저장 때 바뀐 경우만 보낸다(`color` · `me_color`).
4. `app.css`: `:is(.tile-shape, .avatar, .avatar-face, .pm-av).tinted` = 흰 글자(`--on-dark`) · 색별 바탕 여섯(토큰), `.swatch.rail-2`(흑연 단추).

### stage 3 — 실제 화면 · 문서 · README 사진

- 디버그 빌드 + scratchpad 시험 데이터(스키마 29 로 올림) · ego-browser 두 테마 — 레일 · 머리글자 네모 색, 타일 편집 색 저장 · 다시 열기, 그룹 세 사람 그림, 콘솔 오류 0.
- README 사진 셋 다시(레일 색 · 그룹 그림이 보임). 문서: [an_core](../../manual/dev/an_core.md)(스키마 29) · [web-ui](../../manual/dev/web-ui.md) · [사용자 가이드](../../manual/user/user-guide.md) ·
  PRODUCT(#1z-15 줄) · README 1.4.0 줄 · masterPlan 환류(종결 — S4 레일 타일 · 머리글자 색).

### 바꾸는 시험(의도된 변경 — 약화 아님)

- `ux_literals.rs`: `GROUP_SVG` 그림 · `railFace` · `tileFace` · `rowTile` 부르는 꼴이 바뀐 행은 새 꼴로(근거 #1z-15 결정 5 · 7), 새 PRESENT(`tileTint` · `#tile_colors` · `.tinted` 규칙),
  `token_contrast` 에 `--on-dark` × 여섯 색 × 두 테마(4.5:1 — 머리글자 글자) 더함.
- `crates/core` 스키마 시험이 `SCHEMA_VERSION` 28 을 적었으면 29 로.

### stage 4 — 개정 ②(받은 대화 직접 쓰기 · 자동 왕복 표시 · 요청 접기)

> 근거: intent 개정 ②(결정 10 ~ 15). 브랜치 `task/1z-15-direct-post`(main `5150225` 에서). 승인: 2026-10-26 — 사용자 지시 + 질문 답 U1 ~ U4(plan 승인 질문 없이 — 구현자 해석).

1. `continue_conversation`(`POST /api/conversations/{id}/messages`): 받은 대화(`dir = in`)면 `to_agent = false` — 실행 없이 내 이력 + 상대 `post` 미러(기존 실행 없는 사람 글 경로).
   보낸 대화는 그대로 실행 요청. CLI `ask` 는 이미 받은 대화를 거절(그대로).
2. `Chat::post_with`: 받은 쪽 사람 글의 자동 왕복 거절(`set_blocked` · `block_relay`)을 뺀다 — 연 쪽 사람 글의 끔은 그대로, 받은 쪽 멈춤은 `relay(conv, false)` 그대로.
3. 웹: `relayText` — `relay_blocked` 면 `null`(문구 없음). 세부정보 '자동 왕복' 절(`#detail_relay_sec` · `.relay-box` · `.bar`) 없앰. '요청' = `js/logic/request.js`
   `REQ_FOLD = 3` · `reqFold(reqs, open)`(접혔으면 앞 3건, 넘치면 단추 "더보기" ↔ "접기"), `detail.js` `reqView{conv, open}`(대화를 옮기면 접힘).
   (결정 16) 받은 쪽이 멈춘 대화는 '대화 정보' `convInfoRows` 에 컨텍스트 뒤 '자동 왕복' 줄(`RELAY_STOPPED` — `drawInfo` 가 대화 `relay_blocked` 를 덧붙임).
4. 문서: PRODUCT 양쪽 발화 · 자동 왕복 줄, masterPlan D23 · D24 개정 · 기록, [an_agent](../../manual/dev/an_agent.md) · [developer-guide](../../manual/dev/developer-guide.md) 8.3 ·
   [web-ui](../../manual/dev/web-ui.md) 세부정보 · [사용자 가이드](../../manual/user/user-guide.md) 7.2 · 10장 · 세부정보 줄.
5. 바꾸는 시험(의도된 변경 — 결정 10 · 11): `chat_api.rs` `conversation_post_targets` 받은 대화 글 = 실행 없음(A 이력 셋 · B 답 없음),
   `relay.rs` `receiver_stop_refuses_agent_requests` 의 받은 쪽 멈춤을 사람 글 → `relay(false)` 로, 새 `receiver_note_keeps_relay`,
   `view.test.mjs` 거절 중 → `null` · 새 `reqFold`, `ux_literals` 새 행(PRESENT 4 · ABSENT 2).

### stage 5 — 개정 ③(사람 · 그룹 얼굴 = 레일 얼굴)

> 근거: intent 개정 ③(결정 17 ~ 19). 브랜치 `task/1z-15-rail-face`(main `afdacb4` 에서). 승인: 2026-10-26 — 사용자 지시 + 질문 답 U1 · U2(plan 승인 질문 없이 — 구현자 해석).

1. `js/core/widgets.js`: `tileFace`(머리글자) 삭제, `rowTile(cls, icon, color, group)` = `railFace` 감쌈, `railFace(icon, group, color, me)` — `me` 면 `meTint`(흑연 `me-tile`).
   `js/logic/peer.js` `meTint` 새로, `initialOf` 삭제(쓰는 곳 없음).
2. 호출: 개요 · 검색 · 연결 · 그룹설정(`group = true`) 줄 타일, 주고받은 메시지 머리 · 줄(`peerRowEls(rows, peer)` — 나 = `me_icon` · `me_color`), 말풍선 · 요청 의견
   (`conv.js` `peerFace` — 공유 대화 원 상대 글 = 무채 사람), 프로필(`shell.js`), 레일 · 타일 편집 미리보기 '나' = `me = true`. 다시 그림 서명에 그림 더함(`messagesSig` · 주고받은 메시지 `me_icon`).
3. `app.css`: `.tile-shape.me-tile`(`.rail-btn.me .tile-shape` 대신), `.tile-shape svg` 70%, `.avatar` · `.pm-av` 그림, `.tinted` 는 `.tile-shape` 에만.
4. 시험: `view.test.mjs` `initial_of` 삭제 · `me_tint` 새로, `ux_literals` PRESENT 바꿈 · ABSENT `initialOf` · `tileFace` · `peerTint`.
5. 문서: PRODUCT(Graphite · 타일 색 줄) · masterPlan S4 · 기록, web-ui · 사용자 가이드, README 사진 셋 다시.

## 평가 항목

- [ ] E1: A5 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E2: A5 — clippy 경고 0
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: pending
- [ ] E3: A3·A4 — 웹UI 로직 시험(`tileTint`)
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# fail 0/
  EVIDENCE: pending
- [ ] E4: A3·A4 — 화면 원본 literal(색 단추 · 색 붙임 · 그룹 그림) · 토큰 대비 · `:root` 밖 리터럴 0
  CHECK: cargo test -p coworkagent-app --test ux_literals --test css_tokens
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E5: A1·A2 — 저장(스키마 29 · 무작위 색 · 색 바꾸기 · `me_color`) 시험
  CHECK: cargo test -p an-core tile_color
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E6: A5 — 버전 1.4.0 그대로(크레이트 넷)
  CHECK: python -c "import pathlib as P;v=[P.Path('crates/%s/Cargo.toml'%c).read_text(encoding='utf-8') for c in ('core','net','agent','app')];print('crates=%d v140=%d'%(len(v),sum(t.split('version = ',1)[1][1:6]=='1.4.0' for t in v)))"
  EXPECT: crates=4 v140=4
  EVIDENCE: pending
- [ ] E7: A1·A2·A3·A4·A5 — 실제 화면(디버그 빌드 · ego-browser) 두 테마 — 색 저장 · 다시 열기 · 머리글자 네모 색 · 그룹 세 사람, 콘솔 오류 0, README 사진 셋
  EVIDENCE: pending
- [ ] E8: A5 — 문서 규약 · 크레이트 경계 · 코드 정리 · 매뉴얼 반영
  CHECK: python tools/check_ddd_docs.py && python tools/check_crates.py && python tools/check_code_tidy.py && python tools/check_readme.py && python -c "import pathlib as P;r=lambda f:P.Path(f).read_text(encoding='utf-8');print('core=%d web=%d guide=%d product=%d'%(r('docs/manual/dev/an_core.md').count('스키마 29'),r('docs/manual/dev/web-ui.md').count('tileTint'),r('docs/manual/user/user-guide.md').count('아이콘 색'),r('PRODUCT.md').count('#1z-15')))"
  EXPECT: /core=[1-9][0-9]* web=[1-9][0-9]* guide=[1-9][0-9]* product=[1-9][0-9]*/
  EVIDENCE: pending

- [ ] E9: A6 — 받은 대화에 쓴 글 = 실행 없음 · 상대 창에도(로컬 API)
  CHECK: cargo test -p coworkagent-app --test chat_api conversation_post_targets
  EXPECT: /test result: ok\. 1 passed; 0 failed/
  EVIDENCE: pending
- [ ] E10: A6 — 받은 쪽 사람 글은 자동 왕복을 멈추지 않고, 받은 쪽 "멈춤" 은 그대로 거절
  CHECK: cargo test -p an-agent --test relay receiver_
  EXPECT: /test result: ok\. 2 passed; 0 failed/
  EVIDENCE: pending
- [ ] E11: A6·A7 — 실제 화면(디버그 빌드 · ego-browser) 두 테마 — 받은 대화 직접 쓰기(실행 없음) · 거절 중 문구 · 자동 왕복 절 없음 · 요청 3건 + 더보기 · 접기 · 대화 옮기면 접힘, 콘솔 오류 0
  EVIDENCE: pending
- [ ] E12: A6·A7 — 개정 ② 매뉴얼 반영
  CHECK: python -c "import pathlib as P;r=lambda f:P.Path(f).read_text(encoding='utf-8');print('agent=%d web=%d guide=%d product=%d'%(r('docs/manual/dev/an_agent.md').count('#1z-15'),r('docs/manual/dev/web-ui.md').count('reqFold'),r('docs/manual/user/user-guide.md').count('더보기'),r('PRODUCT.md').count('D23 개정 · #1z-15')))"
  EXPECT: /agent=[1-9][0-9]* web=[1-9][0-9]* guide=[1-9][0-9]* product=[1-9][0-9]*/
  EVIDENCE: pending

- [ ] E13: A8 — 웹 원본에 머리글자 네모 코드 없음(`initialOf` · `tileFace` · `peerTint`)
  CHECK: python -c "import pathlib as P;fs=list(P.Path('crates/app/web/js').rglob('*.js'));print('files=%d hits=%d'%(len(fs),sum(sum(f.read_text(encoding='utf-8').count(w) for w in ('initialOf','tileFace','peerTint')) for f in fs)))"
  EXPECT: /files=[1-9][0-9]* hits=0/
  EVIDENCE: pending
- [ ] E14: A8 — 실제 화면(디버그 빌드 · ego-browser) 두 테마 — 프로필 · 레일 · 말풍선 · 요청 의견 · 주고받은 메시지 · 연결 · 그룹설정 얼굴 = 레일 얼굴(그림 있으면 그림), '나' 흑연, 콘솔 오류 0, README 사진 셋
  EVIDENCE: pending
- [ ] E15: A8 — 개정 ③ 문서 반영
  CHECK: python -c "import pathlib as P;r=lambda f:P.Path(f).read_text(encoding='utf-8');print('web=%d guide=%d product=%d master=%d'%(r('docs/manual/dev/web-ui.md').count('개정 ③'),r('docs/manual/user/user-guide.md').count('레일과 같은 얼굴'),r('PRODUCT.md').count('(개정 ③)'),r('docs/masterPlan.md').count('#1z-15(개정 ③)')))"
  EXPECT: /web=[1-9][0-9]* guide=[1-9][0-9]* product=[1-9][0-9]* master=[1-9][0-9]*/
  EVIDENCE: pending

## 개정 기록

- ① 2026-10-26(평가 첫 실행 E1 FAIL) — '바꾸는 시험' 에 `crates/core/tests/store.rs` `schema17_peer_ids_never_reused` 의 peers 열 수 19 → 20 더함:
  그 시험은 지금 정의(스키마 29 `color` 포함)를 16 으로 되돌려 쓰므로 열 수가 하나 는다(검사 뜻 — 열 전부 · 순서 그대로 — 은 같음).
- ② 2026-10-26(종결 뒤 사용자 지시 — "마지막 task 에 포함") — intent 개정 ②(결정 10 ~ 16), stage 4 · E9 ~ E12 더함. E1 ~ E8 은 그대로 다시 잰다.
  결정 16(report-4 답)은 E11 측정 뒤 더했다 — E3(`convInfoRows` 시험) · E4(literal 2행)가 잰다, 실제 화면은 E11 에 덧붙여 다시 쟀다.
- ③ 2026-10-26(사용자 지시 "레일 아이콘을 다른 대화방에서도 동일하게 유지. 해당 기능까지 적용하고 로컬 커밋, push 진행.") — intent 개정 ③(결정 17 ~ 19), stage 5 · E13 ~ E15 더함. E1 ~ E12 는 그대로 다시 잰다.
