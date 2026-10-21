# plan — 05-2.parent-read

> 근거: [intent](intent.md)(승인 2026-10-21 — 결정 1 ~ 7). **구현자는 이 문서만 보고 일한다.** D·S 는 [masterPlan](../../masterPlan.md)
> (D21 에이전트 프로필 · D22 헤드리스 권한 — 종결 때 환류). 브랜치 `task/05-2-parent-read`(작업 폴더 `D:\dev\git\coworkAgent-05-2` — #30 이 열린 채라 따로). 기준 커밋: `5c7e2ea`(origin/main).
> plan 승인: 2026-10-21(질문 답으로 갈음 — intent 결정 7).

## 목표

에이전트별 "상위 한단계 탐색 허용"(기본 끔) — 켜면 `claude -p` 가 에이전트 폴더의 바로 위 폴더를 `--add-dir` 로 더 읽는다(드라이브 루트 · 홈 · 데이터 루트를 품는 위는 제외).

### 성공 기준(intent 사본)

- A1: 옵션을 켠 에이전트의 실행 인자에 `--add-dir <바로 위 폴더>` 가 붙고, 끄면 · 막힌 위(드라이브 루트 · 홈 · 데이터 루트를 품음)면 붙지 않는다(시험).
- A2: 실제 `claude` 를 제품 인자로 돌려 — 켜면 바로 위 폴더 파일을 읽고, 끄면 거부, 켜도 두 단계 위는 거부(실측).
- A3: 등록 화면 · 에이전트 상세에 "상위 한단계 탐색 허용" 체크박스가 있고 저장 · 다시 읽기에 그 값이 남는다. 켰을 때 상세에 실제 여는 위 폴더(또는 "열지 않음")가 보인다(시험 · 실제 화면).
- A4: 있던 에이전트는 끔(스키마 올림 기본값), 옵션을 바꾸면 그 에이전트의 세션을 비운다(시험).
- A5: 회귀 없음 — PROJECT.md 검증 명령 PASS, 매뉴얼 · PRODUCT 반영(검사).

## 설계

### core

- 스키마 24: `agents.parent_read INTEGER NOT NULL DEFAULT 0`(`has_column` 로 건너뜀 — 스키마 23 방식), `SCHEMA_VERSION` 24.
- `Agent.parent_read: bool` · `COLS` · `row`. `add` 시그니처 그대로(끔) + `add_with(…, parent_read)`. `update(…, group, parent_read)` — 폴더나 `parent_read` 가 바뀌면 새 `uid` · `forget_sessions`.
- `paths::parent_read_dir(folder) -> Option<PathBuf>` — `std::path::absolute(folder)` 의 `parent()`. `None`: 위 없음 · 위가 드라이브 루트(그대로 · 정규화 둘 다) ·
  위 = `dirs::home_dir()`(`path_key` 비교) · `agent_folder_problem(위)`. 돌려주는 값은 정규화 전(`\\?\` 접두 없이).

### agent

- `claude_args`: `agent.parent_read` 이고 `parent_read_dir` 가 `Some(up)` 이면 `--add-dir <up>` + 시스템 프롬프트 가드 뒤에 `[읽기 범위]` 한 줄, `None` 이면 `log::info!` 한 줄.
  실행 · 압축 모두 `run_claude` → `claude_args` 를 지나 같은 판단.
- 시험: `tests/runner.rs` `parent_read_adds_dir`(임시 폴더 — 켬 = `--add-dir` 위 · 끔 · 위가 드라이브 루트 = 없음), `tests/parent_read_canary.rs`(`#[ignore]` 실제 claude — 켬 · 끔 · 두 단계 위).

### app

- `NewAgent.parent_read`(`#[serde(default)]`) → `add_with`, `AgentPatch.parent_read: Option<bool>` → `update`, `agent_json` 에 `parent_read` · `parent_dir`(실제 여는 위 폴더, 없으면 `null`).
- 웹UI: 등록 칸 `index.html` `agent_form` 폴더 다음 `<label for="agent_parent">상위 한단계 탐색 허용</label>` + 체크박스(title = 도움말), `app.js` 등록 body ·
  상세(`renderAgentList`) 체크박스 + 켰을 때 "읽는 위 폴더" 줄(`parent_dir` 또는 "열지 않음(…)") · 저장 PATCH. 도움말 글 `view.js` `PARENT_NOTE`.

### 문서

- [an_core](../../manual/dev/an_core.md) · [an_agent](../../manual/dev/an_agent.md) · [an_app](../../manual/dev/an_app.md) · [개발자 가이드](../../manual/dev/developer-guide.md)(실행 인자) ·
  [사용자 가이드](../../manual/user/user-guide.md) 6장 · `PRODUCT.md`(Claude 실행). masterPlan(종결 환류): D21 · D22 개정 · Phase 0 표 · 개정 기록.

## 평가 항목

- [ ] E1: A4·A5 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E2: A5 — clippy 경고 0
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: pending
- [ ] E3: A1·A3·A4 — 상위 폴더 판단 · 실행 인자 · API 시험
  CHECK: cargo test -p an-core --test paths parent_read && cargo test -p an-agent --test runner parent_read && cargo test -p coworkagent-app --test chat_api agents_crud_api
  EXPECT: /test result: ok\. 1 passed; 0 failed/
  EVIDENCE: pending
- [ ] E4: A2 — 실제 claude(사용량): 켬 = 바로 위 읽음 · 끔 = 거부 · 켬 + 두 단계 위 = 거부
  CHECK: cargo test -p an-agent --test parent_read_canary -- --ignored
  EXPECT: /test result: ok\. 1 passed; 0 failed/
  EVIDENCE: pending
- [ ] E5: A3·A5 — 웹UI 로직 시험 · 체크박스 라벨 literal · 스키마 24 실재
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs" && python -c "import pathlib as P;h=P.Path('crates/app/web/index.html').read_text(encoding='utf-8');j=P.Path('crates/app/web/app.js').read_text(encoding='utf-8');d=P.Path('crates/core/src/db.rs').read_text(encoding='utf-8');r=P.Path('crates/agent/src/runner.rs').read_text(encoding='utf-8');L='\uc0c1\uc704 \ud55c\ub2e8\uacc4 \ud0d0\uc0c9 \ud5c8\uc6a9';print('box=%d label=%d js=%d schema=%d adddir=%d'%(h.count('<input type=\x22checkbox\x22 id=\x22agent_parent\x22'),h.count(L)+j.count(L),j.count('parent_read: parent.checked')+j.count('parent_read: $(\x22agent_parent\x22).checked'),d.count('pub const SCHEMA_VERSION: i64 = 24;'),r.count('a.push(\x22--add-dir\x22.into());')))"
  EXPECT: box=1 label=2 js=2 schema=1 adddir=1
  EVIDENCE: pending
- [ ] E6: A5 — 문서 규약 · 크레이트 경계 · 매뉴얼 · PRODUCT 반영
  CHECK: python tools/check_ddd_docs.py && python tools/check_crates.py && python -c "import pathlib as P;t=lambda f:P.Path(f).read_text(encoding='utf-8');L='\uc0c1\uc704 \ud55c\ub2e8\uacc4 \ud0d0\uc0c9 \ud5c8\uc6a9';print('core=%d agent=%d app=%d dev=%d user=%d product=%d'%(t('docs/manual/dev/an_core.md').count('parent_read_dir(folder)'),t('docs/manual/dev/an_agent.md').count('--add-dir <'),t('docs/manual/dev/an_app.md').count('parent_dir'),t('docs/manual/dev/developer-guide.md').count('[--add-dir <'),min(1,t('docs/manual/user/user-guide.md').count(L)),min(1,t('PRODUCT.md').count(L))))"
  EXPECT: core=1 agent=1 app=1 dev=1 user=1 product=1
  EVIDENCE: pending
- [ ] E7: A3 — 실제 화면: 디버그 빌드(시험 데이터 폴더)로 등록 화면 체크 → 등록 → 상세에 체크 · "읽는 위 폴더" → 끄고 저장 → 다시 읽기에 끔 · 위가 홈인 폴더 = "열지 않음"
  EVIDENCE: pending

## 개정 기록

- 없음
