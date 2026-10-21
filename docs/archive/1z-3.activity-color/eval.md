# eval.md — 1z-3.activity-color 적대적 평가

## 평가 원장

<!-- tools/check_gates.py 산출 — 손으로 고치지 않는다. 단 CHECK 가 없는 수동 gate 의
     EVIDENCE 와 ABANDON 줄은 사람이 적고, 재실행해도 보존된다.
     WEAK 줄은 기계가 매 실행 다시 단다 — 약한 gate 를 셀 수 있게 하는 표시이며
     판정에는 영향이 없다. -->

측정 커밋: 20fc5f1bd957df4523c85c3fd1ae6883b494c5ca

- [x] E1: A4 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

- [x] E2: A4 — clippy 경고 0
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.24s

- [x] E3: A2·A4 — 웹UI 로직 시험(종류 색 키 포함)
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# fail 0/
  EVIDENCE: # fail 0

- [x] E4: A1·A2·A3 — 색 · 여백 규칙 실재
  CHECK: python -c "import pathlib as P;c=P.Path('crates/app/web/app.css').read_text(encoding='utf-8');j=P.Path('crates/app/web/app.js').read_text(encoding='utf-8');print('chips=%d tabs=%d picked=%d pad=%d tok=%d plain=%d js=%d'%(sum(c.count('.tag.act-%s {'%k) for k in ('request','mine','answer','failed')),sum(c.count('#act_tabs a[data-kind=\x22%s\x22] {'%k) for k in ('request','mine','answer','failed')),sum(c.count('#act_tabs a[data-kind=\x22%s\x22][aria-current=\x22page\x22] {'%k) for k in ('request','mine','answer','failed')),c.count('#agents_list li.has-edit { display: flex; align-items: center; gap: var(--sp-1); padding-right: var(--sp-5); }'),c.count('--answer-bg: var(--dark-answer-bg);')+c.count('--danger-soft: var(--dark-danger-soft);'),c.count('.tag.plain'),j.count('class: `tag act-${r.tone}`')))"
  EXPECT: chips=4 tabs=4 picked=4 pad=1 tok=2 plain=0 js=1
  EVIDENCE: chips=4 tabs=4 picked=4 pad=1 tok=2 plain=0 js=1

- [x] E5: A4 — 문서 규약 · 매뉴얼 반영
  CHECK: python tools/check_ddd_docs.py && python -c "import pathlib as P;w=P.Path('docs/manual/dev/web-ui.md').read_text(encoding='utf-8');print('webui=%d'%w.count('.tag.act-{tone}'))"
  EXPECT: webui=1
  EVIDENCE: webui=1

- [x] E6: A1·A2·A3 — 실제 화면: 디버그 빌드(시험 데이터 폴더)에서 에이전트 줄 오른쪽 여백 · 탭 글자색 · 고른 탭 바탕 · 칩 색 · 다크 테마 계산 값
  EVIDENCE: Claude(구현자) 2026-10-21 — `target/debug/coworkAgent.exe start --parent <PID>` + `COWORKAGENT_DATA_DIR`=scratchpad `ui1z3\data`(사용자 앱 그대로), ego-browser 로 에이전트 하나 등록 뒤 `#agents` · `#activity` 계산 값 판독(창 최소화로 스크린샷 불가 · 상대 없음 → 칩은 같은 클래스 요소를 잠깐 넣어 읽음). 관측: `#agents_list li.has-edit` padding-right 24px · 연필 오른쪽 ↔ 줄 끝 24px(줄 폭 811). 탭 글자 request rgb(128, 77, 12) · mine rgb(16, 74, 151) · answer rgb(12, 108, 104) · failed rgb(182, 50, 45) · 전체 rgb(23, 26, 29). 칩 바탕 request rgb(253, 241, 224) · mine rgb(232, 241, 254) · answer rgb(224, 241, 239) · failed rgb(251, 233, 231). 받은 요청 탭 고름 → aria-current page · 바탕 rgb(253, 241, 224) · 굵기 600. 다크 → 글자 rgb(234, 187, 121) · 바탕 rgb(48, 36, 18). 끝에 `coworkAgent stop` = "종료함".


WEAK: E6 CHECK 없음 — 수동 gate(사람이 적은 EVIDENCE 로 판정)

## 적대적 평가 — 라운드 1/2

평가자: Orca 가 새로 띄운 Codex(`worker-start --agent codex`, 작업 폴더 `C:/Users/tskim/.claude/skills`), handle `term_ace70654-a108-4b97-83e3-932e86f3c789`.
Run `run_9352c5c2aac9` · Task `task_6466d63da84a` · Dispatch `ctx_f32cdb9877be`, worker_done(`msg_44d3933dfa50`) 발신 handle 일치. 범위 `5c7e2ea..68e75af`(self-collect),
메타 `요청 1z-3-activity-color-r1 · revision 68e75afe98873c01f7633bb4f2236114b40cce0c · 라운드 1/2`. 리뷰 중 작업 폴더 깨끗(HEAD 9a73500 — 범위 뒤는 원장 커밋뿐). 리뷰어 Dispatch `worker-release` → `released`. 원문(render):

```text
# Codex Adversarial Review

Target: branch diff against 5c7e2ea
Verdict: approve

지정 diff와 intent 결정 1~6·plan E1~E6를 읽기 전용 대조했으며 차단할 실질 결함은 발견하지 못했다. label·tone 입력 600조합 불일치 0건, 칩·탭 대비 최저 5.14:1, CSS 우선순위와 .tag.plain 잔여 사용처 점검에서도 회귀 근거가 없었다. 빌드·전체 시험은 재실행하지 않았으며 상대 세션 생성 줄·좁은 화면·긴 이름의 실제 배치는 추가 실측 여지로 남는다.

No material findings.

Next steps:
- 상대 에이전트가 있는 화면에서 세션 생성 단추의 24px 여백을 좁은 화면·긴 이름 조합으로 실측해 E6 증거를 보완한다.
```

판정: 지적 없음 → 수정 없음, 라운드 2 불필요(approve). next step(상대 에이전트 줄 실측)은 시험 PC 에 연결된 상대가 없어 미실행 — 같은 규칙(`#agents_list li.has-edit`)이라 내 에이전트 줄 실측으로 갈음, 미조치 지적 없음.

## 미조치 지적

없음
