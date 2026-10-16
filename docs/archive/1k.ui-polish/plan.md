# plan — 1k.ui-polish

> 근거: [intent](intent.md)(착수 인터뷰 2026-10-15). **구현자는 이 문서만 보고 일한다.** D·S 는 [masterPlan](../../masterPlan.md).
> 기준 커밋: `7420dba`(main — 1j 병합). plan 승인: 2026-10-15 — 사용자 원문 "의견 수용. 진행해."(수락 제출 단추 "추가 · 승인" · 안내 "보내고" — 권장안).

## 목표

1j 뒤 남은 안내문 · 단추를 정리하고, 주고받은 글에 상대 타일 · 에이전트 칩을, 에이전트 화면에 편집 연필을 넣으며, 레일 '+' 를 페어링 카드로 잇고, 설정을
디자인 캔버스 ④(https://claude.ai/artifact/9vjfbXuQEpcHcWFYY9q1yy — 판 11 `Settings.dc.html`)대로 바꾼다.

### 성공 기준(intent 사본)

- A1: 에이전트 화면에 그 안내문이 없고, 내 에이전트 줄마다 오른쪽 ✎ 가 있어 누르면 설정의 그 에이전트 칸이 펼쳐진 채 보인다. 상대 에이전트 줄에는 ✎ 가 없다.
- A2: 주고받은 글에 그 안내문이 없고, 줄마다 왼쪽에 상대 타일, 이름 옆에 에이전트 칩이 있다.
- A3: 대화 말풍선 아래에 "다시 보내기" 가 없다.
- A4: 레일 '+' 를 누르면 설정에서 페어링 카드가 맨 위에 오고 초대 수락 칸에 바로 붙여 넣을 수 있다.
- A5: 설정이 디자인 캔버스 ④ 와 같은 배치(두 열 · 페어링 단추 두 개 · 연결 목록 줄 타일 · 관리 · 내 에이전트 줄 타일 · 관리 · 에이전트 등록)다.
- A6: 루트 `PROJECT.md` 검증 명령이 모두 통과한다.

## 코드 사실(실측, 기준 `7420dba`)

- 에이전트 화면 `index.html` `#agents_view` 안내 `<span class="hint">상대 에이전트를 누르면 …</span>` · `app.js renderAgentsView` — 내 에이전트 줄 = `<a href="#settings/agents">`,
  상대 줄 = `newChat`. 설정 내 에이전트 = `renderAgentList` 의 `remembered("agent"+id, …)`(펼침 = `state.open`).
- 주고받은 글 `index.html` `#peer` 안내 `<span class="hint">주고받은 글 — 모든 대화 …</span>` · `loadPeerMessages` → `view.inboxRows`(`who` · `kind` · `room` · `peer`)
  → `app.js rowEls`(내 활동 · 요청 · 북마크 항목도 같은 `rowEls`).
- 다시 보내기 `view.messageActions` 첫 항목 `{key: "resend"}` → `app.js messageAct`.
- 레일 '+' `$("rail_add")` → `goSettings("invite_accept", "invite_text")` — `#invite_accept` 절로 스크롤(카드 중간). 설정 트리 '연결' = `goSettings("conn")`.
- 설정 `.settings-body`(한 열, `max-width: 1000px`) — 카드 `#me` · `#conn`(페어링 — `window_btn` · `invite_box` · `invite_form` 늘 보임) · `#peers`(연결 목록, 줄 =
  `remembered` `<details>`) · `#agents`(평평한 목록 + `<details class="new-agent">` 등록) · 실행 · 화면 · 시작 · 알림.
- 타일 얼굴 `app.js tileFace(icon, group)` · `tileColor(p)`(`colorOf`). node 시험 96건.

## intent 대비 해석 (plan 승인 대상)

1. **연필** — 내 에이전트 줄 오른쪽 `icon-btn` ✎(`aria-label` "{이름} 편집") → `state.open` 에 `agent{id}` 를 넣고 `goSettings("agents")`, 그 칸에 초점. 줄 자체를 누르면
   지금처럼 설정으로(연필과 같은 곳).
2. **주고받은 글 줄** — `rowEls` 에 선택 칸: `inboxRows` 행에 `agent`(보낸 대화 = `conv.agent`, 받은 대화 = `내 {conv.agent}`) 추가, 주고받은 글만 줄 왼쪽에 상대
   타일(`tileFace` + 상대 색, 30px) · 이름 옆 에이전트 칩. 내 활동 · 요청 · 북마크 항목 줄은 그대로.
3. **다시 보내기** — `messageActions` 에서 `resend` 를 빼고 `messageAct` 의 다시 보내기 갈래 · 안 쓰게 된 도움 함수를 지운다.
4. **'+'** — `goSettings("conn")` 과 같이 페어링 카드를 맨 위로(`scrollIntoView({block: "start"})`), 초대 수락 칸을 펼치고 초점(`preventScroll` — 카드가 위에 남게).
5. **설정 두 열** — `.settings-body` 를 격자: 왼쪽 `#conn`(페어링) · `#peers`(연결 목록), 오른쪽 `#me`(내 정보) · `#agents`(내 에이전트), 나머지 카드는 아래 두 열
   전체 폭. 좁은 창(<1100px)이면 한 열.
   - 페어링 = 제목 · 안내 · 같은 `primary` 큰 단추 둘 "초대 생성"(＋ 아이콘) · "초대 수락"(→ 아이콘). "초대 생성" = 지금처럼 초대를 만들고 초대 칸 · "초대 복사"
     를 보인다. "초대 수락" = 붙여 넣기 칸(`invite_form`)을 펼친다(처음엔 접힘, 다시 누르면 접음). 수락 제출 단추 "추가 · 승인"(1j "초대 수락" 과 이름이 겹쳐
     되돌림).
   - 연결 목록 줄 = `⋮⋮` · 상대 타일(36px) · 이름 · 부제(`{name} · {host} · {접속 상태}`) · "그룹 ▾" · "관리"(누르면 그 줄 아래 상세 — 지문 · 주소 · 모델 · effort ·
     저장 · 승인 · 연결 확인 · 삭제 — 열고 닫음, 펼침은 `state.open`). 절 머리 = 그룹 타일(26px, 그룹 밖 절은 없음) · 이름 · 수 · 이름 바꾸기 ✎ · 삭제(휴지통
     아이콘 · 붉은 글자).
   - 내 에이전트 줄 = 타일(36px) · 이름 · 폴더(mono) · "관리"(상세 — 폴더 · 도구 · 추가 지시 · 저장 · 삭제). 머리 오른쪽 "에이전트 등록" 단추 = 등록 칸 펼침.
6. **문구** — 캔버스 안내 "메신저로 보내고" 는 1j 처럼 "보내고"(화면 "메신저" 금지 시험).

## 설계

- `view.js` — `inboxRows` 행 `agent` · `messageActions`(resend 없음). 시험: `inbox_rows_agent` · `message_actions_no_resend` · 고친 `message_actions` · `inbox_rows`.
- `app.js` — `renderAgentsView` 연필 · `rowEls(rows, empty, opts)` 의 `tile` 선택 · `loadPeerMessages` 가 켬 · `messageAct` 정리 · `rail_add` · 페어링 단추 ·
  `renderPeerList` 줄(`관리` 열고 닫기) · `renderAgentList` 줄 · "에이전트 등록". `index.html` — 안내문 두 줄 없앰 · 페어링 카드 · 내 에이전트 머리 · 설정 두 열 틀.
  `app.css` — 두 열 격자 · 줄 · 큰 단추.
- `ux_literals.rs` — 새 문구 · 없어진 것(두 안내문 · 다시 보내기 단추) 시험.
- 문서 — 사용자 가이드(에이전트 · 주고받은 글 · 대화 · 설정 · 레일 +) · an_app · README(필요하면) · 캡처(설정 · 에이전트 · 주고받은 글).

## Stage

| stage | 목표 | 산출 | gate |
|---|---|---|---|
| 1 | 에이전트 연필 · 주고받은 글 타일 · 칩 · 다시 보내기 없앰 · '+' · 설정 두 열 · 문서 · 이 PC 실측 | node · 문구 시험 · mock 화면 · CDP | E1 ~ E8 |

단계 끝 mock(가상 데이터) 화면을 사용자에게 보인다.

## 평가 항목

- [ ] E1: A6 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E2: A6 — 우리 코드 경고 0(clippy -D warnings)
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: pending
- [ ] E3: A2·A3·A6 — 화면 로직 node 테스트 98건 이상 · 실패 0(주고받은 글 에이전트 칩 · 다시 보내기 없음)
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# pass (9[8-9]|[1-9][0-9]{2})\s+# fail 0/
  EVIDENCE: pending
- [ ] E4: A1·A2·A3·A4·A5 — 웹UI 문구 · 자리(연필 · 상대 타일 · 칩 · 페어링 단추 둘 · 관리 · 에이전트 등록 · 두 열, 없어진 것 0 — 두 안내문 · "다시 보내기")
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
- [ ] E8: A1·A2·A3·A4·A5 — 이 PC 실측(CDP · 비파괴 — 실제 상대 · 이력 · 설정 값은 바꾸지 않음): 에이전트 연필 → 설정 그 에이전트 펼침 · 주고받은 글 타일 · 칩 ·
  다시 보내기 없음 · '+' → 페어링 카드 맨 위 · 수락 칸 초점 · 설정 두 열 · 관리 열고 닫기 · JS 오류 없음.
  EVIDENCE: pending

## 종결 시 할 일

- masterPlan: S4 보강(설정 배치 · 에이전트 편집 · 주고받은 글 타일) · 개정 기록. 원장 없음(예정).
- 이 PC 반영(릴리스 빌드). GitHub 반영 · 릴리스(1h · 1j · 1k)는 사용자 확인 뒤.

## 개정 기록

- 없음
