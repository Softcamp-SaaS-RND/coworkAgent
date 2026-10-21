# plan — 1z-3.activity-color

> 근거: [intent](intent.md)(승인 2026-10-21 — 결정 1 ~ 6). **구현자는 이 문서만 보고 일한다.** D·S 는 [masterPlan](../../masterPlan.md)(S4 내 활동).
> 브랜치 `task/1z-3-activity-color`(작업 폴더 `D:\dev\git\coworkAgent-1z-3`). 기준 커밋: `5c7e2ea`(origin/main). plan 승인: 2026-10-21(시안 선택으로 갈음).

## 목표

등록된 에이전트 줄 오른쪽 여백 24px, 내 활동 종류 색(시안 A — 칩 + 같은 색 탭).

### 성공 기준(intent 사본)

- A1: 에이전트 줄 오른쪽 여백 24px — 단추 · 연필과 줄 끝 사이 24px(실제 화면).
- A2: 내 활동 줄 이름표가 종류별 색 칩(받은 요청 · 내 요청 · 받은 답 · 실패, 실패한 답 · 상대 반영 실패 = 실패 색)(시험 · 실제 화면).
- A3: 거르기 탭 받은 요청 · 내 요청 · 받은 답 · 실패가 칩과 같은 글자색, 고른 탭은 칩과 같은 바탕, 다크 테마도(실제 화면).
- A4: 회귀 없음 — PROJECT.md 검증 명령 PASS, 매뉴얼 반영.

## 설계

- `view.js` `activityRows`: 행에 `tone` — `failed`(내 쪽 실패 · 실패한 답 · 상대 반영 실패) · `mine` · `request` · `answer`(탭 `ACT_KINDS.kind` 와 같은 키).
- `app.js` `activityEls`: 이름표 `tag act-{tone}`(옛 `tag plain`).
- `app.css`: 토큰 `--answer-bg` · `--danger-soft`(+ 다크), `.tag.act-*` 넷, `#act_tabs a[data-kind=…]` 글자색 · 고른 탭 바탕 · 굵게, `#agents_list li.has-edit` `padding-right: var(--sp-5)`,
  `.tag.plain` 삭제.
- 시험 `view.test.mjs` `activity_rows` 에 `tone` 단언.
- 문서: [web-ui](../../manual/dev/web-ui.md) 내 활동 · [사용자 가이드](../../manual/user/user-guide.md) 내 활동 · masterPlan S4 · 개정 기록(종결).

## 평가 항목

- [ ] E1: A4 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E2: A4 — clippy 경고 0
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: pending
- [ ] E3: A2·A4 — 웹UI 로직 시험(종류 색 키 포함)
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# fail 0/
  EVIDENCE: pending
- [ ] E4: A1·A2·A3 — 색 · 여백 규칙 실재
  CHECK: python -c "import pathlib as P;c=P.Path('crates/app/web/app.css').read_text(encoding='utf-8');j=P.Path('crates/app/web/app.js').read_text(encoding='utf-8');print('chips=%d tabs=%d picked=%d pad=%d tok=%d plain=%d js=%d'%(sum(c.count('.tag.act-%s {'%k) for k in ('request','mine','answer','failed')),sum(c.count('#act_tabs a[data-kind=\x22%s\x22] {'%k) for k in ('request','mine','answer','failed')),sum(c.count('#act_tabs a[data-kind=\x22%s\x22][aria-current=\x22page\x22] {'%k) for k in ('request','mine','answer','failed')),c.count('#agents_list li.has-edit { display: flex; align-items: center; gap: var(--sp-1); padding-right: var(--sp-5); }'),c.count('--answer-bg: var(--dark-answer-bg);')+c.count('--danger-soft: var(--dark-danger-soft);'),c.count('.tag.plain'),j.count('class: `tag act-${r.tone}`')))"
  EXPECT: chips=4 tabs=4 picked=4 pad=1 tok=2 plain=0 js=1
  EVIDENCE: pending
- [ ] E5: A4 — 문서 규약 · 매뉴얼 반영
  CHECK: python tools/check_ddd_docs.py && python -c "import pathlib as P;w=P.Path('docs/manual/dev/web-ui.md').read_text(encoding='utf-8');print('webui=%d'%w.count('.tag.act-{tone}'))"
  EXPECT: webui=1
  EVIDENCE: pending
- [ ] E6: A1·A2·A3 — 실제 화면: 디버그 빌드(시험 데이터 폴더)에서 에이전트 줄 오른쪽 여백 · 탭 글자색 · 고른 탭 바탕 · 칩 색 · 다크 테마 계산 값
  EVIDENCE: pending

## 개정 기록

- 없음
