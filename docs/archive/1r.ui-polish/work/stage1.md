# stage1 — 새 대화 · 메뉴 아이콘 · 연필 · 새 에이전트 줄 · ? · 초대 생성/수락 · 들여쓰기

## 목표

plan Stage 1 — intent 결정 1 ~ 9 · 11 을 웹UI 에 반영(plan 해석 1 ~ 9). API · DB · 통신 그대로.

## 산출 파일

- `crates/app/web/index.html` — 새 대화 라벨 아이콘 · "내 응답 에이전트" · 하위 줄 "최대 응답 횟수"(`#chat_relay_cap_label.sub-label` · 칸 처음 `hidden`) · 요청 탭 아이콘 ·
  `#edit_btn` 연필 · 알림 "자동 왕복" ?(`tip_relay`, `.tip.up`) · `#agent_new` = `manage-item new-agent` 줄(빈 타일 + · "새 에이전트" · 취소 `#agent_new_cancel`).
- `crates/app/web/view.js` — `settingsTree` · `newMenu` "초대 생성" `press: "window_btn"` · `profileMenu` `icon` · `ACT_KINDS` `icon`.
- `crates/app/web/app.js` — `renderTree` · 새로 만들기 클릭 `press` · 라우터 `showAccept(r.view === "settings" && r.focus === "invite_accept")` · 레일 + =
  `goSettings("invite_accept", "invite_text")` · `syncRelayCap` · 요청 거절 문구 · `manageItem` 연필 · `renderProfileMenu` · `renderActTabs` 아이콘 · 등록 성공 · 취소 = 닫기.
- `crates/app/web/app.css` — `.ico-gear` · `.ico-title` · `.peer-fold > :not(summary)` 들여쓰기 + 안내선(≤ 720px `--sp-3`) · 옛 `.req-box .peer-fold > summary` 들여쓰기 지움 ·
  `#chat_form` 라벨 · `.sub-label` · `.manage-btn[aria-expanded="true"]` · `.new-tile` · `.new-agent` · `.tabs a` 간격 · `.edit-toggle` · `.tip.up`.
- `crates/app/web/view.test.mjs` — `ui_polish_menu_icons` · `ui_polish_invite_press` · `ui_polish_new_chat`(먼저 3건 모두 실패 확인 → 구현 뒤 통과).
- `crates/app/tests/ux_literals.rs` — 옛 잠금 9행을 새 자리로(편집 · 관리 · 레일 + · 에이전트 등록 · 프로필 설정 · 사용자 정보 · 내 활동 탭 셋), 1r PRESENT 22 · ABSENT 6.

## 설계 결정

- 들여쓰기는 `.peer-fold` 공통 규칙(plan 해석 1) — 내 활동 · 북마크 · 즐겨찾기 상대별 절도 같은 모양.
- 알림 '?' 설명은 알림 카드가 화면 맨 아래라 아래로 열면 잘려(mock 사진) 위로 연다(`.tip.up`).
- '시작과 끝' 설명의 끝 사유는 `notify.rs` · `agent/src/chat.rs`(`RelayEnd` — 상한 · 실패 · 상대가 멈춤) 기준 "상한 도달 · 실패 · 상대가 멈춤 등".

## 검증

- `node --test --test-reporter=tap "crates/app/web/*.test.mjs"` → `# pass 124 / # fail 0`.
- `cargo test --workspace` → `test result: ok` 75줄 · 실패 0, `cargo clippy --workspace --all-targets -- -D warnings` → `Finished`.
- `cargo test -p coworkagent-app --test ux_literals --test css_tokens` → ok 5 · ok 3.
