# plan — 1q.request-overview

> 근거: [intent](intent.md)(승인 2026-10-18 — 결정 1 ~ 14). **구현자는 이 문서만 보고 일한다.** D·S 는 [masterPlan](../../masterPlan.md).
> 기준 커밋: `bb9ff02`(task/1q-request-overview — intent 라운드 2). plan 승인: 2026-10-18(사용자 "의견 수용. 개발진행해." — intent 승인과 함께 개발 진행 지시.
> 아래 "intent 대비 해석" 은 그 지시 아래 구현자가 정한 것 — 보고에서 드러내고, 사용자가 뒤집으면 plan 개정).

## 목표

'요청' 화면을 받은 요청 · 보낸 요청 개요(상태별 건수 · 아코디언)로 열고 상태 바꾸기 단추를 본문 바로 아래 글자 단추로 옮긴다. 내 활동을 "받은 요청 · 내 요청 ·
받은 답 · 실패" 알림함으로 좁히고 내 요청의 상태 바뀜을 안 본 항목으로 올린다. 주고받은 메시지를 상대별 접힘 요약 + 서버 페이지 20건씩 + 행 북마크로 바꾼다.
스키마 · 통신 프레임은 바꾸지 않는다.

### 성공 기준(intent 사본)

- A1: '요청' 메뉴로 들어가면 받은 요청 · 보낸 요청 두 절이 접이식으로 보이고, 절 머리에 전체 건수와 대기 · 수용 · 완료 · 반려 건수가 따로 보인다.
- A2: 탭을 누르면 그쪽 목록만, 눌린 탭을 다시 누르거나 메뉴 '요청' 을 누르면 개요로 돌아간다.
- A3: 요청 목록의 상태 바꾸기 단추가 본문 바로 다음 줄에 글자 단추(✓ 수용 · ✕ 반려 …)로 보이고, 누르면 지금처럼 메모 칸을 거쳐 상태가 바뀐다.
- A4: 내 활동에 받은 요청 · 내 요청(대기 · 수용 · 완료 · 반려) · 받은 답 · 실패만 보이고, 자동 왕복 · 상대 사람의 일반 메시지는 보이지 않는다. 안 본 수도 같은 범위로 센다.
- A5: 내 요청의 상태가 바뀌면 내 활동에 안 본 항목으로 나타난다.
- A6: 주고받은 메시지가 상대별 절 기본 접힘 · 머리 요약으로 보이고, 펼치면 20건씩만 그린다(더 보기).
- A7: 주고받은 메시지 행에서 북마크를 넣고 뺄 수 있고, 말풍선 · 북마크 화면과 같은 상태다.
- A8: 회귀 없음 — `cargo test --workspace` · clippy · node 시험 · 문서 체커 · gate 체커 PASS.
- A9: (개정) `coworkAgent /?` 가 앱이 꺼져 있어도 CLI 명령 · 옵션 · `--json` 출력 형식 · 종료 코드를 표준 출력에 쓰고 0 으로 끝난다.
- A10: (개정) README 에 다른 AI 도구가 `coworkAgent /?` 로 사용법을 확인하고 연동하는 법이 있다.

## 코드 사실(실측, 기준 `bb9ff02`)

- 요청 화면: `index.html` `#requests`(탭 `#req_tab_theirs` · `#req_tab_mine` · `#req_state` · `#req_list`) · `app.js` `loadRequests` / `requestEls` · `view.js`
  `requestRows` · `reqMoves` · `reqResend`. '나' 면 `grouped(screen, rows, toEls)` 가 상대별 `foldEl`(접힘 = `state.folded` · 설정 `ui_folded`, 있으면 접힘).
- 내 활동: `core/src/chat.rs` `ACTIVITY_MESSAGES` · `activity` · `activity_unseen` · `ActivityFilter`(All · Human · Agent · Relay · Failed) · `ActivityKind`
  (Message · Failed · RelayStart · RelayEnd · SyncFailed) · 본 표시 = 상대별 `activity_seen_msg` · `activity_seen_event`(id). `note_failed` 가 `relay_events`
  `failed` 행을 남기는 전례. 받는 쪽 상태 알림 저장 = `agent/src/chat.rs` `inbound_req_state` → `chat::set_req_state`. `relay_events.kind` 는 CHECK 없는 TEXT.
  API `GET /api/activity?kind=all|human|agent|relay|failed`(`app/src/chat_api.rs` `activity`). CLI 는 `/api/activity` 를 쓰지 않는다.
  시험: `core/tests/store.rs` · `core/tests/paths.rs` · `agent/tests/{relay,share,message_tidy}.rs` · `app/tests/{ux_api,message_tidy_api,models_tools}.rs` 가
  `ActivityFilter::Relay` · `kind=relay|human|agent` 를 쓴다.
- 주고받은 메시지: `app.js` `loadPeerMessages`(theirs + mine 각 200 → `peerMessages` 로 합쳐 200) · `rowEls(…, { tile: true })` · `view.js` `inboxRows`.
  `GET /api/messages`(`MessageQuery` — side theirs|mine · peer · group · q · kind · ask · req · limit) → core `messages_across`. 북마크 = `PATCH /api/messages/{id}`
  `{saved}` · `app.js` `toggleBookmark` · `bookmarkButton`(`BOOKMARK_SVG`).
- axum 0.8 — `/api/messages/{id}` 는 PATCH 만, 정적 경로(`/api/messages/delete`)가 이미 옆에 있다.
- 잠금: `app/tests/ux_literals.rs`(PRESENT · ABSENT 표 — `rowEls(rs, "주고받은 메시지 없음", { tile: true })` 1k 행) · `tests/css_tokens.rs`(`:root` 밖 리터럴 0).
  node 시험 112.

## intent 대비 해석 (plan 승인 대상)

1. **요청 개요 데이터** — 개요 · 탭 모두 받은 · 보낸 두 목록을 거르기 없이(`req` 없이, `limit=500`) 받아 화면에서 건수를 세고 상태 거르기를 화면에서 건다. 건수의 한계 =
   쪽마다 최근 500건(`ponytail:` 주석). 탭 화면은 그 쪽만 받는다.
2. **상태 `state.reqBox`** — `null` = 개요(메뉴로 열면 `null`), `"theirs"` · `"mine"` = 그 목록. 눌린 탭을 다시 누르면 `null`.
3. **접힘 열쇠** — 바깥 절 `requests:theirs` · `requests:mine`, 안 상대별 절 `requests:{box}:{상대}`(옛 열쇠 `requests:{상대}` 는 버린다 — 기억된 접힘 하나가 풀릴 뿐).
   처음 접힘인 절(보낸 요청 · 주고받은 메시지 상대 절)은 `open:{열쇠}` 가 `ui_folded` 에 있으면 펼침 — `foldEl` 에 처음 접힘 옵션을 더한다.
4. **머리 문구** — `view.js` `reqSummary(rows)` → `{ total, wait, accepted, done, rejected }`, `reqHead(label, sum)` → `"받은 요청 5 · 대기 2 · 수용 1 · 완료 1 · 반려 1"`(0 상태 뺌,
   전체가 0 이면 `"받은 요청 0"`). 순서 = 대기 · 수용 · 완료 · 반려.
5. **글자 단추** — `button.text-btn`(테두리 · 바탕 없음, `--fs-sm` · `--fw-medium`) + 선 아이콘 `.ico-check` · `.ico-x`(새 마스크 아이콘). 수용 · 완료 = `ok`(✓, `--ok`),
   반려 = `no`(✕, `--danger`), 다시 알리기 = 아이콘 없이 `--accent`. 자리 = 행 링크(`<a>` — 이름 · 상태 · 시각 · 제목 / 본문) 바로 뒤 `div.req-acts`(행 여백과 같은 왼쪽 정렬),
   메모 · "요청자에게 알리지 못함" 은 단추 아래 `div.req-notes`. 메모 칸(`noteForm`)은 단추 줄 안에서 그대로 연다.
6. **내 활동 서버** — `ActivityFilter` = `All · Request · MyRequest · Answer · Failed`, API `kind=all|request|mine|answer|failed`(옛 human · agent · relay 는 400).
   - 받은 요청 = `m.mine = 0 AND m.author = 'human' AND m.request = 1`, 받은 답 = `m.mine = 0 AND m.author = 'agent'` 이고 `reply_to` 가 내 사람 메시지(`mine = 1 AND
     author = 'human'`) — 자동 왕복에서 내 에이전트 글에 단 답은 빠진다. 둘 다 공유 사본(`dir = shared`) 제외, 안 본 = 글 id > `activity_seen_msg`.
   - 내 요청 = `m.mine = 1 AND m.author = 'human' AND m.request = 1 AND m.state <> 'error'`, 시각 = `coalesce(req_at, created_at)`, 안 본 = 그 글의 `relay_events`
     `req_state` 행 id > `activity_seen_event`. 새 `ActivityKind::MyRequest`("my_request").
   - 실패 = 지금 그대로(내 실패 글 + `sync_failed`). 자동 왕복 시작 · 끝은 목록 · 안 본 수에서 뺀다(`ActivityKind::RelayStart` · `RelayEnd` 는 지운다 — 기록 자체는 남는다).
   - 기록: `inbound_req_state` 가 상태를 저장할 때 같은 연결에서 `chat::note_req_state(conn, message_id)`(`relay_events` `req_state` 한 행, 대화 = 그 글의 대화).
     같은 상태 · 메모 재전송은 지금처럼 일찍 돌아가 기록하지 않는다.
   - `activity_unseen` = 받은 요청 · 받은 답 · 실패 글(id) + 실패로 바뀐 글(`failed` 행) + `sync_failed` 행 + `req_state` 가 있는 내 요청(글마다 1).
7. **내 활동 화면** — `#act_kind` select 를 탭 `nav#act_tabs`(`data-kind` = all · request · mine · answer · failed, 이름 전체 · 받은 요청 · 내 요청 · 받은 답 · 실패)로.
   행 라벨(`activityRows`): 받은 요청 = `"받은 요청 · {상태}"`, 내 요청 = `"내 요청 · {상태}"` + 메모(`note`), 받은 답 = `"받은 답"`, 실패 = `"실패"` · `"상대 반영 실패 · {사유}"`.
   열 때 거르기 = 전체.
8. **주고받은 메시지 서버** — `GET /api/messages` 에 `side=both`(= 지금 `mine: None`) · `before`(글 id, 그 id 보다 작은 것만 — 정렬은 그대로 `created_at DESC, id DESC`,
   저장 순서와 id 가 같이 늘어나므로 같은 순서 — `ponytail:` 주석) 추가. 새 `GET /api/messages/peers`(scope = peer | group | 없음) → `[{ peer, count, last }]`
   (`last` = 그 상대 마지막 메시지 JSON, context 글 제외 · 공유 사본 포함 — 지금 목록과 같은 모집단), core `peer_message_counts`.
9. **주고받은 메시지 화면** — 상대별 절(레일 순서 · 메시지 없는 상대 숨김) 기본 접힘(열쇠 `peer:{상대}`, 해석 3 의 `open:`), 머리 = 타일 · 이름 · `"{n}건"` · 마지막
   메시지 한 줄 · 시각. 펼치면 그 상대 20건(`side=both&peer=&limit=20`), 끝에 `"{보인 수} / {전체}"` + "더 보기"(`before` = 보인 마지막 id). 다시 그리기(5초 새로 고침)는
   요약만 받고, 펼친 절은 보인 수만큼 다시 받는다(최대 500). 행 오른쪽 북마크 아이콘(`bookmarkButton` 과 같은 그림 · 클래스 `on`) — 누르면 `PATCH saved` 후 그 행만.
10. **문구 · 잠금** — `ux_literals` 새 1q 행(PRESENT: 개요 · 단추 · 탭 · 더 보기 자리, ABSENT: 옛 `<select id="act_kind">` · `사람 메시지` 옵션 · `.actions` 우하단 단추 자리
    (`requestEls` 의 `class: "actions"`)). 1k 행 `rowEls(rs, "주고받은 메시지 없음", { tile: true })` 은 새 자리로 바꾼다.

## 설계

- `view.js`(export — 파일 전용 함수 아님): `reqSummary` · `reqHead` · `REQ_ACT`(단추 종류 → 아이콘 · 색 클래스) · `activityRows`(라벨 · 메모 · 내 요청) ·
  `ACT_KINDS`(탭 목록) · `peerSummaryRows(items, paired, peerNames)`(상대 절 머리 — 레일 순서 · 건수 · 한 줄 · 시각) · `inboxRows` 에 `saved`.
- `app.js`: `loadRequests`(개요 · 탭) · `requestEls`(단추 자리) · `reqFold` · `foldEl` 처음 접힘 옵션 · 내 활동 탭 · `loadPeerMessages`(요약 + 펼침 페이지 + 더 보기) ·
  `peerRowEls`(북마크 단추).
- `index.html`: `#activity` 거르기 → 탭 · 검색. `#requests` 그대로(목록 자리 `#req_list` 에 개요를 그림).
- `app.css`: `.text-btn` · `.text-btn.ok` · `.text-btn.no` · `.req-acts` · `.req-notes` · `.ico-check` · `.ico-x` · `.box-head`(바깥 절 머리) · `.peer-head`(절 머리 한 줄) ·
  `.more-row` · `.row-mark`(행 북마크 자리). 토큰만.
- core: `ActivityFilter` · `ActivityKind` · `activity` · `activity_unseen` · `note_req_state` · `peer_message_counts` · `messages_across` 에 `before`.
- agent: `inbound_req_state` 에서 `note_req_state`.
- app: `activity` kind 파싱 · `MessageQuery` `side=both` · `before` · `peers` 경로.
- 시험(TDD — 먼저 실패를 본다):
  - node `req_overview_*`(요약 · 머리 문구 · 0 숨김 · 단추 종류) · `activity_rows_*`(새 라벨 · 내 요청 · 메모) · `peer_msgs_*`(절 머리 · 레일 순서 · 숨김 · 북마크 상태).
  - agent `requests.rs` `activity_my_request_*`(상태 바뀜 → `req_state` 행 · 내 활동 맨 위 · 안 본 수 +1 · 재전송 기록 없음 · 본 뒤 0).
  - core `store.rs` `activity_scope`(받은 요청 · 받은 답(내 사람 글에 단 답만) · 실패 · 사람 일반 메시지 · 자동 왕복 기록 · 공유 사본 빠짐 · 거르기별).
  - app `ux_api.rs` `activity_kinds`(새 kind · 옛 kind 400) · `peer_pages`(peers 요약 · both · before · limit 20 · 범위).
  - 기존 시험 중 옛 거르기(relay · human · agent)를 쓰는 것은 새 의미로 고친다(지우지 않는다 — 자동 왕복 기록이 남는 것은 `relay_events` 로 단정).

## Stage

| stage | 목표 | 산출 | gate |
|---|---|---|---|
| 1 | 요청 개요 · 탭 · 글자 단추 | view · app · css · html · node · 문구 잠금 | E3 · E4 · E8 |
| 2 | 내 활동 — core · agent · API · 화면 탭 | Rust · node 시험 | E5 · E6 · E3 |
| 3 | 주고받은 메시지 — 요약 · 페이지 API · 화면 · 북마크 | Rust · node 시험 | E7 · E3 |
| 5(개정) | CLI 도움말 `/?` · `-h` · `--help` · README · 사용자 가이드 §14 | `cli.rs` `is_help` · `help` · `tests/cli.rs` · 문서 | E12 · E13 |
| 4 | 문서(사용자 가이드 · web-ui · an_app · 개발자 가이드 · README 필요 시) · mock 확인 | 문서 | E9 · E10 · E11 |

## 평가 항목

- [ ] E1: A8 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E2: A8 — 우리 코드 경고 0(clippy -D warnings)
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: pending
- [ ] E3: A1·A3·A4·A6·A7·A8 — 화면 로직 node 테스트 119건 이상 · 실패 0(기존 112 + req_overview 4 · activity_rows_tabs 1 · peer_msgs 3 − 죽은 `peerMessages` 시험 1, 기존 시험 회귀 없음)
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# pass (119|1[2-9][0-9])\s+# fail 0/
  EVIDENCE: pending
- [ ] E4: A1·A3 — 요청 개요 시험만 3건 이상(상태별 건수 · 머리 문구 0 숨김 · 상태 거르기와 무관 · 단추 종류 ✓ 수용 · ✕ 반려 · ✓ 완료 · 다시 알리기)
  CHECK: node --test --test-isolation=none --test-reporter=tap --test-name-pattern="^req_overview" crates/app/web/view.test.mjs
  EXPECT: /# pass [3-9]\s+# fail 0/
  EVIDENCE: pending
- [ ] E5: A4 — 내 활동 범위(core): 받은 요청 · 받은 답 · 실패만, 사람 일반 메시지 · 자동 왕복 기록 · 공유 사본 · 내 에이전트 글에 단 답 빠짐, 안 본 수 같은 범위
  CHECK: cargo test -p an-core --test store activity_scope
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E6: A4·A5 — 내 요청 상태 바뀜(두 PC): 수용 · 완료 → 요청자 내 활동 맨 위 · 안 본 수 +1 · 같은 상태 재전송은 기록 없음 · 본 뒤 0
  CHECK: cargo test -p an-agent --test requests activity_my_request
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E7: A6 — API: 상대별 요약(건수 · 마지막) · side=both · before 커서 · limit 20 · 옛 kind 400 · 새 kind
  CHECK: cargo test -p coworkagent-app --test ux_api -- peer_pages activity_kinds
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E8: A1·A2·A3·A4·A6·A7 — 문구 · 자리 잠금(개요 · 글자 단추 · 내 활동 탭 · 더 보기 · 행 북마크, 옛 자리 0)
  CHECK: cargo test -p coworkagent-app --test ux_literals
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E9: A8 — 문서 규약
  CHECK: python tools/check_ddd_docs.py
  EXPECT: PASS
  EVIDENCE: pending
- [ ] E10: A8 — 의존 경계
  CHECK: python tools/check_crates.py
  EXPECT: /crates=4 edges=[0-9]+ violations=0/
  EVIDENCE: pending
- [ ] E11: A1·A2·A3·A4·A5·A6·A7 — 실화면 확인(실제 상대 · 두 PC, 다크 · 라이트): 요청 개요 머리 건수 · 탭 토글 · 글자 단추 → 메모 → 상태 · 내 활동 탭 · 상대가 상태를 바꾸면
  내 활동 맨 위 · 종 점 · 주고받은 메시지 접힘 · 펼침 20건 · 더 보기 · 행 북마크가 말풍선 · 북마크 화면과 같음 · JS 오류 없음.
  EVIDENCE: pending

- [ ] E12: A9 — 실제 실행 파일: `/?` · `-h` · `--help` → 종료 0 · 표준 출력에 명령(ask · start · history) · 종료 코드 · `--json` · progress 줄 · 표준 오류 빔, 다른 자리 `/?` 는 2
  CHECK: cargo test -p coworkagent-app --test cli help_exe
  EXPECT: test result: ok. 1 passed; 0 failed
  EVIDENCE: pending
- [ ] E13: A10 — README 에 `coworkAgent /?` 연동 안내(3곳 이상 — 기능 표 · 연동 절 설명 · 명령 예) · README 사용자 관점 체커
  CHECK: python -c "t=open('README.md',encoding='utf-8').read();print('help_refs=%d section=%d'%(t.count('coworkAgent /?'),t.count('## ')))" && python tools/check_readme.py
  EXPECT: /help_refs=([3-9]|[1-9][0-9]) section=[0-9]+[\s\S]*readme ok/
  EVIDENCE: pending

A8 의 `check_gates` 는 구현 완료 시점에 `eval.md` 평가 원장을 만드는 것으로 충족한다(자기 원장을 읽는 CHECK 는 두지 않는다 — eval-protocol 규약 2).

## 종결 시 할 일

- E11 은 D28 에 따라 `ABANDON`(사유: D28) + 19.prototype-acceptance 에 TC 추가. mock(가상 데이터) 눈 확인은 stage 4 기록.
- masterPlan: D35(내 활동 범위 · 요청 개요 · 주고받은 메시지 접힘) · S4 보강 · 개정 기록. PRODUCT · web-ui · an_app · 사용자 가이드 갱신.
- 이 PC 반영 · GitHub 반영은 사용자 확인 뒤.

## 개정 기록

- 2026-10-18 · intent 개정(CLI 도움말 — 사용자 지시 "`coworkAgent.exe /?` … cli 명령어 출력 기능 병행 개발. readme 에도 …")에 따라 성공 기준 A9 · A10 · Stage 5 ·
  평가 항목 E12 · E13 추가 — 승인 2026-10-18(사용자 지시 자체). `-h` · `--help` 별칭은 구현자 해석(intent 결정 15).
- 2026-10-18 · E3 EXPECT 121 → 119 — 사유: 처음 계산 오류(112 + 3 + 2 + 3 = 120 을 121 로 적음) + 서버 페이지로 바꿔 쓰지 않게 된 `view.js` `peerMessages` 와 그 시험
  1건을 지움(죽은 코드). `activity_rows` 는 옛 시험을 새 범위로 고쳐 1건 + 탭 1건. 승인 2026-10-18(사용자 "plan 개정 승인.")
- 2026-10-18 · E7 CHECK 보정 — 내 활동 kind 시험 이름이 `activity_kinds_scope` 라 `peer_pages` 와 함께 거른다(EXPECT 그대로).
