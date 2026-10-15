# stage2 — 타일 메뉴 · 이미지 지우기 · 그룹 이름 · 그림 · 그룹 삭제 · 타일 색 id 기준

> 근거: [plan](../plan.md) Stage 2. 사용자 검증은 D28 — 19.prototype-acceptance 에서 일괄. stage 1 과 같은 커밋(같은 파일을 이어 고쳐 나누지 않았다).

## 목표

레일 타일('나' · 상대 · 그룹)마다 우클릭 · 메뉴 키 · 사이드 머리 ⋯ 로 여는 타일 메뉴(해석 9), 그룹 이름 · 그림 편집, 그룹 삭제(확인 한 번), 타일 색
상대 id 기준(해석 11). 대상 gate: E1 · E2 · E5 · E6.

## 산출 파일

- `crates/app/web/app.js` — `currentTile` · `tileNow` · `openTileEdit`(대상 = 고른 타일, 그룹이면 라벨 "그룹 이름" · 40자) · 저장(그룹 = `PATCH
  /api/groups/{id}`, 이름 미리 거르기) · `tileMenuEvent`(우클릭 · 메뉴 키) · `selectTile` · `openTileMenu`("타일 편집" · "이미지 지우기" · "그룹 삭제" →
  상자 안 확인) · `tileColor` · `groupColor` = `colorOf(id)` · 바깥 누름 판정에서 떨어져 나간 요소 무시.
- `crates/app/web/view.js` — `colorOf(id)`.
- `crates/app/web/index.html` — 사이드 머리 `#tile_more_btn`(⋯) · `#tile_alias_label`.
- 시험 — `view.test.mjs` `color_of` · `ux_literals.rs` `rail_group_literals` 문구 7개 추가.

## 설계 결정

- **편집 폼은 그대로 하나** — 타일 메뉴의 "타일 편집" 은 그 타일을 고른 뒤 사이드 머리의 지금 폼을 연다(폼이 고른 타일 머리에 있다 — 새 폼을 만들지
  않음).
- **메뉴 키** — 브라우저가 메뉴 키 · Shift + F10 에 같은 `contextmenu` 를 낸다 — 따로 키 처리를 두지 않았다(mock 확인 PASS).
- **그룹 삭제 확인** = 떠 있는 상자 안 두 단추(브라우저 `confirm` 대신 — 화면 문구를 통제). 지운 그룹을 고르고 있었으면 '나' 로.
- **타일 색** — `colorOf(id)` 로 상대 · 그룹 같은 다섯 색. 업데이트 직후 기존 상대 색이 한 번 바뀔 수 있다(plan 해석 11, 사용자 승인).

## 테스트 결과

```text
$ node --test --test-reporter=tap "crates/app/web/*.test.mjs" → # pass 64 · # fail 0
$ cargo test -p coworkagent-app --test ux_literals             → test result: ok. 13 passed; 0 failed
$ cargo test --workspace                                       → EXIT 0 (test result: ok 69 묶음, 실패 0)
$ cargo clippy --workspace --all-targets -- -D warnings        → Finished
```

mock 화면 확인(scratchpad `mock3.py` · `s2.mjs`, 가상 데이터) 15/15 PASS · HTTP 오류 0: 그룹 타일 메뉴(타일 편집 · 그룹 삭제) · 편집 폼 "그룹 이름" ·
같은 이름 거절 · 이름 바꿈 · 그림 넣기 · ⋯ 메뉴에 "이미지 지우기" · 그림 지움 · Shift + F10 메뉴 · 상대 · '나' 메뉴 · 상대 색 id 기준 · 삭제 확인 문구 ·
삭제 뒤 '나' · 대화 수 그대로. 찾아 고친 것 1: "그룹 삭제" 를 누르면 상자 내용이 바뀌며 누른 요소가 떨어져 나가 바깥 누름으로 판정 → 상자가 닫혔다.

## 다음 단계

stage 3 — 드래그 앤 드롭(상대 · 그룹 타일 · 대화) · 그룹 타일에 놓아 넣기 · Alt + ↑ / ↓ · 이 PC 화면 확인(E7) · 문서.
