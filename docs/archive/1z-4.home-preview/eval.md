# eval.md — 1z-4.home-preview 적대적 평가

## 평가 원장

<!-- tools/check_gates.py 산출 — 손으로 고치지 않는다. 단 CHECK 가 없는 수동 gate 의
     EVIDENCE 와 ABANDON 줄은 사람이 적고, 재실행해도 보존된다.
     WEAK 줄은 기계가 매 실행 다시 단다 — 약한 gate 를 셀 수 있게 하는 표시이며
     판정에는 영향이 없다. -->

측정 커밋: 702fcdcc9529c38a09ecba27a64c7d1a7d19d974

- [x] E1: A6 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

- [x] E2: A6 — clippy 경고 0
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.25s

- [x] E3: A2·A5·A7 — 웹UI 로직 시험(창 · 처음 탭 · 고른 탭 · 말풍선 · 다시 읽을 때 · 탭 3개 + "+n개")
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# fail 0/
  EVIDENCE: # fail 0

- [x] E4: A1·A5 — 설정 기억 · 400, 글 `limit`(마지막 N · 범위 밖 400)
  CHECK: cargo test -p coworkagent-app --test ux_api --test chat_api
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.16s

- [x] E5: A1·A2·A4·A5 — 보기 전환 · 격자 · 마지막 글만 · 글자만 실재
  CHECK: python -c "import pathlib as P;c=P.Path('crates/app/web/app.css').read_text(encoding='utf-8');j=P.Path('crates/app/web/app.js').read_text(encoding='utf-8');h=P.Path('crates/app/web/index.html').read_text(encoding='utf-8');r=P.Path('crates/core/src/run_opts.rs').read_text(encoding='utf-8');print('grid=%d seg=%d box=%d tail=%d save=%d def=%d read=%d html=%d'%(c.count('grid-template-columns: repeat(auto-fill, minmax(min(100%, 360px), 1fr)); grid-auto-rows: 320px;'),h.count('<button id=\x22home_view_preview\x22 type=\x22button\x22 aria-pressed=\x22true\x22>미리보기</button>'),h.count('<div id=\x22home_preview\x22 class=\x22pv-grid\x22></div>'),j.count('/messages?limit=${PREVIEW_MSGS}`'),j.count('api(\x22PUT\x22, \x22/api/settings\x22, { ui_home_view: v })'),r.count('pub const HOME_VIEWS: [&str; 2] = [\x22preview\x22, \x22list\x22];'),j[j.index('async function renderHomePreview'):j.index('async function pickPreviewTab')].count('/read'),j[j.index('function previewWindowEl'):j.index('async function pickPreviewTab')].count('innerHTML')))"
  EXPECT: grid=1 seg=1 box=1 tail=1 save=1 def=1 read=0 html=0
  EVIDENCE: grid=1 seg=1 box=1 tail=1 save=1 def=1 read=0 html=0

- [x] E6: A6 — 문서 규약 · 크레이트 경계 · 매뉴얼 반영
  CHECK: python tools/check_ddd_docs.py && python tools/check_crates.py && python -c "import pathlib as P;w=P.Path('docs/manual/dev/web-ui.md').read_text(encoding='utf-8');a=P.Path('docs/manual/dev/an_app.md').read_text(encoding='utf-8');g=P.Path('docs/manual/user/user-guide.md').read_text(encoding='utf-8');print('webui=%d app=%d guide=%d'%(w.count('#home_preview.pv-grid'),a.count('ui_home_view 는 preview · list'),g.count('| \x27개요\x27 미리보기(기본) |')))"
  EXPECT: webui=1 app=1 guide=1
  EVIDENCE: webui=1 app=1 guide=1

- [x] E7: A1·A2·A3·A4·A5·A7 — 실제 화면: 디버그 빌드(시험 데이터 폴더, 상대 4 · 대화 6 시드)에서 기본 미리보기 · 창 · 처음 탭 · 탭 유지 · 상대 타일 창 · 격자 열 수 · 잘림 · 안 읽음 그대로 · 목록 기억
  EVIDENCE: Claude(구현자) 2026-10-22 — `target/debug/coworkAgent.exe start --parent <PID>` + `COWORKAGENT_DATA_DIR`=scratchpad `ui1z4\data`(사용자 앱 그대로, 상대 4 · 대화 6 · 글 16 sqlite 시드), ego-browser DOM 판독(창 최소화로 스크린샷 불가). 관측: 처음 `#home_view_preview` pressed · `#home_summary` hidden → 창 4(김도윤 탭 3 "*배포 절차 점검 3"(최근) · 글 6, 박서연 1, 이준호 2, 최하은 "아직 주고받은 대화가 없습니다 · 세션 생성") · 말풍선 순서 = 글 순서(시드 `lc` 보정 뒤) · `<b>굵게</b>` 글자 그대로(태그 0) · 첫 말풍선 top 129 < 본문 top 243(위에서 잘림) · 마지막 bottom 443 ≤ 본문 bottom 455 → "API 오류 원인" 탭 → 본문 `#c/3` 2개(error 말풍선) · 초점 그 탭 → 6.5초 뒤 그대로 선택 → 안 읽음 전후 `배포 절차 점검 3` 그대로 → 격자 폭 380/800/1200/1600 = 1/1/3/4열(800 은 스크롤바 뒤 737px < 744) · 자연 폭 820 = 2열 · `overflow-y: auto` · 창 높이 320px → 목록 누름 = 설정 `list` · 새로 고침 뒤에도 목록 → 미리보기 = `preview` → 레일 김도윤 = "개요 · 김도윤" 창 3(대화마다, 탭 0, `#c/1` · `#c/2` · `#c/3`) → 다크 테마 말풍선 토큰 색. 끝에 `coworkAgent stop`. 재확인(R1 · 사용자 Q2 · Q3 반영 뒤, 같은 방식 · 김도윤 대화 5개로 시드 추가): 탭 "*배포 절차 점검 3" · "로그 수집 스크립트" · "API 오류 원인" + "+2개" → 11초 가만히 = 미리보기 글 읽기 0회(5초 새로 고침 두 번) → 마지막이 아닌 글을 DB 에서 고침(신호 없음) → 6초 뒤 그대로 · 읽기 0회 → 내 활동 갔다 개요로 = 읽기 3회(창 3) · "다시 고친 둘째 답" → 새 글 하나(요약 바뀜) → 6초 안 그 대화만 1회 · 마지막 말풍선 "새 글 하나" → "+2개" 누름 = "개요 · 김도윤" 창 5. R1 반영 직후(그릴 때마다 읽던 판)에는 앱이 뜬 채 둘째 글을 고치자 6.5초 뒤 "고친 둘째 답".


WEAK: E7 CHECK 없음 — 수동 gate(사람이 적은 EVIDENCE 로 판정)

## 적대적 평가 — 라운드 1/2

평가자: Orca 가 새로 띄운 Codex(`worker-start --agent codex`, 작업 폴더 `C:/Users/tskim/.claude/skills`), handle `term_17fd225a-7950-4b6e-b080-6d4b2e1c102c`.
Run `run_7a036b967f4f` · Task `task_ce3dd62b4f5c` · Dispatch `ctx_10398a7e5134`, worker_done(`msg_3cc02b35306d`) 발신 handle 일치. 범위 `af034a2..7b4369c`(self-collect),
메타 `요청 1z-4-home-preview-r1 · revision 7b4369c97b482b8d26a0461be1823210b5674c36 · 라운드 1/2`. 리뷰 동안 대상 작업 폴더 편집 없음(HEAD 7b4369c 그대로). 리뷰어 Dispatch `worker-release` → `released` · `captured`. 원문(render):

```text
# Codex Adversarial Review

Target: branch diff against af034a2
Verdict: needs-attention

지정 범위를 읽기 전용으로 검토하고 재시도 경로를 메모리 재현했다. 미리보기 갱신을 막는 독립 결함 2건이 있어 출시 보류가 필요하다. 캐시 무효화와 렌더 서명 보완 및 회귀 검증이 남았다.

Findings:
- [medium] 이전 말풍선 변경을 감지하지 못하는 캐시 키 (crates/app/web/view.js:1117-1118)
  previewKey는 마지막 글의 시각·상태·본문 앞부분만 사용한다. 최근 6개 중 마지막이 아닌 글을 기존 편집 API로 수정하면 chat.rs의 edit_text는 text·edited_at만 갱신하므로 이 키는 그대로다. renderHomePreview의 need 필터가 조회를 계속 생략하여 5초 새로 고침과 개요 재진입 후에도 수정 전 글이 표시된다. 마지막 글이 아닌 실행 글의 상태 변경도 같은 문제다. summaries가 제공하는 마지막 글 정보만으로 최근 6개 전체의 변경 여부를 판단할 수 없다.
  Recommendation: 최근 글 중 편집·상태 변경에도 갱신되는 대화 revision을 API에 제공하여 캐시 키로 사용하거나, 해당 대화 변경 신호에서 캐시를 무효화한다. 마지막이 아닌 미리보기 글 편집·상태 변경 후 실제 재조회와 표시를 검증한다.
- [medium] 조회 실패 후 성공한 재시도가 오래된 화면을 갱신하지 않음 (crates/app/web/app.js:1784-1791)
  기존 캐시가 있는 대화에 새 글이 생기고 첫 조회만 실패하면, 이전 캐시로 그리면서 새 previewKey와 has=true를 렌더 서명에 저장한다. 다음 조회가 성공해 캐시 내용을 교체해도 서명은 같은 새 previewKey와 has=true라서 needsRender가 반환하고 화면은 이전 말풍선에 머문다. 지정 revision의 renderHomePreview를 추출한 메모리 실행에서 rendered=[[old],[old]], cached=new를 확인했다. 이후 대화 메타데이터나 접속 표시 등이 바뀌기 전까지 성공한 재시도가 화면에 반영되지 않는다.
  Recommendation: 렌더 서명에 실제 캐시의 key 또는 메시지 revision을 포함한다. 이전 캐시가 있는 상태에서 새 글 조회 실패→동일 메타데이터로 재시도 성공 시 새 말풍선이 그려지는 회귀 검증을 추가한다.

Next steps:
- 대화 변경 revision 또는 명시적 캐시 무효화로 최근 글 전체의 변경을 추적한다.
- 렌더 서명을 실제 표시 데이터에 연결하고 일시적 실패 후 성공 경로를 검증한다.
```

판정 · 조치:

| 지적 | 판정 | 조치 |
|---|---|---|
| R1-1 medium 마지막이 아닌 글 편집 · 상태 변경을 `previewKey` 가 놓침 | 사실 — 요약은 마지막 글만 안다 | `previewKey` 삭제. 창의 대화를 그릴 때마다 다시 읽는다 — 서버 `chat::messages_tail`(SQL `LIMIT`, 대화 전체를 읽지 않음). plan 개정(A5 · 설계 줄) · intent A5 같이. 실제 화면: 앱이 뜬 채로 김도윤 대화의 둘째 글(마지막 아님)을 고치자 6.5초 뒤 미리보기 말풍선 "고친 둘째 답" |
| R1-2 medium 실패 뒤 성공한 재시도가 서명이 같아 안 그려짐 | 사실 — 서명이 메타데이터만 봤다 | 서명 = 실제 말풍선(`previewBubbles` 결과) · 머리 값. 실패한 창은 지난 글로 그리고, 다음 성공 때 말풍선이 바뀌면 서명이 바뀐다. 시험 `preview_bubbles` 에 마지막이 아닌 글 편집 · 상태 변경 → 말풍선 달라짐 |

시험: `cargo test --workspace`(PowerShell) test result: ok 80 · `cargo clippy --workspace --all-targets -- -D warnings` Finished · node 162 pass, gate 재측정(위 원장).

## 사용자 추가 지시 — Q2 · Q3 반영(라운드 1 뒤)

사용자 "Q2, Q3 반영."(2026-10-22) — 탭 3개 + "+n개", 미리보기 다시 읽기 = 받은 적 없음 · 요약 변화 · 변경 신호 · 개요에 들어옴 · 보기 바꿈 · 탭 고름 · 읽기 실패만
(plan 개정 기록 2 · intent 결정 11). R1-1 을 "그릴 때마다 읽기"로 고친 것을 이 지시로 다시 줄였다 — 남는 경우는 아래 "미조치 지적".

## 적대적 평가 — 라운드 2/2

평가자: Orca 가 새로 띄운 Codex(`worker-start --agent codex`, 작업 폴더 `C:/Users/tskim/.claude/skills`), handle `term_43862fe4-4c08-4574-82c1-f0e0e3f2c537`.
Run `run_7a036b967f4f` · Task `task_606e864f6bdb` · Dispatch `ctx_db59d5d04928`, worker_done(`msg_3a9765bc574d`) 발신 handle 일치. 범위 `af034a2..048202a`(self-collect),
메타 `요청 1z-4-home-preview-r2 · revision 048202ad6bb4983401965b3a6308408701d87070 · 라운드 2/2`. 리뷰 동안 대상 작업 폴더 편집 없음(HEAD 048202a 그대로). 리뷰어 Dispatch `worker-release` → `released` · `captured`. 원문(render):

```text
# Codex Adversarial Review

Target: branch diff against af034a2
Verdict: needs-attention

지정 revision을 읽기 전용으로 검토하고 추출 함수로 두 경로를 확인했다. SSE 재연결 시 미리보기 갱신 누락과 기본 화면의 첫 연결 안내 노출이 남았다. 두 결함 수정과 회귀 검증이 필요하다.

Findings:
- [medium] SSE 재연결 후 놓친 변경이 캐시에 영구 잔류 (crates/app/web/app.js:1773-1779)
  캐시가 있는 상태에서 SSE가 끊긴 동안 마지막이 아닌 글이 편집·삭제되면 previewKey는 그대로다. events는 새 broadcast 구독만 만들고 과거 이벤트를 재생하지 않으며, listen의 onopen(4431)은 /api/seen만 호출한다. 따라서 재연결 후에도 all=false·stale 없음으로 이 필터가 매 5초 읽기를 계속 생략하여, 개요 재진입이나 별도 변경 전까지 오래된 본문·상태를 표시한다. 추출한 실제 onopen 실행에서도 previewAll=false, previewStale.size=0을 확인했다. 두 PC 단절 실측은 미실행이며 지속 누락 경로는 코드에 근거한 추론이다.
  Recommendation: EventSource onopen에서 previewAll=true로 무효화하고 현재 개요를 갱신하라. 구독 전후 변경 누락도 막도록 재연결 구독이 성립한 뒤 읽고, 동일 요약의 중간 글 변경을 놓친 단절·복구 회귀를 검증하라.
- [medium] 기본 미리보기에서 첫 연결 안내 숨김을 건너뜀 (crates/app/web/app.js:1720-1722)
  새 분기의 조기 return이 home_intro.hidden 갱신(1725)을 건너뛴다. index.html의 home_intro는 처음부터 visible이고 다른 갱신 위치도 없어, 기본 preview로 시작하면 연결된 상대와 대화가 있어도 '처음 연결하려면 상대에게 받은 초대를…' 안내가 계속 노출된다. .view는 세로 flex이므로 이 안내가 미리보기 격자 높이도 차지한다. 지정 revision의 renderHome을 대화가 있는 입력으로 실행해 home_intro.hidden=false를 확인했다. 반대로 빈 상태에서 안내가 이전 목록 상태에 따라 숨겨질 수도 있다.
  Recommendation: home_intro.hidden 갱신을 preview/list 분기 이전으로 옮겨라. 기존 대화가 있는 기본 preview와 상대가 없는 빈 preview 양쪽에서 안내 표시를 검증하라.

Next steps:
- SSE 재연결 뒤 동일 previewKey의 본문 변경을 한 번 다시 읽도록 보완한다.
- 보기 분기 이전에 첫 연결 안내 상태를 갱신한다.
- 라운드 2/2 결과로 기록하고 추가 리뷰는 시작하지 않는다.
```

판정 · 조치(상한 도달, 재리뷰 없음):

| 지적 | 판정 | 조치 |
|---|---|---|
| R2-1 medium 변경 신호가 끊긴 동안 놓친 변경이 다시 붙은 뒤에도 안 보임 | 사실 — 신호는 다시 보내지 않고 onopen 은 `/api/seen` 만 | `es.onopen` 에서 `previewAll = true` + `refreshHome()` — 붙을 때마다(처음 포함) 미리보기 글 전부 새로. `ux_literals` 한 줄. 끊김 · 복구 실측은 미실행(EventSource 를 밖에서 끊을 수단 없음) |
| R2-2 medium 미리보기에서 첫 연결 안내(`home_intro`)를 숨기지 않음 | 사실 — 조기 return 이 갱신 줄을 건너뜀(E7 판독 항목에 없었다) | 갱신을 보기 분기 앞으로(두 보기 공통) · `ux_literals` 한 줄. 실제 화면: 시드 데이터 = 안내 숨김 · 창 4, 빈 데이터 폴더 = 안내 보임 · 창 0(둘 다 미리보기 기본) |

## 미조치 지적

| 지적 원문 | 판정 | 미조치 사유 |
|---|---|---|
| R1-1 Recommendation "최근 글 중 편집·상태 변경에도 갱신되는 대화 revision을 API에 제공하여 캐시 키로 사용하거나, 해당 대화 변경 신호에서 캐시를 무효화한다." — 이 PC 에서 마지막이 아닌 글만 바뀌는 경우(신호 없음 · 요약 그대로) | 사실(남은 경우) | 사용자 "Q3 반영"(5초마다 다시 읽지 않음)으로 변경 신호 · 요약 변화 · 개요에 들어옴 · 다시 붙음으로 무효화 — 이 PC 변경에는 신호가 없어(D39 ②) 개요에 다시 들어올 때 보인다. 대화 revision API 는 범위 밖. 리뷰 상한 2/2 |
