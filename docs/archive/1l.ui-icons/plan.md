# plan — 1l.ui-icons

> 근거: [intent](intent.md)(착수 인터뷰 2026-10-16). **구현자는 이 문서만 보고 일한다.** D·S 는 [masterPlan](../../masterPlan.md).
> 기준 커밋: `473441a`(main — 1k 병합). plan 승인: 2026-10-16 — 사용자 원문 "의견 수용. 진행. 아이콘 크기의 일관성 유지 병행."(화면 카드 이름 바꿈 · 서버 설정 열쇠 하나 — 권장안).

## 목표

메뉴 · 화면 제목 · 설정 항목 · ⋯ 메뉴에 뜻에 맞는 아이콘을 붙이고, 말풍선을 색으로 내 글 · 상대 글로 가르며, 설정을 한 열 · '?' 도움말 · 사용자 정보 팝업으로
정리한다. 레일 타일 순서를 환경설정에서 고르게 하고, "+ 새로 만들기" 메뉴를 다듬는다.

### 성공 기준(intent 사본)

- A1: 사이드바 메뉴 · 화면 머리 제목 · 설정 트리 · 대화 ⋯ · 타일 ⋯ 메뉴 항목마다 이름 앞에 뜻에 맞는 아이콘이 있다.
- A2: 대화에서 내 글은 강조색 바탕, 상대 글은 그 상대 타일 색의 연한 바탕 + 띠라 색으로 구별된다(다크 · 라이트).
- A3: 설정 카드에 늘 보이는 안내문이 없고, 제목 옆 '?' 에 마우스를 올리거나 초점을 주면 그 안내가 뜬다.
- A4: 설정에 '내 정보' 카드 · 트리 항목이 없고, 프로필 메뉴 "사용자 정보" 가 같은 내용을 팝업으로 보인다.
- A5: 설정이 한 열(페어링 → 연결 목록 → 내 에이전트 → 실행 → 화면 → 시작 → 알림)이고, 내 에이전트에 검색칸이 없으며 "에이전트 등록" 이 등록 칸을 보이고 숨기고,
  "추가 지시" 옆 '?' 에 사용 예가 뜬다.
- A6: 루트 `PROJECT.md` 검증 명령이 모두 통과한다.
- A7: 환경설정에서 타일 순서를 "그룹 → 사용자" 로 고르면 레일에 그룹 타일이 상대 타일보다 먼저 오고(키보드 이동 순서도), 다시 열어도 유지된다. 기본은 사용자 → 그룹.
- A8: "+ 새로 만들기" 메뉴에 "자동 왕복 대화" 가 없고, 남은 항목마다 이름 앞에 아이콘이 있다.

## 코드 사실(실측, 기준 `473441a`)

- 사이드바 메뉴 `view.sideNav(tile, unseen, waiting, starred)` → `{label, href, count}` · `app.renderSideNav` 가 `<a><span>이름</span><span class="count"></a>`.
- 화면 머리 `index.html` `.view-head .head-title > strong` — `#home_title` · `#agents_title` · `#peer_title` 은 `app.js` 가 `textContent` 로 덮어쓴다.
- 설정 트리 `view.settingsTree()`(첫 항목 `me` "내 정보") · `route("#me")` = 설정 내 정보. 설정 틀 `.settings-cols` 두 열(왼쪽 `#conn` · `#peers`, 오른쪽 `#me` ·
  `#agents`) + 아래 `#run` · `#screen` · `#startup` · `#notify`. 안내문 = 카드 안 `<p class="hint">`(페어링 · 초대 생성 · 초대 수락 · 연결 목록 · 내 에이전트 · 실행 ·
  시작 · 알림). 연결 목록 안내에 "내 정보 → 지문".
- 내 에이전트 머리 = `h2` · `#agent_q`(검색 — `renderAgentList` · `editAgent` 가 씀) · `#agent_new_btn` → `<details id="agent_new">`(`summary` "새 에이전트 등록")
  `open = true`. 그 아래 `<details class="guide">` "추가 지시 사용 예".
- 말풍선 `app.messageEl` — `article.msg.{author}.side-{mine|theirs}`(`view.bubbleSide` = `m.mine`), CSS 내 글 `--sent-bg`(#ECEDFD) · 상대 `--surface-2`(#F7F8FB) ·
  상대 에이전트 `--surface`. 타일 색 `view.colorOf(id)` = `peer-1` ~ `peer-5`(CSS `.peer-n` 은 `background` 를 깐다).
- 프로필 메뉴 `view.profileMenu` → `app.renderProfileMenu` · `profileAct`. 팝업 선례 `<dialog id="scope_dialog">` + `showModal`.
- 타일 ⋯ `app.openTileMenu`(타일 편집 · 이미지 지우기 · 그룹 체크 `peerGroupMenu` · 위로 · 아래로 옮기기), 대화 ⋯ `app.openConvMenu`(머리 "그룹" + 그룹 체크).
- 레일 `index.html` `#rail` = `#rail_me` · `#rail_pcs`(상대) · `#rail_add` · `#rail_sep` · `#rail_groups` · `#rail_group_add` · `#rail_settings` 순 DOM.
- 화면 설정 저장 = `GET/PUT /api/settings`(`notify.rs` `SettingsPatch` 검증 → `settings` 표 열쇠 · 값). 테마 선례 `run_opts::THEME_KEY` · `THEMES` · `theme()`
  (틀린 값은 400, 없거나 모르는 값은 기본). 스키마 바꾸지 않고 열쇠만 더할 수 있다.
- "+ 새로 만들기" `view.newMenu()` = 새 대화 · 자동 왕복 대화(`chat_relay_agent` 초점) · 초대 생성 · 초대 수락 · 에이전트 등록(`#settings/agents` · `agent_name` 초점).
- 아이콘 선례 = `index.html` 인라인 SVG(24 viewBox · stroke 1.8). node 시험 98건.

## intent 대비 해석 (plan 승인 대상)

1. **아이콘 한 벌** — `app.css` 에 `.ico`(16px · `currentColor` 를 SVG 모양 `mask` 로 칠함) + `.ico-{이름}`(data URI 선 아이콘). HTML · JS 모두
   `<span class="ico ico-{이름}" aria-hidden="true">` 만 쓴다(아이콘 정의는 CSS 한 곳). 이름 · 뜻:

   | 자리 | 항목 → 아이콘 |
   |---|---|
   | 사이드바 · 화면 제목 | 개요 → 집 · 내 활동 → 종(상단 바 알림과 같음) · 요청 → 받은 편지함 · 주고받은 글 → 좌우 화살표 · 즐겨찾기 → 별 · 북마크 항목 → 책갈피 · 에이전트 → 로봇 |
   | 설정 트리 | 연결 → 고리 · 초대 생성 → ＋ · 초대 수락 → 들어오는 화살표 · 연결 목록 → 사람 둘 · 내 에이전트 → 로봇 · 실행 → 재생 · 화면 → 모니터 · 시작 → 전원 · 알림 → 종 |
   | 타일 ⋯ | 타일 편집 → 연필 · 이미지 지우기 → 지우개 그림 · 그룹 머리 · 그룹 항목 → 사람 둘 · 위로 · 아래로 옮기기 → ↑ · ↓ 화살표 |
   | 대화 ⋯ | 그룹 머리 · 그룹 항목 → 사람 둘 |
   | + 새로 만들기 | 새 대화 → 말풍선 · 초대 생성 → ＋ · 초대 수락 → 들어오는 화살표 · 에이전트 등록 → 로봇 |

   `sideNav` · `settingsTree` 항목에 `icon` 열쇠를 더한다(시험 대상). 동적 제목 3개는 `<strong>` 안에 아이콘 + `<span id="…_title">` 로 나눠 `textContent` 가
   아이콘을 지우지 않게 한다.
2. **말풍선 색** — 내 글 = `--accent` 바탕 · `--on-accent` 글자 · 테두리 없음(안의 링크 · 아이콘은 글자색, 코드 · 표는 `--text` 로 되돌려 읽히게). 상대 글 =
   `color-mix(그 상대 타일 색 12%, --surface)` 바탕 + 왼쪽 3px 타일 색 띠(상대 사람 · 에이전트 같음 — 1j 의 상대 에이전트 `--surface` 갈래 없앰). 타일 색은
   `article` 의 `data-tone="peer-n"` → CSS `--tone`(`.peer-n` 클래스는 배경을 깔아 쓰지 않음).
3. **'?' 도움말** — 카드 안내 `<p class="hint">` 를 `h2` 옆 `<span class="help" tabindex="0" role="button" aria-label="{제목} 도움말">?<span class="tip"
   role="tooltip">…</span></span>` 로. `:hover` · `:focus` 에서 `.tip` 이 뜬다(JS 없음). 페어링 카드는 카드 안내 + 초대 생성 안내를 한 '?' 에, 초대 수락 안내는 수락
   칸 `label` 옆 '?'. 연결 목록 안내의 "내 정보 → 지문" → "프로필 → 사용자 정보 → 지문". 화면(테마) 카드는 안내가 없어 '?' 없음.
4. **사용자 정보 팝업** — `#me` 카드의 표(같은 id)를 `<dialog id="me_dialog">`(제목 "사용자 정보" · "닫기", Esc 로도 닫힘)로 옮긴다. 프로필 메뉴 구분선 아래 첫
   항목 `{key: "me", label: "사용자 정보"}` → `showModal`. 설정 트리 · 경로 `#settings/me` · `#me` 는 없앤다(`#me` 는 안내 화면 — 앱 안에 그 링크 없음).
5. **설정 한 열** — `.settings-cols` · `.settings-col` 틀을 없애고 카드 순서 페어링 → 연결 목록 → 내 에이전트 → 실행 → 화면 → 시작 → 알림. 트리도 같은 순서.
6. **내 에이전트** — `#agent_q`(검색 · `searchItems` 호출 · `editAgent` 의 비움) 없앰. `<details id="agent_new">` → `<div id="agent_new" hidden>`(제목 줄 없음),
   "에이전트 등록" = `aria-expanded` · `aria-controls` 로 보이기 · 숨기기(보이면 이름 칸 초점 — 1k "초대 수락" 과 같은 꼴). "추가 지시 사용 예" 목록 → 등록 칸
   "추가 지시" `label` 옆 '?'.
7. **타일 순서** — 설정 열쇠 `ui_rail_order` = `peers_first`(기본) · `groups_first`(`run_opts::RAIL_ORDER_KEY` · `RAIL_ORDERS` · `rail_order()`, 틀린 값 400 ·
   없거나 모르는 값은 기본 — 테마와 같은 꼴). 화면 카드에 "타일 순서" 고르기("사용자 → 그룹" · "그룹 → 사용자"), 바꾸면 바로 저장 · 반영. 반영은 레일 DOM
   자체를 옮긴다(`#rail_groups` · `#rail_group_add` 를 `#rail_pcs` 앞으로 — CSS `order` 는 Tab 순서가 화면과 어긋나 쓰지 않음). 구분선은 두 묶음 사이 그대로.
   카드 · 트리 이름 "화면 — 테마" → "화면 — 테마 · 타일 순서"(내용과 맞춤 — 이름 바꿈, 승인 대상).
8. **+ 새로 만들기** — `newMenu` 에서 "자동 왕복 대화" 를 빼고(새 대화 화면의 자동 왕복 칸은 그대로) 항목에 `icon`. "에이전트 등록" 은 6 의 숨긴 등록 칸을
   펼친 뒤 `agent_name` 초점.

## 설계

- 서버 — `an_core::run_opts` 타일 순서 열쇠 · 검증 · 읽기, `notify.rs` `SettingsPatch.ui_rail_order` · `settings_json`. 시험: `ux_api` 타일 순서(기본 · 바꿈 · 틀린 값 400).
- `view.js` — `newMenu`(자동 왕복 없음 · `icon`) · `sideNav` · `settingsTree` 항목 `icon` · `settingsTree` 에서 `me` 빼고 순서 · `profileMenu` 에 `me` · `route` 의 `#me` 없앰. 시험: `side_nav_icons` ·
  `settings_tree_icons` · 고친 `settings_tree` · `profile_menu` · `route`.
- `app.js` — `renderSideNav` 아이콘 · 설정 트리 아이콘 · `messageEl` `data-tone` · `openTileMenu` · `peerGroupMenu` · `openConvMenu` 아이콘 · `profileAct` "me" →
  `me_dialog` · 에이전트 검색 없앰 · 등록 칸 보이기 · 숨기기.
- `index.html` — 화면 제목 아이콘 · 설정 한 열 · '?' · `me_dialog` · 등록 칸. `app.css` — `.ico` · 아이콘 · 말풍선 · `.help` · `.tip`.
- `ux_literals.rs` — 새 문구 · 자리 · 없어진 것(`agent_q` · "새 에이전트 등록" · 설정 `#me` 카드 · 늘 보이는 카드 안내).
- 문서 — 사용자 가이드(설정 · 사용자 정보 · 대화) · an_app · PRODUCT(필요하면) · README(필요하면) · 캡처(설정 · 대화).

## Stage

| stage | 목표 | 산출 | gate |
|---|---|---|---|
| 1 | 아이콘 · 말풍선 색 · 설정 '?' · 한 열 · 사용자 정보 팝업 · 내 에이전트 · 타일 순서 · 새로 만들기 · 문서 · 이 PC 실측 | node · 문구 시험 · mock 화면(다크 · 라이트) · CDP | E1 ~ E8 |

단계 끝 mock(가상 데이터) 화면을 사용자에게 보인다.

## 평가 항목

- [ ] E1: A6·A7 — 워크스페이스 테스트 전부 통과(타일 순서 설정 기본 · 저장 · 틀린 값 400)
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E2: A6 — 우리 코드 경고 0(clippy -D warnings)
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: pending
- [ ] E3: A1·A4·A5·A6·A8 — 화면 로직 node 테스트 100건 이상 · 실패 0(메뉴 · 트리 아이콘 · 트리에 내 정보 없음 · 프로필 "사용자 정보" · 새로 만들기에 자동 왕복 없음)
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# pass (1[0-9]{2})\s+# fail 0/
  EVIDENCE: pending
- [ ] E4: A1·A2·A3·A4·A5 — 웹UI 문구 · 자리(아이콘 · `data-tone` · '?' · `me_dialog` · 한 열 · 등록 칸, 없어진 것 0 — `agent_q` · "새 에이전트 등록" · 설정 내 정보 카드)
  CHECK: cargo test -p coworkagent-app --test ux_literals
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E5: A6 — 문서 규약
  CHECK: python tools/check_ddd_docs.py
  EXPECT: PASS
  EVIDENCE: pending
- [ ] E6: A6 — README 사용자 관점
  CHECK: python tools/check_readme.py
  EXPECT: readme ok
  EVIDENCE: pending
- [ ] E7: A6 — 의존 경계
  CHECK: python tools/check_crates.py
  EXPECT: /crates=4 edges=[0-9]+ violations=0/
  EVIDENCE: pending
- [ ] E8: A1·A2·A3·A4·A5 — 이 PC 실측(CDP · 비파괴 — 실제 상대 · 이력 · 설정 값은 바꾸지 않음): 메뉴 · 제목 · 트리 · ⋯ 아이콘 · 말풍선 색(내 글 강조색 · 상대 타일
  색) · '?' 띄움 · 설정 한 열 · 내 정보 없음 · 프로필 "사용자 정보" 팝업 · 등록 칸 보이기 · 숨기기 · 타일 순서 바꿈 → 레일 순서 · Tab 순서 · 새로 고침 뒤 유지(확인 뒤 원래 값으로 되돌림) · 새로 만들기 메뉴 아이콘 · 자동 왕복 없음 · JS 오류 없음.
  EVIDENCE: pending

## 종결 시 할 일

- masterPlan: D35(메뉴 아이콘 · 타일 순서) · S4 보강(설정 한 열 · 사용자 정보 팝업 · 말풍선 색 · 새로 만들기) · 개정 기록. 원장 없음(예정).
- 이 PC 반영(릴리스 빌드). GitHub 반영 · 릴리스(1h · 1j · 1k · 1l)는 사용자 확인 뒤.

## 개정 기록

- 2026-10-16 · plan 승인 전 사용자 추가 지시 — 타일 순서 설정 · "+ 새로 만들기" 자동 왕복 없앰 · 아이콘(해석 7 · 8, A7 · A8). 승인 때 추가 — 아이콘 크기 일관성(stage1 설계 결정).
