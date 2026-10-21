# eval.md — 1z-2.ui-refine 적대적 평가

## 평가 원장

<!-- tools/check_gates.py 산출 — 손으로 고치지 않는다. 단 CHECK 가 없는 수동 gate 의
     EVIDENCE 와 ABANDON 줄은 사람이 적고, 재실행해도 보존된다.
     WEAK 줄은 기계가 매 실행 다시 단다 — 약한 gate 를 셀 수 있게 하는 표시이며
     판정에는 영향이 없다. -->

측정 커밋: fbd4fed36c5119eae4f7813a91226a8c4b973941

- [x] E1: A10 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

- [x] E2: A10 — clippy 경고 0
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.21s

- [x] E3: A10 — 웹UI 화면 로직 시험 전부(지금 158 - 1 + 새 시험)
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# pass (1[6-9][0-9])\s+# fail 0/
  EVIDENCE: # skipped 0 / # todo 0 / # duration_ms 164.8764

- [x] E4: A1·A2·A6·A9 — 메뉴 · 상대 타일 · 놓을 자리 · 아바타(node)
  CHECK: node --test --test-reporter=tap --test-name-pattern="^side_nav|rail_slot|tile_unread|avatar_kind|inbox_rows_author" crates/app/web/view.test.mjs
  EXPECT: /# pass 11\s+# fail 0/
  EVIDENCE: # skipped 0 / # todo 0 / # duration_ms 63.3635

- [x] E5: A1·A2·A3·A4·A5 — 메뉴 이름 · 배지 · 개요 아이콘 · 레일 + · 대화 줄 한 줄 코드 실재
  CHECK: python -c "import pathlib;j=pathlib.Path('crates/app/web/app.js').read_text(encoding='utf-8');v=pathlib.Path('crates/app/web/view.js').read_text(encoding='utf-8');h=pathlib.Path('crates/app/web/index.html').read_text(encoding='utf-8');ci=j[j.index('function convItem'):j.index('function renderConvList')];print('menu=%d old=%d leave=%d badge=%d head=%d row=%d plus=%d gplus=%d bot=%d sub=%d prev=%d'%(v.count('...(me ? [{ label: \x22에이전트 응답\x22'),v.count('label: \x22에이전트 대화\x22')+h.count('>에이전트 대화<'),j.count('if (peerLeaves(r.view) && state.railPeer != null) pickRail(null);'),j.count('const unread = tileUnread(state.convs);'),j.count('rowTile(`small ${colorOf(g.peer)}`, p?.icon),'),j.count('speakerEl(\x22agent\x22)'),h.count('title=\x22초대 수락\x22>+ 사용자</button>'),h.count('title=\x22새 그룹\x22>+ 그룹</button>'),ci.count('ico(\x22bot\x22)'),ci.count('{ class: \x22sub\x22 }'),ci.count('room-preview')))"
  EXPECT: menu=1 old=0 leave=1 badge=1 head=1 row=1 plus=1 gplus=1 bot=1 sub=0 prev=0
  EVIDENCE: menu=1 old=0 leave=1 badge=1 head=1 row=1 plus=1 gplus=1 bot=1 sub=0 prev=0

- [x] E6: A6·A7·A8·A9 — 레일 놓을 자리 · 그룹설정 B · 포인터 넣기 · 주고받은 메시지 코드 실재
  CHECK: python -c "import pathlib;j=pathlib.Path('crates/app/web/app.js').read_text(encoding='utf-8');c=pathlib.Path('crates/app/web/app.css').read_text(encoding='utf-8');tg=j[j.index('function tileGrab(ev)'):j.index('function tileGrab(ev)')+4000];print('slot=%d point=%d b=%d card=%d table=%d keep=%d join=%d html5=%d lost=%d pm=%d bg=%d sel=%d oldA=%d'%(tg.count('railSlot(same.map(mid)'),tg.count('document.elementFromPoint'),j.count('replaceChildren(gsGroupsEl(list, secs, q), gsUsersEl(list, convs, q))'),c.count('.gs-cards { display: grid; grid-template-columns: repeat(auto-fill, minmax(220px, 1fr))'),j.count('{ class: \x22gs-table\x22 }'),j[j.index('function gsToggle'):j.index('function gsGroupsEl')].count('const open = state.open.has(key);'),j.count('convGrab(ev, ev.currentTarget.parentElement, { join: true })'),j.count('draggable: \x22true\x22'),j.count('grip.addEventListener(\x22lostpointercapture\x22, end);'),j.count('pm-av pm-av-${kind}'),c.count('.rows li.pm-agent { background: var(--agent-soft); }'),c.count('#rail { user-select: none; }'),j.count('groupSettingsListEl')))"
  EXPECT: slot=1 point=0 b=1 card=1 table=1 keep=1 join=1 html5=0 lost=1 pm=1 bg=1 sel=1 oldA=0
  EVIDENCE: slot=1 point=0 b=1 card=1 table=1 keep=1 join=1 html5=0 lost=1 pm=1 bg=1 sel=1 oldA=0

- [x] E7: A10 — 문서 규약 · 크레이트 경계 · 화면 문구 목록 · CSS 토큰
  CHECK: python tools/check_ddd_docs.py && python tools/check_crates.py && cargo test -p coworkagent-app --test ux_literals --test css_tokens
  EXPECT: /test result: ok\. 3 passed; 0 failed[\s\S]*test result: ok\. 5 passed; 0 failed/
  EVIDENCE: Finished `test` profile [unoptimized + debuginfo] target(s) in 0.19s / Running tests\css_tokens.rs (target\debug\deps\css_tokens-e921a55a3b82e8dd.exe) / Running tests\ux_literals.rs (target\debug\deps

- [x] E8: A3·A6·A7·A8·A9 — 실 화면(`rebuild_run` 릴리스 빌드): 개요 아이콘 · 레일 이름 · 틈 · 끝 밖 놓기(보통 · 작게) · 그룹설정 카드 · 표 펼침 유지 · 카드에 포인터 놓기 · 주고받은 메시지 줄
  EVIDENCE: 2026-10-21 Claude(구현 세션) — 이 PC 릴리스 빌드(`tools/rebuild_run.py`) 웹UI 를 ego-browser(Chromium) 실제 마우스로. 메뉴 '나' 끝 = 등록된 에이전트 · 에이전트 응답, 상대 타일 = 요청 · 즐겨찾기 · 북마크 · 등록된 에이전트(에이전트 응답 화면에서 상대 타일 → `#`). 개요 절 머리 사용자 아이콘 6/6 · 대화 줄 로봇 12/12. 레일 "+ 사용자" · "+ 그룹". 대화 줄 4 = 로봇 4 · 2행 0 · 높이 40px. 레일 끌기: 보통 크기 둘째 타일 아래 이름 위 → 2:after · 순서 바뀜 → 되돌림, 작게 마지막 타일 아래 밖 → 6:after · 첫 타일 위 틈 → 2:before, 매번 서버 `GET /api/peers` = 화면, 끝 = 시작 순서 1,2,3,5,4,6. 그룹설정: 카드 3(3열) · 사용자 표 6줄(사용자 · PC · 상태 · 모델 · effort · 대화 · 관리) · `draggable="true"` 0, 줄 빈 칸 누름 → 펼침 · 6초 뒤 유지, 대화 줄 손잡이 → 이미 든 그룹 카드 → `drop-into` · 놓으면 "이미 그룹 AI Discovery 에 있습니다" · 사본 0. 주고받은 메시지 줄 9 = 에이전트 4 · 나 2 · 상대 3, 에이전트 줄 바탕 rgb(242, 249, 248) · 칩 4 · 북마크 9. 미확인: 그림 아이콘 타일 · 다크 테마 대비 · 실제 넣기(데이터 변경 피함)


WEAK: E8 CHECK 없음 — 수동 gate(사람이 적은 EVIDENCE 로 판정)

## 적대적 평가 — 라운드 1/2

평가자: Orca 가 새로 띄운 Codex(`worker-start --agent codex`, 작업 폴더 `C:/Users/tskim/.claude/skills`), handle `term_0855d86e-3756-4462-83cd-2ca2616c78d3`.
Run `run_f50378826139` · Task `task_e75de05d160c` · Dispatch `ctx_0114f2edb2c2`, worker_done(`msg_8a03e514b4af`) 발신 handle 일치. 범위 `0bdca87..3f9e28f`(self-collect),
메타 `요청 1z-2-ui-refine-r1 · revision 3f9e28fcfbc011bdb331708846239000b8e7f5b8 · 라운드 1/2`. 리뷰 뒤 HEAD 3f9e28f 그대로(작업 폴더 변경은 리뷰 중 내가 쓴 종결 초안
(masterPlan · PRODUCT 환류)와 다른 세션 파일뿐). 리뷰어 Dispatch `worker-release` → `captured`. 원문(render):

```text
# Codex Adversarial Review

Target: branch diff against 0bdca87
Verdict: needs-attention

고정 범위의 A1~A10·E1~E8과 구현을 읽기 전용으로 대조했습니다. 시안 B에서 다른 사용자 설정 저장이 미저장 입력을 지우는 회귀가 있어 출하 보류가 필요합니다. 사용자별 입력 보존 수정과 DOM 회귀 검증이 남았습니다.

Findings:
- [medium] 다른 사용자 저장 시 열린 행의 미저장 설정이 사라짐 (crates/app/web/app.js:1038-1044)
  사용자 A·B를 함께 펼쳐 A의 모델을 바꾼 뒤 B를 저장하면 A의 선택도 서버의 기존 값으로 돌아갑니다. 새 gsUsersEl은 사용자별 userDetail을 동시에 만들지만 입력값은 각 select DOM에만 있고, userDetail의 peerAct는 저장 뒤 loadPeers(true)를 호출하여 이 replaceChildren에서 모든 사용자 폼을 재생성합니다. force는 needsRender의 입력 보호도 우회하며 state.open은 펼침만 복구합니다. 고정 revision의 renderPeerList·needsRender를 실행한 메모리 DOM 모형에서 A의 미저장 opus가 B 저장 후 sonnet으로 초기화됨을 확인했습니다(실 브라우저 검증은 미실시). 여러 사용자 설정을 편집할 때 경고 없이 입력을 잃으며, A7의 펼침 유지 시험과 E6 문자열 검사는 이를 검출하지 못합니다.
  Recommendation: 사용자 id별 dirty draft를 state에 보관하고 재생성 시 복구하거나, 저장한 사용자 행만 갱신하여 다른 행의 입력 DOM을 보존하십시오. A·B 동시 펼침 → A 모델·effort 변경 → B 저장 → A 입력 유지의 DOM 회귀 시험을 추가하십시오.

Next steps:
- 사용자별 미저장 입력 보존 후 두 사용자 동시 편집·저장 및 배경 갱신을 DOM 수준에서 검증하십시오.
- 기존 gate 8/8과 순수 함수 시험은 이 회귀의 통과 근거로 사용하지 마십시오.
```

raw JSON: 보관 `scratchpad/review_1z2_r1.json`.

### 판정 · 조치

| 지적 | 판단 | 조치 |
|---|---|---|
| [medium] 다른 사용자 저장 시 열린 행의 미저장 설정이 사라짐 | 사실(시안 B 에서 새로 생긴 회귀 — 시안 A 는 한 사람만 보였다) | `gsDrafts`(상대 id → 저장 전 모델 · effort, `change` 때 기록) — `userDetail` 이 다시 만들 때 되살리고, 그 사람 저장이 성공하면 지운다(실패면 남김). `ux_literals` PRESENT 1. 실 화면: 사용자 둘 펼침 → A 모델 opus(저장 안 함) → B "저장" → 다시 그린 뒤 A = opus · 둘 다 펼침(ego-browser), 뒤에 A 를 원래 값으로 되돌림. DOM 자동 시험은 없음(웹UI 시험은 `view.js` 만) |

## 적대적 평가 — 라운드 2/2

평가자: Orca 가 새로 띄운 Codex(`worker-start --agent codex`), handle `term_26ef0d2b-834e-4d12-8711-76abc41a483d`. Run `run_f50378826139` · Task `task_ab5671c53d77` ·
Dispatch `ctx_474341e6b5c8`, worker_done(`msg_5a925b4853c1`) 발신 handle 일치. 범위 `0bdca87..6aa923f`(self-collect), 메타 `요청 1z-2-ui-refine-r2 · revision
6aa923f0880bb46865573b2d1708e5ec737bf596 · 라운드 2/2`. 리뷰 뒤 HEAD 6aa923f 그대로(작업 폴더 변경은 리뷰 중 내가 쓴 report 초안 · 환류와 다른 세션 파일뿐).
리뷰어 Dispatch `worker-release` → `captured`. 원문(render):

```text
# Codex Adversarial Review

Target: branch diff against 0bdca87
Verdict: needs-attention

고정 범위의 A1~A10·E1~E8과 구현을 읽기 전용으로 대조하고 view.test.mjs 160개를 고정 소스의 메모리 주입 실행으로 통과시켰습니다. R1의 다른 사용자 저장 경로는 보존되지만 저장 중 재편집을 잃는 잔여 결함 때문에 출하 보류입니다. 저장 완료의 draft 삭제를 세대별로 제한하고 지연 응답 회귀 검증이 남았습니다.

Findings:
- [medium] 이전 저장 완료가 요청 이후의 미저장 편집까지 삭제 (crates/app/web/app.js:1240-1242)
  opus/high로 저장을 누르고 PATCH 응답을 기다리는 동안 같은 사용자를 sonnet/medium으로 바꾸면, change 핸들러가 새 draft를 기록해도 이전 요청 완료가 gsDrafts.delete(p.id)로 그것까지 삭제합니다. 입력·저장 잠금이나 draft 세대 비교가 없고 peerAct의 loadPeers(true)가 즉시 전체 폼을 재생성하므로 이후 편집이 경고 없이 서버의 opus/high로 되돌아갑니다. 고정 revision의 실제 userDetail·act를 추출한 메모리 DOM 모형과 지연 API에서 이 전이를 재현했습니다(실 브라우저 미실시). R1의 A 편집→B 저장 경로는 같은 모형에서 보존됐으나 저장 대기 중 편집 보호는 닫히지 않았습니다.
  Recommendation: 요청 시작 시 제출값과 draft 객체 또는 세대 번호를 캡처하고, 성공 시 현재 draft가 제출한 세대와 같을 때만 삭제하십시오. 응답 대기 중 새 change는 유지하고, 지연 PATCH→같은 사용자 재편집→응답→새 값 유지의 DOM 회귀 시험을 추가하십시오.

Next steps:
- 저장 완료의 무조건 draft 삭제를 고치고 지연 응답·후속 편집 보존을 검증하십시오.
- 라운드 2/2 종료: 이 미조치 지적을 인계하고 추가 리뷰는 수행하지 않습니다.
```

raw JSON: 보관 `scratchpad/review_1z2_r2.json`.

### 판정 · 조치

| 지적 | 판단 | 조치 |
|---|---|---|
| [medium] 이전 저장 완료가 요청 이후의 미저장 편집까지 삭제 | 사실(R1 수정이 만든 잔여 결함) | 저장 시작 때의 고친 값 객체(`sent`)를 잡아 두고, 성공 뒤 지금 값이 그 객체일 때만 지운다(`change` 마다 새 객체라 응답 대기 중 고친 값은 남는다). `ux_literals` PRESENT 1. 실 화면: 이 탭의 PATCH 응답을 1.5초 늦추고 A = opus 저장 → 기다리는 사이 haiku → 응답 뒤 A = haiku(ego-browser), 뒤에 원래 값(기본값)으로 저장해 서버 값 되돌림(`GET /api/peers` model · effort 모두 기본값) |

수정 뒤 라운드 상한(2/2) — 재리뷰 없이 종료.

## 미조치 지적

| 지적 | 판단 | 사유 |
|---|---|---|
| R1 · R2 "A·B 동시 펼침 → A 변경 → B 저장 → A 유지" · "지연 PATCH → 같은 사용자 재편집 → 응답 → 새 값 유지" 의 DOM 회귀 시험을 추가하라 | 사실(시험 공백) | 코드는 고쳤고 실 화면(ego-browser)으로 두 경로를 확인했으나 웹UI 시험은 DOM 없는 `view.js` 순수 함수뿐이라 자동 시험이 없다 — #1z 미조치(DOM 시험 도입 안 함, 사용자 2026-10-21 "기록만하고 종료")와 같은 판단 |
