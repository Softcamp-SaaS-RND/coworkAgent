# plan — 1z-5.home-thumb

> 근거: [intent](intent.md)(승인 2026-10-22 — 결정 1 ~ 7). **구현자는 이 문서만 보고 일한다.** D·S 는 [masterPlan](../../masterPlan.md)(S4 개요 · D35).
> 브랜치 `task/1z-5-home-thumb`(작업 폴더 `D:\dev\git\coworkAgent-1z-5`). 기준 커밋: `4dd486c`(origin/main). plan 승인: 2026-10-22(요청 · 질문 답으로 갈음).

## 목표

개요 '미리보기' 보기 = 2차 시안 A(상대별 절 + 대화방마다 축소 채팅 카드), '나' · 그룹은 절마다 한 줄(폭 맞춤) + "+n개".

### 성공 기준(intent 사본)

- A1: 개요 미리보기 = 상대마다 절 + 대화방마다 카드(제목 · 상태 · 최근 글 말풍선 · 에이전트 · 시각 · 안 읽음), 카드를 누르면 그 대화, 글은 글자로만(시험 · 실제 화면).
- A2: '나' · 그룹은 절마다 한 줄 — 열 수가 브라우저 폭에 따라 바뀌고, 넘치면 끝 칸 "+n개"(숨은 수) → 누르면 그 상대 개요(카드 전부)(시험 · 실제 화면).
- A3: 대화 없는 페어링 상대 = 점선 한 줄 + 새 세션 단추(실제 화면).
- A4: 기본 보기 · 보기 기억 · 다시 읽기 규칙(#1z-4 A1 · A5)이 그대로 — 미리보기만으로 안 읽음이 줄지 않는다(시험 · 실제 화면).
- A5: 회귀 없음 — PROJECT.md 검증 명령 PASS, 매뉴얼 반영.

## 설계

- web `view.js`: `previewSections(groups, single, cols)` — 절마다 `cards`(최근 활동 순, '나' · 그룹은 넘치면 `cols - 1` 개) · `more`(숨은 수), 상대 타일은 전부.
  `previewWindows` · `previewTabs` · `PREVIEW_TABS` 삭제. `PREVIEW_MSGS` · `previewKey` · `previewStaleOf` · `previewBubbles` 그대로.
- web `app.js`: `state.previewCols`(한 줄 열 수, `state.previewTab` 삭제), `renderHomePreview` = 절 · 카드의 대화만 다시 읽기(규칙 그대로) · 서명 = 절 · 카드 · 말풍선 · 열 수,
  `fitPreviewCols`(그린 `.pv-cards` 의 `gridTemplateColumns` 트랙 수 — 바뀌면 다시 그림) · `ResizeObserver(#home_preview)`, `previewSectionEl` · `previewCardEl`.
  `previewWindowEl` · `pickPreviewTab` 삭제.
- web `index.html`: `#home_preview.pv-list`. `app.css`: `.pv-list` · `.pv-sec` · `.pv-none` · `.pv-cards`(`repeat(auto-fill, minmax(min(100%, 240px), 1fr))`) · `.pv-card*` ·
  `.pv-state` · `.pv-body`(120px) · `.pv-msg` · `.pv-more`(토큰만 — `css_tokens` 시험). `.pv-grid` · `.pv-win` · `.pv-tab*` · `.pv-dot` · `.pv-empty` 삭제.
- 시험: `view.test.mjs` `preview_sections`(`preview_windows` 대신, 탭 단언 삭제) · `ux_literals`(#1z-5 다섯 줄 — #1z-4 격자 · 상자 두 줄 대체).
- 문서: [web-ui](../../manual/dev/web-ui.md) 개요 보기 줄 · [사용자 가이드](../../manual/user/user-guide.md) 화면 표 · PRODUCT · masterPlan S4 · 개정 기록(종결).

## 평가 항목

- [ ] E1: A5 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E2: A5 — clippy 경고 0
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: pending
- [ ] E3: A1·A2·A4 — 웹UI 로직 시험(절 · 카드 순서 · 한 줄 + 숨은 수 · 상대 타일 전부 · 말풍선 · 다시 읽을 때)
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# fail 0/
  EVIDENCE: pending
- [ ] E4: A1·A2·A4 — 절 목록 · 카드 격자 · 열 재기 · "+n개" · 마지막 글만 실재, B 창 · 탭 없음, 글자만
  CHECK: python -c "import pathlib as P;c=P.Path('crates/app/web/app.css').read_text(encoding='utf-8');j=P.Path('crates/app/web/app.js').read_text(encoding='utf-8');h=P.Path('crates/app/web/index.html').read_text(encoding='utf-8');v=P.Path('crates/app/web/view.js').read_text(encoding='utf-8');print('box=%d grid=%d fit=%d more=%d tail=%d old=%d html=%d'%(h.count('<div id=\x22home_preview\x22 class=\x22pv-list\x22></div>'),c.count('.pv-cards { display: grid; grid-template-columns: repeat(auto-fill, minmax(min(100%, 240px), 1fr));'),j.count('}).observe($(\x22home_preview\x22));'),j.count('text: `+${s.more}개` })'),j.count('/messages?limit=${PREVIEW_MSGS}`'),sum(x.count(k) for x in (c,j,v,h) for k in ('pv-win','pv-tab','previewTab','previewWindows')),j[j.index('function previewSectionEl'):j.index('/// 개요 보기 바꾸기')].count('innerHTML')))"
  EXPECT: box=1 grid=1 fit=1 more=1 tail=1 old=0 html=0
  EVIDENCE: pending
- [ ] E5: A5 — 문서 규약 · 크레이트 경계 · 매뉴얼 반영
  CHECK: python tools/check_ddd_docs.py && python tools/check_crates.py && python -c "import pathlib as P;w=P.Path('docs/manual/dev/web-ui.md').read_text(encoding='utf-8');g=P.Path('docs/manual/user/user-guide.md').read_text(encoding='utf-8');print('webui=%d guide=%d'%(w.count('#home_preview.pv-list'),g.count('| \x27개요\x27 미리보기(기본) | 상대마다 절')))"
  EXPECT: webui=1 guide=1
  EVIDENCE: pending
- [ ] E6: A1·A2·A3·A4 — 실제 화면: 디버그 빌드(시험 데이터 폴더, #1z-4 시드 — 상대 4 · 대화 8)에서 기본 미리보기 · 절 · 카드 · 폭별 열 수 · 한 줄 + "+n개" · 상대 개요 전부 · 빈 상대 점선 · 글자만 · 안 읽음 그대로
  EVIDENCE: pending

## 개정 기록

- 2026-10-22 · E4 CHECK 괄호 하나 많음(첫 실행 SyntaxError) 고침 — 검사 항목 · EXPECT 그대로.
- 2026-10-22 · 적대적 평가 R1 반영 — 전체 무효화(`previewAll`)를 쓸 때 한 줄 밖에 숨은 카드의 받아 둔 글도 `previewStale` 로(다시 보일 때 읽음). 설계 줄 그대로(규칙 이어받음의 보완),
  `ux_literals` 한 줄.
