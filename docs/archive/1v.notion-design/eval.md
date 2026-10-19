# eval.md — 1v.notion-design 적대적 평가

## 평가 원장

<!-- tools/check_gates.py 산출 — 손으로 고치지 않는다. 단 CHECK 가 없는 수동 gate 의
     EVIDENCE 와 ABANDON 줄은 사람이 적고, 재실행해도 보존된다.
     WEAK 줄은 기계가 매 실행 다시 단다 — 약한 gate 를 셀 수 있게 하는 표시이며
     판정에는 영향이 없다. -->

측정 커밋: 67b8664345efe53b705e353a5309d9002e92ccdf

- [x] E1: A10 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

- [x] E2: A10 — 경고 0
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.21s

- [x] E3: A4·A10 — 화면 로직 node 테스트 · 실패 0 · 7건 이상 늘어남(기준 133)
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# pass (14[0-9]|1[5-9][0-9])\s+# fail 0/
  EVIDENCE: # skipped 0 / # todo 0 / # duration_ms 146.9671

- [x] E4: A1·A2 — 보드 열 · 놓기 판정 시험
  CHECK: node --test --test-reporter=tap --test-name-pattern="req_board" crates/app/web/view.test.mjs
  EXPECT: /# pass [2-9]\s+# fail 0/
  EVIDENCE: # skipped 0 / # todo 0 / # duration_ms 61.7577

- [x] E5: A5 — 대화 목록 내 대화만 시험
  CHECK: node --test --test-reporter=tap --test-name-pattern="conv_list_mine" crates/app/web/view.test.mjs
  EXPECT: /# pass [1-9]\s+# fail 0/
  EVIDENCE: # skipped 0 / # todo 0 / # duration_ms 60.586

- [x] E6: A6 — 사이드바 에이전트 하위 메뉴 시험
  CHECK: node --test --test-reporter=tap --test-name-pattern="side_nav" crates/app/web/view.test.mjs
  EXPECT: /# pass [4-9]\s+# fail 0/
  EVIDENCE: # skipped 0 / # todo 0 / # duration_ms 62.1014

- [x] E7: A7 — 에이전트 대화 화면 절 시험
  CHECK: node --test --test-reporter=tap --test-name-pattern="agent_convs" crates/app/web/view.test.mjs
  EXPECT: /# pass [1-9]\s+# fail 0/
  EVIDENCE: # skipped 0 / # todo 0 / # duration_ms 61.8816

- [x] E8: A8 — 피드백 말풍선 시험
  CHECK: node --test --test-reporter=tap --test-name-pattern="req_feedback" crates/app/web/view.test.mjs
  EXPECT: /# pass [1-9]\s+# fail 0/
  EVIDENCE: # skipped 0 / # todo 0 / # duration_ms 68.3835

- [x] E9: A8 — 말풍선 메타 줄에 "메모:" 없음 · 피드백 말풍선 그리기 1곳
  CHECK: python -c "s=open('crates/app/web/app.js',encoding='utf-8').read();b=s.split('function messageEl(')[1].split(chr(10)+'function ')[0];print('meta_note=%d feedback=%d'%(b.count(chr(0xba54)+chr(0xbaa8)+':'),s.count('reqFeedback(')))"
  EXPECT: /meta_note=0 feedback=[1-9]/
  EVIDENCE: meta_note=0 feedback=1

- [x] E10: A9 — 입력창 정적 시험
  CHECK: node --test --test-reporter=tap --test-name-pattern="composer_a" crates/app/web/view.test.mjs
  EXPECT: /# pass [1-9]\s+# fail 0/
  EVIDENCE: # skipped 0 / # todo 0 / # duration_ms 60.7319

- [x] E11: A10 — 문서 규약 · 의존 경계
  CHECK: python tools/check_ddd_docs.py && python tools/check_crates.py
  EXPECT: crates=4 edges=6 violations=0
  EVIDENCE: crates=4 edges=6 violations=0

- [x] E13: A11 — 편집 모드 흔적 없음
  CHECK: python -c "i=open('crates/app/web/index.html',encoding='utf-8').read();a=open('crates/app/web/app.js',encoding='utf-8').read();print('index=%d app=%d'%(sum(i.count(k) for k in ('edit-toggle','edit_bar','scope_dialog')),a.count('setEditMode')))"
  EXPECT: index=0 app=0
  EVIDENCE: index=0 app=0

- [ ] E12: A3·A4 — 실화면: 보드에서 대기 카드를 수용 열로 끌면 메모 칸이 열리고 확인 뒤 요청자 PC 에 수용 · 메모가 보인다, 취소면 그대로, 허용 안 된 열엔 놓이지 않는다, 목록 보기는 지금과 같다. 프로토타입 인수 Task(D28)로 이월.
  EVIDENCE: pending

ABANDON: E12 실화면 두 PC 확인(보드 끌어 놓기 → 메모 칸 → 확인 시 요청자 PC 에 수용 · 메모, 취소면 그대로, 허용 안 된 열 놓기 불가, 목록 보기 동일) — 수동 테스트는 프로토타입 인수 Task 에서 일괄(masterPlan D28). 19 TC 로 넘김 · 자동 근거는 E4(canDrop) · E3(목록 회귀)

## 적대적 평가 — 라운드 1/2

평가자: `codex-review` 세션 · `/codex:adversarial-review --base 1ad4d8f --scope branch "1v.notion-design: …"`(job `review-mverij12-e2vpkf`, 요청 HEAD `32a3f85` ·
실행 HEAD `1e3fcd9` — 직계 후손, 사이 커밋은 STATE.md 만 → 코드 동일, worktree clean) · Verdict needs-attention(medium 1). 원문:

```text
출시 보류. `dir=in` 대화만 있는 범위에서 기본 대화 목록이 빈 화면으로 렌더링됩니다.

Findings:
- [medium] 숨긴 에이전트 대화가 빈 상대 절을 생성함 (crates/app/web/app.js:1463-1506)
  `rows`와 `sections`를 `dir=in` 필터링 전에 만들고, 이후 `convListParts()`만 빈 배열을 반환합니다. 렌더러는 원본 `rows.length`가 1 이상이므로 "대화 없음"도 출력하지 않습니다. 실제 확인 결과 inbound 대화 1건에서 `sourceRows=1`, `visibleParts=0`이었습니다. 따라서 에이전트 대화만 있는 상대 타일은 완전히 빈 목록을, '나' 타일은 내용 없는 상대 절을 표시합니다. 검색어가 숨긴 에이전트 대화에만 일치할 때도 "맞는 대화 없음" 대신 빈 결과가 나옵니다.
  Recommendation: 비그룹 화면에서는 `dir !== "in"`으로 먼저 필터링한 visible rows로 `convSections`와 empty-state 판정을 모두 수행하십시오. inbound-only 및 검색 결과가 inbound-only인 렌더링 회귀 테스트도 추가하십시오.
```

| 지적(요지) | 판단 | 조치 |
|---|---|---|
| [medium] 에이전트 대화를 빼기 전 줄로 절 · 빈 판정 → 빈 상대 절 · 빈 목록 · 빈 검색 결과 | 사실 — stage 1 메모("에이전트 대화만 있는 상대 절은 빈 안내로 남는다")도 틀렸다(원본 `rows.length` 가 1 이상이라 안내도 없음) | `view.js` `visibleConvRows`(나 · 상대 = `in` 제외, 그룹 = 그대로) — `renderConvList` 가 절(`convSections`) · 빈 안내 · 서명을 이 줄로, `convListParts` 도 같은 함수. node 시험 `conv_list_mine_sections`(에이전트 대화뿐 · 상대별 절 · 검색이 에이전트 대화에만 맞음 · 그룹 그대로). 커밋 `a3cc8d0` |

## 적대적 평가 — 라운드 2/2

평가자: `codex-review` 세션(라운드 1 과 같은 세션) · `/codex:adversarial-review --base 1ad4d8f --scope branch "1v 라운드 2: …"`(job `review-mves0emq-7fzlxs`,
실행 HEAD `b06e9d3` = 요청 일치, worktree clean) · Verdict needs-attention(medium 1). **라운드 상한 도달 — 반영 뒤 재리뷰 없음.** 원문:

```text
출시 보류. A9 placeholder 계약이 실제 대화 화면에서 깨지며, 현재 정적 시험은 이를 놓칩니다.

Findings:
- [medium] 런타임에 placeholder 계약을 덮어씀 (crates/app/web/app.js:1897)
  `index.html`의 `placeholder="메시지 입력"`은 대화를 열 때마다 `showConversation()`이 `${conv.agent} 에게 보내기`로 덮어씁니다. 따라서 승인된 A9와 달리 실제 화면에서는 "메시지 입력"이 보이지 않습니다. `composer_a` 시험은 정적 HTML만 검사해 141개 시험 통과가 이 회귀를 탐지하지 못합니다.
  Recommendation: 해당 대입을 제거하거나 값을 `메시지 입력`으로 고정하고, `showConversation()` 실행 후 placeholder를 검증하는 DOM 시험 또는 런타임 재대입 금지 검사를 추가하십시오.

Next steps:
- 의견: `visibleConvRows`의 라운드 1 수정 자체는 절·빈 안내·검색 결과 경로에 올바르게 적용됐습니다.
```

| 지적(요지) | 판단 | 조치 |
|---|---|---|
| [medium] 대화를 열 때 `reply_text` placeholder 를 "{에이전트} 에게 보내기" 로 덮어씀 → A9 "메시지 입력" 깨짐 | 사실 — intent 결정 10 은 "placeholder 는 지금 '메시지 입력' 그대로" 로 적었으나 지금 값은 런타임 대입이었다(코드 사실 확인 누락). 사용자 이유("이미 어떤 에이전트와 통신하는지 결정")와도 같은 축 | 대입 삭제(정적 값 "메시지 입력" 고정). 재대입 금지 검사 — node `composer_a` 에 `app.js` 의 `$("reply_text").placeholder` 없음, `ux_literals` 없음 표 2줄(`" 에게 보내기`"` · 대입), 있음 표의 #17 라벨 줄 삭제. DOM 시험은 미조치(아래) |

## 미조치 지적

| 지적 원문 | 판단 | 미조치 사유 |
|---|---|---|
| R2 "`showConversation()` 실행 후 placeholder를 검증하는 DOM 시험 … 추가" | 사실(정적 시험은 런타임 대입을 못 본다) — 권고 둘 중 "런타임 재대입 금지 검사" 는 반영 | DOM 시험은 jsdom 등 새 의존이 필요(저장소 화면 시험은 DOM 없는 순수 함수 · 정적 원문 검사 규약). 실화면 확인은 19 인수(E12 와 함께) |
