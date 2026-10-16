# plan — 1j.messenger-ui

> 근거: [intent](intent.md)(승인 2026-10-15 — 라운드 1 · 2 · 3, 디자인 캔버스 판 11). **구현자는 이 문서만 보고 일한다.** D·S 는 [masterPlan](../../masterPlan.md).
> 기준 커밋: `35ac69b`(main — 1h 병합). plan 승인: 2026-10-15 — 사용자 원문 "승인. 진행."(질문 Q1 옛 자유 입력 그룹 → 레일 그룹으로 옮김 · Q2 요청 상태 ·
> 다시 보내기 단추는 말풍선 아래 작은 단추로 남김 · Q3 단계 3개, 단계마다 화면 확인 — 모두 권장안).

## 목표

대화 화면을 메신저형 말풍선 · 대화방 목록(☆)으로 바꾸고, 메뉴 · 상단 바 · 레일 아이콘을 정리하며, 상대를 레일 그룹에 넣는 길(타일 메뉴 · 연결 목록)과 설정 정리를
넣는다. 디자인 캔버스(판 11)가 화면의 기준이다.

### 성공 기준(intent 사본)

- A1: 대화를 열면 글이 말풍선으로 보인다 — 내 글은 오른쪽, 상대 사람 · 에이전트 글은 왼쪽에 이름과 함께, 날짜가 바뀌면 구분선 — 말풍선 · 목록에 "사람 →
  에이전트" 같은 방향 라벨이 없다.
- A2: 대화 머리에 "대화 보기" · "원문 보기" · "북마크 n" · "세부정보" 가 있고 "보낸 메시지" 는 없다 — "북마크 n" 은 이 대화의 북마크 목록을 연다(n = 이 대화에서
  북마크한 글 수).
- A3: 말풍선 우상단 북마크 아이콘을 누르면 그 글이 북마크 항목에 들어가고 다시 누르면 빠진다(북마크한 글은 노란 아이콘이 늘 보임), 복사 아이콘은 본문을
  복사한다 — 말풍선 아래 "북마크" · "복사" 단추가 없다.
- A4: 대화방 목록 줄에 '보냄' · '받음' 표시가 없고, 제목 왼쪽 ☆ 로 즐겨찾기를 넣고 빼며, 대화 머리 ☆ 와 같은 상태다.
- A5: 사이드바 메뉴 — '나' = 개요 | 내 활동 · 요청 · 주고받은 글 | 즐겨찾기 · 북마크 항목 · 에이전트, 상대 타일 = 요청 | 즐겨찾기 · 북마크 항목 · 에이전트,
  그룹 타일 = 개요 | 요청 | 즐겨찾기 · 북마크 항목. "즐겨찾기" 는 그 범위의 ★ 대화방을 보인다. '나' 의 주고받은 글은 상대별 절이다.
- A6: 상단 바에 "받기 가능" 칩이 없고, 프로필 아이콘 색이 상태를 보인다(받기 가능 초록 · 집중 주황 · 앱과 연결 끊김 붉은) — 프로필 메뉴에서 받기 가능 · 집중을
  고른다. 표시 이름은 한 번만 보인다.
- A7: 레일 타일이 사람 모양(나 · 상대) · 사람 + 그림자(그룹) 아이콘에 이름이 아래에 있고, 그림을 넣은 타일은 그림이다.
- A8: 상대 타일 메뉴 "그룹" 체크 · 연결 목록 끌어 옮기기 · "그룹 ▾" 로 상대를 그룹에 넣고 빼면 그 그룹 타일 화면에 그 상대의 대화 전부(뒤에 생긴 대화
  포함)가 보이고, 연결 목록 · 레일이 바로 바뀐다. 연결 목록에서 그룹을 추가 · 이름 바꾸기 · 삭제(대화 · 연결은 남음)하면 레일 그룹 타일에 바로 반영된다.
- A9: 타일을 마우스로 누른 뒤 바로 Alt + ↑ / ↓ 로 자리가 바뀌고, 타일 메뉴 "위로 옮기기 · 아래로 옮기기" 도 같다. 끌어 놓기도 된다.
- A10: 설정 — "페어링" 의 "초대 생성" · "초대 수락" 이 같은 모양이고 "주소 · 코드로 직접 추가" 가 없다, 내 정보에 "읽기 전용" 이 없다, 내 에이전트 · 연결
  목록에 자유 입력 그룹 칸 · "그룹 없음 n" 머리가 없다.
- A11: 루트 `PROJECT.md` 검증 명령이 모두 통과한다.

## 코드 사실(실측, 기준 `35ac69b`)

- **말풍선** `app.js messageEl`(1437) — `.avatar` 첫 글자 · `.msg-meta`(이름 · `kindText` "사람 → 에이전트" 등 · `askTag` · 메모 · 시각) · `.msg-icons`(복사,
  편집 모드 연필) · `bodyPair`(뷰어 + 원문 `pre`) · `.msg-actions`(`messageActions` — 북마크 · 다시 보내기 · 요청 상태). `m.mine` = 이 PC 쪽 글(보낸 대화의
  사람 글 · 받은 대화의 에이전트 답 — `db::MINE_BY_DIR`).
- **대화 머리** `index.html:190 ~ 219` — `star_btn` · 탭 `tab_conv` "대화" · `tab_sent` "보낸 메시지"(`#s/{id}` → `openSent`, 그 상대에게 내가 쓴 글 전부) ·
  `raw_btn`(`show-raw` 전환) · `saved_btn`(세부정보 패널을 열고 북마크 절로 — 북마크하지 않는다) · `detail_btn`.
- **대화방 목록** `convRows` · `convLabel`(tag 보냄 / 받음) · `convSections`(★ 절 먼저) · `app.js convItem`(1073) · `renderConvList`(1111). `Summary` 에 마지막 글
  본문은 없다(`chat.rs:151`). ☆ 는 머리 `star_btn` → `PATCH /api/conversations/{id} {starred}`.
- **주고받은 글** `#p` · `#p/{id}` → `loadPeerMessages`(2411) · `inboxRows`(kind = `kindText` · tag 보냄 / 받음) · `rowEls`. 상대 타일 첫 화면 = `#p/{id}`(D35).
- **상단 바** `focus_chip`(`renderFocusChip` 3124 · `setFocus` 3131 · 프로필 메뉴 "집중 — 요청 받지 않음") · `renderAvatar`(496 — 머리글자 + 이름 → '나 나').
  앱이 죽으면 웹UI 는 모른다 — `disconnected()` 는 토큰 없음 · 401 때만, 5초 새로 고침 실패는 상태 줄에만.
- **레일** `renderRail`(315) — 고른 타일이 서명에 들어 누를 때마다 `replaceChildren` → 누른 단추가 사라져 초점이 `body`(Alt + 방향키 무반응의 원인).
  `renderConvList` 도 같은 구조. 끌기 · 키보드 = `app.js:2775 ~ 2840`(`moveItem` · `stepTarget`).
- **설정** 페어링 `index.html:332 ~ 374`(`window_btn` · `invite_form` · `<details>` "주소 · 코드로 직접 추가" `add_form` → `POST /api/peers {addr, code}`) · 내 정보 "읽기
  전용" `index.html:318 ~ 327` · `kvTable`(805) · 연결 목록 `renderPeerList`(833 — `groupFilter` · `NO_GROUP` "그룹 없음" · 자유 입력 그룹 칸 → `peers.grp`) · 내
  에이전트 `renderAgentList`(977 — 그룹 칸 → `agents.grp`). `peers.grp` · `agents.grp` 를 읽는 곳은 설정 화면뿐.
- **그룹**(#1i) `groups.rs` · `groups_api.rs` · `chat::Scope::Group` = `group_convs` 의 대화만. 웹 `inGroup` · `scopedConvs`. 스키마 17.
- node 시험 83건.

## intent 대비 해석 (plan 승인 대상)

1. **말풍선 쪽** = `m.mine`(이 PC 쪽 글은 오른쪽 — 내 에이전트가 상대 요청에 한 답도 오른쪽, 이름 "내 {에이전트}" 작게). 왼쪽 = 상대 사람 · 상대 에이전트(아바타 ·
   이름 · 시각). 날짜 구분선 = 지역 날짜가 바뀔 때(`view.dayLabel` — 오늘 · 어제 · M.D).
2. **라벨** — 말풍선 · 주고받은 글에서 `kindText`(사람 · 에이전트 · 사람 → 에이전트 …)를 뺀다. 남기는 표시: 요청 칩(`askTag` — "요청 · 대기" 등) · 실행 중 시간 ·
   CLI 출처(이름 "내 AI 에이전트(CLI)" 그대로). 대화방 목록 · 주고받은 글의 보냄 / 받음 표시 없앰.
3. **머리** — 탭 · `panel_sent` · `#s/{id}` 화면을 없앤다(`#s/{id}` 로 오면 `#c/{id}`). "대화 보기" · "원문 보기" = 한 쌍의 전환(`role=tablist`, 지금 `show-raw`
   그대로). `saved_btn` = "북마크 n" — 세부정보 패널을 열고 북마크 절로(n = 이 대화에서 북마크한 글 수).
4. **말풍선 아이콘** — `.msg-icons` 에 북마크 아이콘(넣기 · 빼기, `aria-pressed`, 북마크한 글은 늘 노랑 · 그 밖은 마우스를 올리거나 초점일 때) + 복사. `messageActions` 에서
   북마크를 뺀다 — 남는 다시 보내기 · 요청 상태(수용 · 반려 · 완료 · 다시 알리기)는 그 말풍선 아래 작은 단추(Q2).
5. **대화방 목록** — 줄 = ☆ 단추(넣기 · 빼기 — `PATCH starred`) + 아바타(에이전트 첫 글자, 상대 색) · 제목 · 마지막 글 미리보기(말한 이 + 본문 앞 60자) · 시각 · 안 읽음.
   `★ 즐겨찾기` 절은 없앤다(별은 줄에, 모아 보기는 메뉴 "즐겨찾기"). 서버 `Summary` 에 `last_text` · `last_author`(마지막 글 — 앞 120자) 추가.
6. **메뉴** — `sideNav`: '나' 개요 / 내 활동 · 요청 · 주고받은 글 / 즐겨찾기 · 북마크 항목 · 에이전트, 상대 요청 / 즐겨찾기 · 북마크 항목 · 에이전트, 그룹 개요 / 요청 /
   즐겨찾기 · 북마크 항목. "즐겨찾기" = 새 화면 `#starred`(그 범위의 ★ 대화방 — 대화방 줄 그대로, 상대별 절). 상대 타일 첫 화면 = 그 상대 하나의 개요(`peerSummary`
   상대 하나 — D35 "첫 화면 주고받은 글" 개정). `#p/{id}` 로 오면 그 상대 개요로.
7. **상태 색** — 프로필 아이콘 테두리 · 점: 집중 주황 · 아니면 초록, 앱과 연결 끊김이면 붉은(5초 새로 고침이 네트워크 오류로 연달아 2번 실패 → 끊김, 성공하면 되돌림 —
   `view.presenceOf(focus, offline)`). 프로필 메뉴 = "받기 가능" · "집중" 고르기(`menuitemradio`) + 표시 이름 · 아이콘 · 알림 일시 중지 · 환경설정. `focus_chip` 없앰.
   이름은 한 번(아이콘 없으면 사람 모양).
8. **레일 아이콘 A안** — 타일 = 사람(나 · 상대) · 사람 + 그림자(그룹) SVG(그림이 있으면 그림), 아래 이름(11px · 말줄임 · 툴팁), 레일 칸 60 → 72px. 1h 의
   `tileText` 글자 타일 · `.rail-btn.group` 겹친 카드 · `.tile-text` 를 없앤다(intent 편차).
9. **초점 유지** — `renderRail` · `renderConvList` 가 다시 그리기 전에 초점이 그 안에 있으면 그 항목 열쇠(`data-drag` + `data-id` · `data-conv`)를 기억했다가 새 요소로
   되돌린다. 타일 메뉴에 "위로 옮기기 · 아래로 옮기기"(`moveItem`).
10. **상대 소속(스키마 18)** — `group_peers(group_id, peer_id, PRIMARY KEY)`. 그룹의 대화 = `group_convs` ∪ 그 그룹 상대의 대화(계산 — 뒤에 생긴 대화도).
    `Scope::Group` SQL 에 `OR c.peer_id IN (SELECT peer_id FROM group_peers WHERE group_id = ?)`. `groups::list` 에 `peers`. API `PUT /api/groups/{id}/peers {add,
    remove}`(모르는 상대 → 400). 그룹 삭제 시 `group_peers` 도 지움. 웹 `inGroup(c, g)` = `convs` 또는 `peers`. 상대를 빼도 대화를 하나씩 넣은 것(`group_convs`)은 남는다.
    올릴 때 `peers.grp` 가 있으면 같은 이름 그룹(없으면 만들고 · 이름 규칙 맞춤)에 그 상대를 넣는다(Q1). `agents.grp` 는 읽지 않는다(열은 그대로).
11. **연결 목록** — 레일 그룹별 절(그룹 자리 순, 한 상대는 여러 절에) + "그룹에 없는 연결". 줄 끌기(HTML 기본 드래그) → 다른 절에 놓으면 옮김(새 절 넣기 · 옛 절 빼기,
    "그룹에 없는 연결" 에 놓으면 그 절에서 빼기) · "그룹 ▾"(그룹마다 체크) · "+ 그룹 추가" · 절 머리 이름 바꾸기 · 삭제(확인 한 번). 바꾸면 그룹을 다시 읽어 레일 ·
    목록을 함께 그린다. 지금의 그룹 고르기 칸(`peer_group`)은 없애고 검색 칸은 둔다.
12. **설정 정리** — "페어링" 카드(제목) · "초대 생성" · "초대 수락" 같은 단추 모양 · "주소 · 코드로 직접 추가" `<details>` 와 그 웹 코드 없앰(서버 `POST /api/peers
    {addr, code}` 경로는 시험 · 하위 호환으로 둔다) · 내 정보 · 연결 상세(`kvTable`)의 "읽기 전용" 없앰 · 내 에이전트 그룹 칸 · 그룹 묶기 없앰(평평한 목록).

## 설계

### `an-core`
- `db.rs` 스키마 18(`group_peers` · `peers.grp` 옮기기). `groups.rs` — `Group.peers` · `change_peers` · `unknown_peers` · `remove` 가 `group_peers` 도. `chat.rs` —
  `Scope::Group` SQL · `Summary.last_text` · `last_author`.
- 시험 `tests/store.rs` `schema18_*`(옮기기 · 빈 값 · 같은 이름 · 이름 규칙), `paths.rs` 스키마 번호.

### `an-app` 서버
- `groups_api.rs` `PUT /api/groups/{id}/peers` · 그룹 JSON `peers` · `chat_api.rs` 대화 JSON `last_text` · `last_author`.
- 시험 `tests/groups_api.rs` — 상대 넣기 · 빼기 · 모르는 상대 400 · 그룹 거르기에 그 상대의 뒤에 생긴 대화 포함 · 삭제 시 함께 지움.

### `an-app` 웹UI
- `view.js` — `bubbleSide` · `dayLabel` · `convLabel`(tag 없음) · `roomPreview` · `inboxRows`(kind 없음) · `sideNav` · `presenceOf` · `inGroup`(peers) ·
  `peerSections`(연결 목록 절) · `messageActions`(북마크 뺌). `app.js` — `messageEl` · 머리 · 대화방 줄 · `#starred` · 상대 개요 · 상단 바 · 프로필 메뉴 · 레일 아이콘 ·
  초점 유지 · 타일 메뉴 · 연결 목록 · 설정. `index.html` · `app.css`.
- 시험 `view.test.mjs` · `ux_literals.rs`(새 문구 · 없어진 문구 0).

### 문서
- README · user-guide(4 · 5 · 6 · 7 · 11장) · developer-guide(스키마 18) · an_core · an_app · PRODUCT · masterPlan(D35 · D37 개정 · S4) · 캡처.

## Stage

| stage | 목표 | 산출 | gate |
|---|---|---|---|
| 1 | 대화 화면 — 말풍선 · 날짜 · 라벨 · 머리(대화 보기 · 원문 · 북마크 n · 세부정보) · 말풍선 아이콘 · 대화방 줄(☆ · 미리보기) · 주고받은 글 라벨 | node · 문구 시험 · mock 화면 | E1 ~ E4 |
| 2 | 메뉴(즐겨찾기 · 상대 개요) · 상단 바 · 상태 색 · 프로필 · 레일 아이콘 A · 초점 유지 · 타일 메뉴 옮기기 | node · 문구 시험 · mock 화면 | E1 ~ E4 |
| 3 | 상대 그룹(스키마 18 · API · 타일 메뉴 · 연결 목록) · 설정 정리 · 문서 · 이 PC 실측 | core · API · node · CDP | E1 ~ E10 |

단계마다 mock(가상 데이터) 화면을 사용자에게 보이고 다음 단계로 간다(Q3).

## 평가 항목

- [ ] E1: A11 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E2: A11 — 우리 코드 경고 0(clippy -D warnings)
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: pending
- [ ] E3: A1·A2·A4·A5·A6·A8·A11 — 화면 로직 node 테스트 95건 이상 · 실패 0(말풍선 쪽 · 날짜 구분 · 라벨 없음 · 대화방 미리보기 · 메뉴 세 범위 · 상태 색 · 그룹에 상대
  대화 포함 · 연결 목록 절 · 북마크 작업 빠짐)
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# pass (9[5-9]|[1-9][0-9]{2})\s+# fail 0/
  EVIDENCE: pending
- [ ] E4: A2·A3·A4·A6·A7·A10 — 웹UI 문구 · 자리("대화 보기" · "원문 보기" · "북마크" · "즐겨찾기" · "페어링" · 프로필 상태 · 사람 아이콘, 없어진 것 0 — "보낸 메시지" ·
  "받기 가능" 칩 · "읽기 전용" 표시 · "주소 · 코드로 직접 추가" · "그룹 없음" · 사람 → 에이전트)
  CHECK: cargo test -p coworkagent-app --test ux_literals
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E5: A8 — 상대 그룹 API(넣기 · 빼기 · 모르는 상대 400 · 그 상대의 뒤에 생긴 대화도 그룹 거르기에 · 삭제 시 함께 지움 · 대화를 하나씩 넣은 것은 남음)
  CHECK: cargo test -p coworkagent-app --test groups_api
  EXPECT: /test result: ok\. (1[6-9]|[2-9][0-9]) passed; 0 failed/
  EVIDENCE: pending
- [ ] E6: A8 — 스키마 18(`group_peers` · 옛 자유 입력 그룹 → 같은 이름 레일 그룹 · 사본 `.v17.bak`)
  CHECK: cargo test -p an-core --test store schema18
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E7: A11 — 문서 규약
  CHECK: python tools/check_ddd_docs.py
  EXPECT: PASS
  EVIDENCE: pending
- [ ] E8: A11 — README 사용자 관점
  CHECK: python tools/check_readme.py
  EXPECT: readme ok
  EVIDENCE: pending
- [ ] E9: A11 — 의존 경계
  CHECK: python tools/check_crates.py
  EXPECT: /crates=4 edges=[0-9]+ violations=0/
  EVIDENCE: pending
- [ ] E10: A1·A2·A3·A4·A5·A6·A7·A8·A9·A10 — 이 PC 실측(CDP · 비파괴 — 실제 상대 · 이력은 바꾸지 않고, 즐겨찾기 · 북마크 · 그룹은 바꿨다 되돌린다): 말풍선 쪽 · 날짜 · 라벨 없음 · 머리 ·
  아이콘 북마크 넣고 빼기 · 대화방 ☆ · 메뉴 · 즐겨찾기 화면 · 상태 색 · 프로필 · 레일 아이콘 · 누른 뒤 Alt + ↓ · 상대를 그룹에 넣고 빼기 · 연결 목록 · 설정 · 스키마 18 ·
  JS 오류 없음.
  EVIDENCE: pending

## 종결 시 할 일

- masterPlan: D35 개정(메뉴 · 상대 첫 화면 · 레일 아이콘) · D37 개정(상대 단위) · S4 보강 · 개정 기록. 원장 `#1h R2-F2`(A10 gate — 이 plan 부터 PROJECT.md
  명령 전부 gate). README · 매뉴얼 · 캡처.
- 이 PC 반영(릴리스 빌드 · 사본 `.v17.bak`).
- GitHub 반영 · 릴리스(1h · 1j 함께)는 사용자 확인 뒤.

## 개정 기록

- 2026-10-15 stage 1 — 해석 5 의 `last_text` · `last_author` → `last` 객체(`text` · `author` · `mine` · `agent`) · 미리보기 120자(기존 `preview`) ·
  해석 3 의 `role=tablist` → `role=group` + `aria-pressed`. 근거 [stage1](work/stage1.md).
- 2026-10-15 사용자 지시(stage 1 검토) — 말풍선 최대 폭 = 창(대화 칸)의 90%(긴 에이전트 답도 말풍선, 좌 · 우 붙음으로 구별). stage 2 —
  해석 7 끊김 판정을 `api()` fetch 실패 연달아 2번으로 · 메뉴 "즐겨찾기 n". 근거 [stage2](work/stage2.md).
