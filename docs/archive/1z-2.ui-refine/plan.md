# plan — 1z-2.ui-refine

> 근거: [intent](intent.md)(승인 2026-10-21 — 결정 1 ~ 9). **구현자는 이 문서만 보고 일한다.** D·S 는 [masterPlan](../../masterPlan.md)
> (D35 사이드바 메뉴 · 레일 · 대화 줄 · D37 그룹설정 · S4 주고받은 메시지 — 종결 때 환류). 브랜치 `task/1z-2-ui-refine`. 기준 커밋: `0bdca87`(task/1z-chat-look — #1z 종결,
> main 미반영). plan 승인: 2026-10-21(사용자 "위 작업까지 완료 후 병합, push 진행").

## 목표

에이전트 응답 메뉴('나' 에만) · 개요 아이콘 · 레일 "+ 사용자 / + 그룹" · 대화 줄 한 줄 · 레일 끌기 놓는 자리 · 그룹설정 시안 B · 주고받은 메시지 시안 Peer.

### 성공 기준(intent 사본)

- A1: '에이전트 응답' 메뉴(`#agents/convs`)가 '나' 타일에만 있고 상대 · 그룹 타일에는 없다, 화면 제목 · 빈 안내도 "에이전트 응답", 사용자에게 보이는 "에이전트 대화" 메뉴 이름 0건(node 시험 · 검사).
- A2: 상대 타일을 고를 때 에이전트 응답 화면이면 그 상대 개요로 바뀐다, 상대 타일 레일 배지는 그 타일에서 보이는 대화(에이전트 대화 제외)의 안 읽음만 센다(node 시험 · 검사).
- A3: 개요 아코디언 머리에 사용자 아이콘, 대화 줄에 로봇 아이콘이 붙는다(검사 · 실 화면).
- A4: 레일 "+" 단추 글자가 "+ 사용자" · "+ 그룹" 이고 보이는 글자가 접근 이름이다(검사).
- A5: 대화 목록 줄이 한 줄(2행 없음) — 손잡이 · 로봇 아이콘 · 제목 · 실행 중 점 · 안 읽음 수 · 시각 · ⋯, 미리보기 · 발화자 아이콘 · 칩 없음(검사).
- A6: 레일 타일을 같은 묶음 타일 사이 틈 · 이름 · 양 끝 밖에 놓아도 가장 가까운 자리로 옮겨진다(경계 = 타일 가운데)(node 시험 · 실 화면).
- A7: 그룹설정 = 그룹 카드 격자 + 사용자 표, 카드 누르면 펼침(대화 전체 · 그룹에서 빼기) · 이름 바꾸기 · 삭제, 사용자 줄 누르면 관리 + 대화 펼침, 펼침이 5초 새로 고침에도 유지(검사 · node 시험 · 실 화면).
- A8: 그룹설정 대화 줄을 포인터로 끌어 그룹 카드에 놓으면 그 그룹에 넣는다(복사, 대화 순서 안 바뀜), 설정 대화 줄에 HTML 끌어 놓기 없음(검사 · 실 화면).
- A9: 주고받은 메시지 줄 = 원형 아바타(상대 사람 · 나 · 에이전트 구분) · 메타(이름 · 사람 / 에이전트 칩 · 대화 제목) · 본문 · 시각 · 북마크, 에이전트 줄 바탕 구분(node 시험 · 검사 · 실 화면).
- A10: 회귀 없음 — PROJECT.md 검증 명령 PASS.

## 설계

### 웹UI — `view.js`(순수 함수 · node 시험)

- `sideNav`: '에이전트' 하위 셋째 = `...(me ? [{ label: "에이전트 응답", href: "#agents/convs", icon: "chat", sub: true, count: agentUnread }] : [])`.
- `export function peerLeaves(view)` → `view === "agentConvs"`(상대 타일에서 열 수 없는 화면).
- `export function tileUnread(convs)` → `unreadByPeer(convs.filter((c) => c.dir !== "in"))`(상대 타일 배지).
- `export function railSlot(centers, pos)` → 가장 가까운 가운데의 `{ index, after: pos > centers[index] }`, 빈 목록 `null`(같은 거리면 앞 것).
- `export function avatarKind(author, mine)` → `"agent"` · `"me"` · `"peer"`. `inboxRows` 행에 `mine: m.mine === true`.
- `groupSettingsPick` 삭제(시안 A 의 "고른 항목" — 시안 B 에는 고르기가 없다) — 그 시험 `group_settings_pick` 도 삭제(기능 삭제, 단정 완화 아님). `groupSettingsList` 그대로(검색).
- 시험(`view.test.mjs`): `side_nav` · `side_nav_sections` · `side_nav_agents` 갱신(상대 타일 에이전트 응답 없음 · 옛 이름 없음 · `peerLeaves`), `inbox_rows_author` 에 `mine`,
  새 `rail_slot`(가운데 정확히 = 앞 · 1px 지나면 뒤 · 틈 양쪽 · 한가운데 틈 = 앞 타일 · 양 끝 밖) · `tile_unread` · `avatar_kind`.

### 웹UI — `app.js`

- `render`: `peerLeaves(r.view) && state.railPeer != null` → `pickRail(null)`(직접 연 주소). 타일 고르기는 기존 `pickTile` 이 대화 화면 밖이면 `#`(그 상대 개요).
- `renderRail`: `tileUnread(state.convs)`.
- 에이전트 응답 화면 빈 안내 "에이전트 응답 없음".
- `renderHome`: 절 머리 맨 앞 `rowTile(\`small ${colorOf(g.peer)}\`, p?.icon)`, 대화 줄 에이전트 = `el("span", { class: "home-agent" }, speakerEl("agent"), …)`.
- `convItem`: `room-av agent-av` + `ico("bot")`, 제목 줄 = 제목 · 실행 중 `spin`(role img · 상태 이름) · 안 읽음 수 · 시각, `sub` 줄 삭제(미리보기 · 발화자 · 자동 왕복 · 태그 칩).
- `tileGrab`: 같은 종류 타일(`same`)의 가운데로 `railSlot`, 포인터가 레일 밖(세로 레일 = X, 가로 레일 = Y)이면 놓을 자리 없음, 자기 자신이면 없음.
- 그룹설정: `renderPeerList` = `gsGroupsEl`(카드 격자 — 카드 `data-drag="group"`, 머리 `gsToggle` · 연필 · 휴지통, 접힘 = 대화 3개 + "+n 더 보기", 펼침 = `settingsConvRow` 전체) +
  `gsUsersEl`(표 — 줄 `onclick`(단추 · 입력 밖) · 이름 칸 `gsToggle` · 연필, 펼침 줄 `userDetail` = 부제 · 지문 · 주소 · 모델 · effort · 단추 · "대화 n" 줄). 펼침 기억 = `state.open`
  (`gsg{id}` · `gsu{id}`). `gsDot` = 점 + 상태 글자(`data-peer-dot` — 서명 밖 부분 갱신). `gsDragging` 동안 다시 그리지 않음. 시안 A 함수(`groupSettingsListEl` · `groupSection` ·
  HTML `dragstart`/`dragover`/`drop` · `convDrag`) 삭제.
- `settingsConvRow`: `data-drag="conv"`, 손잡이 단추 `convGrab(ev, li, { join: true })`(HTML `draggable` 없음).
- `convGrab(ev, li, { join })`: `join` 이면 포인터 아래 가장 가까운 `[data-drag="group"]` 만 대상, 놓으면 `gsJoin`(`convCopy` · `changeGroup` · `peer_msg`), 아니면 미룬 `renderPeerList`.
  공통: `lostpointercapture` 도 끝 처리, 끝 처리는 한 번만(`done`).
- `peerRowEls`: 줄 = `li.has-mark.pm-row.pm-{kind}` › `a.pm-link`(`pm-av pm-av-{kind}` 아이콘(role img · "사람" / "에이전트") · `pm-text`(메타: 이름 · "사람" 또는 `tag agent-tag` "에이전트" ·
  대화 제목 · 요청 태그 / `pm-body` 본문) · `pm-time`) + 북마크 단추. `rowEls` 의 `tile` 갈래 삭제(주고받은 메시지 전용이었다).

### 웹UI — `index.html` · `app.css`

- `index.html`: `rail_add` = `class="rail-btn add add-text" title="초대 수락">+ 사용자`(aria-label 삭제 — 보이는 글자가 이름), `rail_group_add` = `class="rail-btn add add-text" title="새 그룹">+ 그룹`,
  에이전트 응답 화면 제목 "에이전트 응답".
- `app.css`: 토큰 `--agent-soft: #F2F9F8`(다크 `--dark-agent-soft: #13211F`), `.rail-btn.add-text`(옛 `.group-add`), `#rail { user-select: none; }`, 대화 줄 한 줄(`.room-av.agent-av` ·
  `.title` 왼쪽 정렬), 그룹설정 B(`.gs-cards` 격자 `minmax(220px, 1fr)` · `.gs-card` · `.gs-table` · `.gs-more`), 주고받은 메시지(`.pm-*` · `.rows li.pm-agent { background: var(--agent-soft); }`),
  시안 A · 옛 연결 목록 CSS(`.gs-list` · `.gs-row` · `.peer-section*` · `.settings-tree` · `.row-with-tile`) 삭제.
- `ux_literals.rs`: PRESENT · ABSENT 갱신(옛 문구 → ABSENT: `draggable: "true"` · `class: "room-preview"` · `label: "에이전트 대화"` · 레일 "+" 글자 · `groupSettingsListEl(`).

### 문서

- [사용자 가이드](../../manual/user/user-guide.md) · [웹UI 매뉴얼](../../manual/dev/web-ui.md) · `README.md` — 에이전트 응답 · 개요 아이콘 · 레일 + · 대화 줄 · 레일 끌기 · 그룹설정 B · 주고받은 메시지.
- masterPlan(종결 환류): D35(에이전트 응답 '나' 에만 · 대화 줄 한 줄 · 레일 놓는 자리 · + 사용자), D37(그룹설정 B), S4(주고받은 메시지 줄 · 개요 아이콘), 개정 기록.

## 평가 항목

- [ ] E1: A10 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E2: A10 — clippy 경고 0
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: pending
- [ ] E3: A10 — 웹UI 화면 로직 시험 전부(지금 158 - 1 + 새 시험)
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# pass (1[6-9][0-9])\s+# fail 0/
  EVIDENCE: pending
- [ ] E4: A1·A2·A6·A9 — 메뉴 · 상대 타일 · 놓을 자리 · 아바타(node)
  CHECK: node --test --test-reporter=tap --test-name-pattern="^side_nav|rail_slot|tile_unread|avatar_kind|inbox_rows_author" crates/app/web/view.test.mjs
  EXPECT: /# pass 11\s+# fail 0/
  EVIDENCE: pending
- [ ] E5: A1·A2·A3·A4·A5 — 메뉴 이름 · 배지 · 개요 아이콘 · 레일 + · 대화 줄 한 줄 코드 실재
  CHECK: python -c "import pathlib;j=pathlib.Path('crates/app/web/app.js').read_text(encoding='utf-8');v=pathlib.Path('crates/app/web/view.js').read_text(encoding='utf-8');h=pathlib.Path('crates/app/web/index.html').read_text(encoding='utf-8');ci=j[j.index('function convItem'):j.index('function renderConvList')];print('menu=%d old=%d leave=%d badge=%d head=%d row=%d plus=%d gplus=%d bot=%d sub=%d prev=%d'%(v.count('...(me ? [{ label: \x22에이전트 응답\x22'),v.count('label: \x22에이전트 대화\x22')+h.count('>에이전트 대화<'),j.count('if (peerLeaves(r.view) && state.railPeer != null) pickRail(null);'),j.count('const unread = tileUnread(state.convs);'),j.count('rowTile(`small ${colorOf(g.peer)}`, p?.icon),'),j.count('speakerEl(\x22agent\x22)'),h.count('title=\x22초대 수락\x22>+ 사용자</button>'),h.count('title=\x22새 그룹\x22>+ 그룹</button>'),ci.count('ico(\x22bot\x22)'),ci.count('{ class: \x22sub\x22 }'),ci.count('room-preview')))"
  EXPECT: menu=1 old=0 leave=1 badge=1 head=1 row=1 plus=1 gplus=1 bot=1 sub=0 prev=0
  EVIDENCE: pending
- [ ] E6: A6·A7·A8·A9 — 레일 놓을 자리 · 그룹설정 B · 포인터 넣기 · 주고받은 메시지 코드 실재
  CHECK: python -c "import pathlib;j=pathlib.Path('crates/app/web/app.js').read_text(encoding='utf-8');c=pathlib.Path('crates/app/web/app.css').read_text(encoding='utf-8');tg=j[j.index('function tileGrab(ev)'):j.index('function tileGrab(ev)')+4000];print('slot=%d point=%d b=%d card=%d table=%d keep=%d join=%d html5=%d lost=%d pm=%d bg=%d sel=%d oldA=%d'%(tg.count('railSlot(same.map(mid)'),tg.count('document.elementFromPoint'),j.count('replaceChildren(gsGroupsEl(list, secs, q), gsUsersEl(list, convs, q))'),c.count('.gs-cards { display: grid; grid-template-columns: repeat(auto-fill, minmax(220px, 1fr))'),j.count('{ class: \x22gs-table\x22 }'),j[j.index('function gsToggle'):j.index('function gsGroupsEl')].count('const open = state.open.has(key);'),j.count('convGrab(ev, ev.currentTarget.parentElement, { join: true })'),j.count('draggable: \x22true\x22'),j.count('grip.addEventListener(\x22lostpointercapture\x22, end);'),j.count('pm-av pm-av-${kind}'),c.count('.rows li.pm-agent { background: var(--agent-soft); }'),c.count('#rail { user-select: none; }'),j.count('groupSettingsListEl')))"
  EXPECT: slot=1 point=0 b=1 card=1 table=1 keep=1 join=1 html5=0 lost=1 pm=1 bg=1 sel=1 oldA=0
  EVIDENCE: pending
- [ ] E7: A10 — 문서 규약 · 크레이트 경계 · 화면 문구 목록 · CSS 토큰
  CHECK: python tools/check_ddd_docs.py && python tools/check_crates.py && cargo test -p coworkagent-app --test ux_literals --test css_tokens
  EXPECT: /test result: ok\. 3 passed; 0 failed[\s\S]*test result: ok\. 5 passed; 0 failed/
  EVIDENCE: pending
- [ ] E8: A3·A6·A7·A8·A9 — 실 화면(`rebuild_run` 릴리스 빌드): 개요 아이콘 · 레일 이름 · 틈 · 끝 밖 놓기(보통 · 작게) · 그룹설정 카드 · 표 펼침 유지 · 카드에 포인터 놓기 · 주고받은 메시지 줄
  EVIDENCE: pending

## 개정 기록

- 2026-10-21 · E6 CHECK 보정(EXPECT 그대로 — eval-protocol 규약 2): 펼침 기억 `const open = state.open.has(key);` 는 `manageItem` 에도 있어(`keep=2`) `gsToggle` 본문 안에서만 센다.
