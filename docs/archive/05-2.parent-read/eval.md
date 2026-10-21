# eval.md — 05-2.parent-read 적대적 평가

## 평가 원장

<!-- tools/check_gates.py 산출 — 손으로 고치지 않는다. 단 CHECK 가 없는 수동 gate 의
     EVIDENCE 와 ABANDON 줄은 사람이 적고, 재실행해도 보존된다.
     WEAK 줄은 기계가 매 실행 다시 단다 — 약한 gate 를 셀 수 있게 하는 표시이며
     판정에는 영향이 없다. -->

측정 커밋: 20fc5f1bd957df4523c85c3fd1ae6883b494c5ca

- [x] E1: A4·A5 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

- [x] E2: A5 — clippy 경고 0
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.24s

- [x] E3: A1·A3·A4 — 상위 폴더 판단 · 실행 인자 · API 시험
  CHECK: cargo test -p an-core --test paths parent_read && cargo test -p an-agent --test runner parent_read && cargo test -p coworkagent-app --test chat_api agents_crud_api
  EXPECT: /test result: ok\. 1 passed; 0 failed/
  EVIDENCE: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 8 filtered out; finished in 0.10s

- [x] E4: A2 — 실제 claude(사용량): 켬 = 바로 위 읽음 · 끔 = 거부 · 켬 + 두 단계 위 = 거부
  CHECK: cargo test -p an-agent --test parent_read_canary -- --ignored
  EXPECT: /test result: ok\. 1 passed; 0 failed/
  EVIDENCE: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 18.10s

- [x] E5: A3·A5 — 웹UI 로직 시험 · 체크박스 라벨 literal · 스키마 24 실재
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs" && python -c "import pathlib as P;h=P.Path('crates/app/web/index.html').read_text(encoding='utf-8');j=P.Path('crates/app/web/app.js').read_text(encoding='utf-8');d=P.Path('crates/core/src/db.rs').read_text(encoding='utf-8');r=P.Path('crates/agent/src/runner.rs').read_text(encoding='utf-8');L='\uc0c1\uc704 \ud55c\ub2e8\uacc4 \ud0d0\uc0c9 \ud5c8\uc6a9';print('box=%d label=%d js=%d schema=%d adddir=%d'%(h.count('<input type=\x22checkbox\x22 id=\x22agent_parent\x22'),h.count(L)+j.count(L),j.count('parent_read: parent.checked')+j.count('parent_read: $(\x22agent_parent\x22).checked'),d.count('pub const SCHEMA_VERSION: i64 = 24;'),r.count('a.push(\x22--add-dir\x22.into());')))"
  EXPECT: box=1 label=2 js=2 schema=1 adddir=1
  EVIDENCE: box=1 label=2 js=2 schema=1 adddir=1

- [x] E6: A5 — 문서 규약 · 크레이트 경계 · 매뉴얼 · PRODUCT 반영
  CHECK: python tools/check_ddd_docs.py && python tools/check_crates.py && python -c "import pathlib as P;t=lambda f:P.Path(f).read_text(encoding='utf-8');L='\uc0c1\uc704 \ud55c\ub2e8\uacc4 \ud0d0\uc0c9 \ud5c8\uc6a9';print('core=%d agent=%d app=%d dev=%d user=%d product=%d'%(t('docs/manual/dev/an_core.md').count('parent_read_dir(folder)'),t('docs/manual/dev/an_agent.md').count('--add-dir <'),t('docs/manual/dev/an_app.md').count('parent_dir'),t('docs/manual/dev/developer-guide.md').count('[--add-dir <'),min(1,t('docs/manual/user/user-guide.md').count(L)),min(1,t('PRODUCT.md').count(L))))"
  EXPECT: core=1 agent=1 app=1 dev=1 user=1 product=1
  EVIDENCE: core=1 agent=1 app=1 dev=1 user=1 product=1

- [x] E7: A3 — 실제 화면: 디버그 빌드(시험 데이터 폴더)로 등록 화면 체크 → 등록 → 상세에 체크 · "읽는 위 폴더" → 끄고 저장 → 다시 읽기에 끔 · 위가 홈인 폴더 = "열지 않음"
  EVIDENCE: Claude(구현자) 2026-10-21 — `target/debug/coworkAgent.exe start --parent <PID>` + `COWORKAGENT_DATA_DIR`=scratchpad `ui052\data`(사용자 앱 그대로), ego-browser 로 `#settings/agents` 조작 · DOM 판독(창 최소화로 스크린샷 불가). 관측: 등록 칸 `label[for=agent_parent]` = "상위 한단계 탐색 허용" · 체크 → "등록 완료" → 상세 체크 = true · "읽는 위 폴더 …\ui052\proj" → 끄고 "저장" = "에이전트 저장 완료" → 새로 고침 뒤 체크 = false · 위 폴더 줄 없음 → `C:\Users\tskim\Desktop` 켜고 등록 → 상세 "읽는 위 폴더 열지 않음(드라이브 루트 · 사용자 홈 · 데이터 폴더를 품음)". 끝에 `coworkAgent stop` = "종료함".


WEAK: E7 CHECK 없음 — 수동 gate(사람이 적은 EVIDENCE 로 판정)

## 적대적 평가 — 라운드 1/2

평가자: Orca 가 새로 띄운 Codex(`worker-start --agent codex`, 작업 폴더 `C:/Users/tskim/.claude/skills`), handle `term_06d42507-6ba8-4ff0-92c0-0d42d4b00cad`.
Run `run_4593c3b57db1` · Task `task_b8c94056b97b` · Dispatch `ctx_c672c0fd42fa`, worker_done(`msg_97fb348b5574`) 발신 handle 일치. 범위 `5c7e2ea..cb6e340`(self-collect),
메타 `요청 05-2-parent-read-r1 · revision cb6e3404acb55b64d3cf94abd652b0f4c07f6ba7 · 라운드 1/2`. 리뷰 중 작업 폴더 깨끗(HEAD 3339b16 — 범위 뒤는 원장 커밋뿐). 리뷰어 Dispatch `worker-release` → `released`. 원문(render):

```text
# Codex Adversarial Review

Target: branch diff against 5c7e2ea
Verdict: needs-attention

고정 범위의 경로 차단·실행·압축·API·UI·명세를 읽기 전용으로 검토했다. 압축 대기 중 옵션을 끄면 옛 세션을 새 세대의 요약으로 되살릴 수 있어 출하 보류가 필요하다. 세션·세대 결합 수정과 경쟁 조건 회귀 검증이 남았으며 런타임 재현은 수행하지 않았다.

Findings:
- [high] 압축 대기 중 옵션을 끈 경우 옛 세션이 새 세대 요약으로 복원됨 (crates/core/src/agents.rs:86-91)
  여기서 parent_read 변경 시 uid와 DB 세션·요약을 비우지만, compact는 이미 세션 ID를 메모리에 보관할 수 있다. crates/agent/src/chat.rs:3864에서 세션을 읽고 3871~3873에서 전역 실행 자리를 기다리는 동안 parent_read를 true→false로 바꾸면 이 초기화가 실행된다. 이후 compact는 3886에서 변경 후 Agent를 읽어 새 uid를 붙인 채 3903에서 옛 세션을 Resume한다. 따라서 3817~3821의 uid 비교도 통과하여 위 폴더 내용이 포함된 옛 세션의 요약을 ctx_seeds에 다시 저장하고, 다음 끔 상태 실행이 이를 입력으로 받는다. intent 결정 4를 위반한다. 정적 호출 순서에 근거한 경쟁 조건 추론이며 런타임 재현은 미실행이다.
  Recommendation: compact가 실행 자리를 얻은 뒤 Conversation도 다시 읽고, 세션 ID와 해당 Agent의 uid를 동일 DB 스냅샷에서 묶어 가져오도록 수정한다. 실행·저장 시 그 결합을 검증하고, 대기 중 parent_read를 끈 경우 옛 세션을 Resume하거나 ctx_seed를 복원하지 않는 결정적 회귀 테스트를 추가한다.

Next steps:
- 압축의 세션·uid 스냅샷과 재검증 수정 후, 실행 상한을 채운 상태에서 압축 대기→옵션 끔→자리 해제 순서로 회귀 검증한다.
```

판정 · 조치(`87bb64a`):

| 지적 | 판정 | 조치 |
|---|---|---|
| R1-1 high 압축 대기 중 옵션을 바꾸면 옛 세션을 새 세대로 요약 | 사실 — 세션은 실행 자리 대기 전에, 세대는 뒤에 읽는다(폴더 변경도 같은 경로 — #1m 부터 있던 틈을 이번 옵션이 한 번 더 연다) | `compact` 가 세션마다 실행 직전 한 DB 잠금 안에서 대화를 다시 읽어 그 세션이 아직 있는지 · 에이전트 세대를 함께 읽고, 없으면 건너뜀. 회귀 시험 `context::compact_skips_session_cleared_while_waiting`(첫 세션 요약 중 다른 에이전트 옵션 변경 → 그 세션 Resume 없음 · 요약 없음) — 수정 전 코드로 실패 확인 |

시험: `cargo test -p an-agent --test context` 11 passed(수정 전 새 시험 FAILED), gate 재측정(위 원장).

## 적대적 평가 — 라운드 2/2

평가자: Orca 가 새로 띄운 Codex(`worker-start --agent codex`, 작업 폴더 `C:/Users/tskim/.claude/skills`), handle `term_1f84e6be-c060-4bc0-838c-1d9c31e3fff4`.
Run `run_4593c3b57db1` · Task `task_530b657ad8a3` · Dispatch `ctx_ada59175ddc5`, worker_done(`msg_c30e727d917e`) 발신 handle 일치. 범위 `5c7e2ea..500fc77`(self-collect),
메타 `요청 05-2-parent-read-r2 · revision 500fc778f52e80c38c07a32be402ed7873aa8eec · 라운드 2/2`. 리뷰 중 작업 폴더 깨끗 · HEAD 500fc77 그대로. 리뷰어 Dispatch `worker-release` → `released`. 원문(render):

```text
# Codex Adversarial Review

Target: branch diff against 5c7e2ea
Verdict: needs-attention

고정 revision의 변경·명세·R1 수정을 읽기 전용으로 검토했다. 자동 왕복에서 옵션을 끈 뒤 옛 세션을 새 세대로 복원할 수 있어 출하 보류가 필요하다. 세션·실행 에이전트의 원자적 조회와 경쟁 조건 회귀 검증이 남았다.

Findings:
- [high] 자동 왕복은 옵션 변경 후에도 옛 세션을 새 uid로 복원할 수 있음 (crates/agent/src/chat.rs:3076-3080)
  process는 3050~3060에서 conv.session을 읽지만, 자동 왕복은 여기서 읽은 Agent를 버리고 3076~3080에서 relay_agent를 별도 DB 잠금으로 다시 읽는다. 두 조회 사이 parent_read를 true→false로 PATCH하면 agents::update가 DB 세션을 비우고 uid를 바꿔도 메모리의 conv.session은 남는다. execute의 3456~3459는 그 옛 세션을 Resume하고, 새 Agent의 parent_read=false·새 uid로 실행하므로 record의 3961 uid 비교도 통과하여 세션을 다시 저장한다. 따라서 상위 폴더에서 이미 읽은 내용이 끔 상태의 후속 실행·압축으로 이어져 intent 결정 4를 위반한다. 정적 호출 순서에 근거한 경쟁 조건 추론이며 런타임 재현은 미실행이다. 87bb64a의 compact 재확인은 이 일반 자동 왕복 경로에 적용되지 않는다.
  Recommendation: process의 한 DB 잠금 안에서 실제 실행 주체(relay_agent 포함)를 결정하고 해당 Agent·uid·세션을 함께 읽어라. 이후 에이전트만 별도로 다시 읽지 말고, 대화 조회→parent_read 끔→왕복 실행 순서를 강제하는 회귀 테스트로 옛 세션 Resume·복원·요약이 모두 없는지 검증하라.

Next steps:
- 자동 왕복의 세션·uid 조회를 결합하고 옵션 변경 경쟁 조건 회귀 테스트를 추가한다.
```

판정 · 조치(`dd2e624`):

| 지적 | 판정 | 조치 |
|---|---|---|
| R2-1 high 자동 왕복이 왕복 에이전트를 대화와 다른 DB 잠금으로 다시 읽음 | 사실 — 두 잠금 사이(같은 스레드, `await` 없음 — 매우 좁다)에 옵션 · 폴더가 바뀌면 옛 세션 + 새 세대(폴더 변경도 같은 틈, #12 부터) | `process` 의 대화 읽기와 같은 잠금 안에서 실행 에이전트를 정한다(자동 왕복 = `conv.relay.agent`, 멘션 = 부른 에이전트, 그 밖 = 대화 에이전트). 뒤의 별도 조회 삭제 |

시험: `cargo test -p an-agent --test relay --test context --test conv_leave` 11 · 13 · 12 passed, clippy Finished, gate 재측정(위 원장). 라운드 2/2 — 재리뷰 없음(상한).

## 미조치 지적

| 지적 원문 | 판정 | 미조치 사유 |
|---|---|---|
| R2-1 Recommendation "대화 조회→parent_read 끔→왕복 실행 순서를 강제하는 회귀 테스트" | 사실(시험 없음) | 고친 뒤 두 조회가 한 잠금 안이라 사이에 끼울 자리가 없다 — 순서를 강제할 시험 훅을 제품 코드에 넣어야 한다. 대신 왕복 · 압축 · 나가기 시험 전부 통과로 동작 회귀만 확인 |
| (R1 · R2 공통 귀결) 판단 시점 뒤 · 실행 중에 옵션을 끄면 그 실행은 이미 정한 옛 세션 · 인자로 끝까지 돈다 | 사실(설계 한계) | 끝난 실행은 세대가 달라 세션 · 요약을 남기지 않는다(`record` · `compact` 저장의 `uid` 비교) — 폴더 변경과 같은 규칙(#1h). 진행 중 실행 중단은 범위 밖 |
