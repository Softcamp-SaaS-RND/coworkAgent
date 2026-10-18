# eval.md — 1r.ui-polish 적대적 평가

## 평가 원장

<!-- tools/check_gates.py 산출 — 손으로 고치지 않는다. 단 CHECK 가 없는 수동 gate 의
     EVIDENCE 와 ABANDON 줄은 사람이 적고, 재실행해도 보존된다.
     WEAK 줄은 기계가 매 실행 다시 단다 — 약한 gate 를 셀 수 있게 하는 표시이며
     판정에는 영향이 없다. -->

측정 커밋: 2b181986cff48ef35dafaf02994cbe625f97b582

- [x] E1: A10 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

- [x] E2: A10 — 우리 코드 경고 0(clippy -D warnings)
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.23s

- [x] E3: A1·A2·A5·A9·A10 — 화면 로직 node 테스트 124건 이상 · 실패 0(기존 121 + ui_polish 3)
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# pass (12[4-9]|1[3-9][0-9])\s+# fail 0/
  EVIDENCE: # skipped 0 / # todo 0 / # duration_ms 131.7197

- [x] E4: A1·A2·A5·A9 — 1r 시험만 3건(메뉴 아이콘 · 초대 생성 누름 · 새 대화 폼)
  CHECK: node --test --test-isolation=none --test-reporter=tap --test-name-pattern="^ui_polish" crates/app/web/view.test.mjs
  EXPECT: /# pass [3-9]\s+# fail 0/
  EVIDENCE: # skipped 0 / # todo 0 / # duration_ms 17.4307

- [x] E5: A1·A3·A4·A6·A7·A8·A9 — 문구 · 자리 잠금(내 응답 에이전트 · 최대 응답 횟수 · 들여쓰기 규칙 · ? 설명 · 수락 접기 · 새 에이전트 줄 · 연필 · 탭 아이콘, 옛 자리 0)
  CHECK: cargo test -p coworkagent-app --test ux_literals
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

- [x] E6: A10 — CSS 토큰(색 리터럴 `:root` 밖 0)
  CHECK: cargo test -p coworkagent-app --test css_tokens
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

- [x] E7: A10 — 문서 규약
  CHECK: python tools/check_ddd_docs.py
  EXPECT: PASS
  EVIDENCE: PASS — DDD 문서 규약 위반 없음

- [x] E8: A10 — 의존 경계
  CHECK: python tools/check_crates.py
  EXPECT: /crates=4 edges=[0-9]+ violations=0/
  EVIDENCE: crates=4 edges=6 violations=0

- [x] E9: A1·A2·A3·A4·A5·A6·A7·A8·A9 — 실화면 확인(mock, 다크 · 라이트): 새 대화 "안 함" ↔ 에이전트 고름에 하위 줄 숨김 · 보임 · 라벨 아이콘 · 요청 개요 / 주고받은 메시지 /
  EVIDENCE: mock(Edge headless, 다크 · 라이트) 확인 — 새 대화 하위 줄 숨김/보임 · 라벨 아이콘 7 · 요청 개요 2단 들여쓰기 + 안내선 · 알림 ? 위로 4줄 · check1r 스크립트(수락 칸 펼침 → 다른 항목 접힘 → 레일 + 펼침 → 다른 화면 접힘 · 해시로 초대 생성 옴 = 초대 없음 · 트리/새로 만들기 클릭 = 초대 상자) · 연필 펼침 · 새 에이전트 줄 · 프로필 메뉴 아이콘 · JS 오류 0 — work/stage2.md


WEAK: E9 CHECK 없음 — 수동 gate(사람이 적은 EVIDENCE 로 판정)

## 적대적 평가 — 라운드 1/2

평가자: `codex-review` 세션 · `/codex:adversarial-review --base main "웹UI 1r: …"`(job `review-mvd8bfy6-5fzweq`, 실행 HEAD `9ae9c95` — 요청 `5ad2bf9` 의 직계 후손,
사이 커밋은 `STATE.md` 만이라 범위 코드 같음, base `main` = `8ecf555`) · Verdict needs-attention(high 1 · medium 2).

| 지적(원문 요지) | 판단 | 조치 |
|---|---|---|
| [high] 숨긴 `relay_cap` 이 새 대화 전송을 보이지 않게 막는다 — 에이전트 고름 → 범위 밖 값 → "안 함" 이면 `hidden` 칸이 여전히 폼 검사에 걸려 submit 이 안 일어난다. 에이전트를 고른 채 비우면 `required` 가 없어 `Number("")` = 0 전송(`app.js` 3199-3202) | 사실 — 1r 이 칸을 숨기면서 생긴 회귀(전에는 칸이 보여 고칠 수 있었다) | 칸 `required disabled hidden` · `step="1"` 으로 시작, `syncRelayCap` 이 숨김과 `disabled` 를 함께 바꾼다(숨긴 동안 검사 제외, 보이면 브라우저가 1 ~ 50 정수 · 빈 값 검사). 싣는 값 `valueAsNumber`. 잠금 node `ui_polish_new_chat`(칸 속성) · `ux_literals` |
| [medium] 에이전트 등록 성공 뒤 키보드 초점이 숨긴 폼에 남는다(`app.js` 1338-1344) | 사실 | 등록 성공 = 칸 닫고 `agent_new_btn` 에 초점(취소와 같은 자리). `ux_literals` |
| [medium] 초대 생성 중복 호출 — 트리 · 메뉴 `click()` 에 진행 중 막음이 없어 POST 가 겹치면 서버 코드가 바뀌고 늦게 온 옛 응답이 화면에 남을 수 있다(`app.js` 3617-3620) | 사실(단추 두 번 누름으로도 전부터 가능했고, 1r 의 새 진입점이 늘렸다) | `window_btn` 처리기가 도는 동안 단추를 끈다(`disabled` — 꺼진 단추는 `click()` 도 무시) → 단추 · 트리 · 새로 만들기 모두 한 번에 하나. `ux_literals` |

재검증(mock DOM — `check1r_r1.mjs`, 고치기 전 `9ae9c95` 웹 파일로 같은 스크립트):

| 확인 | 고치기 전 | 고친 뒤 |
|---|---|---|
| 에이전트 고름 + 99 → "안 함" 뒤 폼 유효 | `false`(전송 막힘) | `true` |
| 에이전트 고름 + 빈 값 → 폼 유효 | `true`(0 전송) | `false` |
| 트리 "초대 생성" 두 번 + 단추 한 번 → `POST /api/pairing-window` | 3 | 1 |
| 등록 성공 뒤 초점 | 없음(`body`) | `agent_new_btn` |

`cargo test --workspace` → `test result` 실패 0 · clippy `Finished` · node `# pass 124` `# fail 0` · `ux_literals` ok · JS 오류 0.

## 적대적 평가 — 라운드 2/2(상한)

평가자: `codex-review` 세션(라운드 1 과 같은 세션) · `/codex:adversarial-review --base 8ecf555 "웹UI 1r 라운드 2: R1 반영 확인 …"`(job `review-mvd905k1-zsgt1r`,
실행 HEAD `50cb229` = 요청 일치) · Verdict needs-attention(medium 1). 라운드 1 의 숨김 칸 검사 · 등록 뒤 초점은 새 지적 없음.

| 지적(원문 요지) | 판단 | 조치 |
|---|---|---|
| [medium] 새로 만들기 → 초대 생성 시 초점이 사라진다 — 메뉴를 닫고 이동한 직후 `window_btn.click()` 이 단추를 `disabled` 로 만들어, hashchange 뒤 `applyFocus()` 가 그 단추에 초점을 줄 수 없다(`app.js` 3893-3898) | 사실 — R1 수정(`disabled`)이 만든 회귀 | `disabled` 대신 진행 중 표시 `inviteBusy` + `aria-disabled="true"`(누름은 버리고 초점은 받는다) · 끝나면 둘 다 풂 · 진행 중 모양 `#window_btn[aria-disabled="true"]`. `ux_literals`(새 자리 PRESENT · 옛 `disabled = true` ABSENT) |

재검증(mock DOM — `check1r_r2.mjs`, 키보드로 새로 만들기 → 초대 생성 Enter, 고치기 전 `50cb229` 웹 파일로 같은 스크립트):

| 확인 | 고치기 전 | 고친 뒤 |
|---|---|---|
| 새로 만들기 → 초대 생성 뒤 `document.activeElement` | `BODY` | `window_btn` |
| 이어서 단추 `click()` 세 번 → 더해진 `POST /api/pairing-window` | 1 | 1 |

`cargo test --workspace` → 실패 0 · clippy `Finished` · node `# pass 124` `# fail 0` · JS 오류 0. 라운드 상한 도달 — 재리뷰 없음.

## 미조치 지적

없음
