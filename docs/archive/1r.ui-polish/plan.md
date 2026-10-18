# plan — 1r.ui-polish

> 근거: [intent](intent.md)(승인 2026-10-18 — 결정 1 ~ 11). **구현자는 이 문서만 보고 일한다.** D·S 는 [masterPlan](../../masterPlan.md).
> 기준 커밋: `a7ab363`(task/1r-ui-polish — intent 승인 · 시안). plan 승인: 2026-10-18(사용자 "시안 확인. 개발 진행해." — 시안 = 결정 10 의 캔버스).
> 아래 "intent 대비 해석" 은 그 지시 아래 구현자가 정한 것 — 보고에서 드러내고, 사용자가 뒤집으면 plan 개정.

## 목표

새 대화의 "자동 왕복" 칸을 "내 응답 에이전트" + 하위 줄 "최대 응답 횟수" 로 바꾸고 줄마다 아이콘을 붙인다. 접이식 목록의 하위 절 · 행을 단계마다 들여 쓴다.
설정 알림 "자동 왕복" 에 ? 설명을 달고, '초대 생성' 메뉴가 초대를 만들고 '초대 수락' 칸이 다른 곳으로 가면 접히게 고친다. 에이전트 등록 칸을 목록 줄 모양으로,
"관리" · 대화 "편집" 을 연필로 통일하고 아이콘 없는 메뉴(프로필 메뉴 · 내 활동 · 요청 탭)에 아이콘을 넣는다. 웹UI 만 — API · DB · 통신 그대로.

### 성공 기준(intent 사본)

- A1: 새 대화 폼에 "자동 왕복" 낱말이 없고 "내 응답 에이전트" 줄이 있다. "최대 응답 횟수"(기본 10, 1 ~ 50)는 그 아래 한 단 들여쓴 줄로, "안 함" 이면 숨고 에이전트를 고르면 보이며 보낼 때 그 값이 `relay_cap` 으로 간다.
- A2: 새 대화 폼의 줄 라벨마다 아이콘이 있다.
- A3: 개요 · 요청 개요 · 주고받은 메시지에서 하위 절 머리와 행이 상위보다 단계마다 한 단씩 들여쓰이고 하위 절 왼쪽에 안내선이 있다(좁은 화면은 더 좁게).
- A4: 설정 알림 "자동 왕복" 옆 ? 에 마우스를 올리거나 초점을 주면 기능 설명과 세 옵션(시작과 끝 · 메시지마다 · 알리지 않음) 설명이 보인다.
- A5: 설정 트리 · 새로 만들기의 "초대 생성" 을 누르면 초대가 만들어진다(초대 상자 · 안내 문구). 같은 해시로 새로 고침 · 뒤로 가기로 와서는 만들지 않는다.
- A6: "초대 수락" 칸은 메뉴 "초대 수락" · 레일 "+" · `accept_btn` 으로 펼치고, 다른 설정 항목 · 다른 화면으로 가면 접힌다.
- A7: "에이전트 등록" 을 누르면 목록 마지막 줄 아래에 목록 줄 모양의 "새 에이전트" 줄(펼친 상세 = 이름 · 폴더 · 도구 · 추가 지시 · 등록 · 취소)이 나오고, 등록하면 닫히고 목록에 그 줄이 생긴다.
- A8: 연결 목록 · 내 에이전트 줄의 "관리" 가 연필 아이콘 단추(이름 "{이름} 관리", 펼침 `aria-expanded`)이고, 대화 "편집" 단추는 연필 + "편집" 이다.
- A9: 프로필 메뉴(사용자 정보 · 프로필 편집 · 알림 일시 중지 · 설정) · 내 활동 탭 · 요청 탭의 항목마다 아이콘이 있다.
- A10: 기존 검증 명령(cargo test · node 테스트 · 문서 · gate 체커)이 green 이다.

## 코드 사실(실측, 기준 `a7ab363`)

- 새 대화: `index.html` `#chat_form`(`form.grid` — 라벨 · 칸 두 열, ≤ 720px 한 열). `chat_relay_agent` 옵션은 `app.js` `loadAgents` 가 "안 함" + 내 에이전트로 채우고,
  `chat_relay_cap` 처음 값은 `fillPrefs` 가 설정 `relay_cap_default`(처음 10)로. 보내기(`chat_form` submit)는 `chat_relay_agent` 가 있으면 `relay_agent` · `relay_cap` 을 싣고,
  요청이면 "요청으로는 자동 왕복을 열 수 없습니다".
- 아이콘: `app.js` `ico(name)` = `span.ico.ico-{name}`(CSS 마스크 · `app.css` `.ico-{name} { --ico: url(data:…) }`). 레일 설정 단추는 인라인 SVG(톱니).
- 접이식: `foldEl` → `details.peer-fold`(머리 `summary` + 몸). 쓰는 곳 = 개요 `home:{상대}` · `grouped()`(내 활동 · 북마크 · 즐겨찾기 등 '나' 목록) · 요청 개요 `req-box` ·
  주고받은 메시지 `peer-msgs`. 들여쓰기 = `.req-box .peer-fold > summary { padding-left: var(--sp-6) }` 하나뿐.
- 설정 트리 `renderTree` — 누르면 `goSettings(n.id)` 만. 새로 만들기 메뉴 — `go(i.href)` + `focusAfter`. 초대 만들기는 `window_btn` click(`POST /api/pairing-window`).
- 라우터 `render()` 끝 — `if (r.view === "settings" && r.focus === "invite_accept") showAccept(true)` 만(닫는 곳 없음). 레일 "+" = `showAccept(true)` + `goSettings("conn", "invite_text")`.
  `render()` 는 hashchange 와 같은 해시 `go()` 에서만 돈다(5초 새로 고침은 `render()` 를 부르지 않는다).
- 에이전트: `#agent_new`(맨 `form.grid`)가 `#agent_groups` 뒤. 등록 성공 = 폼 비움 + `loadAgents`(칸은 열린 채).
- `manageItem(key, label, head, ...body)` — 글자 단추 "관리"(`small manage-btn`, `aria-label` "{이름} 관리", `aria-expanded`). 연결 목록 · 내 에이전트가 쓴다.
- 대화 "편집" = `index.html` `#edit_btn.edit-toggle`(글자만, `aria-pressed`).
- 프로필 메뉴 = `view.js` `profileMenu()`(아이콘 없음 — 상태 셋은 `state` 색 점) → `app.js` `renderProfileMenu`. 내 활동 탭 = `ACT_KINDS` → `renderActTabs`(글자만).
  요청 탭 = `index.html` `#req_tab_theirs` · `#req_tab_mine`(글자만).
- 잠금: `app/tests/ux_literals.rs` — 바뀌는 행: `aria-pressed="false" title="편집 모드">편집</button>`(#1f) · `text: "관리",`(#1k) · `goSettings("conn", "invite_text")`(#1k 레일 +) ·
  `<div id="agent_new" class="new-agent" hidden>`(#1l). `tests/css_tokens.rs`(색 리터럴은 `:root` 안만). node 시험 121.

## intent 대비 해석 (plan 승인 대상)

1. **들여쓰기 범위** — 규칙을 `.peer-fold` 공통에 건다(`foldEl` 을 쓰는 '나' 목록 전부 — 개요 · 요청 · 주고받은 메시지 + 같은 접이식인 내 활동 · 북마크 · 즐겨찾기 상대별 절도 같은 모양).
   몸(머리 아닌 자식)에 `margin-left` 한 단 + `border-left` 안내선 — 겹친 절은 저절로 두 단. 요청 개요 옛 `padding-left: var(--sp-6)` 은 지운다. 사이드바 `side-section` 은 대상 아님.
2. **'초대 생성' 누름** — `view.js` 항목에 `press: "window_btn"`(설정 트리 `invite_make` · 새로 만들기 "초대 생성"). `app.js` 는 그 항목 클릭 때만 화면 이동 뒤 `$(press).click()`.
   해시 경로(`#settings/invite_make`)에는 묶지 않는다.
3. **'초대 수락' 접기** — 라우터가 `r.view === "settings" && r.focus === "invite_accept"` 이면 펼치고 **그 밖의 모든 경로에서 접는다**. 레일 "+" 는 새로 만들기 "초대 수락" 과 같게
   `goSettings("invite_accept", "invite_text")`(그 칸으로 스크롤 — 옛 `conn` 카드 맨 위 대신). `accept_btn` 토글은 경로 변화가 없어 그대로.
4. **최대 응답 횟수 줄** — 라벨 `#chat_relay_cap_label`(`class="sub-label"`) · 칸 `#chat_relay_cap` 둘 다 `hidden` 으로 시작, `chat_relay_agent` 값이 있으면 보인다(`syncRelayCap` —
   change · 후보 다시 채움 뒤). 값 · 범위(설정 기본 10 · 1 ~ 50) · 싣는 규칙은 그대로. 요청 거절 문구 → "요청으로는 내 응답 에이전트를 쓸 수 없습니다".
5. **새 그림** — `.ico-gear`(레일 톱니와 같은 선) · `.ico-title`(T). 그 밖은 기존 아이콘. 레일 설정 단추의 인라인 SVG 는 그대로(바꿀 이유 없음).
6. **연필 단추** — `manageItem` 단추 = `icon-btn manage-btn` + `ico("pencil")`(글자 없음, `title` · `aria-label` "{이름} 관리", `aria-expanded`). 펼침이면 `[aria-expanded="true"]`
   강조(`--accent`). 대화 "편집" = `ico-pencil` + "편집".
7. **새 에이전트 줄** — `#agent_new` 를 `manage-item new-agent` 로: 머리(점선 빈 타일 + `plus` · "새 에이전트" · 부제 "등록하면 위 목록에 들어갑니다") + 몸(지금 폼 + "취소"(`#agent_new_cancel`)).
   등록 성공이면 폼 비움 + 칸 닫기. 머리 "에이전트 등록" 단추는 그대로 열고 닫음(`aria-expanded`).
8. **? 설명** — 알림 "자동 왕복" 라벨을 `label-row` + `help-wrap`(`tip_relay`) 로, 줄 넷: "자동 왕복 — 새 대화에서 '내 응답 에이전트' 를 고르면 상대 에이전트의 답에 내 에이전트가
   이어 답하기를 최대 응답 횟수까지 자동으로 주고받습니다." · "• 시작과 끝 — 왕복이 시작할 때와 끝날 때(상한 도달 · 실패 · 상대가 멈춤 등)만 알림" · "• 메시지마다 — 시작 · 끝에 더해 상대
   에이전트의 답이 올 때마다 알림" · "• 알리지 않음 — 자동 왕복 알림 없음"(의미 = `notify.rs` `RelayStart` · `RelayEnd` · `RelayStep`).
9. **탭 아이콘** — `ACT_KINDS` 에 `icon`(bell · arrow-down · arrow-up · chat · x), 요청 탭 `arrow-down` · `arrow-up`. 프로필 메뉴 `icon`(user · pencil · bell · gear), 상태 셋은 점 그대로.

## 설계

- `index.html`: `#chat_form` 라벨에 `span.ico` · 줄 이름 · 하위 줄 · `#edit_btn` 연필 · 요청 탭 아이콘 · 알림 `notify_relay` ? · `#agent_new` 줄 모양 + 취소.
- `view.js`: `settingsTree` · `newMenu` `press` · `profileMenu` `icon` · `ACT_KINDS` `icon`.
- `app.js`: `renderTree` · 새로 만들기 클릭 `press` · 라우터 `showAccept` · 레일 + · `syncRelayCap` · 보내기 문구 · `manageItem` 연필 · `renderProfileMenu` · `renderActTabs` 아이콘 ·
  에이전트 등록 닫기 · 취소.
- `app.css`: `.ico-gear` · `.ico-title` · `.peer-fold` 들여쓰기 · `#chat_form` 라벨 아이콘 · `.sub-label` · `.manage-btn` · `.new-agent` · `.tabs a` 간격 · `.edit-toggle` 아이콘. 토큰만.
- 시험(먼저 실패를 본다):
  - node `ui_polish_menu_icons`(프로필 메뉴 비상태 항목 · 내 활동 탭 아이콘 · CSS 정의) · `ui_polish_invite_press`(초대 생성 둘만 `press`) ·
    `ui_polish_new_chat`(폼에 "자동 왕복" 없음 · 라벨마다 아이콘 · 하위 줄 처음 숨김).
  - `ux_literals` 1r 행(PRESENT · ABSENT) — 옛 잠금 넷은 새 자리로.

## Stage

| stage | 목표 | 산출 | gate |
|---|---|---|---|
| 1 | 새 대화 · 아이콘(프로필 · 탭) · 연필 · 에이전트 줄 · ? · 초대 생성/수락 · 들여쓰기 | html · css · view · app · node · 잠금 | E1 ~ E6 |
| 2 | 문서(사용자 가이드 · web-ui) · mock 실화면 확인(다크 · 라이트) | 문서 · 화면 사진 | E7 · E8 · E9 |

## 평가 항목

- [ ] E1: A10 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E2: A10 — 우리 코드 경고 0(clippy -D warnings)
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: pending
- [ ] E3: A1·A2·A5·A9·A10 — 화면 로직 node 테스트 124건 이상 · 실패 0(기존 121 + ui_polish 3)
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# pass (12[4-9]|1[3-9][0-9])\s+# fail 0/
  EVIDENCE: pending
- [ ] E4: A1·A2·A5·A9 — 1r 시험만 3건(메뉴 아이콘 · 초대 생성 누름 · 새 대화 폼)
  CHECK: node --test --test-isolation=none --test-reporter=tap --test-name-pattern="^ui_polish" crates/app/web/view.test.mjs
  EXPECT: /# pass [3-9]\s+# fail 0/
  EVIDENCE: pending
- [ ] E5: A1·A3·A4·A6·A7·A8·A9 — 문구 · 자리 잠금(내 응답 에이전트 · 최대 응답 횟수 · 들여쓰기 규칙 · ? 설명 · 수락 접기 · 새 에이전트 줄 · 연필 · 탭 아이콘, 옛 자리 0)
  CHECK: cargo test -p coworkagent-app --test ux_literals
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E6: A10 — CSS 토큰(색 리터럴 `:root` 밖 0)
  CHECK: cargo test -p coworkagent-app --test css_tokens
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E7: A10 — 문서 규약
  CHECK: python tools/check_ddd_docs.py
  EXPECT: PASS
  EVIDENCE: pending
- [ ] E8: A10 — 의존 경계
  CHECK: python tools/check_crates.py
  EXPECT: /crates=4 edges=[0-9]+ violations=0/
  EVIDENCE: pending
- [ ] E9: A1·A2·A3·A4·A5·A6·A7·A8·A9 — 실화면 확인(mock, 다크 · 라이트): 새 대화 "안 함" ↔ 에이전트 고름에 하위 줄 숨김 · 보임 · 라벨 아이콘 · 요청 개요 / 주고받은 메시지 /
  개요 들여쓰기 · 안내선 · 알림 ? 설명 · 트리 "초대 생성" → 초대 상자 · 다른 항목으로 가면 수락 칸 접힘 · 새 에이전트 줄 · 연필 단추 펼침 · 프로필 메뉴 · 탭 아이콘 · JS 오류 없음.
  EVIDENCE: pending
