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

## 개정 기록

- ① 2026-10-26(평가 첫 실행 E1 FAIL) — '바꾸는 시험' 에 `crates/core/tests/store.rs` `schema17_peer_ids_never_reused` 의 peers 열 수 19 → 20 더함:
  그 시험은 지금 정의(스키마 29 `color` 포함)를 16 으로 되돌려 쓰므로 열 수가 하나 는다(검사 뜻 — 열 전부 · 순서 그대로 — 은 같음).
