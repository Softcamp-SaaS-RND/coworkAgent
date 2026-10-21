# stage 2 — 레일 타일 순서 포인터 끌기 · 그룹설정(시안 A)

## 목표

intent 개정(결정 9 ~ 11, A11 ~ A13) — 레일 사용자 · 그룹 타일 끌어 놓아 순서 바꾸기, 설정 "연결 목록" → "그룹설정", 그룹설정 두 칸(목록 + 상세).
시안 2개(Design 캔버스 "그룹설정 A안 · B안") → 사용자 "A안 목록+상세".

## 산출 파일

- `crates/app/web/view.js` — `DRAG_SLOP` · `pastSlop`, `groupSettingsList` · `groupSettingsPick`, 트리 라벨 "그룹설정".
- `crates/app/web/app.js` — `tileGrab`(레일 타일 포인터 끌기) · 타일 `draggable` 제거 · 레일 HTML 끌기 처리 · `dragging` 삭제, `renderPeerList` 두 칸 · `groupSettingsListEl` ·
  `userDetail`(옛 `userListSection` 의 한 사람), `state.gsPick`, "+ 그룹 추가" 뒤 그 그룹 고름, "연결 목록" 문구 · 주석 → "그룹설정".
- `crates/app/web/index.html` — 카드 제목 · 안내 2곳 · 검색 라벨 · placeholder. `crates/app/src/notify.rs` — 새 연결 알림 문구.
- `crates/app/web/app.css` — `.rail-ghost` · 타일 `touch-action: none`, `.gs-*`.
- 시험 — `view.test.mjs` 새 3(`past_slop` · `group_settings_list` · `group_settings_pick`), `settings_tree` 라벨, `ux_literals.rs` PRESENT 6 · ABSENT 2.
- 문서 — 사용자 가이드(4.1 · 5장) · 웹UI 매뉴얼(설정 · 레일 그룹 · 순서) · an_app · 개발자 가이드 · README.

## 설계 결정

- 끌기 시작 = 5px(`DRAG_SLOP`) — 그 미만이면 click 이 그대로 타일을 고른다. 끈 뒤 click 은 캡처 단계에서 한 번 먹고, 다음 틱에 먹는 처리를 지운다(click 이 안 오면 다음 누름을 먹지 않게).
- 놓을 자리 = `dropKind` "move" 만(사용자 ↔ 사용자, 그룹 ↔ 그룹 — '나' · 묶음 사이는 안 됨). 대화 줄을 그룹 타일에 넣는 `convGrab` 은 그대로(타일이 `data-drag` 를 그대로 가진다).
- 좁은 화면(가로 레일)은 앞 / 뒤를 X 로 가른다.
- 그룹설정 검색은 시안 placeholder("그룹 · 이름 · PC · 대화 제목")대로 그룹 이름 · 사용자 이름/별칭/PC/지문 · 대화 제목. 그룹 상세의 대화 줄 거르기는 전처럼 보이는 사용자 기준.
- 왼쪽 그룹 줄이 대화 놓는 자리(`data-peer-section`) — 기존 HTML 끌어 놓기 처리(`peer_groups` dragover · drop)를 그대로 쓴다(설정 대화 줄 HTML 끌기는 제외 — intent).
- 사용자 상세는 옛 관리 줄 + 접이식 "대화 n" 을 그대로(연필로 상세 여닫기).
- `USER_LIST` · `settingsGroupSections` 끝 절은 남김(상세가 그룹 절을 찾는 데 씀 — 시험 그대로).

## 테스트 결과

```text
node --test --test-reporter=tap --test-name-pattern="past_slop|group_settings_list|group_settings_pick" crates/app/web/view.test.mjs → # pass 3 / # fail 0
node --test --test-reporter=tap "crates/app/web/*.test.mjs"   → # pass 158 / # fail 0
cargo test -p coworkagent-app --test css_tokens --test ux_literals → ok 3 · ok 5
cargo test --workspace                                        → 78 바이너리 481 passed · 0 failed
cargo clippy --workspace --all-targets -- -D warnings         → Finished(경고 0)
```

RED 확인: `settings_tree`(옛 라벨 "연결 목록") 실패 → 새 라벨로 갱신, `css_tokens` `.gs-rows { gap: 2px }` 리터럴 실패 → `--sp-1`.

## 다음 단계

- gate 원장 → 적대적 리뷰 라운드 1/2(1z 전체, base `2b68dbc`). E11 실 화면(레일 끌기 · 그룹설정 두 칸 포함)은 앱 재실행 사용자 확인 뒤.
