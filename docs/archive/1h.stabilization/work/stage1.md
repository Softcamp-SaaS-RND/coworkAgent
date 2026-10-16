# stage1 — 화면 5건: 타일 글자 · 겹친 카드 · 메뉴 세 절 · 개요 · 북마크 · 프로필 알약

> 근거: [plan](../plan.md) Stage 1. 사용자 검증은 D28 — 19.prototype-acceptance 에서 일괄.

## 목표

해석 1 ~ 6 — 레일 타일 글자 폭 예산 · 그룹 겹친 카드 · 사이드바 메뉴 세 절(구분선) · "대화" → "개요" · "저장된 항목" → "북마크 항목"(메시지 작업
"북마크") · 우상단 프로필 알약. 대상 gate: E1 ~ E4.

## 산출 파일

- `crates/app/web/view.js` — `tileText(name, budget = 4)`(넓이 1 · 2 — 한글 · 한자 · 가나 · 그림 문자 · 국기 2) · `sideNav` 세 절 + `{ sep: true }` ·
  `messageActions` "북마크" / "북마크 해제".
- `crates/app/web/app.js` — `tileFace`(`<span class="tile-text">`, 세 글자 이상 `long`) · `renderAvatar`(아이콘 또는 머리글자(예산 2) + 이름 · `title`) ·
  `renderSideNav` 구분선(`li.nav-sep[role=separator]`) · `#home_title` "개요" · 북마크 상태 문구 · 빈 목록 문구.
- `crates/app/web/index.html` — `#saved` 제목 · 안내 · `#saved_btn` · 실행 상세 절 "북마크 항목" · `#home_title` "개요".
- `crates/app/web/app.css` — 긴 타일 글자 · `.rail-btn.group` 겹친 카드(`box-shadow` 두 겹) · `.nav-sep` · 알약 단추 · `.avatar-name`(12em 말줄임).
- 시험 — `view.test.mjs` 76 → 82(`tile_text_latin` · `tile_text_wide` · `tile_text_budget` · `side_nav_sections` · `side_nav_counts` ·
  `message_actions_bookmark`, 기존 `tile_text` · `side_nav` · `side_nav_group` · `message_actions` 고침) · `ux_literals.rs` `stabilization_literals`(새) ·
  `workspace_labels` · `rail_scope_literals` 고침.

## 설계 결정

- **국기도 넓이 2** — plan 해석 1 은 한글 · 한자 · 가나 · 그림 문자였으나 국기(지역 표시 글자 둘)는 `Extended_Pictographic` 이 아니라 넓이 1 로 세어져 넘쳤다
  — `\p{Regional_Indicator}` 를 더했다.
- **프로필 머리글자는 예산 2** — 이름이 옆에 다 보이므로 머리글자는 한 자(한글) · 두 자(영문)만. 머리글자는 `aria-hidden`(단추 이름 = 표시 이름).
- **겹친 카드 = `box-shadow`** — 가상 요소는 타일 안 그림 · 배지 · 접속 점과 쌓임 순서를 다퉈서, 그림자 두 겹(레일 바탕 · 흐린 글자색)으로 테두리 카드 한
  장을 그렸다. 선택 테두리(`outline`)와 겹치지 않는다.

## 검증

- `node --test` — pass 82 · fail 0. `cargo test -p coworkagent-app --test ux_literals` — 14 passed.
- mock(가상 데이터) 화면 — 레일 "buil"(작은 글씨) · 그룹 두 타일 겹친 카드 · '나' 메뉴 개요 | 내 활동 · 요청 · 주고받은 글 | 북마크 항목 · 에이전트 ·
  프로필 "박 박지훈" · 긴 이름(17자) 단추 폭 208px · 말줄임 · `title` 전체 이름.
