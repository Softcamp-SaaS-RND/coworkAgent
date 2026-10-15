# plan — 1i.rail-group

> 근거: [intent](intent.md)(승인 2026-10-15 — 라운드 1 · 2 · 3). **구현자는 이 문서만 보고 일한다.** D·S 는 [masterPlan](../../masterPlan.md).
> 기준 커밋: `136e829`(`task/1i-rail-group` — 분기점 main). plan 승인: 2026-10-15 — 사용자 원문 "권장방안 승인. 개발진행"(해석 16건 · 타일 색
> 상대 id 기준(해석 11) · "대화 추가" 에서 새 대화 만들기 없음).

## 목표

레일에 **그룹**(이 PC 에만 있는 대화 묶음 — 한 대화는 여러 그룹에)을 더하고, 레일 타일마다 메뉴(이름 · 이미지 · 그룹 삭제)를, 그룹 · 상대 타일 ·
대화 목록에 드래그 앤 드롭 위치 조정(키보드 Alt + ↑ / ↓)을 넣는다. 정렬 기능은 따로 두지 않는다.

### 성공 기준(intent 사본)

- A1: 레일 "그룹 +" 로 이름을 넣으면 상대 타일 아래 구분선 뒤에 그룹 타일이 생기고, 타일 메뉴로 이름 바꾸기 · 삭제가 된다 — 삭제해도 그 그룹의
  대화는 남는다.
- A2: 대화 하나를 여러 그룹에 넣을 수 있고, 대화 ⋯ 메뉴 "그룹" 의 체크로 그룹마다 넣고 뺀다 — 체크 상태가 지금 소속과 같다.
- A3: 그룹 화면 "대화 추가" 의 검색 + 체크로 여러 대화를 한 번에 넣고, 그룹 목록 줄 "그룹에서 빼기" 로 뺀다(대화 자체는 그대로).
- A4: 그룹 · 소속은 이 PC 에만 있다 — 상대 PC 로 가는 프레임이 없고, 앱을 다시 켜도 남는다.
- A5: 레일 타일('나' · 상대 · 그룹)마다 우클릭 · ⋯ 메뉴로 이름 · 이미지를 바꾸고 이미지를 지운다 — 그룹 타일은 같은 메뉴에 "그룹 삭제".
- A6: 그룹 타일을 고르면 메뉴가 대화 · 요청 · 저장된 항목 · 주고받은 글 4개(첫 화면 = 그룹 요약)이고, 보이는 목록 · 화면은 그 그룹 대화만 · 상대별
  접이식 절이다. 대화 목록은 ★(그룹 안) + 상대별 절이고, 그 화면에서 만든 새 대화는 그 그룹에 들어 있다.
- A7: 그룹 이름은 1 ~ 40자 · 같은 이름 금지(거절 문구가 보인다).
- A8: 상대 타일끼리 · 그룹 타일끼리 · 같은 절 안의 대화끼리 끌어 놓아 자리를 바꾸고, 포커스한 항목에서 Alt + ↑ / ↓ 로 한 칸 옮긴다 — 앱을 다시 켜도
  자리가 남는다. '나' · "+" · 설정은 옮겨지지 않고, 묶음 사이(상대 ↔ 그룹 · 다른 상대 절)로는 놓이지 않는다.
- A9: 대화 순서는 대화마다 하나다 — 한 곳(예: 그룹)에서 옮기면 '나' · 상대 타일 · ★ 에서도 같은 앞뒤이고, 새 글이 와도 자리가 바뀌지 않으며 새 대화는
  맨 위에 생긴다. 상대 타일을 옮기면 '나' · 그룹의 상대별 절 순서도 따라 바뀐다.
- A10: 대화를 그룹 타일에 끌어 놓으면 그 그룹에 들어간다(이미 있으면 그대로).
- A11: 연결을 지운 상대의 대화는 그룹에 그대로 남아 레일 상대 절 뒤에 보인다.
- A12: 루트 `PROJECT.md` 검증 명령이 통과한다.

## 코드 사실(실측, 기준 `ceb6fd4` 뒤 main)

- **스키마 15**(`crates/core/src/db.rs` `SCHEMA_VERSION`) — 올리기 전 `{db}.v{N}.bak` 사본, 단계는 `if v < N { tx … PRAGMA user_version = N }`.
  `rusqlite` bundled(창 함수 사용 가능).
- **대화 INSERT 는 한 곳** — `chat.rs:320`(보낸 · 받은 대화 모두). 대화를 지우는 코드는 없다. 연결을 지워도(`peers::remove`) 대화는 남는다.
- **대화 목록** `chat::summaries` = 최근 활동순(`ORDER BY` 마지막 글 시각) — 웹UI(`GET /api/conversations`) · `chat.rs:691`(통합 검색) · 시험이 쓴다.
  웹UI 는 절 안에서 다시 `last_at` 내림차순(`view.js` `peerSummary`)이고 `convSections` 는 들어온 순서.
- **상대 목록** `peers::list` = `ORDER BY id`. 레일 = `state.peers` 의 페어링된 것(`app.js` `renderRail`). 타일 색 = 페어링 목록 **자리**(`tileColor` — 자리를
  바꾸면 색이 바뀐다).
- **레일 선택** = `state.railPeer`(상대 id · `null` = 나) 하나. 목록 거르기 = `filterByPeer` · API `?peer=`(`/api/messages` · `/api/saved` · `/api/activity`).
- **타일 편집** = 사이드바 머리 ✎ → `openTileEdit()`(지금 고른 타일 대상) · 이름 + 그림(80px PNG data URL — 서버 `run_opts::clean_icon`). '나' =
  `PUT /api/settings` `me_alias` · `me_icon`, 상대 = `PATCH /api/peers/{id}` `alias` · `icon`.
- **팝오버 메뉴** = `.popover.menu`(`#new_menu` · `#profile_menu`). 대화 행에 ⋯ 메뉴 · 드래그 코드는 없다.
- **접힘 기억** = 설정 `ui_folded` 열쇠 `^[a-z]+:([0-9]+|me)$`(300개).
- 웹UI node 시험 57건(`view.test.mjs`).

## intent 대비 해석 (plan 승인 대상)

1. **저장(스키마 16)** — `groups(id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL UNIQUE, icon TEXT, pos INTEGER NOT NULL)` ·
   `group_convs(group_id, conversation_id, PRIMARY KEY(group_id, conversation_id))` ·
   `peers.pos` · `conversations.pos`. 그룹 id 는 다시 쓰지 않는다(AUTOINCREMENT — #17 S1-P 같은 번호 재사용을 피함). 이 DB 는 `PRAGMA foreign_keys` 를 켜지 않는다(`db::open` —
   기본 꺼짐) — 외래 키 · `ON DELETE CASCADE` 에 기대지 않고 `groups::remove` 가 같은 트랜잭션에서 `group_convs` 를 지운다(켜면 기존 표의 선언이 새로
   검사된다 — 범위 밖). 올릴 때 첫 자리 =
   지금 보이는 순서(상대 = id 순, 대화 = 지금 최근 활동순) — 업데이트 직후 화면이 그대로다.
2. **새 대화는 맨 위** — `chat.rs:320` INSERT 에 `pos = coalesce(min(pos), 0) - 1`(받은 대화 포함). 새 상대는 맨 뒤(`max + 1`), 새 그룹도 맨 뒤.
3. **순서 = `pos` 하나** — 웹UI 는 대화 · 상대 · 그룹을 `pos` 로 그린다(절 안 · 요약 · ★ · 그룹 모두). `chat::summaries` 의 최근 활동순은 그대로 둔다
   (통합 검색 · CLI 영향 없음) — JSON 에 `pos` 를 싣고 웹UI 가 정렬. `peers::list` 는 `ORDER BY pos, id`.
4. **옮기기 = 전체 순서 보내기** — `PUT /api/order {kind: "peers" | "groups" | "conversations", ids: [...]}`: 준 순서대로 `pos = 0..n`. 모르는 id ·
   중복 → 400, 빠진 id(그 사이 생긴 새 대화 등)는 제 `pos` 그대로 — 같은 `pos` 가 생기면 정렬은 `pos, id`(웹UI · `peers::list` 같은 규칙). 웹UI 는 걸러 보인 목록에서 옮겨도 **전체 순서**에서 "놓은 대상 바로 앞 / 뒤" 로
   끼워 계산(`view.moveInOrder`) — 그래서 그룹에서 옮긴 앞뒤가 '나' · 상대 타일에서도 같다(A9).
5. **그룹 API** — `GET /api/groups` → `[{id, name, icon, pos, convs: [대화 id]}]` · `POST /api/groups {name}` · `PATCH /api/groups/{id} {name?, icon?}` ·
   `DELETE /api/groups/{id}` · `PUT /api/groups/{id}/conversations {add: [...], remove: [...]}`(이미 있음 · 없음은 무시, 모르는 대화 → 400). 이름 =
   앞뒤 공백 뺀 1 ~ 40자(문자 수), 같은 이름 → 409 "같은 이름의 그룹이 있습니다". 그림 = `clean_icon` 그대로. 새 대화 `POST /api/conversations` 에
   선택 `group` — 같은 트랜잭션으로 소속. 피어 프레임 · CLI 변경 없음(A4).
6. **목록 거르기** — `/api/messages` · `/api/saved` 에 선택 `group`(그 그룹 대화만 — `conversation_id IN (SELECT … FROM group_convs …)`). 내 활동은
   그룹에 없다(D35). 대화 목록 · 요약은 웹UI 가 `convs` 로 거른다.
7. **레일 선택** — `state.railGroup`(그룹 id · `null`)을 더한다. 상대 타일 · '나' 를 누르면 `null`, 그룹을 누르면 `railPeer = null`. 그룹이면
   `sideNav("group")` = 대화 `#` · 요청 · 저장된 항목 · 주고받은 글 `#p`(4개). 첫 화면 `#` = 그룹 요약(`peerSummary` 를 그룹 대화로 — 단 **그룹에 대화가 있는 상대만** 머리를 보인다. '나' 요약은 대화 없는 페어링 상대도
   보이지만, 그룹에서 그러면 그룹과 무관한 상대 머리가 늘어선다). 그룹이 비었으면 "대화 추가" 안내 한 줄. 그룹 타일인 채
   `#activity` · `#agents` 로 오면 '나' 로 바꿔 연다(1g 의 상대 타일 규칙과 같음).
8. **그룹 사이드바 대화 목록** — `convSections` 에 그룹 대화만 넣는다: ★(그룹 안) + 상대별 절(레일 순서, 지운 상대 뒤 — A11) · 머리에 "대화 추가".
   접힘 열쇠는 '나' 와 같은 화면 열쇠를 함께 쓴다(`convs:{상대}` — 설정 키 · 형식 그대로, 그룹마다 따로 기억하지 않음).
9. **타일 메뉴** — 레일 타일 우클릭(키보드 메뉴 키 · Shift + F10 도 같은 `contextmenu`) 과 사이드바 머리 ✎ 옆 **⋯** 가 같은 팝오버를 연다: "타일 편집"
   (지금 ✎ 폼 — 대상 타일을 인자로) · "이미지 지우기"(그림이 있을 때) · 그룹이면 "그룹 삭제"(확인 한 번 — "대화는 지우지 않습니다"). 그룹의 "타일
   편집" 이름 = 그룹 이름(바꾸면 `PATCH`, 409 문구를 폼에).
10. **"그룹 +"** — 그룹 타일들 아래 버튼 → 이름 한 칸 팝오버 → 만들고 그 그룹을 고른다. 그룹 타일 = 이름 머리글자(`tileText`) 또는 그림, 색 = 그룹 id
    기준 5색.
11. **타일 색 고정**(intent 밖 — plan 승인 때 따로 확인) — 상대 타일 색을 자리 대신 **상대 id** 기준으로(`tileColor`) — 지금은 자리 기준이라 끌어
    옮기면 색이 바뀐다. id 기준이면 색이 타일을 따라가지만, 업데이트 직후 한 번 기존 색이 바뀔 수 있다.
12. **대화 ⋯ 메뉴** — 사이드바 대화 행(마우스를 올리거나 초점이 오면 보이는 ⋯) · 대화 머리 ⋯: "그룹" ▸ 그룹마다 체크(누를 때마다 `PUT …/conversations`
    add · remove) · 그룹 타일을 고른 채면 "그룹에서 빼기". 그룹이 없으면 "그룹 +" 안내 한 줄.
13. **"대화 추가"** — 그룹 사이드바 대화 목록 머리 버튼 → 팝오버: 제목 검색 칸 + 모든 대화 체크 목록(지금 소속은 체크된 채, 상대 이름 · 제목) · "적용"
    = add · remove 한 번에.
14. **드래그 앤 드롭** — 브라우저 기본 HTML 드래그(`draggable` · `dragstart` · `dragover` · `drop`, 라이브러리 없음). 끄는 것 종류(`peer` · `group` · `conv`)와
    대화의 절(★ · 상대 id)을 `dataTransfer` 에 싣고, **같은 종류 · 같은 절**에만 놓인다(놓을 수 없으면 `dropEffect = "none"`). 놓을 자리 = 대상 위쪽 반 →
    앞, 아래쪽 반 → 뒤(선 표시). 대화를 **그룹 타일**에 놓으면 그 그룹에 넣는다(A10). '나' · "+" · 설정 · "그룹 +" 는 끌리지도 놓이지도 않는다.
15. **키보드** — 초점이 상대 · 그룹 타일 · 대화 행에 있을 때 Alt + ↑ / ↓ = 같은 묶음 · 같은 절 안에서 한 칸(끝이면 그대로) · 초점 유지 · 화면 읽기
    프로그램용 알림(`aria-live` "n번째로 옮김").
16. **여러 창** — 보내고 받은 뒤 그 창은 바로 다시 그리고, 다른 창은 다음 새로 고침(5초)에 그룹 · 순서를 다시 읽는다. 동시에 옮기면 마지막 저장이 이긴다
    (1g 접힘과 같은 수준 — 위험 수용).

## 설계

### `an-core`
- `db.rs` — 스키마 16(해석 1 · 2), `SCHEMA_VERSION = 16`.
- `groups.rs`(새) — `list` · `add` · `rename` · `set_icon` · `remove` · `change(group, add, remove)` · `clean_name`. 대화 INSERT 의 `pos`(chat.rs) ·
  `chat::reorder(kind, ids)`(해석 4 — 모르는 id 거절) · `Summary.pos` · `messages_across` · `saved` 에 `group`.
- 시험 — `tests/store.rs`(스키마 15 → 16 첫 자리 · 새 대화 맨 위 · 순서 바꾸기 · 그룹 넣고 빼기 · 그룹 삭제 뒤 대화 남음 · 같은 이름 거절).

### `an-app` 서버
- `chat_api.rs` — 그룹 라우트(해석 5) · `PUT /api/order` · `group` 거르기 · 새 대화 `group` · 대화 JSON `pos`.
- 시험 — `tests/groups_api.rs`(새: 만들기 · 이름 거절 1 ~ 40자 · 409 · 넣고 빼기 · 삭제 · 순서 · 모르는 id 400 · 새 대화 소속 · 거르기 · 다시 열어도 남음 ·
  피어 프레임 없음 — outbox 0건).

### `an-app` 웹UI
- `view.js`(순수 · node 시험) — `sortByPos` · `moveInOrder(order, id, target, after)` · `stepInOrder(order, id, dir, sameSection)` · `sideNav("group")` ·
  `groupConvs(convs, group)` · `canDrop(drag, target)` · `groupNameError(name, groups)` · `tileColor` id 기준.
- `app.js` — `state.groups` · `state.railGroup` · `renderRail`(그룹 타일 · 구분선 · "그룹 +") · `pickGroup` · 타일 메뉴 · 대화 ⋯ 메뉴 · "대화 추가" ·
  드래그 · Alt + ↑ / ↓ · 목록 `group` 거르기 · 새 대화 `group` · `openTileEdit(target)`.
- `index.html` — 레일 그룹 자리 · 팝오버 셋(타일 메뉴 · 대화 메뉴 · 대화 추가 · 그룹 이름) · ✎ 옆 ⋯ · `aria-live`. `app.css` — 구분선 · 놓을 자리 선 ·
  행 ⋯.
- 시험 — `view.test.mjs` · `ux_literals.rs`(라벨 "그룹 +" · "대화 추가" · "그룹에서 빼기" · "그룹 삭제" · "타일 편집" · "이미지 지우기").

### 문서
- README(무엇을 할 수 있나 — 그룹 · 위치 조정), user-guide(레일 · 그룹 · 타일 메뉴 · 끌어 옮기기 · 키보드), developer-guide(스키마 16 · API), an_core ·
  an_app 매뉴얼. masterPlan — D35 개정(레일 순서 = 손으로 · 그룹 타일) · D37(그룹 — 이 PC 정리용 · 대화 단위 · 다자 채널 아님) · S4 보강 · 개정 기록.

## Stage

| stage | 목표 | 산출 | gate |
|---|---|---|---|
| 1 | 스키마 16 · 그룹 API · 순서 API · 그룹 타일 · 그룹 사이드바 · 대화 ⋯ "그룹" · "대화 추가" · 새 대화 소속 | core · API 시험 · node 시험 | E1 ~ E5 |
| 2 | 타일 메뉴(우클릭 · ⋯) · 이미지 지우기 · 그룹 이름 · 그림 · 그룹 삭제 · 타일 색 id 기준 | node · 문구 시험 | E1 ~ E5 |
| 3 | 드래그 앤 드롭 · 그룹 타일에 놓아 넣기 · Alt + ↑ / ↓ · 이 PC 화면 확인 · 문서 | node 시험 · CDP 확인 · 문서 체커 | E1 ~ E7 |

## 평가 항목

- [ ] E1: A12 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E2: A12 — 우리 코드 경고 0(clippy -D warnings)
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: pending
- [ ] E3: A1·A2·A3·A4·A7·A9·A11 — 그룹 · 순서 API 시험(만들기 · 이름 1 ~ 40자 · 같은 이름 409 · 넣고 빼기 · 여러 그룹 소속 · 삭제 뒤 대화 남음 · 순서
  저장 · 모르는 id 400 · 새 대화 맨 위 · 새 대화 그룹 소속 · `group` 거르기 · 지운 상대 대화 남음 · outbox 0건)
  CHECK: cargo test -p coworkagent-app --test groups_api
  EXPECT: /test result: ok\. (1[2-9]|[2-9][0-9]) passed; 0 failed/
  EVIDENCE: pending
- [ ] E4: A4·A9 — 스키마 16 올리기(첫 자리 = 지금 보이는 순서 · 사본 `.v15.bak`) · 다시 열어도 그룹 · 순서 남음
  CHECK: cargo test -p an-core --test store schema16
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E5: A2·A6·A8·A9·A10·A11 — 화면 로직 node 테스트 75건 이상 · 실패 0(`pos` 정렬 · 전체 순서에 끼우기 · 한 칸 옮기기 경계 · 놓을 수 있는 곳(종류 · 절) ·
  그룹 메뉴 4 · 그룹 대화 거르기 · ★ + 상대별 절 · 지운 상대 뒤 · 이름 거절 · 타일 색 id 기준)
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# pass (7[5-9]|[89][0-9]|[1-9][0-9]{2})\s+# fail 0/
  EVIDENCE: pending
- [ ] E6: A1·A3·A5 — 웹UI 문구 · 자리("그룹 +" · "대화 추가" · "그룹에서 빼기" · "그룹 삭제" · "타일 편집" · "이미지 지우기" · ⋯ · `aria-live`)
  CHECK: cargo test -p coworkagent-app --test ux_literals
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E7: A1 ~ A11 — 이 PC 실측(사람 · CDP 확인, 비파괴 — 실제 상대 · 실제 이력은 건드리지 않고 시험용 그룹만 만들고 지운다): 그룹 만들기 · 이름 거절
  문구 · 대화 ⋯ 체크 · "대화 추가" · "그룹에서 빼기" · 그룹 메뉴 4 · 요약 · 타일 메뉴(우클릭 · ⋯ · 메뉴 키) · 그림 넣고 지우기 · 그룹 삭제 뒤 대화 남음 ·
  끌어 옮기기(그룹 · 대화) · 그룹 타일에 놓아 넣기 · Alt + ↑ / ↓ · 앱 다시 실행 뒤 남음 · JS 오류 없음. 상대가 하나라 상대 타일 끌기 · 여러 상대 절은 E5 가
  잰다.
  EVIDENCE: pending

## 종결 시 할 일

- masterPlan: D35 개정 · D37 추가 · S4 보강 · 개정 기록. README · 매뉴얼.
- 이 PC 반영(릴리스 빌드 · DB 사본 `.v15.bak` 확인).
- GitHub 반영(push)은 사용자 확인 뒤.

## 개정 기록

- 없음
