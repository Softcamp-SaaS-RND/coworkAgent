# stage 1 — 저장(an_core · an_net · an_app)

## 목표

plan stage 1 — 상대 · 그룹 타일 색 열(스키마 29, 처음 무작위 저장 — 결정 2 · 9) · '나' 색 설정(`me_color`, 빈 값 = 흑연 — 결정 4) · API.

## 산출 파일

- `crates/core/src/run_opts.rs` — `ME_COLOR_KEY` · `TILE_COLOR_RANDOM`(SQL 식) · `clean_tile_color` · `me_tile` → `MeTile`(표시 이름, 아이콘, 색).
- `crates/core/src/db.rs` — 스키마 29(`peers.color` · `groups.color` 더한 뒤 있던 행 무작위), 스키마 18 옛 그룹 옮기기는 그 때 표에 직접 INSERT(`groups::add` 는 지금 표 가정).
- `crates/core/src/peers.rs` — `Peer.color` · `upsert` 새 행만 무작위(다시 hello 는 그대로) · `set_color`. `crates/core/src/groups.rs` — `Group.color` · `add` 무작위 · `set_color`.
- `crates/net/src/node.rs` — `Node::set_peer_color`. `crates/app/src/peers_api.rs` — 보기 `color` · PATCH `color`(여섯 밖 400, 준 쪽만).
  `crates/app/src/groups_api.rs` — 보기 · PATCH `color`. `crates/app/src/notify.rs` — 설정 `me_color`(빈 값 = 흑연 · 여섯 밖 400).
- 시험: `crates/core/tests/paths.rs` `schema29_tile_color`(새로) · `SCHEMA_VERSION` 29 · `schema28_conv_info` 의 28 → `SCHEMA_VERSION`, `crates/app/tests/ux_api.rs` `peer_and_me_tile_color`(새로) · `groups_api.rs` 이름 바꾸기 시험에 색.

## 설계 결정

- 무작위는 SQL `abs(random()) % 6` — 새 의존성 없음, 같은 식을 스키마 29 · 새 상대 · 새 그룹이 쓴다.
- 상대 색은 `peers::set_tile` 을 넓히지 않고 `set_color` 따로(부르는 곳 셋 그대로). PATCH 에 색과 타일을 함께 주면 색 → 타일 차례(한 트랜잭션 아님 — 기존 meta · tile 과 같음, 다 검증한 뒤 씀).

## 테스트 결과

- RED — `cargo test -p an-core --test paths` 컴파일 실패(`color` · `set_color` · `clean_tile_color` 없음).
- GREEN — `--test paths` ok 19, `coworkagent-app --test ux_api --test groups_api --test settings_api` ok 50 · 15 · 3.

## 다음 단계

stage 2 — 화면(색 단추 · 색 붙임 · 그룹 세 사람 그림).
