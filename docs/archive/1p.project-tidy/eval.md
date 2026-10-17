# eval.md — 1p.project-tidy 적대적 평가

## 평가 원장

<!-- tools/check_gates.py 산출 — 손으로 고치지 않는다. 단 CHECK 가 없는 수동 gate 의
     EVIDENCE 와 ABANDON 줄은 사람이 적고, 재실행해도 보존된다.
     WEAK 줄은 기계가 매 실행 다시 단다 — 약한 gate 를 셀 수 있게 하는 표시이며
     판정에는 영향이 없다. -->

측정 커밋: 6cfad0a96cdba779e70d9a687484f52b938a7a6a

- [x] E1: A1·A2 — 필독 4문서 합 20KB 이하 · `CLAUDE.md` 필독 절이 정확히 STATE · PROJECT · ddd-workflow · `PRODUCT.md` 15KB 이하
  CHECK: python {TASK}/design/gates.py size
  EXPECT: /files=4 bytes=\d+ under20k=True must_read_ok=True product=\d+ under15k=True/
  EVIDENCE: files=4 bytes=18915 under20k=True must_read_ok=True product=15253 under15k=True

- [x] E2: A1·A2 — 줄인 4문서(`CLAUDE.md` · `PROJECT.md` · `PRODUCT.md` · ddd-workflow)에서 지운 줄의 사실(ASCII 코드 이름 · 단위 붙은 수)이 살아 있는 문서에 모두 있다(기준 `ceaaa6f`)
  CHECK: python {TASK}/design/gates.py moved ceaaa6f
  EXPECT: targets=167 missing=0
  EVIDENCE: targets=167 missing=0

- [x] E3: A3 — `notice_for` 0 · `db_err` 정의 1(an_core) · `an-agent` `serde` 의존 없음 · `ux_literals.rs` 시험 6개 이하
  CHECK: python {TASK}/design/gates.py code
  EXPECT: /notice_for=0 db_err_defs=1 agent_serde=False ux_tests=[1-6] end/
  EVIDENCE: notice_for=0 db_err_defs=1 agent_serde=False ux_tests=5 end

- [x] E4: A4 — `app.css` 첫 `:root` 밖 색 · 글꼴 크기 · 굵기 · 줄높이 · 둥글기 · 간격 리터럴 0(검사한 선언 300개 이상을 함께 단정)
  CHECK: cargo test -p coworkagent-app --test css_tokens
  EXPECT: /test result: ok\. [1-9]\d* passed; 0 failed/
  EVIDENCE: test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

- [x] E5: A5 — `var(--mono)` 를 쓰는 선택자가 허용 목록과 같다
  CHECK: python {TASK}/design/gates.py mono
  EXPECT: mono=.count|.fp|.kbd|.md code|.mono|.raw|.steps .at|kbd end
  EVIDENCE: mono=.count|.fp|.kbd|.md code|.mono|.raw|.steps .at|kbd end

- [x] E6: A6 — plan-copy 승인(✅) 행이 화면 원본에 모두 반영(현재 문구 사라짐 · 제안 문구 있음)되고 사용자 가이드 · README 에 옛 문구 0
  CHECK: python {TASK}/design/gates.py copy
  EXPECT: rows=193 applied=193 stale=0 doc_stale=0
  EVIDENCE: rows=193 applied=193 stale=0 doc_stale=0

- [x] E7: A7·A9 — web-ui.md 용어 표가 있고(개념 1개 이상) 쓰지 않는 말이 화면 문구에 0
  CHECK: python {TASK}/design/gates.py terms
  EXPECT: terms=7 banned=16 banned_hits=0
  EVIDENCE: terms=7 banned=16 banned_hits=0

- [ ] E8: A8 — 실제 앱(릴리스 빌드)에서 다크 · 라이트 주요 화면(개요 · 대화 · 세부정보 · 설정 · 새 대화)을 바꾸기 전 캡처와 눈으로 비교 — 글꼴 · 크기 · 간격이 어긋나거나 잘린 곳 없음(수동, D28 이면 19 인수)
  EVIDENCE: pending

- [x] E9: A9 — `target-t12/` · 빈 워크트리 폴더 없음 · `STATE.md` 낡은 항목 0 · README · 사용자 가이드 캡처 전부 새로 · 참조 없는 캡처 0
  CHECK: python {TASK}/design/gates.py extra ceaaa6f
  EXPECT: t12=False wt=False state_stale=0 shots=5 stale_shots=0 orphans=0
  EVIDENCE: t12=False wt=False state_stale=0 shots=5 stale_shots=0 orphans=0

- [x] E10: A8·A10 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9]\d* passed; 0 failed/
  EVIDENCE: test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

- [x] E11: A8·A10 — 우리 코드 경고 0(clippy -D warnings) · node 화면 시험 실패 0
  CHECK: cargo clippy --workspace --all-targets -- -D warnings && node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# pass [1-9]\d*\s+# fail 0/
  EVIDENCE: # todo 0 / # duration_ms 118.7416 / Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.22s

- [x] E12: A10 — 문서 규약 · README · 의존 경계 체커 통과
  CHECK: python tools/check_ddd_docs.py && python tools/check_readme.py && python tools/check_crates.py
  EXPECT: /crates=4 edges=\d+ violations=0/
  EVIDENCE: crates=4 edges=6 violations=0

- [x] E13: A11 — 고른 시안의 토큰(`design/tokens.json`)이 `app.css` 첫 `:root` 값과 모두 같고, 시안이 정한 AI 티 패턴이 화면 원본에 0
  CHECK: python {TASK}/design/gates.py design
  EXPECT: tokens=92 mismatch=0 slop_rules=7 slop=0
  EVIDENCE: tokens=92 mismatch=0 slop_rules=7 slop=0

- [x] E14: A11 — 첫 `:root` 토큰으로 계산한 글자/바탕 대비(짝 37 × 2 테마 · 타일 흰 글자 6 × 2 · 말풍선 18 × 2)가 모두 4.5:1 이상
  CHECK: python {TASK}/design/gates.py contrast
  EXPECT: pairs=122 fail=0
  EVIDENCE: pairs=122 fail=0

ABANDON: E8 실제 앱(릴리스 빌드) 다크 · 라이트 주요 화면 눈 비교 — 수동 테스트는 프로토타입 인수 Task 에서 일괄(masterPlan D28, plan E8 "D28 이면 19 인수" · 종결 시 할 일). 19 TC 로 넘김 · 구현 중 mock 눈 확인은 work/stage3.md(다크 · 라이트) · stage4.md(다크) — gate 증거 아님(plan 해석 16)

## 적대적 평가 — 라운드 1/2

- 2026-10-17 · `/codex:adversarial-review --base 0c5d81e --scope branch` + focus(intent A1 ~ A11 ↔ plan E1 ~ E14 · plan-copy 193행 · 용어 T1 ~ T7 ·
  plan-docs, 완화된 시험 단정 · 부정형 gate 모집단 · 필독 문서 사실 유실 · 글꼴 공개 경로 · [보안] 문구 · 경계값 · 조용한 실패) · 세션 codex-review ·
  HEAD `a827856`(요청과 일치 · worktree clean) · job `review-mvc86mwc-wbjrux` — Verdict needs-attention(high 2 · medium 1). base `0c5d81e` = plan 승인
  (구현 직전, main 과의 merge-base).

원문:

```text
Target: branch diff against 0c5d81e
Verdict: needs-attention

No-ship. A4 위반 리터럴이 현재 CSS에 남았는데 E4가 통과하며, E6·E7·E13·E14 모집단도 축소해 헛통과시킬 수 있습니다.

Findings:
- [high] E4가 현재 남은 색·간격 리터럴을 놓친다 (crates/app/tests/css_tokens.rs:115-173)
  A4는 첫 `:root` 밖 색·간격 리터럴 0을 요구하고 `1px`는 테두리만 예외로 정했습니다. 그러나 `tokens()`는 `var(...)` 전체를 무조건 토큰으로 처리해 fallback을 검사하지 않습니다. 그 결과 `app.css:519`의 `var(--danger, #e5484d)`가 통과합니다. 또한 `Kind::Space`는 모든 `1px`·`-1px`를 허용해 현재 `app.css:402,580`의 `padding: 1px ...`도 통과합니다. E4의 green은 명세 충족 증거가 아닙니다.
  Recommendation: `var()` fallback을 재귀 검사하고, `1px`는 border 속성에만, `-1px`는 승인된 `.sr-only` 규칙에만 허용하십시오. 현재 fallback과 `padding: 1px`를 토큰화하고 해당 세 경우의 regression test를 추가하십시오.
- [high] 평가 gate가 모집단 축소를 성공으로 인정한다 (docs/tasks/1p.project-tidy/plan.md:136-184)
  EXPECT가 정확한 승인 모집단을 고정하지 않습니다. 현재 증거는 moved 167, copy 193, terms 7/16, shots 5, design 92/7, contrast 122지만 정규식은 각 값이 1 이상이기만 하면 됩니다. 따라서 plan-copy 행, 용어, 토큰, slop 규칙, 대비 pair, 캡처 참조를 대부분 삭제해도 `stale=0`, `banned_hits=0`, `mismatch=0`, `fail=0`으로 통과합니다. 특히 E14 본문은 정확히 122 pair를 약속하지만 `pairs=1 fail=0`도 성공입니다.
  Recommendation: 승인된 정확값인 `targets=167`, `rows=193`, `terms=7 banned=16`, `shots=5`, `tokens=92 slop_rules=7`, `pairs=122`를 EXPECT와 gate 내부에서 단정하십시오. 승인된 plan 개정 때만 기준값을 함께 변경하도록 하십시오.
- [medium] JS 토크나이저가 중첩 template literal의 화면 문구를 누락한다 (docs/tasks/1p.project-tidy/design/gates.py:114-129)
  `js_literals()`는 backtick을 일반 따옴표처럼 처리해 다음 backtick에서 즉시 종료합니다. 현재 `app.js:426,458`처럼 outer template의 `${...}` 안에 inner template이 있는 구문에서는 inner literal 문구가 추출되지 않습니다. `terms()`가 이 결과에 의존하므로 금지 용어가 inner template에 들어가도 `banned_hits=0`으로 통과할 수 있습니다.
  Recommendation: Acorn 등 실제 JS parser로 StringLiteral·TemplateElement를 수집하거나 `${...}` 중첩을 추적하는 lexer로 교체하십시오. 현재 중첩 template 사례에 금지 용어를 넣어 `terms`가 실패하는 regression test를 추가하십시오.

Next steps:
- E4 parser와 현재 CSS 리터럴을 먼저 수정한다.
- JS parser 기반 문구 추출과 중첩 template regression test를 추가한다.
- 모집단 정확값을 고정한 뒤 E1~E14를 다시 실행해 eval 원장을 갱신한다.
```

판단 · 조치(커밋 `5e8ee07`):

- **[high] E4 리터럴 놓침** — 판단: **사실**. plan 해석 10 의 예외는 `1px`(테두리) · `-1px`(시각적 숨김)인데 시험은 간격 전체에 둘 다 허용했고 `var()` 대체
  값을 보지 않았다. **고침** — `css_tokens.rs`: `tokens()` 가 `var(--이름, 대체)` 의 대체 값을 다시 조각으로 검사, 간격의 `1px` · `-1px` 허용을 지우고
  `.sr-only { margin: -1px }` 한 곳만 예외(`HIDDEN`, 그 한 곳이 실재함도 단정). RED: 시험만 고친 상태에서 `literals_only_in_root` 가 정확히 지적한 3곳
  (`.md code` · `.day-break` `padding: 1px` · `.danger-text` `var(--danger, #e5484d)`)으로 실패 → `app.css` 간격 1px → 0(stage 3 규칙 "가까운 4pt 토큰",
  1 → 0) · 대체 값 지움 → GREEN 3 passed. 양방향 경계 시험: `1px solid var(--border)` · `var(--tone, var(--wait))` 는 통과, `1px var(--sp-3)` ·
  `-1px` · `var(--danger, #e5484d)` 는 잡힘. web-ui.md 예외 문구도 맞춤.
- **[high] gate 모집단** — 판단: **사실**(eval-protocol 규약 2 "대상 개수가 계약"). **고침(CHECK 보정)** — `gates.py` `POP` 에 승인 정확값(moved 167 ·
  copy 193 · terms 7/16 · shots 5 · design 92/7 · contrast 122)을 두고 각 gate 가 결과 줄을 찍은 뒤 다르면 `population … want=… got=…` + rc 1. RED 재현:
  표에서 5행을 빼면 고치기 전 `rows=188 applied=188 stale=0 doc_stale=0` rc 0(EXPECT 일치 — 헛통과) → 고친 뒤 `population copy: want=193 got=188` rc 1,
  용어 2행을 빼면 `population terms: want=(7, 16) got=(5, 6)` rc 1. **EXPECT 를 정확값으로 바꾸는 것은 항목 변경**(규약 3 — plan 개정 + 사용자 승인)이라
  하지 않았다 — 사용자 결정 대기(gate 안 단정만으로도 축소는 실패한다).
- **[medium] 중첩 템플릿** — 판단: **사실**. RED: `` `${x ? ` · 실행 상세 ${n}` : ""} · 끝` `` 에서 옛 추출은 `['${x ?', ': ""} · 끝']` — 안쪽 문구(금지어
  "실행 상세")가 빠짐. **고침** — `js_literals` 를 lexer 로: 템플릿은 통째(`${…}` 원문 포함 — 표의 `${…}` 행 비교 그대로) + `${…}` 안의 문자열 · 템플릿을
  따로, 주석 · 정규식 건너뜀. 회귀 `tokenizer_selftest`(중첩 템플릿의 금지어 · 정규식 속 백틱 · 주석 속 따옴표 → 한글 literal 집합이 정확히 같아야 함)를
  `copy` · `terms` 앞에서 돌린다(깨지면 gate rc 1). 새로 잡힌 literal app.js 8 · view.js 4(" · 안 읽음 ${n}" · "공유 해제됨" · "읽기 전용" · "요약 중…" ·
  "비우는 중…" 등) — `copy` 193/0 · `terms` 0 그대로. Acorn 은 쓰지 않음(의존을 늘리지 않음 — 화면 원본 3파일에 필요한 문법만).

| 지적 | plan 항목 | 판단 | 조치 |
|---|---|---|---|
| E4 가 대체 값 · 간격 1px 리터럴을 놓침 | E4(A4) | 사실 | `css_tokens` 강화 · `app.css` 3곳 |
| 모집단 축소를 성공으로 인정 | E2 · E6 · E7 · E9 · E13 · E14 | 사실 | gate 안 정확값 단정 — EXPECT 개정은 사용자 결정 |
| 중첩 템플릿 문구 누락 | E6 · E7 | 사실 | lexer · `tokenizer_selftest` |

재검증: `cargo test -p coworkagent-app --test css_tokens` 3 passed · `check_gates` 14건 미충족 0(측정 `5e8ee07`) · clippy · rustfmt 차이 없음 · 문서 캡처 5장
다시(새 mock 하나 — 17시에 띄운 mock 이 같은 포트를 함께 쥐고 있어 옛 시각 · 409 가 섞이던 것을 정리).

## 적대적 평가 — 라운드 2/2

- 2026-10-17 · `/codex:adversarial-review --base 0c5d81e --scope branch` + focus(라운드 1 고침 3건이 닫혔는지 먼저 — `css_tokens` 대체 값 · 1px · `HIDDEN`,
  `gates.py` `POP` · `pin`, lexer · `tokenizer_selftest` — 뒤 라운드 1 과 같은 축) · 세션 codex-review(라운드 1 과 같음) · HEAD `558ac09`(요청과 일치 ·
  worktree clean) · job `review-mvc98fao-y8qyc8` — Verdict needs-attention(high 2 · medium 1). 라운드 상한 — 고친 뒤 재리뷰하지 않는다.

원문:

```text
Target: branch diff against 0c5d81e
Verdict: needs-attention

No-ship. 라운드 1의 현존 CSS 3곳은 수정됐지만, A4·E6·E7 gate에 재현 가능한 헛통과 경로가 남아 수정 완료로 볼 수 없다.

Findings:
- [high] 비-root custom property로 A4 리터럴 검사를 우회할 수 있다 (crates/app/tests/css_tokens.rs:37-40)
  `kind()`는 모든 `--*` 선언을 `Kind::Color`로 분류한다. 따라서 `.x { --pad: 1px; padding: var(--pad); }`에서 `--pad: 1px`는 색 리터럴이 아니어서 통과하고, `padding`도 `var()`라 통과한다. `every_var_is_declared` 역시 선언된 변수로 인정한다. 결과적으로 첫 `:root` 밖 간격 리터럴이 존재해도 E4의 3개 시험이 모두 green일 수 있어 A4의 재발 방지 계약이 깨진다.
  Recommendation: 비-root custom property를 값 종류별로 검사하거나 `--st`, `--tone`, `--ico`처럼 필요한 지역 변수만 명시적으로 허용한다. `--pad: 1px; padding: var(--pad)`와 radius/font-size 우회 사례를 regression test로 추가한다.
- [high] POP pin은 모집단의 내용이 아니라 개수만 고정한다 (docs/tasks/1p.project-tidy/design/gates.py:397-442)
  `copy_rows()`는 행 ID 중복이나 승인된 행 집합을 검증하지 않고, `copy()`는 최종 행 수만 `POP["copy"] == 193`과 비교한다. 승인 행 하나를 삭제하고 이미 통과하는 다른 행을 복제하면 `rows=193 applied=193 stale=0` 및 rc 0을 유지한다. 삭제된 행의 옛 문구는 더 이상 검사되지 않는다. 같은 방식으로 보안 표시 행도 계약에서 조용히 빠질 수 있어 라운드 1의 '모집단이 다르면 rc 1' 조치가 완전히 닫히지 않았다.
  Recommendation: 승인 행 ID의 정확한 집합과 유일성을 고정하고, 가능하면 `(id, file, before, after)` manifest 또는 digest를 pin한다. 중복 ID와 동일 개수 행 치환이 rc 1이 되는 selftest를 추가한다.
- [medium] JS regex 판별 휴리스틱이 유효한 문법에서 문자열을 누락한다 (docs/tasks/1p.project-tidy/design/gates.py:71-109)
  정규식 시작 여부가 앞 16자의 제한된 패턴에만 의존한다. 예를 들어 유효한 JS `if (ok) /'/.test(v); const label = '실행 상세';`에서 `/` 앞이 `)`라 정규식으로 인식되지 않는다. 정규식 안 `'`가 문자열 시작으로 오인되어 실제 `'실행 상세'`가 literal 집합에서 빠지고 `terms()`가 `banned_hits=0`으로 헛통과할 수 있다. `tokenizer_selftest()`의 정규식은 `=` 뒤에만 있어 이 경로를 검증하지 않는다.
  Recommendation: Acorn 등 실제 JS parser로 StringLiteral·TemplateElement를 수집한다. 자체 lexer를 유지한다면 statement 위치의 regex와 quote/backtick을 포함한 regex 뒤 금지 문구 사례를 selftest에 추가한다.

Next steps:
- CSS 지역 custom property 우회 regression을 추가하고 E4를 재실행한다.
- plan-copy 승인 행의 정확한 ID·내용 집합과 중복 금지를 pin한다.
- 실제 JS parser로 교체하거나 regex 문맥 selftest를 강화한 뒤 E6·E7을 다시 측정한다.
```

판단 · 조치(커밋 `547c2cf`):

- **[high] 지역 사용자 속성 우회** — 판단: **사실**. 지금 CSS 의 첫 `:root` 밖 사용자 속성은 모두 `var(…)`(다크 대입 · `--st` · `--tone`) · `url(…)`
  (`--ico`)이지만 시험은 `--pad: 1px` 를 막지 못한다. RED: 회귀 사례를 `kind("--pad")` 로 더하자 `left: None right: Some("1px")` 로 실패. **고침** —
  `Kind::Custom`: 첫 `:root` 밖 `--이름` 값은 `var(…)` · `url(…)` · `0` 외에는 무엇이든 리터럴 → GREEN. 회귀 7건(`--pad: 1px` · `--r: 6px` ·
  `--fs: 13px` · `--c: #fff` · `--w: 600` 은 잡힘, `--st: var(--ok)` · `--ico: url(…)` 는 통과). 지금 CSS 위반 0 그대로.
- **[high] 모집단 내용** — 판단: **사실**. RED: 승인 행 하나를 지우고 다른 행을 복제(193 그대로)하면 개수만 보는 `pin` 은 rc 0. **고침** — `POP` =
  (개수, 정렬 항목 sha256 앞 12자리) + 열쇠 유일: copy = (id · 파일 · 현재 · 제안) · 열쇠 id, terms = 표 행 · 열쇠 개념, design = 토큰 + slop(99), contrast =
  짝 이름, moved = 사실, shots = 참조. 같은 RED 가 이제 `population copy: want=(193, '8fece94740b2') got=(193, '6ab183d41589') dup_keys=1` rc 1.
  `pin_selftest`(복제 치환 · 내용 바꿈 · 열쇠 겹침 → 실패)를 `copy` 앞에서 돈다.
- **[medium] 문장 자리 정규식** — 판단: **사실**. RED: `if (ok) /'/.test(v); const c = '셋째';` 에서 옛 판별은 `[]`(문구 빠짐) → 고친 뒤 `['셋째']`.
  **고침** — `regex_start`: 앞 글자 규칙에 더해 `if (…)` · `while (…)` · `for (…)` · `with (…)` 의 닫는 괄호 뒤 `/` 는 정규식. `tokenizer_selftest` 에 이 사례와
  나눗셈 사례(`(a) / 2 / 4` 뒤 문구)를 더함. 실제 파서(Acorn)는 쓰지 않았다 — 아래 미조치.

| 지적 | plan 항목 | 판단 | 조치 |
|---|---|---|---|
| 지역 사용자 속성으로 리터럴 우회 | E4(A4) | 사실 | `Kind::Custom` · 회귀 7건 |
| 모집단 개수만 고정 | E2 · E6 · E7 · E9 · E13 · E14 | 사실 | 개수 + 내용 digest + 열쇠 유일 · `pin_selftest` |
| 문장 자리 정규식 누락 | E6 · E7 | 사실 | `regex_start` · selftest 보강(파서 교체는 미조치) |

재검증: `cargo test -p coworkagent-app --test css_tokens` 3 passed · `cargo test --workspace` 418 passed 0 failed · clippy · node 112/0 · rustfmt 차이 없음 ·
`check_gates` 14건 미충족 0(측정 `547c2cf`).

상한 뒤 사용자 결정(2026-10-17 사용자 "의견 수용. 권장 방안으로 진행"):

- **R1 → plan 개정 2**(항목 변경 — 규약 3): E2 · E6 · E7 · E9 · E13 · E14 의 EXPECT 를 승인 모집단 정확값으로(`targets=167` · `rows=193 applied=193` ·
  `terms=7 banned=16` · `shots=5` · `tokens=92 … slop_rules=7` · `pairs=122`). CHECK 는 그대로 — 원장을 다시 렌더해 새 EXPECT 로 판정(위 원장).
- **R2 → 수용**: 자체 lexer 를 유지한다(원장 `#1p R2`).
- **plan 48행 "화면 한글 문구 719개" 다시 셈**(stage4 편차): 고친 lexer 로 기준 `ceaaa6f` 중복 포함 737(index.html 213 · app.js 341 · view.js 183 — 옛
  토크나이저가 정규식 속 백틱 · 중첩 템플릿에서 어긋나 app.js 9 · view.js 9 를 놓침), 서로 다른 문구 598 → 지금 566.
- **종결 환류와 E2**: masterPlan S4 말풍선 층을 새 값(토큰 10% · 0%)으로 고치자 E2 가 `missing=2`(1o 값 `14%` · `7%` — 옛 `PRODUCT.md` 에서 옮긴 사실)로
  떨어졌다. 이 Task 가 바꾼 값(개정 1 · 해석 19)이라 S4 에 되돌리지 않고 masterPlan 개정 기록 `#1p` 줄에 바뀐 값("14% · 7% + 옆 띠 → 토큰 10% · 0%")으로
  남겼다 — 다시 잰 원장(측정 `6cfad0a`)의 `targets=167 missing=0` 은 그 줄을 찾은 결과다.

## 미조치 지적

| 지적 원문(요약) | 판단 | 미조치 사유 |
|---|---|---|
| R1 [high] 승인 모집단 정확값을 **EXPECT** 에도 고정(`targets=167` · `rows=193` · `terms=7 banned=16` · `shots=5` · `tokens=92 slop_rules=7` · `pairs=122`) | 사실(계약) | EXPECT 변경은 plan 항목 변경(eval-protocol 규약 3 — 사용자 승인). gate 안에서 개수 · 내용 digest · 열쇠 유일을 단정해 축소 · 치환은 이미 rc 1 — EXPECT 개정 여부는 사용자 결정 → 2026-10-17 승인 · plan 개정 2 로 닫힘(원장 `#1p R1`) |
| R2 [medium] JS 문구 추출을 실제 파서(Acorn — StringLiteral · TemplateElement)로 교체 | 사실(권고) | 자체 lexer 유지(의존을 늘리지 않음). 지적 경로(문장 자리 정규식 · 중첩 템플릿)는 `regex_start` · lexer · selftest 로 닫았고, 남은 한계는 휴리스틱 경계(괄호 안 문자열 속 괄호 · 드문 `}` 뒤 정규식) → 2026-10-17 사용자 수용(원장 `#1p R2`) |
