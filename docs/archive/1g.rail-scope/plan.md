# plan — 1g.rail-scope

> 근거: [intent](intent.md)(승인 2026-10-14 — 라운드 1 · 2). **구현자는 이 문서만 보고 일한다.** D·S 는 [masterPlan](../../masterPlan.md).
> 기준 커밋: `075c36f`(`task/1g-rail-scope` — 분기점 main). plan 승인: 2026-10-14 — 사용자 원문 "Q1,2,3 의견도 반영하여
> 승인. 개발 진행"(intent 미해소 3건 권장안 + 접힘 기억 + 대기 요청 수).

## 목표

레일 '나' = 모든 상대의 내용을 상대별 절로 묶어 보는 자리(메뉴 6개), 상대 타일 = 그 상대만(메뉴 4개, 첫 화면 주고받은 글). 겹치는 '대화' 안내 ·
상대 타일의 '내 활동' · 사이드바 에이전트 절을 없앤다.

### 성공 기준(intent 사본)

- A1: '나' 를 고르면 사이드바 메뉴가 대화 · 내 활동 · 요청 · 저장된 항목 · 주고받은 글 · 에이전트 6개(이 순서)다.
- A2: 상대 타일을 고르면 메뉴가 요청 · 저장된 항목 · 주고받은 글 · 에이전트 4개(이 순서)이고 '대화' · '내 활동' 은 없다. 첫 화면은 그 상대 주고받은 글
  이고, 보이는 목록 · 화면은 그 상대 것만이다.
- A3: '나' 의 내 활동 · 요청(받은 · 보낸 탭 안) · 저장된 항목 · 주고받은 글 · 에이전트는 상대별 접이식 절(제목 = 상대 이름 · 건수, 레일 순서, 절 안
  최신순, 글 없는 상대는 숨김)로 나온다. 에이전트는 '내 에이전트' 절이 맨 앞. 접은 절은 다시 열어도(새로 고침 · 앱 다시 실행) 접혀 있다.
- A4: '나' 의 '대화' 는 메인에 상대별 요약(대화 수 · 안 읽음 · 대기 요청 수 · 최근 활동 시각 + 그 상대 대화 목록 — 누르면 그 대화)을, 사이드바 대화
  목록은 ★ 즐겨찾기 아래 상대별 절을 보인다.
- A5: 사이드바에 '내 에이전트' · '{상대}의 에이전트' 절이 없다.
- A6: 루트 `PROJECT.md` 검증 명령이 통과한다.
- A7: 상대 타일을 고른 채 내 활동 주소로 오면 '나' 로 바뀌어 열리고, '대화' 주소(`#`)로 오면 그 상대 주고받은 글이 열린다. '나' 내 활동에 '상대' 거르기
  칸이 없다.

## 코드 사실(실측)

- 사이드바 메뉴 = `view.js` `sideNav(tile, unseen, waiting)` — 지금 '나' 5개, 상대 6개(끝에 주고받은 글 `#p/{id}`). 그리기 `app.js` `renderSideNav`.
- 모든 목록은 이미 고른 타일(`state.railPeer`)로 거른다 — 내 활동 `GET /api/activity?peer=` · 요청 · 주고받은 글 `GET /api/messages?peer=` · 저장
  `GET /api/saved?peer=` · 에이전트 화면 `renderAgentsView`('나' = 내 것 + 페어링된 상대 전부를 한 줄로). `peer` 가 없으면 전부 — **서버 변경 없음**.
- 목록 행 `inboxRows` · `requestRows` · `activityRows` 는 상대 id 를 싣지 않는다(이름만) — 묶으려면 행에 `peer` 를 싣는다.
- 대화 목록 JSON 에 `peer_id` · `unread` · `last_state` · `last_at` 이 있다(본문 미리보기는 없다 — 요약에 넣지 않는다). 사이드바 절 = `convSections`
  (★ 즐겨찾기 · 대화).
- '나' 목록은 서버 한 번 읽기 — 건수 상한 `MESSAGES_DEFAULT` = 200(주고받은 글은 받은 · 보낸 합쳐 `peerMessages` 200)이 상대별이 아니라 전체에 걸린다.
  조용한 상대의 오래된 글은 그 절에서 빠질 수 있다(상한 안내 없음 — 지금과 같다).
- 타일 누름 `pickTile` → 열린 대화가 그 타일 것이 아니면 `#`(안내 화면). '나' 일 때만 내 활동 '상대' 거르기 칸(`act_peer`)이 보인다.
- 사이드바 에이전트 절 = `index.html` `#side_agents` · `app.js` `renderSideAgents`. 상대 에이전트 목록(`loadPeerAgents`)은 에이전트 화면 · 새 대화
  칸도 쓴다 — 받기는 남긴다.
- 화면 설정은 서버 설정(`PUT /api/settings`, `run_opts` 검증 · 키) — 웹UI 포트가 실행마다 바뀌어(`127.0.0.1:0`) `localStorage` 는 이어지지 않는다
  (#15 `ui_side_width`). 받은 요청 대기 수 = `loadCounts` 가 읽는 `GET /api/messages?side=theirs&ask=request&req=wait` 목록 길이.
- 문구 시험 `ux_literals.rs` 가 "주고받은 글" 개수를 "받은 글" 개수와 맞춘다(`no_received_label`).

## intent 대비 해석 (plan 승인 대상)

1. **메뉴** — `sideNav(null)` = 대화 `#` · 내 활동 `#activity` · 요청 `#requests` · 저장된 항목 `#saved` · 주고받은 글 `#p` · 에이전트 `#agents`.
   `sideNav(id)` = 요청 · 저장된 항목 · 주고받은 글 `#p/{id}` · 에이전트. 배지(내 활동 안 본 수 · 요청 대기 수)는 그대로.
2. **묶음 하나** — `view.js` `peerGroups(rows, peers)` → `[{peer, name, rows}]`: 레일 순서(페어링된 상대 순), 그 뒤 페어링 풀린 상대(id 순), 행 없는
   상대는 뺀다, 절 안 순서는 들어온 순서(이미 최신순). 행에 `peer` 를 싣는다. 절 = `<details>`(제목 `{이름} · {건수}`), 기본 펼침.
3. **사이드바 대화 목록('나')** — ★ 즐겨찾기 절 하나를 맨 위, 그 아래 상대별 절(제목 = 상대 이름, 절마다 `+` = 새 대화). 상대 타일이면 지금처럼
   ★ · 대화 두 절.
4. **'나' 의 '대화'(`#`) = 상대별 요약** — 상대마다 머리(이름 · 접속 상태 · 대화 n · 안 읽음 n · 대기 요청 n · 최근 활동 시각) + 그 상대 대화 행(제목 · 상태 · 안
   읽음, 최근 활동 순 — 누르면 그 대화). 대화 없는 상대도 머리는 보인다(대화 0 · "새 대화" 링크). 페어링된 상대가 없으면 지금 안내 문구.
5. **타일과 주소** — 상대 타일을 누르면 열린 대화가 그 상대 것이 아니면 `#p/{id}`. 상대 타일인 채 `#` 로 오면 `#p/{id}` 로, `#activity`(종 ·
   직접 주소)로 오면 '나' 로 바꿔 연다. `#p`(id 없음)는 '나' 로 바꿔 전체 주고받은 글.
6. **내 활동 '상대' 거르기 칸 제거** — 상대별 절이 대신한다. 종류 · 본문 검색은 그대로. 본 표시 범위 = 전부(지금 '나' 와 같다).
7. **에이전트 화면('나')** — 절: '내 에이전트'(맨 앞) · 상대마다 '{이름}의 에이전트'. 상대 타일이면 그 상대 절 하나.
8. **사이드바 에이전트 절 제거** — `#side_agents` · 제목 · `renderSideAgents` 와 부르는 곳.
9. **접힘 기억** — 설정 `ui_folded` = 접은 절 열쇠 목록 `["{화면}:{상대 id}" …]`(화면 = `home` · `convs` · `activity` · `requests` · `saved` · `peer` ·
   `agents`, '내 에이전트' = `agents:me`). 검증: 300개까지, 열쇠마다 `^[a-z]+:([0-9]+|me)$` · 32자까지 — 어기면 400. 저장 = 쉼표로 이은 한 값(빈 값 =
   없음). 절을 접거나 펴면 그 목록을 통째로 보낸다.
10. **대기 요청 수** — '나' 일 때 `loadCounts` 가 받은 대기 목록을 상대별로 센다(`waitingByPeer`) → 요약 머리 "대기 요청 n"(0 이면 뺀다).

## 설계

### `an-core` · `an-app` 서버 (설정 키 하나)
- `run_opts.rs` — `FOLDED_KEY` · `clean_folded` · `folded(conn)`. `notify.rs` — `SettingsPatch.ui_folded` · 설정 JSON `ui_folded`.
- 시험 — `ux_api.rs` 설정 왕복 · 거절 1건.

### `an-app` 웹UI
- `view.js` — `sideNav` · `route`(`#p` → `{view:"peer", id:null}`) · `peerGroups` · `peerSummary(convs, peers)` · `convSections(rows, byPeer)` ·
  `inboxRows` · `requestRows` · `activityRows` 행에 `peer` · `waitingByPeer`.
- `app.js` — `renderSideNav` · `pickTile` · `render`(해석 5) · `loadActivity` · `loadRequests` · `loadSaved` · `loadPeerMessages(null)` ·
  `renderAgentsView` · `renderConvList` · 홈 요약 · 사이드바 에이전트 절 제거.
- `index.html` — 사이드바 에이전트 절 · `act_peer` 칸 제거, 홈 절에 요약 자리, 안내 문구. `app.css` — 절(`details`) 머리.
- 시험 — `view.test.mjs`(메뉴 · 경로 · 묶음 · 요약 · 대화 절) · `ux_literals.rs`(메뉴 문구 · 사이드바 에이전트 절 없음).

### 문서
- README(무엇을 할 수 있나), 매뉴얼 user-guide(레일 · 사이드바 절) · developer-guide(웹UI 화면) · an_app. masterPlan(D35 · S4 보강 · 개정 기록).

## Stage

| stage | 목표 | 산출 | gate |
|---|---|---|---|
| 1 | `view.js` 로직 — 메뉴 · 경로 · 묶음 · 요약, 설정 `ui_folded` | node 시험 · 설정 시험 | E1 · E3 · E6 |
| 2 | `app.js` · `index.html` · `app.css` 연결, 이 PC 화면 확인 | 문구 시험 · CDP 확인 | E1 ~ E6 |
| 3 | README · 매뉴얼 · masterPlan | 문서 체커 | E1 ~ E6 |

## 평가 항목

- [ ] E1: A6 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E2: A6 — 우리 코드 경고 0(clippy -D warnings)
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: pending
- [ ] E3: A1·A2·A3·A4·A6·A7 — 화면 로직 node 테스트 54건 이상 · 실패 0(메뉴 '나' 6 · 상대 4 / `#p` 경로 / 상대별 묶음 — 레일 순서 · 빈 상대 숨김 · 풀린 상대 뒤 / 대화 요약 — 수 · 안 읽음 · 최근 / ★ 아래 상대별 대화 절 / 입력칸만 새로 고침을 미룸 · 접힘 저장 마지막 값만(개정 1))
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# pass (5[4-9]|[6-9][0-9])\s+# fail 0/
  EVIDENCE: pending
- [ ] E4: A1·A2·A5 — 웹UI 문구 · 자리(메뉴 라벨 · 사이드바 에이전트 절 없음 · `act_peer` 없음)
  CHECK: cargo test -p coworkagent-app --test ux_literals
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E5: A1·A2·A3·A4·A5·A7 — 이 PC 실측(사람 · CDP 확인, 비파괴): '나' 메뉴 6 · 요약 · 상대별 절 · 접힘이 새로 고침 뒤에도 남음 / 상대 타일 메뉴 4 ·
  첫 화면 주고받은 글 · 그 상대만 / 상대 타일인 채 `#activity` → '나' / 사이드바 에이전트 절 없음 / 절 머리 초점 중에도 목록 새로 고침(개정 1) / JS 오류 없음. 페어링된 상대가 하나라 여러 상대
  묶음은 E3 이 잰다.
  EVIDENCE: pending
- [ ] E6: A3 — 접힘 설정 `ui_folded` 왕복 · 거절(형식 · 개수)
  CHECK: cargo test -p coworkagent-app --test ux_api folded
  EXPECT: test result: ok. 1 passed; 0 failed
  EVIDENCE: pending

## 종결 시 할 일

- masterPlan: D35(레일 범위) 추가 · S4 보강 · 개정 기록.
- README, 매뉴얼.
- GitHub 반영(push)은 사용자 확인 뒤.

## 개정 기록

- 개정 1(2026-10-14, 적대적 평가 라운드 1/2 — high 1 · medium 1 사실): 해석 2 · 9 — (1) 배경 새로 고침은 **입력칸**(`input` · `textarea` · `select`)에
  초점이 있을 때만 미룬다(`view.holdsRender`) — 절 머리(`summary`)를 눌러 초점이 남으면 목록이 멈췄다. (2) 접힘 저장은 한 번에 하나, 보내는 동안 바뀐
  값은 마지막 목록만 이어서 보낸다(`view.SendLatest`) — 동시에 보낸 옛 목록이 늦게 닿아 새 값을 덮지 않게. E3 55 → 57건, E5 에 "summary 초점 중에도
  목록 새로 고침" 추가. 여러 웹UI 창의 마지막 저장 우선은 남는다(eval "미조치 지적").
