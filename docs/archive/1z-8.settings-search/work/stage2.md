# stage 2 · 3 — 설정 틀 · 디버깅 · 연결 / 그룹설정 / 에이전트 목록 · 상세(시안 B)

## 목표

plan "설계 · stage 2 · 3" — 설정 메뉴 8개 · 고른 화면만 · 첫 화면 연결(A5), 스위치(A6), 목록 · 상세 2단 · 닫기 · 초안(A7), 2열 타일 · 좁은 창(A8), 디버깅(A9). (plan 개정 기록 ① — 한 커밋)

## 산출 파일

- app `server.rs` — 보호 경로 `GET /api/debug` · `GET /api/debug/log`, `LOG_TAIL_LINES`(200) · `LOG_TAIL_BYTES`(64 KiB) · `tail_lines`. 새 시험 `tests/debug_api.rs`(5).
- web `view.js` — `settingsTree` 평평한 8개 · `SETTINGS_ALIAS` · `route` 설정 갈래 `{page, sub}` · `newMenu` 에이전트 등록 = `#settings/agent_new` · `fileSize` · `logLines` · `diagText`.
  지움: `USER_LIST` · `settingsGroupSections` · `convCopy`.
- web `index.html` — 설정 = 메뉴 + 화면 8개(`section.set-page`), 연결 · 그룹설정 · 에이전트 = `.md`(목록 · 상세 — 초대 생성 · 초대 수락 · 새 에이전트는 상세 칸 안 정적 폼), 실행 · 화면 · 시작 · 알림 = `.set-tiles`,
  디버깅 화면, 설정 체크박스 전부 `role="switch"`.
- web `app.js` — `pick` · `setPick` · `closePick` · `mdRow` · `mdHead` · `mdFoot` · `renderConnPage` · `renderGroupPage` · `groupDetail` · `convLine` · `convAct` · `userDetail`(상세 머리 · 몸 · 아래 줄) ·
  `renderAgentList` · `agentDetail` · `agentDrafts` · `showAccept` · `showAgentNew` · `openSettings`(고른 화면 · `aria-current`) · `editAgent`(고름 + 줄 초점) · 디버깅(`loadDebug` · `renderDebugLog` · `debugCopy`).
  지움: `manageItem` · `gsToggle` · `gsGroupsEl` · `gsUsersEl` · `settingsConvRow` · `gsJoin` · `gsDragging` · `kvTable` · `state.open` · `convGrab` 의 `join` 갈래 · 트리 `press` · 설정 `scrollIntoView`.
- web `app.css` — `.set-page` · `.set-head` · `.set-tiles` · `.set-tile` · `.md*` · 디버깅 · `#settings_tree a[aria-current="page"]` · `.ico-bug` · 960px 아래 1열, `.settings-body` 폭 제한 없앰,
  옛 설정 규칙(`.card` · `.sub-head` · `.pair-btns` · `.manage-*` · `.gs-*` 카드 · 표 · `.settings-conv*` · `.item-body`) 지움.
- 시험: `view.test.mjs`(`settings_tree` · 새 `settings_route` · `new_menu` · `profile_menu` · `menu_icons` · `ui_polish_invite_press` · `shell_confirm` · `screen_settings_1x` 개정,
  `settings_group_sections` · `conv_copy` 지움, 새 `debug_text`), `ux_literals.rs`(바뀐 행 #1z-8 출처로, 없어야 할 행 추가 — 옛 연필 펼침 · 카드 · 끌기 · 사용자 목록 절).
- 문서: web-ui · an_app · an_core · 사용자 가이드 · README(1.3.4).

## 설계 결정

- 정적 폼(초대 생성 · 초대 수락 · 새 에이전트)은 상세 칸 안에 늘 두고 `hidden` 만 바꾼다 — 기존 처리기가 id 로 찾는다(조언 반영).
- 닫기 초점: 줄을 고른 상세 → 그 줄, 초대 생성 · 초대 수락 · 새 에이전트 → 머리 단추.
- 다시 그리기: 화면 단위 서명(`needsRender` — 고름 포함), 입력 중이면 배경 새로 고침은 미룸. 사용자 · 에이전트 상세의 저장 전 값은 `gsDrafts` · `agentDrafts`.
- 그룹설정 = 넣기 · 빼기 단추(설정 화면 끌기 지움 — 레일 · 사이드바 끌기 `convGrab` 는 그대로).
- 디버깅 로그 시각 = 줄 앞 유닉스 초를 이 PC 시각 "월-일 시:분:초" 로. 로그 줄은 그대로(자기 지문이 적힌 시작 줄 포함 — intent A9 명확화).
- plan 개정 기록 ② ~ ⑤(에이전트 검색 없음 · 화면 머리 설명 없음 · 검색 스위치 값 · 클래스 이름).

## 테스트 결과

- `node --test --test-reporter=tap "crates/app/web/*.test.mjs"` → `# pass 169` · `# fail 0`.
- `cargo test -p coworkagent-app --test debug_api` → `test result: ok. 5 passed; 0 failed`.
- `cargo test -p coworkagent-app --test ux_literals --test css_tokens` → 5 · 3 passed.
- 실제 화면(ego-browser, 디버그 빌드 · `COWORKAGENT_DATA_DIR` = scratchpad `ui1z8\data`, 상대 3 · 대화 5(나감 1 · 상대가 나감 1) · 그룹 2 시드) — plan E8 EVIDENCE 에 관측.
  발견 · 고침 2건: 그룹 상세 아래 단추 줄이 칸 중간에 뜸(몸 `flex: none`) → 몸이 칸을 채우게, 검색 스위치가 검색 칸 `padding` 을 물려받아 넓어짐(96px) → 스위치 `padding: 0; border: 0`(36 × 20).
  디버깅 크기 글자 줄바꿈 → `nowrap`.

## 다음 단계

gate 판정(`python tools/check_gates.py docs/tasks/1z-8.settings-search`) → eval.md → 적대적 평가(codex-peer-review, 2라운드 상한) → 종결.
