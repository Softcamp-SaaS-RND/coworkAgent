# eval.md — 1g.rail-scope 적대적 평가

## 평가 원장

<!-- tools/check_gates.py 산출 — 손으로 고치지 않는다. 단 CHECK 가 없는 수동 gate 의
     EVIDENCE 와 ABANDON 줄은 사람이 적고, 재실행해도 보존된다.
     WEAK 줄은 기계가 매 실행 다시 단다 — 약한 gate 를 셀 수 있게 하는 표시이며
     판정에는 영향이 없다. -->

측정 커밋: 6447b645db1be4f5a46e18dd4b18743a5ed28867

- [x] E1: A6 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

- [x] E2: A6 — 우리 코드 경고 0(clippy -D warnings)
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.24s

- [x] E3: A1·A2·A3·A4·A6·A7 — 화면 로직 node 테스트 54건 이상 · 실패 0(메뉴 '나' 6 · 상대 4 / `#p` 경로 / 상대별 묶음 — 레일 순서 · 빈 상대 숨김 · 풀린 상대 뒤 / 대화 요약 — 수 · 안 읽음 · 최근 / ★ 아래 상대별 대화 절 / 입력칸만 새로 고침을 미룸 · 접힘 저장 마지막 값만(개정 1))
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# pass (5[4-9]|[6-9][0-9])\s+# fail 0/
  EVIDENCE: # skipped 0 / # todo 0 / # duration_ms 126.284

- [x] E4: A1·A2·A5 — 웹UI 문구 · 자리(메뉴 라벨 · 사이드바 에이전트 절 없음 · `act_peer` 없음)
  CHECK: cargo test -p coworkagent-app --test ux_literals
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

- [x] E5: A1·A2·A3·A4·A5·A7 — 이 PC 실측(사람 · CDP 확인, 비파괴): '나' 메뉴 6 · 요약 · 상대별 절 · 접힘이 새로 고침 뒤에도 남음 / 상대 타일 메뉴 4 ·
  EVIDENCE: 2026-10-14 headless Edge · CDP(`ui_check_1g.mjs`, 이 PC 실제 앱 — 릴리스 `daed561`, 페어링된 상대 1) 27/27 PASS — '나' 메뉴 6 · 요약 절 home:{id} · 사이드바 convs:{id} · 접힘 ui_folded 저장 · 새로 고침 뒤 접힘 유지 · 펴면 빠짐 · #activity · #requests · #saved · #p · #agents 낱 행 없음 · 내 에이전트 절 맨 앞 · 상대 타일 첫 화면 #p/{id} · 메뉴 4 · 묶음 없음 · '#' → #p/{id} · #activity → '나' · summary 초점 중 새로 고침(개정 1) · 접힘 설정 원래대로 · JS 오류 0. 요청 · 저장된 항목은 이 PC 에 글이 없어 빈 안내만(묶음 그리기는 E3)

- [x] E6: A3 — 접힘 설정 `ui_folded` 왕복 · 거절(형식 · 개수)
  CHECK: cargo test -p coworkagent-app --test ux_api folded
  EXPECT: test result: ok. 1 passed; 0 failed
  EVIDENCE: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 28 filtered out; finished in 0.07s


WEAK: E5 CHECK 없음 — 수동 gate(사람이 적은 EVIDENCE 로 판정)

## 적대적 평가 — 라운드 1/2

- 2026-10-14 · `/codex:adversarial-review --base 075c36f` · 세션 codex-review · 실행 HEAD `e6ae781`(요청과 일치) · Verdict needs-attention(high 1 ·
  medium 1).

### 지적 원문

- [high] 접힘 머리에 초점이 남으면 새 요청 목록이 갱신되지 않음 (crates/app/web/app.js:1995-1996) — `grouped()`가 `req_list` 안에 focusable
  `<summary>`를 추가했지만, HEAD의 `needsRender`는 자식 요소가 초점을 가진 동안 모든 재렌더를 거부한다. 사용자가 절을 접거나 펼치면 summary에 초점이
  남아 이후 5초 주기 `loadRequests(false)`가 새 요청을 받아도 DOM 반영을 계속 건너뛴다. 같은 문제가 대화 요약·내 활동·저장된 항목·주고받은 글에도
  적용되어 새 작업을 놓칠 수 있다. 현재 worktree에는 입력 컨트롤만 보호하도록 좁힌 수정이 있으나 대상 브랜치 HEAD에는 포함되지 않았다.
- [medium] 동시 접힘 저장에서 오래된 전체 스냅샷이 최신 상태를 덮어씀 (crates/app/web/app.js:894-900) — 각 toggle은 현재 `state.folded` 전체를
  독립적인 `PUT /api/settings`로 즉시 전송하며 요청을 직렬화하거나 버전을 확인하지 않는다. 두 절을 빠르게 조작하면 `[A]`와 `[A,B]`가 동시에 전송되고,
  서버 도착·커밋 순서가 뒤집힐 경우 오래된 `[A]`가 마지막에 저장되어 재시작 후 B의 상태가 사라진다. 여러 웹UI 창에서도 동일한 last-writer-wins 손실이
  발생한다.

### 판단 · 조치

| 지적 | 판단 | 조치 |
|---|---|---|
| high 초점 | 사실(리뷰 요청 직후 자체 검토로 찾아 worktree 에 고치던 중이었다) | `view.holdsRender(tag)` — 입력칸(`INPUT` · `TEXTAREA` · `SELECT`)만 미룬다. node 시험 `hold_render` + CDP 회귀(summary 초점 중 서명을 낡게 하고 6.5초 뒤 표지 행이 사라짐) |
| medium 동시 저장 | 사실(한 창 안의 역전) | `view.SendLatest` — 한 번에 하나, 보내는 동안 바뀐 값은 마지막 것만 이어서. node 시험 `send_latest`(가운데 값 건너뜀 · 실패해도 다음 값). 여러 창의 마지막 저장 우선은 "미조치 지적" |

RED: 새 시험 두 건은 export 가 없어 불러오기에서 실패(# pass 0 · # fail 1) → 구현 뒤 57/57. 문구 시험 literal(`{ ui_folded: [...state.folded] }`)이 새
코드와 어긋나 E1 · E4 가 한 번 실패 → `{ ui_folded: keys }` · `foldSaver.push(...)` 로 맞춤(`1ebbd28`, 시험만).

## 적대적 평가 — 라운드 2/2

- 2026-10-14 · `/codex:adversarial-review --base 075c36f` · 세션 codex-review(라운드 1 과 같은 세션) · 실행 HEAD `daed561`(요청과 일치, 작업 트리 깨끗) ·
  Verdict needs-attention(medium 1 · low 1, 라운드 1 재지적 없음). 상한 도달 — 재리뷰하지 않는다.

### 지적 원문

- [medium] 500건 제한 결과를 상대별 전체 대기 건수로 사용한다 (crates/app/web/app.js:1965-1978) — `waitingByPeer(w)`는 `limit=500`으로 잘린 최신 요청만
  집계한다. 서버도 `MESSAGES_MAX=500`이며 SQL `LIMIT`을 적용한다. 대기 요청이 500건을 넘으면 오래된 요청을 가진 상대가 0건으로 보이거나 요약에서 빠져
  업무를 놓칠 수 있다. API 실패도 조용히 삼켜 초기 화면에서는 실패와 0건을 구분할 수 없다.
- [low] 여러 웹UI 창이 접힘 목록 전체를 서로 덮어쓴다 (crates/app/web/app.js:895-907) — `SendLatest`는 한 JS 인스턴스 안에서만 요청을 직렬화한다. 두
  창이 같은 초기 목록을 읽은 뒤 각각 다른 절을 접으면 각 창이 자기 전체 스냅샷을 PUT하고 마지막 응답이 다른 창의 변경을 삭제한다.

### 판단 · 조치

두 건 모두 고치지 않았다 — 아래 "미조치 지적". medium 은 정확한 집계에 서버 집계 API 가 필요한데 intent `## 제외` 가 "목록 API 변경" 을 뺐다(조용한
범위 확장 금지 → 사용자 결정). low 는 라운드 1 medium 의 남은 범위(여러 창)와 같다.

## 미조치 지적

| 지적 원문(요약) | 판단 | 미조치 사유 |
|---|---|---|
| R2 medium — 상대별 대기 요청 수가 `limit=500` 으로 잘린 목록에서 세어져 500건을 넘으면 오래된 요청을 가진 상대가 0 으로 보임 · 읽기 실패와 0 을 구분 못 함 | 사실 | 정확한 수는 서버 상대별 집계 API 가 필요 — intent 제외("목록 API 변경")와 충돌해 사용자 결정 대기. 사이드바 '요청' 대기 배지(#1d)도 같은 500 상한(1g 이전부터). 실패하면 이전 값이 남는다(첫 화면은 0) |
| R2 low(= R1 medium 남은 범위) — 여러 웹UI 창이 접힘 목록 전체를 서로 덮어씀 | 사실 | 한 창 안의 역전은 `SendLatest` 로 고침. 여러 창 동시 조작은 한 PC 한 사용자(D33)에서 드물고, 잃는 것은 절 접힘 표시뿐(데이터 아님). 고치려면 설정 API 에 키 단위 연산 · revision 이 필요 — 라운드 상한 |
