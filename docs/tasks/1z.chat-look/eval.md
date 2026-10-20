# eval.md — 1z.chat-look 적대적 평가

## 평가 원장

<!-- tools/check_gates.py 산출 — 손으로 고치지 않는다. 단 CHECK 가 없는 수동 gate 의
     EVIDENCE 와 ABANDON 줄은 사람이 적고, 재실행해도 보존된다.
     WEAK 줄은 기계가 매 실행 다시 단다 — 약한 gate 를 셀 수 있게 하는 표시이며
     판정에는 영향이 없다. -->

측정 커밋: 8d5322bc202fae0be087f53d9aef614422f17f5b

- [x] E1: A10 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

- [x] E2: A10 — clippy 경고 0
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.23s

- [x] E3: A10 — 웹UI 화면 로직 시험 전부(지금 151 + 새 시험)
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# pass (15[5-9]|1[6-9][0-9])\s+# fail 0/
  EVIDENCE: # skipped 0 / # todo 0 / # duration_ms 161.1594

- [x] E4: A8 — 의견 여러 줄 · 거부 · 호환 형태(core)
  CHECK: cargo test -p an-core --test requests note_
  EXPECT: /test result: ok\. 2 passed; 0 failed/
  EVIDENCE: test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.00s

- [x] E5: A8 — 의견 줄바꿈이 요청자에게 그대로 간다 · 거부 규칙(agent)
  CHECK: cargo test -p an-agent --test requests -- receiver_moves_state_with_note note_lines_reach_requester
  EXPECT: /test result: ok\. 2 passed; 0 failed/
  EVIDENCE: test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 8 filtered out; finished in 0.29s

- [x] E6: A3·A4·A7·A9 — 발화자 아이콘 · 행 author · 레일 경계 · 글자 수(node)
  CHECK: node --test --test-reporter=tap --test-name-pattern="speaker_icon|inbox_rows_author|rail_size|note_count" crates/app/web/view.test.mjs
  EXPECT: /# pass 4\s+# fail 0/
  EVIDENCE: # skipped 0 / # todo 0 / # duration_ms 65.1014

- [x] E7: A1·A2·A5·A9 — 말풍선 · 아이콘 · 입력창 · 레일 손잡이 코드 실재
  CHECK: python -c "import pathlib;c=pathlib.Path('crates/app/web/app.css').read_text(encoding='utf-8');j=pathlib.Path('crates/app/web/app.js').read_text(encoding='utf-8');h=pathlib.Path('crates/app/web/index.html').read_text(encoding='utf-8');print('recv=%d agentb=%d av=%d chip=%d margin=%d minh=%d maxh=%d handle=%d drag=%d'%(c.count('.msg.side-theirs:not(.agent) .bubble { background: var(--recv-bg)'),c.count('.msg.agent .bubble { background: var(--surface); border: 1.5px solid var(--agent)'),j.count('class: \x22avatar agent-av\x22'),j.count('text: \x22에이전트\x22'),(c.count('--chat-gutter: max(48px, 8%);')==1)*c.count('var(--chat-gutter)'),c.count('min-height: 44px; max-height: 200px'),j.count('200) + \x22px\x22'),h.count('id=\x22rail_resize\x22'),j.count('saveTileSize(railSize(')))"
  EXPECT: recv=1 agentb=1 av=1 chip=1 margin=2 minh=1 maxh=1 handle=1 drag=1
  EVIDENCE: recv=1 agentb=1 av=1 chip=1 margin=2 minh=1 maxh=1 handle=1 drag=1

- [x] E8: A3·A7 — 목록 카드 · 의견 칸 코드 실재
  CHECK: python -c "import pathlib;c=pathlib.Path('crates/app/web/app.css').read_text(encoding='utf-8');j=pathlib.Path('crates/app/web/app.js').read_text(encoding='utf-8');n=j[j.index('function noteForm'):j.index('function noteForm')+2500];print('card=%d speaker=%d ta=%d rows=%d max=%d cnt=%d enter=%d btn=%d wrap=%d'%(c.count('.side-list li:not(.hint) { background: var(--surface)'),j.count('speakerEl(c.last.author)'),n.count('el(\x22textarea\x22'),n.count('rows: \x223\x22'),n.count('maxlength: \x22200\x22'),n.count('noteCount('),n.count('enterSends(ev)'),c.count('.req-note button { height: 44px; }'),c.count('.feedback-note { white-space: pre-wrap; }')))"
  EXPECT: card=1 speaker=1 ta=1 rows=1 max=1 cnt=2 enter=1 btn=1 wrap=1
  EVIDENCE: card=1 speaker=1 ta=1 rows=1 max=1 cnt=2 enter=1 btn=1 wrap=1

- [x] E9: A6 — 사용자에게 보이는 "피드백" 0 · "의견" 있음(웹UI 문자열 · 서버 · 문서 4곳)
  CHECK: python -c "import pathlib,re;w=['crates/app/web/app.js','crates/app/web/view.js','crates/app/web/index.html'];d=['docs/manual/user/user-guide.md','docs/manual/dev/web-ui.md','README.md'];lit=lambda s:len(re.findall(r'[\x22\x27\x60>][^\x22\x27\x60<\n]*(피드백|메모:)',s));ws=''.join(pathlib.Path(f).read_text(encoding='utf-8') for f in w);ds=''.join(pathlib.Path(f).read_text(encoding='utf-8') for f in d);r=pathlib.Path('crates/core/src/chat.rs').read_text(encoding='utf-8');print('targets=%d web_old=%d doc_old=%d rs_new=%d web_new=%d doc_new=%d'%(len(w)+len(d),lit(ws),ds.count('피드백'),r.count('의견은 {MAX_REQ_NOTE}자까지'),min(ws.count('요청 의견'),1),min(ds.count('의견'),1)))"
  EXPECT: targets=6 web_old=0 doc_old=0 rs_new=1 web_new=1 doc_new=1
  EVIDENCE: targets=6 web_old=0 doc_old=0 rs_new=1 web_new=1 doc_new=1

- [x] E10: A10 — 문서 규약 · 크레이트 경계 · 화면 문구 목록
  CHECK: python tools/check_ddd_docs.py && python tools/check_crates.py && cargo test -p coworkagent-app --test ux_literals -- literals_present literals_absent literal_counts
  EXPECT: /test result: ok\. 3 passed; 0 failed/
  EVIDENCE: test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.05s

- [x] E12: A11·A13 — 끌기 시작 경계 · 그룹설정 목록 · 고른 항목(node)
  CHECK: node --test --test-reporter=tap --test-name-pattern="past_slop|group_settings_list|group_settings_pick" crates/app/web/view.test.mjs
  EXPECT: /# pass 3\s+# fail 0/
  EVIDENCE: # skipped 0 / # todo 0 / # duration_ms 63.2124

- [x] E13: A11 — 레일 타일 포인터 끌기 실재 · HTML 끌기 없음
  CHECK: python -c "import pathlib;j=pathlib.Path('crates/app/web/app.js').read_text(encoding='utf-8');r=j[j.index('function renderRail'):j.index('function renderRail')+6000];print('grab=%d fn=%d slop=%d move=%d eat=%d html5=%d rail_dnd=%d'%(r.count('onpointerdown: tileGrab'),j.count('function tileGrab(ev)'),j.count('pastSlop(e.clientX - sx, e.clientY - sy)'),j[j.index('function tileGrab(ev)'):j.index('function tileGrab(ev)')+3000].count('moveItem(d.kind, d.id, drop.t.id, drop.after)'),j.count('tile.addEventListener(\x22click\x22, eat'),r.count('draggable'),j.count('$(box).addEventListener(\x22dragstart\x22')))"
  EXPECT: grab=2 fn=1 slop=1 move=1 eat=1 html5=0 rail_dnd=0
  EVIDENCE: grab=2 fn=1 slop=1 move=1 eat=1 html5=0 rail_dnd=0

- [x] E14: A12·A13 — "연결 목록" 0 · "그룹설정" · 두 칸 코드(웹UI · 알림 · README)
  CHECK: python -c "import pathlib;fs=['crates/app/web/app.js','crates/app/web/view.js','crates/app/web/index.html','crates/app/src/notify.rs','README.md'];t=''.join(pathlib.Path(f).read_text(encoding='utf-8') for f in fs);c=pathlib.Path('crates/app/web/app.css').read_text(encoding='utf-8');print('targets=%d old=%d new=%d list=%d detail=%d'%(len(fs),t.count('연결 목록'),min(t.count('그룹설정'),1),t.count('replaceChildren(groupSettingsListEl(list, pick)'),c.count('.gs-detail { flex: 999 1 420px')))"
  EXPECT: targets=5 old=0 new=1 list=1 detail=1
  EVIDENCE: targets=5 old=0 new=1 list=1 detail=1

- [x] E11: A2·A5 — 실 화면: 에이전트 답 말풍선 옆 로봇 아바타(내 쪽 포함) · 스크롤 중 말풍선 경계와 입력창 사이 24px · 좌우 여백
  EVIDENCE: 2026-10-21 Claude(구현 세션) — 이 PC(Windows 11, `tools/rebuild_run.py` 로 b971fca 릴리스 빌드 실행, 실제 사용자 DB) 웹UI 를 ego-browser(Chromium)로 열어 DOM 측정(창 최소화라 스크린샷 불가 — 계산된 레이아웃 · 스타일 값). 대화 #1(보낸 대화): 에이전트 글 2/2 로봇 아바타(`.avatar.agent-av .ico-bot`) · "에이전트" 칩 · 흰 면 + 테두리 rgb(12,108,104). 받은 대화 #2: 내 쪽(오른쪽) 에이전트 글 2/2 로봇 아바타 · 아바타가 말풍선 오른쪽, 상대 사람 글 3 개 면 rgb(253,241,224). 대화 칸 폭 992px 에서 말풍선 영역 좌우 79.375px = 입력창 좌우 margin 79.375px(8%), `#messages` padding-bottom 0, 가운데로 스크롤한 상태에서 스크롤 영역 아래 경계 ↔ 입력창 위 = 24px, 글 칸 min-height 44px(높이 46.4px). 대비(라이트/다크): 에이전트 글 17.47/14.45, 칩 · 아바타 · 목록 발화자 아이콘 6.25/8.58, 상대 사람 글 15.67/12.33. 추가 확인 — 레일 폭 손잡이 실제 마우스 끌기: 138px 에서 작게 유지 · 90px 에서 크게 전환 · 놓으면 72px · 설정 선택 normal, 다시 150px 로 끌어 168px · small, 새로 고침 뒤 유지. 레일 타일 끌기: 사용자 1↔2 · 그룹 12↔11 순서 바뀜(끄는 중 사본 1 · 표시 1, 놓은 뒤 사본 0) 후 원래 순서로 되돌림, 짧은 클릭은 타일 고름, 레일에 `draggable="true"` 0. 그룹설정: 트리 · 제목 "그룹설정", 왼쪽 280px · 오른쪽 611px 나란히, "그룹 3" · "사용자 6" 머리, 접속 점 online/offline, 그룹 이름 "AI Discovery" 검색 → 상세 대화 2 전부. 미확인: 그림 아이콘 타일(이 PC 타일에 그림 없음) · 끌기 중 새 메시지 도착 · 포인터 취소 경로


WEAK: E11 CHECK 없음 — 수동 gate(사람이 적은 EVIDENCE 로 판정)

## 적대적 평가 — 라운드 1/2

평가자: Orca 가 띄운 Codex(`worker-start --agent codex` — 사용자 승인 2026-10-21, 기존 `review` · `review2` Codex 터미널은 `agent_unconfigured` "not running a recognized agent" 로 배정 거절),
handle `term_2f3ad5c7-4177-4207-a2b0-4e099064e0ce`, 작업 폴더 `C:/Users/tskim/.claude/skills`. Run `run_18273b6a83a0` · Task `task_b9ce1790a077` · Dispatch `ctx_dc54437baf09`,
worker_done payload taskId · dispatchId 일치 · outcome succeeded. 범위 `2b68dbc..e6f3080`(self-collect), 메타 `요청 1z-chat-look-r1 · revision e6f308063d58cbb2410a32767077be3d0cb5d9ce · 라운드 1/2`.
리뷰 뒤 작업 폴더 HEAD e6f3080 그대로(다른 세션 PRODUCT.md · security-posture.md 외 변경 없음). 리뷰어 터미널은 라운드 2 에 다시 쓰려 retain. 원문(render):

```text
# Codex Adversarial Review

Target: branch diff against 2b68dbc
Verdict: needs-attention

출하 보류: A9·A11의 끌기 종료 경로와 A13의 검색 상세에 결함 3건이 있다. 고정 diff·명세를 대조하고 최소 Node harness로 검색 및 이벤트 정리 누락을 확인했다. 전체 테스트 재실행·실 브라우저 검증은 하지 않았으며 E11은 여전히 미검증이다.

Findings:
- [medium] 그룹 이름으로 검색하면 해당 그룹의 대화가 상세에서 사라진다 (crates/app/web/app.js:1030-1035)
  renderPeerList는 그룹 이름 일치로 그룹을 검색 결과에 포함하면서 상세에는 list.users의 ID만 전달하고, groupSection은 검색어가 있으면 그 사용자들의 대화만 남긴다. 그룹 이름이 사용자 정보나 대화 제목과 겹치지 않으면 왼쪽에는 '대화 1'인 그룹이 나오지만 오른쪽에는 '맞는 대화 없음'이 나온다. 고정 revision의 view.js를 로드한 Node 재현에서 그룹 '배포 준비', 대화 '로그인 오류', 사용자 'jiwon'을 두고 '배포 준비' 검색 시 groups=[7], users=[], detailRows=[]였다. 검색으로 찾은 그룹의 대화 확인·제거가 불가능해 A13의 검색과 상세 연결이 깨진다. E12는 왼쪽 목록만 시험해 이 경우를 검출하지 않는다.
  Recommendation: 그룹 이름 자체가 일치하면 해당 그룹의 전체 대화를 상세에 제공하고, 대화 제목·사용자 검색에는 동일한 검색 판정을 상세까지 전달하라. 비어 있지 않은 그룹의 이름 검색과 같은 사용자의 여러 대화 제목 검색을 상세 행까지 검증하라.
- [medium] 폭 손잡이의 pointercancel 뒤에도 레일 폭 변경이 계속된다 (crates/app/web/app.js:4278-4287)
  폭 끌기는 window의 pointermove를 등록하고 pointerup에서만 해제한다. pointercancel·캡처 상실·창 이탈에 대한 종료 처리가 없고 손잡이에 포인터 캡처도 없다. 특히 터치 스크롤 등으로 pointercancel이 발생하면 인라인 --rail-w와 리스너가 남는다. 추출한 실제 핸들러의 최소 이벤트 harness에서 pointerdown→pointermove→pointercancel→버튼 없이 pointermove 후에도 폭이 190px로 바뀌었고 move/up 리스너가 각각 1개 남았다. 다음 무관한 pointerup이 이 오래된 끌기 값을 저장하므로 A9의 놓을 때 72/168px 확정·설정 동기화가 깨진다. 실 브라우저의 취소 발생 조건은 미검증이다.
  Recommendation: 활성 pointerId를 추적하고 손잡이에 setPointerCapture를 적용하라. pointerup·pointercancel·lostpointercapture에 공통 정리 함수를 연결해 모든 리스너와 인라인 폭을 제거하고, 성공한 pointerup에서만 저장하라. 취소 후 일반 이동·다음 클릭이 폭과 저장값을 바꾸지 않는지 시험하라.
- [medium] 백그라운드 레일 재렌더링이 진행 중인 타일 끌기를 고아 상태로 남긴다 (crates/app/web/app.js:3897-3899)
  tileGrab은 캡처한 타일에만 pointermove/up/cancel을 연결하고 lostpointercapture를 처리하지 않는다. renderRail은 안 읽음 수·접속 상태 문구가 변하면 drawRailTiles의 replaceChildren으로 모든 타일을 교체한다. 따라서 타일을 끄는 동안 새 메시지나 5초 refresh가 레일 서명을 바꾸면 캡처 대상이 DOM에서 제거된다. 이후 종료 이벤트가 기존 타일에 도달하지 않아 body에 붙인 rail-ghost와 drop 표시가 남고 순서 저장도 실행되지 않는다는 것은 DOM 제거·캡처 동작에 근거한 추론이며 실 브라우저 재현은 확인 필요다. 최소 harness에서는 끌기 시작 뒤 lostpointercapture를 보내도 ghost=1, move/up 리스너 각 1개가 그대로 남았다. E13의 문자열 존재 검사는 이 A11 중단 경로를 덮지 않는다.
  Recommendation: lostpointercapture를 취소 정리 경로에 연결하고 정리를 멱등하게 하라. 끌기 중 타일 DOM 교체를 유예하거나 안정된 상위 요소에서 활성 끌기를 관리하라. 새 메시지·접속 상태 갱신으로 타일이 교체되는 동안 ghost·표시·리스너가 남지 않는지 브라우저 시험을 추가하라.

Next steps:
- 검색 상세 및 포인터 취소·재렌더링 결함을 수정하고 동작 기반 회귀 시험을 추가한다.
- E11 실 화면에서 양쪽 에이전트 아바타·스크롤 간격·두 테마 대비·레일 끌기와 그룹설정 두 칸을 확인한다.
- 수정 revision을 고정해 허용된 라운드 2/2에서 남은 위험을 재평가한다.
```

raw JSON: 보관 `scratchpad/review_1z_r1.json`.

### 판정 · 조치

| 지적 | 판단 | 조치 |
|---|---|---|
| [medium] 그룹 이름으로 검색하면 그 그룹 대화가 상세에서 사라진다 | 사실 | `view.groupSettingsList` 그룹 항목에 `convs`(상세에 보일 대화 — 빈 검색 · 이름이 맞으면 전부, 아니면 제목 · 사용자가 맞는 대화만), `groupSection(s, visible, q)` 가 그것으로 거름. node `group_settings_list` 에 상세 대화 단정 5개(이름 · 사용자와 겹치는 이름 · 제목 · 지문) |
| [medium] 폭 손잡이 pointercancel 뒤에도 폭이 계속 바뀐다 | 사실 | 손잡이에 포인터 캡처, `pointermove` · `pointerup` · `pointercancel` · `lostpointercapture` 를 손잡이에만 걸고 한 번만 정리(`done`), 저장은 `pointerup` 만 · 취소면 시작 때 크기로 되돌림. DOM 동작 시험은 없음(웹UI 시험은 DOM 없는 `view.js` 만 — E11 실 화면에서 확인) |
| [medium] 백그라운드 레일 다시 그리기가 끄는 중인 타일을 고아로 만든다 | 사실(추론 포함 — 실 브라우저 재현 없음) | `railDragging` 동안 `renderRail` 건너뜀(서명을 적지 않아 끝난 뒤 그림), 끝나면 `moveItem` 이 · 취소면 `renderRail()` 로 미룬 그리기, `lostpointercapture` 도 끝 처리 · 정리는 한 번만(`done`). DOM 동작 시험은 없음(E11) |

수정 뒤 `cargo test --workspace` 78 바이너리 481 passed · 0 failed, clippy 0, node 158 pass · 0 fail.

## 적대적 평가 — 라운드 2/2

평가자: Orca 가 새로 띄운 Codex(`worker-start --agent codex` — 라운드 1 리뷰어 터미널 `term_2f3ad5c7…` 재배정은 `agent_unconfigured` 로 거절), handle
`term_21ec3ab5-88d2-47c7-8ae5-a2260df62b15`, 작업 폴더 `C:/Users/tskim/.claude/skills`. Run `run_18273b6a83a0` · Task `task_729f2f03449c` · Dispatch `ctx_c66d968eafa7`,
worker_done payload 일치 · outcome succeeded. 범위 `2b68dbc..636cd7d`(self-collect), 메타 `요청 1z-chat-look-r2 · revision 636cd7d9d9dcb1c7ea9479ae5d04028a71fd361b · 라운드 2/2`.
리뷰 뒤 작업 폴더 HEAD 636cd7d 그대로(다른 세션 PRODUCT.md · security-posture.md 외 변경 없음). 두 리뷰어 Dispatch 는 `worker-release` 요청 → Orca 가 `retained`
(`user_takeover`, 프로세스 동작 없음)로 처리 — 두 Codex 터미널이 Orca 에 남아 있다. 원문(render):

```text
# Codex Adversarial Review

Target: branch diff against 2b68dbc
Verdict: needs-attention

출하 보류: 고정 diff와 A1~A13·E1~E14를 대조했고 R1 세 조치의 코드 반영을 확인했다. 사용자 지정 이미지 타일의 HTML 끌기 잔존과 그룹설정 접속 점의 갱신 누락이 남는다. 전체 테스트 재실행과 실 브라우저 검증은 수행하지 않았으며 E11과 포인터 실제 동작 확인이 남았다.

Findings:
- [medium] 이미지 아이콘에서는 기본 HTML 끌기가 포인터 순서 변경을 가로챈다 (crates/app/web/app.js:3858-3863)
  tileGrab은 pointerdown 기본 동작을 막지 않고 캡처만 설정한다. 사용자·그룹 아이콘을 만드는 tileFace(395~396)는 draggable을 지정하지 않은 img를 반환하며, 이미지가 채운 타일(.rail-btn img는 width/height 100%)에는 기본 이미지 끌기가 그대로 남는다. 타일 버튼의 draggable 및 기존 dragstart 처리만 제거했으므로 이미지 위에서 끌면 브라우저의 HTML drag가 시작되어 pointercancel로 순서 변경이 취소될 수 있다는 추론이다(실 브라우저 재현은 확인 필요). 추출한 tileFace 호출에서도 img에 draggable 속성이 없고 tileGrab/레일에 기본 끌기 차단이 없음을 확인했다. 기본 SVG 타일만 점검하면 놓치는 A11 기능 분기이며, E13의 버튼 문자열 검사는 이를 검출하지 못한다.
  Recommendation: 레일 아이콘 img에 draggable="false"를 지정하고 레일의 native dragstart를 차단하라. 사용자·그룹 모두 사용자 지정 이미지의 중앙에서 끌어 순서 저장 및 뒤따르는 click 억제가 동작하는지 검증하라.
- [medium] 통신이 끊긴 사용자의 접속 점이 계속 온라인으로 남는다 (crates/app/web/app.js:1085-1088)
  새 .gs-dot은 groupSettingsListEl에서만 presence(p)를 계산하지만 renderPeerList의 매 갱신 경로는 기존 [data-peer-sub] 글자만 바꾼다. seen_at이 그대로인 채 마지막 통신 후 75초가 지나도 서명에는 현재 시각/presence 결과가 없어서 needsRender가 반환하며 점의 class·title·aria-label은 갱신되지 않는다. 고정 revision에서 추출한 renderPeerList의 최소 harness에 동일 peer를 두고 80초 뒤 재호출하자 목록 그리기는 1회뿐이었다. 상대 종료 후에도 왼쪽 목록은 '방금 통신' 녹색 점을 유지하면서 선택한 사용자 상세는 오래된 통신으로 표시될 수 있어, 많은 사용자 중 접속 상태를 비교하는 시안 A의 핵심 표시가 틀린다.
  Recommendation: 점에 peer 식별자를 부여하고 needsRender 판정 전에 class·title·aria-label을 부분 갱신하라. 전체 목록 재생성 없이 seen_at 고정 상태의 74→75초 경계 및 focus 전환을 검증하라.

Next steps:
- R1의 그룹 이름 검색 상세·폭 pointercancel·레일 재렌더링 조치는 정적으로 반영됐으나, 실제 캡처 상실·click 억제·새 메시지 도착 중 끌기는 E11에서 확인한다.
- 이미지 타일 native drag 차단과 그룹설정 접속 점 부분 갱신을 반영하고 해당 분기·75초 경계 회귀 검증을 추가한다.
- U+2028 변환과 이전 clean_label 통과는 코드·시험에 부합한다; E11에서 양쪽 아바타·24px 간격·두 테마 대비를 확인하고 라운드 2/2로 리뷰를 종료한다.
```

raw JSON: 보관 `scratchpad/review_1z_r2.json`.

### 판정 · 조치

| 지적 | 판단 | 조치 |
|---|---|---|
| [medium] 이미지 아이콘에서는 기본 HTML 끌기가 포인터 순서 변경을 가로챈다 | 사실(추론 포함 — 실 브라우저 재현 없음) | `tileFace` 그림 `draggable="false"`, 레일 `dragstart` 는 `preventDefault`(타일 순서 · 대화 넣기 모두 포인터 끌기라 HTML 끌기 쓸 곳 없음). `ux_literals` PRESENT 2 |
| [medium] 통신이 끊긴 사용자의 접속 점이 계속 온라인으로 남는다 | 사실 | 점을 `gsDot(p)`(`data-peer-dot`)로 만들고 `renderPeerList` 가 서명 판정 전에 매번 점만 다시 만든다(부제 `data-peer-sub` 와 같은 방식). `ux_literals` PRESENT 1. 75초 경계는 `presence`(기존 · 시험 있음) 그대로 |

수정 뒤 `cargo test --workspace` 78 바이너리 481 passed · 0 failed, clippy 0, node 158 pass · 0 fail. 라운드 상한(2/2) — 재리뷰 없이 종료.

## 미조치 지적

| 지적 | 판단 | 사유 |
|---|---|---|
| R1 폭 손잡이 취소 · R1 타일 끌기 중 다시 그리기 · R2 그림 기본 끌기 — "취소 · 캡처 상실 · click 억제 · 새 메시지 도착 중 끌기를 브라우저 시험으로 검증하라" | 사실(시험 공백) | 코드는 고쳤으나 웹UI 시험은 DOM 없는 `view.js` 순수 함수뿐이라 이벤트 경로 자동 시험이 없다 — E11 실 화면 확인으로 대신(사용자 앱 재실행 확인 대기). DOM 시험 도입(jsdom · Playwright)은 별도 판단 |
| R2 "seen_at 고정 상태 74→75초 경계 · focus 전환을 검증하라"(그룹설정 접속 점) | 사실(시험 공백) | 경계 판정 `presence` 는 기존 시험이 덮고, 이번 수정은 점을 다시 만드는 DOM 경로라 자동 시험 없음 — E11 |
