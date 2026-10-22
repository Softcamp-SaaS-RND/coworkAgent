# plan — 1z-6.home-zoom

> 근거: [intent](intent.md)(승인 2026-10-22 — 결정 1 ~ 6). **구현자는 이 문서만 보고 일한다.** D·S 는 [masterPlan](../../masterPlan.md)(S4 개요 · D35).
> 브랜치 `task/1z-6-home-zoom`(작업 폴더 `D:\dev\git\coworkAgent-1z-6`). 기준 커밋: `b83579f`(origin/main). plan 승인: 2026-10-22(요청 · 질문 답으로 갈음).

## 목표

개요 미리보기 머리에 "미리보기 크기" 100 ~ 200%(25% 단계) — 말풍선 칸 높이 · 줄 수만 키우고 설정에 기억.

### 성공 기준(intent 사본)

- A1: 개요 미리보기 머리에 "미리보기 크기" 드롭다운(100 ~ 200%, 25% 단계)이 미리보기 보기에서만 보인다(실제 화면).
- A2: 고르면 카드 본문 높이가 배율만큼(100% = 지금), 말풍선 줄 수가 2 · 3 · 3 · 4 · 4 로 바뀌고 글자 크기 · 카드 폭 · 한 줄 열 수는 그대로(시험 · 실제 화면).
- A3: 고른 배율이 설정 `ui_home_zoom` 에 남아 다시 열어도 같다 — 목록 밖 값은 400(시험 · 실제 화면).
- A4: 회귀 없음 — PROJECT.md 검증 명령 PASS, 매뉴얼 반영.

## 설계

- core `run_opts`: `HOME_ZOOM_KEY`(`ui_home_zoom`) · `HOME_ZOOMS` `[100, 125, 150, 175, 200]`(첫 값 = 기본) · `home_zoom(conn)`.
- app `notify`: `SettingsPatch.ui_home_zoom: Option<i64>`(틀린 값 400 "ui_home_zoom 는 100 · 125 · 150 · 175 · 200") · 쓰기 · `Notifier::home_zoom` · `settings_json`.
- web `view.js`: `HOME_ZOOMS` · `previewZoom(z)` → `{scale, lines}`(목록 밖 = 100%), `PREVIEW_MSGS` 12.
- web `app.js`: `applyHomeZoom`(`#home_preview` 에 `--pv-scale` · `--pv-lines` · 선택 값) — 시작할 때(`showSettings`) · 고를 때(change → `PUT /api/settings`)만,
  `renderHome` 이 `#home_zoom_pick` 을 미리보기에서만.
- web `index.html`: 개요 머리 `label#home_zoom_pick.head-pick` "미리보기 크기" `select#ui_home_zoom[data-num]`. `app.css`: `:root` `--pv-scale` · `--pv-lines`,
  `.pv-body` 높이 · `.pv-msg` 줄 수 · `.pv-more` 높이가 그 값.
- 시험: `ux_api` `home_zoom_setting_persists` · `view.test.mjs` `preview_zoom`(+ `PREVIEW_MSGS` 12) · `ux_literals`(#1z-6 네 줄).
- 문서: [web-ui](../../manual/dev/web-ui.md) · [an_app](../../manual/dev/an_app.md) 설정 · [an_core](../../manual/dev/an_core.md) `run_opts` ·
  [사용자 가이드](../../manual/user/user-guide.md) 화면 표 · PRODUCT · masterPlan S4 · 개정 기록(종결).

## 평가 항목

- [ ] E1: A4 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E2: A4 — clippy 경고 0
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: pending
- [ ] E3: A2 — 웹UI 로직 시험(배율 → 본문 배수 · 줄 수 · 목록 밖 100%)
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# fail 0/
  EVIDENCE: pending
- [ ] E4: A3 — 설정 기억 · 400
  CHECK: cargo test -p coworkagent-app --test ux_api home_zoom
  EXPECT: /test result: ok\. 1 passed; 0 failed/
  EVIDENCE: pending
- [ ] E5: A1·A2 — 드롭다운 · 높이 · 줄 수 · 미리보기에서만 실재, 축소 값 없음
  CHECK: python -c "import pathlib as P;c=P.Path('crates/app/web/app.css').read_text(encoding='utf-8');j=P.Path('crates/app/web/app.js').read_text(encoding='utf-8');h=P.Path('crates/app/web/index.html').read_text(encoding='utf-8');r=P.Path('crates/core/src/run_opts.rs').read_text(encoding='utf-8');print('pick=%d body=%d clamp=%d only=%d def=%d small=%d'%(h.count('<label id=\x22home_zoom_pick\x22 class=\x22head-pick\x22>미리보기 크기 <select id=\x22ui_home_zoom\x22 data-num>'),c.count('.pv-body { height: calc(120px * var(--pv-scale));'),c.count('-webkit-line-clamp: var(--pv-lines);'),j.count('$(\x22home_zoom_pick\x22).hidden = !pv;'),r.count('pub const HOME_ZOOMS: [i64; 5] = [100, 125, 150, 175, 200];'),h.count('<option value=\x2275\x22>')+h.count('<option value=\x2250\x22>')))"
  EXPECT: pick=1 body=1 clamp=1 only=1 def=1 small=0
  EVIDENCE: pending
- [ ] E6: A4 — 문서 규약 · 크레이트 경계 · 매뉴얼 반영
  CHECK: python tools/check_ddd_docs.py && python tools/check_crates.py && python -c "import pathlib as P;w=P.Path('docs/manual/dev/web-ui.md').read_text(encoding='utf-8');a=P.Path('docs/manual/dev/an_app.md').read_text(encoding='utf-8');g=P.Path('docs/manual/user/user-guide.md').read_text(encoding='utf-8');print('webui=%d app=%d guide=%d'%(w.count('select#ui_home_zoom'),a.count('ui_home_zoom 는 100 · 125 · 150 · 175 · 200'),g.count('미리보기 크기\x22 100 ~ 200%')))"
  EXPECT: webui=1 app=1 guide=1
  EVIDENCE: pending
- [ ] E7: A1·A2·A3 — 실제 화면: 디버그 빌드(시험 데이터 폴더, #1z-4 시드)에서 드롭다운 · 배율별 본문 높이 · 줄 수 · 글자 · 카드 폭 · 열 수 · 보이는 말풍선 · 다시 열어도 남음 · 목록 보기에서 숨음
  EVIDENCE: pending

## 개정 기록

- 2026-10-22 · 적대적 평가 R1 반영 — 배율을 `PREFS`(모든 설정 응답이 `fillPrefs` 로 덮음)에서 빼고 시작할 때 · 고를 때만 적용(다른 설정 저장의 늦은 응답이
  막 고른 배율을 되돌리던 결함). 설계 줄 갱신, EXPECT 그대로, `ux_literals` 한 줄.
- 2026-10-22 · 적대적 평가 R2 반영 — 배율 저장을 `SendLatest`(한 번에 하나, 보내는 동안 고른 마지막 값만 이어서)로. 연속 선택의 동시 PUT 이 앞 값으로 끝나던 결함.
  설계 줄 그대로(저장 방식의 보완), `ux_literals` 한 줄. 상한 2/2 — 재리뷰 없음.
