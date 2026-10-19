# eval.md — 1u.scoped-refresh 적대적 평가

## 평가 원장

<!-- tools/check_gates.py 산출 — 손으로 고치지 않는다. 단 CHECK 가 없는 수동 gate 의
     EVIDENCE 와 ABANDON 줄은 사람이 적고, 재실행해도 보존된다.
     WEAK 줄은 기계가 매 실행 다시 단다 — 약한 gate 를 셀 수 있게 하는 표시이며
     판정에는 영향이 없다. -->

측정 커밋: 61ecb86584678e63d7e24a04750a2defe61a2f17

- [x] E1: A6 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

- [x] E2: A6 — 경고 0
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.21s

- [x] E3: A6 — 화면 로직 node 테스트 · 실패 0 · 6건 이상 늘어남(기준 124)
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# pass (13[0-9]|1[4-9][0-9])\s+# fail 0/
  EVIDENCE: # skipped 0 / # todo 0 / # duration_ms 149.3324

- [x] E4: A1·A2 — 소비자 표 · 합치기 node 시험 6건 이상
  CHECK: node --test --test-reporter=tap --test-name-pattern="loadersFor|mergeChanges" crates/app/web/view.test.mjs
  EXPECT: /# pass ([6-9]|[1-9][0-9])\s+# fail 0/
  EVIDENCE: # skipped 0 / # todo 0 / # duration_ms 68.0416

- [x] E5: A3·A4·A5 — 알림기 변경 신호 시험 4건(설정 무관 · 원천별 키 · 건수 없음 · 뒤처짐)
  CHECK: cargo test -p coworkagent-app --test notifier change_
  EXPECT: test result: ok. 4 passed; 0 failed
  EVIDENCE: test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 12 filtered out; finished in 0.08s

- [x] E6: A5 — SSE 가 `event: change` 를 흘린다
  CHECK: cargo test -p coworkagent-app --test events_api change
  EXPECT: test result: ok. 1 passed; 0 failed
  EVIDENCE: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.18s

- [x] E7: A3 — 상대 편집(알림 없음)이 받은 쪽에 그 대화 id 의 변경 신호를 낸다
  CHECK: cargo test -p an-agent --test message_tidy signals_change
  EXPECT: test result: ok. 1 passed; 0 failed
  EVIDENCE: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 11 filtered out; finished in 0.22s

- [x] E8: A6 — 5초 새로 고침 1곳 그대로 · 변경 신호 구독 1곳 · 알림 처리기에서 `refresh()` 호출 0(대상 함수 존재 함께)
  CHECK: python -c "s=open('crates/app/web/app.js',encoding='utf-8').read();q=chr(34);b=s.split('function listen()')[1].split(chr(10)+'function ')[0] if 'function listen()' in s else '';m=b.split('es.onmessage')[1].split('};')[0] if 'es.onmessage' in b else 'refresh(';p=s.count('setInterval(() => refresh(), 5000)');c=b.count('addEventListener('+q+'change'+q);r=m.count('refresh(');print(f'listen={int(bool(b))} poll={p} change={c} notice_refresh={r}')"
  EXPECT: listen=1 poll=1 change=1 notice_refresh=0
  EVIDENCE: listen=1 poll=1 change=1 notice_refresh=0

- [x] E9: A6 — 문서 규약 · 의존 경계
  CHECK: python tools/check_ddd_docs.py && python tools/check_crates.py
  EXPECT: crates=4 edges=6 violations=0
  EVIDENCE: crates=4 edges=6 violations=0

- [ ] E10: A7 — 실화면(두 PC): 5초 새로 고침을 끈 빌드에서 상대가 글 · 편집 · 삭제 · 요청 상태를 바꾼 뒤 변경 신호 한 번으로 대화 본문 · 대화 목록 · 배지 · 내 활동 · 연결 표시가 맞는다. 알림을 끈 · 집중 상태에서도 같다. 프로토타입 인수 Task(D28)로 이월.
  EVIDENCE: pending

ABANDON: E10 실제 상대 · 두 PC 실화면 확인(5초 새로 고침을 끈 빌드 · 상대 글 · 편집 · 삭제 · 요청 상태 → 본문 · 목록 · 배지 · 내 활동 · 연결 표시 일관, 알림 끔 · 집중 포함) — 수동 테스트는 프로토타입 인수 Task 에서 일괄(masterPlan D28). 19 TC 로 넘김 · 자동 근거는 E4 · E5 · E6 · E7

## 적대적 평가 — 라운드 1/2

평가자: `codex-review` 세션 · `/codex:adversarial-review --base 3c95734 --scope branch "1u.scoped-refresh: …"`(job `review-mvem14yx-eyvcon`, 실행 HEAD `de8ea93` =
요청 일치, worktree clean) · Verdict needs-attention(high 1 · medium 1). 원문:

```text
No-ship. A7/E10이 미충족이고, 변경 신호와 주기 refresh가 겹치면 오래된 응답이 최신 UI를 덮을 수 있다.

Findings:
- [high] 핵심 실화면 성공 기준 A7이 검증되지 않았다 (docs/tasks/1u.scoped-refresh/eval.md:57-60)
  E10은 체크되지 않았고 EVIDENCE도 pending이며, 정확히 A7의 두 PC·poll-off·알림 끔·집중 시나리오가 ABANDON 처리됐다. E4~E7은 순수 매핑과 개별 스트림만 검증하고 production의 5초 refresh는 유지하므로, 누락된 원천·소비자·순서 문제가 polling에 가려질 수 있다. 명시된 성공 기준을 충족하지 않은 상태라 완료 또는 ship 근거가 없다.
  Recommendation: E10 시나리오를 5초 polling을 끈 두 PC 빌드에서 실행해 증거를 남기기 전까지 task를 미완료로 유지한다. 이월하려면 A7을 이번 변경의 성공 기준에서 제거하는 명세 개정과 사용자 승인을 먼저 받는다.
- [medium] 변경 신호 refresh가 single-flight가 아니어서 최신 상태를 오래된 응답이 덮을 수 있다 (crates/app/web/app.js:3794-3802)
  50ms 후 `changeQ`를 먼저 비우고 `refresh(...)`를 await하지 않은 채 시작한다. 따라서 refresh 실행 중 다음 변경 신호가 오거나 line 4131의 5초 전체 refresh가 겹치면 여러 refresh가 동시에 돈다. `loadPeers`, `loadGroups`, `loadConvs`, `loadRuns` 등에는 요청 세대 검사가 없어 먼저 시작한 오래된 조회가 나중 조회 뒤에 완료되면 최신 state/DOM을 다시 과거 값으로 덮을 수 있다. 다음 polling까지 본문·목록·배지·연결 표시가 서로 어긋나며, polling을 끄는 A7 환경에서는 그대로 남을 수 있다.
  Recommendation: 변경 신호와 주기 refresh를 하나의 single-flight coordinator로 직렬화한다. 실행 중 도착한 자원은 계속 합쳐 완료 직후 후속 refresh를 수행하고, 로더별 generation/AbortController로 오래된 응답 반영을 차단한다. 지연 응답과 5초 refresh가 겹치는 결정적 테스트를 추가한다.
```

| 지적(요지) | 판단 | 조치 |
|---|---|---|
| [high] A7 · E10 미충족(ABANDON) | 사실 · 승인된 이월 — plan 승인(2026-10-19)이 "E10 은 19 인수로 이월 — D28" 을 포함했고, 수동 실화면은 프로토타입 인수 Task 일괄(D28)이 저장소 규약. 앞 Task(1q E11 등)와 같은 처리 | 코드 조치 없음. 원장 `조건 대기`(19 TC). 5초 새로 고침 축소는 A7 확인 뒤(intent 제외 · D39) — 실화면 전에는 줄이지 않으므로 지금 사용자 화면은 polling 이 그대로 받친다 |
| [medium] refresh 겹침 → 오래된 응답이 덮음 | 사실 — 겹침 자체는 1u 전(알림 refresh + 5초)에도 있었으나 변경 신호가 잦아 커졌다. `Latest` 는 일부 조회에만 있음 | `view.js` `OneAtATime`(한 번에 하나 · 도는 중 요청은 `unionLoaders` 로 합쳐 끝난 뒤 한 번 더 · 기다리는 쪽은 자기 요청 반영 뒤 풀림), `app.js` `refresh` 가 그것을 지난다(변경 신호 · 5초 · 동작 뒤 모두). 결정적 node 시험 2(`OneAtATime` — 지연 응답 · 5초 겹침 · 동시 실행 0, `unionLoaders`). 로더별 AbortController 는 직렬화로 겹침이 없어져 넣지 않음 |

재측정: `python tools/check_gates.py docs/tasks/1u.scoped-refresh` → E1 ~ E9 충족 · E10 ABANDON(측정 커밋 `53ee813`, 수정 커밋 clean tree), node 132.

## 적대적 평가 — 라운드 2/2

평가자: `codex-review` 세션(라운드 1 과 같은 세션) · `/codex:adversarial-review --base 3c95734 --scope branch "1u 라운드 2: …"`(job `review-mvemj83c-pl6jo5`,
실행 HEAD `9ac8410` = 요청 일치, worktree clean) · Verdict needs-attention(medium 1). **라운드 상한 도달 — 반영 뒤 재리뷰 없음.** 원문:

```text
No-ship. 성공 경로 직렬화는 됐지만 refresh 오류 시 대기 요청이 유실된다. A7·E10의 승인된 D28 이월 기록은 확인됐다.

Findings:
- [medium] 실행 중 refresh가 실패하면 누적된 후속 요청이 실행되지 않는다 (crates/app/web/view.js:346-362)
  `call()`은 busy 중 요청을 `again`에 합치지만, `await this.run(next)`가 throw하면 루프를 빠져나가 `finally`가 `again`을 지운다. 재현 결과 첫 실행 중 두 번째 요청을 넣고 첫 실행을 reject했을 때 `runs=["first"]`, 두 Promise 모두 rejected, `busy=null`이었다. 즉 busy는 풀리지만 queued 요청은 실행되지 않는다. 변경 신호·5초 polling·편집/보내기/이름 변경 후 refresh가 일시적 API 실패와 겹치면 호출자의 변경을 반영할 후속 조회가 사라진다. 현재는 다음 polling까지 stale UI가 남고, A7처럼 polling을 끄면 다음 신호까지 계속 남는다. 기존 테스트는 성공 경로만 검증한다.
  Recommendation: 실패한 batch와 이후 queued batch를 분리해, 첫 `run`이 실패해도 `again`을 보존하고 다음 batch를 실행한 뒤 busy를 해제하라. 각 `call()` Promise는 자기 요청이 포함된 batch 완료에 연결하라. 첫 run throw 중 두 번째 요청 enqueue → 두 번째 실행 확인, waiter 정산, `busy === null`, 이후 세 번째 호출 성공을 검증하는 결정적 node 테스트를 추가하라.

Next steps:
- 오류 경로 회귀 테스트와 coordinator 수정을 반영한 뒤 node 테스트를 재실행한다.
- `onChange`·polling·`act` 후처리의 refresh rejection도 사용자 상태 줄 또는 명시적 catch로 노출한다.
```

| 지적(요지) | 판단 | 조치 |
|---|---|---|
| [medium] 앞 실행이 실패하면 쌓인 요청이 사라짐 | 사실 — 라운드 1 수정의 오류 경로 | `OneAtATime` 을 묶음 단위로: 도는 중 요청은 다음 묶음(`queued`)에 합치고, 앞 묶음이 실패해도 `finally` 에서 다음 묶음을 시작한다. 각 `call` 약속 = 자기 묶음 결과. 결정적 node 시험 추가(앞 묶음 reject → 쌓인 요청 실행 · 각자 정산 · `busy === null` · 다음 호출 실행) |
| Next steps — rejection 노출 | 일부 반영 | 변경 신호 다시 읽기 실패 → 상태 줄 `새로 고침 실패: …`(`refreshFailed`). 5초 polling · 동작 뒤 다시 읽기는 아래 미조치 |

재측정: `python tools/check_gates.py docs/tasks/1u.scoped-refresh` → E1 ~ E9 충족 · E10 ABANDON(측정 커밋 `61ecb86`, 수정 커밋 clean tree), node 133.

## 미조치 지적

| 지적 | 판단 | 사유 |
|---|---|---|
| 라운드 1 [high] — A7 · E10 실화면(두 PC · 5초 끔 · 알림 끔 · 집중) 미검증 | 사실 · 승인된 이월 | plan 승인(2026-10-19) 때 19 인수 이월(D28) 포함. 5초 새로 고침 축소는 A7 확인 뒤 별도 Task(D39) — 그 전까지 polling 이 받친다 |
| 라운드 2 Next steps — 5초 polling 의 다시 읽기 실패가 상태 줄에 안 보임(전과 같이 처리 안 된 rejection) | 사실 · 1u 전부터 | plan E8 EXPECT 가 `setInterval(() => refresh(), 5000)` literal 을 잠근다 — 바꾸려면 plan 개정 · 승인. 다음 5초 · 변경 신호가 다시 읽는다 |
| 라운드 2 Next steps — 동작 뒤 `await refresh()`(편집 끝 · 보내기 끝 · 이름 바꾸기)의 실패 노출 | 사실 · 1u 전부터 | 그 호출자의 기존 오류 처리(`act` 등)로 그대로 전파 — 1u 가 바꾼 경로 아님 |
