# plan — 1z-4.home-preview

> 근거: [intent](intent.md)(승인 2026-10-22 — 결정 1 ~ 10). **구현자는 이 문서만 보고 일한다.** D·S 는 [masterPlan](../../masterPlan.md)(S4 개요 · D35).
> 브랜치 `task/1z-4-home-preview`(작업 폴더 `D:\dev\git\coworkAgent-1z-4`). 기준 커밋: `af034a2`(origin/main). plan 승인: 2026-10-22(시안 선택 · 질문 답으로 갈음).

## 목표

개요 기본 보기 = 미리보기(시안 B — 상대마다 작은 대화 창), "목록 | 미리보기" 전환을 설정에 기억, 열 수는 브라우저 폭.

### 성공 기준(intent 사본)

- A1: 개요 기본 보기가 미리보기이고, 목록으로 바꾸면 설정에 남아 다시 열어도 목록 — 틀린 값은 400(시험 · 실제 화면).
- A2: '나' · 그룹 개요 미리보기 = 상대마다 창, 처음 탭 = 최근 활동 대화방, 탭을 누르면 그 대화방 글로 바뀌고 5초 새로 고침에도 유지(시험 · 실제 화면).
- A3: 상대 타일 개요 미리보기 = 대화방마다 창(실제 화면).
- A4: 창 열 수가 브라우저 폭에 따라 바뀌고(넓으면 여럿, 좁으면 하나) 넘치면 세로 스크롤(실제 화면 계산 값).
- A5: 미리보기는 대화 전체를 읽지 않고(`limit` — 서버가 SQL 에서 마지막 N 개만) 5초 새로 고침마다 읽지 않으며 필요한 대화만 다시 읽고(요약이 바뀜 · 변경 신호 · 개요에 들어옴 · 탭 고름), 미리보기만으로 안 읽음이 줄지 않는다(시험 · 실제 화면).
- A7: 창의 대화방 탭은 3개까지(고른 대화는 늘 보임), 넘치면 "+n개" — 누르면 그 상대 개요(대화마다 창)(시험 · 실제 화면).
- A6: 회귀 없음 — PROJECT.md 검증 명령 PASS, 매뉴얼 반영.

## 설계

- core `run_opts`: `HOME_VIEW_KEY`(`ui_home_view`) · `HOME_VIEWS` `["preview", "list"]`(첫 값 = 기본) · `home_view(conn)`.
- app `notify`: `SettingsPatch.ui_home_view`(틀린 값 400 "ui_home_view 는 preview · list") · 쓰기 · `Notifier::home_view` · `settings_json`.
- core `chat`: `messages_tail(conn, id, n)` — `messages` 와 같은 순서(`lc`, `uuid`)의 끝 n 개를 SQL `LIMIT` 로(오래된 것부터).
- app `chat_api`: `GET /api/conversations/{id}/messages?limit=N`(`TailQuery`, 1 ~ `MESSAGES_MAX` 밖 400) — `messages_tail`, 그 글만 진행 · 보관 · 편집 가능을 읽는다.
  인자 없으면 지금 그대로(대화 화면).
- web `view.js`: `PREVIEW_MSGS`(6) · `PREVIEW_TABS`(3) · `previewWindows(groups, single, picked)` · `previewTabs(tabs, picked, max)` · `previewKey(c)` ·
  `previewStaleOf(res)` · `previewBubbles(conv, ms, peerName, n)`.
- web `app.js`: `state.homeView` · `previewTab` · `previewMsgs`, `previewLatest`, `renderHome` 이 보기 분기, `renderHomePreview` · `previewWindowEl` ·
  `pickPreviewTab` · `setHomeView`(누르면 그리고 `PUT /api/settings {ui_home_view}`), 시작 때 `showSettings` 가 `ui_home_view` 를 읽는다.
- web `index.html`: 개요 머리 `.seg`(`#home_view_list` · `#home_view_preview`) · `#home_preview.pv-grid`. `app.css`: `.pv-*`(토큰만 — `css_tokens` 시험).
- 시험: `chat_api`(limit) · `ux_api`(`home_view_setting_persists`) · `view.test.mjs`(`preview_windows` · `preview_bubbles`) · `ux_literals`(#1z-4 다섯 줄).
- 문서: [web-ui](../../manual/dev/web-ui.md) 사이드바 절 · [an_app](../../manual/dev/an_app.md) 설정 · 글 API · [an_core](../../manual/dev/an_core.md) `run_opts` ·
  [사용자 가이드](../../manual/user/user-guide.md) 화면 표 · PRODUCT · masterPlan S4 · 개정 기록(종결).

## 평가 항목

- [ ] E1: A6 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E2: A6 — clippy 경고 0
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: pending
- [ ] E3: A2·A5·A7 — 웹UI 로직 시험(창 · 처음 탭 · 고른 탭 · 말풍선 · 다시 읽을 때 · 탭 3개 + "+n개")
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# fail 0/
  EVIDENCE: pending
- [ ] E4: A1·A5 — 설정 기억 · 400, 글 `limit`(마지막 N · 범위 밖 400)
  CHECK: cargo test -p coworkagent-app --test ux_api --test chat_api
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E5: A1·A2·A4·A5 — 보기 전환 · 격자 · 마지막 글만 · 글자만 실재
  CHECK: python -c "import pathlib as P;c=P.Path('crates/app/web/app.css').read_text(encoding='utf-8');j=P.Path('crates/app/web/app.js').read_text(encoding='utf-8');h=P.Path('crates/app/web/index.html').read_text(encoding='utf-8');r=P.Path('crates/core/src/run_opts.rs').read_text(encoding='utf-8');print('grid=%d seg=%d box=%d tail=%d save=%d def=%d read=%d html=%d'%(c.count('grid-template-columns: repeat(auto-fill, minmax(min(100%, 360px), 1fr)); grid-auto-rows: 320px;'),h.count('<button id=\x22home_view_preview\x22 type=\x22button\x22 aria-pressed=\x22true\x22>미리보기</button>'),h.count('<div id=\x22home_preview\x22 class=\x22pv-grid\x22></div>'),j.count('/messages?limit=${PREVIEW_MSGS}`'),j.count('api(\x22PUT\x22, \x22/api/settings\x22, { ui_home_view: v })'),r.count('pub const HOME_VIEWS: [&str; 2] = [\x22preview\x22, \x22list\x22];'),j[j.index('async function renderHomePreview'):j.index('async function pickPreviewTab')].count('/read'),j[j.index('function previewWindowEl'):j.index('async function pickPreviewTab')].count('innerHTML')))"
  EXPECT: grid=1 seg=1 box=1 tail=1 save=1 def=1 read=0 html=0
  EVIDENCE: pending
- [ ] E6: A6 — 문서 규약 · 크레이트 경계 · 매뉴얼 반영
  CHECK: python tools/check_ddd_docs.py && python tools/check_crates.py && python -c "import pathlib as P;w=P.Path('docs/manual/dev/web-ui.md').read_text(encoding='utf-8');a=P.Path('docs/manual/dev/an_app.md').read_text(encoding='utf-8');g=P.Path('docs/manual/user/user-guide.md').read_text(encoding='utf-8');print('webui=%d app=%d guide=%d'%(w.count('#home_preview.pv-grid'),a.count('ui_home_view 는 preview · list'),g.count('| \x27개요\x27 미리보기(기본) |')))"
  EXPECT: webui=1 app=1 guide=1
  EVIDENCE: pending
- [ ] E7: A1·A2·A3·A4·A5·A7 — 실제 화면: 디버그 빌드(시험 데이터 폴더, 상대 4 · 대화 6 시드)에서 기본 미리보기 · 창 · 처음 탭 · 탭 유지 · 상대 타일 창 · 격자 열 수 · 잘림 · 안 읽음 그대로 · 목록 기억
  EVIDENCE: pending

## 개정 기록

- 2026-10-22 · 적대적 평가 R1 반영 — 미리보기 다시 읽기를 "마지막 글 요약(`previewKey`)이 바뀐 대화만" → "창의 대화를 새로 고침마다(SQL `LIMIT` 로 마지막 N 개)"로,
  다시 그리기 서명 = 실제 말풍선(R1-1 마지막이 아닌 글 편집 · 상태 변경, R1-2 실패 뒤 성공한 재시도). A5 문구 · 설계 줄 갱신(intent A5 같이), EXPECT 그대로.
- 2026-10-22 · 사용자 "Q2, Q3 반영"(intent 결정 11) — 탭 3개 + "+n개"(A7 신설), 다시 읽기 = 받은 적 없음 · 요약 변화(`previewKey` 되살림) · 변경 신호(`previewStaleOf`) ·
  개요에 들어옴 · 보기 바꿈 · 탭 고름 · 읽기 실패만(5초마다 전부 읽지 않음). A5 문구 · E3 · E7 의 A 목록 · 설계 줄 갱신, EXPECT 그대로.
