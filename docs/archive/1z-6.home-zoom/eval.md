# eval.md — 1z-6.home-zoom 적대적 평가

## 평가 원장

<!-- tools/check_gates.py 산출 — 손으로 고치지 않는다. 단 CHECK 가 없는 수동 gate 의
     EVIDENCE 와 ABANDON 줄은 사람이 적고, 재실행해도 보존된다.
     WEAK 줄은 기계가 매 실행 다시 단다 — 약한 gate 를 셀 수 있게 하는 표시이며
     판정에는 영향이 없다. -->

측정 커밋: 638948b623b509525a731a79d4158646fa0edfd7

- [x] E1: A4 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

- [x] E2: A4 — clippy 경고 0
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.92s

- [x] E3: A2 — 웹UI 로직 시험(배율 → 본문 배수 · 줄 수 · 목록 밖 100%)
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# fail 0/
  EVIDENCE: # fail 0

- [x] E4: A3 — 설정 기억 · 400
  CHECK: cargo test -p coworkagent-app --test ux_api home_zoom
  EXPECT: /test result: ok\. 1 passed; 0 failed/
  EVIDENCE: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 47 filtered out; finished in 0.09s

- [x] E5: A1·A2 — 드롭다운 · 높이 · 줄 수 · 미리보기에서만 실재, 축소 값 없음
  CHECK: python -c "import pathlib as P;c=P.Path('crates/app/web/app.css').read_text(encoding='utf-8');j=P.Path('crates/app/web/app.js').read_text(encoding='utf-8');h=P.Path('crates/app/web/index.html').read_text(encoding='utf-8');r=P.Path('crates/core/src/run_opts.rs').read_text(encoding='utf-8');print('pick=%d body=%d clamp=%d only=%d def=%d small=%d'%(h.count('<label id=\x22home_zoom_pick\x22 class=\x22head-pick\x22>미리보기 크기 <select id=\x22ui_home_zoom\x22 data-num>'),c.count('.pv-body { height: calc(120px * var(--pv-scale));'),c.count('-webkit-line-clamp: var(--pv-lines);'),j.count('$(\x22home_zoom_pick\x22).hidden = !pv;'),r.count('pub const HOME_ZOOMS: [i64; 5] = [100, 125, 150, 175, 200];'),h.count('<option value=\x2275\x22>')+h.count('<option value=\x2250\x22>')))"
  EXPECT: pick=1 body=1 clamp=1 only=1 def=1 small=0
  EVIDENCE: pick=1 body=1 clamp=1 only=1 def=1 small=0

- [x] E6: A4 — 문서 규약 · 크레이트 경계 · 매뉴얼 반영
  CHECK: python tools/check_ddd_docs.py && python tools/check_crates.py && python -c "import pathlib as P;w=P.Path('docs/manual/dev/web-ui.md').read_text(encoding='utf-8');a=P.Path('docs/manual/dev/an_app.md').read_text(encoding='utf-8');g=P.Path('docs/manual/user/user-guide.md').read_text(encoding='utf-8');print('webui=%d app=%d guide=%d'%(w.count('select#ui_home_zoom'),a.count('ui_home_zoom 는 100 · 125 · 150 · 175 · 200'),g.count('미리보기 크기\x22 100 ~ 200%')))"
  EXPECT: webui=1 app=1 guide=1
  EVIDENCE: webui=1 app=1 guide=1

- [x] E7: A1·A2·A3 — 실제 화면: 디버그 빌드(시험 데이터 폴더, #1z-4 시드)에서 드롭다운 · 배율별 본문 높이 · 줄 수 · 글자 · 카드 폭 · 열 수 · 보이는 말풍선 · 다시 열어도 남음 · 목록 보기에서 숨음
  EVIDENCE: Claude(구현자) 2026-10-22 — `target/debug/coworkAgent.exe start --parent <PID>` + `COWORKAGENT_DATA_DIR`=scratchpad `ui1z5\data`(사용자 앱 그대로, #1z-4 시드 사본), ego-browser DOM 판독. 관측: 처음 설정 `ui_home_zoom` 100 · `#ui_home_zoom` "100" · 머리 순서 제목(left 468) → "목록 | 미리보기"(951) → "미리보기 크기"(1094) → 드롭다운 change 100/125/150/175/200 = 첫 카드(`#c/1`, 글 7) 본문 높이 138/173/207/242/276px · line-clamp 2/3/3/4/4 · 말풍선 글자 12px · 카드 폭 378 · 격자 2열 · 한 줄 칸 2 그대로 · 본문 안에 다 보이는 말풍선 3/3/4/5/6 → 설정 200 저장 · 새로 고침 뒤 "200" · 276px · 4줄 → 목록 보기 = `#home_zoom_pick` hidden. R1 뒤 같은 확인 반복(같은 값). R2 뒤: 125 → 150 → 175 → 200 을 잇달아 고름 = PUT 2회(125, 200 — 가운데는 합쳐짐, 차례대로) · 화면 `--pv-scale` 2 · 저장 200. 끝에 설정 100 으로 되돌리고 `coworkAgent stop`.


WEAK: E7 CHECK 없음 — 수동 gate(사람이 적은 EVIDENCE 로 판정)

## 적대적 평가 — 라운드 1/2

평가자: Orca 가 새로 띄운 Codex(`worker-start --agent codex`, 작업 폴더 `C:/Users/tskim/.claude/skills`), handle `term_dac56dd9-1ace-45b0-a65e-e7aff55f86a6`.
Run `run_808691f28bae` · Task `task_16b4a522df73` · Dispatch `ctx_5b77b01b7cf4`, worker_done(`msg_1721b51e7d4d`) 발신 handle 일치. 범위 `b83579f..849f7e9`(self-collect),
메타 `요청 1z-6-home-zoom-r1 · revision 849f7e98b9594c86456c70c72bd90965eb64a8e6 · 라운드 1/2`. 리뷰 동안 대상 작업 폴더 HEAD 849f7e9 그대로(그 사이 `cargo test` · clippy 만 — `target/`). 리뷰어 Dispatch `worker-release` → `released` · `captured`. 원문(render):

```text
# Codex Adversarial Review

Target: branch diff against b83579f
Verdict: needs-attention

지정 revision을 읽기 전용으로 검토하고 배율 매핑·PREVIEW_MSGS 및 실제 설정 적용 함수를 검사했다. 오래된 다른 설정 응답이 저장 완료한 미리보기 배율을 되돌리는 경합을 확인해 출하 보류를 권고한다. 응답 적용 범위 수정과 지연 응답 회귀 검증이 남았다.

Findings:
- [medium] 다른 설정의 늦은 응답이 저장한 배율을 되돌림 (crates/app/web/app.js:4270-4270)
  fillPrefs는 모든 설정 PUT 응답의 ui_home_zoom을 무조건 적용한다. 집중 설정 PUT이 zoom=100인 응답을 만든 뒤 전달이 지연되고, 그 사이 배율 PUT=200이 완료되면 늦은 집중 응답이 선택값과 CSS를 다시 100%·2줄로 바꾼다. DB는 200이므로 현재 화면과 재시작 화면도 달라진다. 실제 revision의 fillPrefs/applyHomeZoom을 stub DOM에서 200 응답→100 응답 순서로 호출해 --pv-scale 2→1, --pv-lines 4→2를 재현했다. 네트워크 순서 역전 자체는 미실측이며, 독립적인 setFocus 및 PREFS 요청과 전체 설정 응답에 근거한 추론이다.
  Recommendation: 시작 GET은 전체 설정을 적용하되 개별 PUT은 요청한 키만 적용하거나, 설정 응답에 단조 revision을 부여해 오래된 응답을 거부한다. 집중 PUT 응답을 지연하고 배율 200 저장 뒤 해제해 선택값·CSS·DB가 모두 200을 유지하는 회귀 검증을 추가한다.

Next steps:
- 다른 설정 응답으로 ui_home_zoom이 덮이지 않도록 수정하고 지연 응답 시나리오를 검증한다.
```

판정 · 조치:

| 지적 | 판정 | 조치 |
|---|---|---|
| R1-1 medium 다른 설정의 늦은 응답이 저장한 배율을 되돌림 | 사실 — `PREFS` 의 값은 모든 설정 응답을 `fillPrefs` 가 적용 | 배율을 `PREFS` 에서 빼고 시작할 때(`showSettings`) · 고를 때만 적용(`ui_home_view` 와 같은 규칙) · `ux_literals` 한 줄 · 실제 화면 재확인. 다른 `PREFS` 설정(테마 등)의 같은 경합은 이 Task 전부터의 구조 — 범위 밖(report) |

## 적대적 평가 — 라운드 2/2

평가자: Orca 가 새로 띄운 Codex, handle `term_361b7928-e6ef-48e5-8271-949cb2d555f0`. Run `run_808691f28bae` · Task `task_f4a70fad2956` · Dispatch `ctx_7f7c2aa4c94f`,
worker_done(`msg_6a911cc080eb`) 발신 handle 일치. 범위 `b83579f..677552c`(self-collect), 메타 `요청 1z-6-home-zoom-r2 · revision 677552c887783fb5c27234f7b060e7018b044d70 · 라운드 2/2`. 리뷰 동안 대상 작업 폴더 편집 없음(HEAD 677552c 그대로).
리뷰어 Dispatch `worker-release` → `released` · `captured`. 원문(render):

```text
# Codex Adversarial Review

Target: branch diff against b83579f
Verdict: needs-attention

고정 범위의 diff·결정 1~6·저장 경로를 읽기 전용 검토하고 배율 순수 로직을 확인했다. R1의 fillPrefs 덮어쓰기는 제거됐지만 연속 선택의 저장 순서 경합이 남아 출시 보류다. 저장 직렬화와 역순 처리 회귀 검증이 필요하다.

Findings:
- [medium] 연속 배율 선택의 동시 PUT이 마지막 선택을 이전 값으로 저장할 수 있음 (crates/app/web/app.js:1915-1919)
  change마다 화면을 즉시 바꾼 뒤 독립적인 PUT을 시작하며 선택 비활성화나 저장 직렬화가 없다. 125% 다음 200%를 고른 뒤 서버가 200 요청을 먼저, 125 요청을 나중에 처리하면 둘 다 성공해도 화면은 200%, 저장값은 125%가 되고 재시작 때 이전 배율로 돌아간다. put_settings는 도착한 patch를 그대로 update하므로 선택 순서를 복구하지 못한다. 지정 revision에서 추출한 실제 handler의 제어 실험은 동시 요청 [125,200], 역순 저장 후 displayed=200·persisted=125·warning 빈 값을 확인했다. 실서버에서 요청 처리 순서가 역전되는 상황은 미실측이며 해당 발생 조건은 추론이다.
  Recommendation: 화면 즉시 적용은 유지하되 기존 SendLatest로 ui_home_zoom PUT을 한 번에 하나만 보내고 대기 중 마지막 선택을 이어서 저장하라. 첫 저장을 지연시킨 연속 선택에서 최종 저장값이 마지막 선택과 일치하는 회귀 검증을 추가하라.

Next steps:
- 배율 저장에 SendLatest 직렬화를 적용하고 지연된 첫 요청·후속 선택의 최종 저장값을 검증한다.
- 라운드 2/2 종료이며 남은 지적은 coordinator가 처리한다.
```

판정 · 조치(상한 도달, 재리뷰 없음):

| 지적 | 판정 | 조치 |
|---|---|---|
| R2-1 medium 연속 배율 선택의 동시 PUT 이 마지막 선택을 이전 값으로 저장할 수 있음 | 사실 — 고를 때마다 따로 PUT, 서버는 도착 순서대로 씀 | 저장을 `SendLatest`(접힘 저장과 같은 도구 — 한 번에 하나, 보내는 동안 고른 마지막 값만 이어서) · `ux_literals` 한 줄 · 실제 화면(잇달아 넷 = PUT 125 → 200 차례대로 · 저장 200) |

## 미조치 지적

없음
