# eval.md — 1q.request-overview 적대적 평가

## 평가 원장

<!-- tools/check_gates.py 산출 — 손으로 고치지 않는다. 단 CHECK 가 없는 수동 gate 의
     EVIDENCE 와 ABANDON 줄은 사람이 적고, 재실행해도 보존된다.
     WEAK 줄은 기계가 매 실행 다시 단다 — 약한 gate 를 셀 수 있게 하는 표시이며
     판정에는 영향이 없다. -->

측정 커밋: 1d92c8436a53329bc1362b0b77d58f620c2dc27e

- [x] E1: A8 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

- [x] E2: A8 — 우리 코드 경고 0(clippy -D warnings)
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.23s

- [x] E3: A1·A3·A4·A6·A7·A8 — 화면 로직 node 테스트 119건 이상 · 실패 0(기존 112 + req_overview 4 · activity_rows_tabs 1 · peer_msgs 3 − 죽은 `peerMessages` 시험 1, 기존 시험 회귀 없음)
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# pass (119|1[2-9][0-9])\s+# fail 0/
  EVIDENCE: # skipped 0 / # todo 0 / # duration_ms 136.5644

- [x] E4: A1·A3 — 요청 개요 시험만 3건 이상(상태별 건수 · 머리 문구 0 숨김 · 상태 거르기와 무관 · 단추 종류 ✓ 수용 · ✕ 반려 · ✓ 완료 · 다시 알리기)
  CHECK: node --test --test-isolation=none --test-reporter=tap --test-name-pattern="^req_overview" crates/app/web/view.test.mjs
  EXPECT: /# pass [3-9]\s+# fail 0/
  EVIDENCE: # skipped 0 / # todo 0 / # duration_ms 17.1724

- [x] E5: A4 — 내 활동 범위(core): 받은 요청 · 받은 답 · 실패만, 사람 일반 메시지 · 자동 왕복 기록 · 공유 사본 · 내 에이전트 글에 단 답 빠짐, 안 본 수 같은 범위
  CHECK: cargo test -p an-core --test store activity_scope
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 15 filtered out; finished in 0.13s

- [x] E6: A4·A5 — 내 요청 상태 바뀜(두 PC): 수용 · 완료 → 요청자 내 활동 맨 위 · 안 본 수 +1 · 같은 상태 재전송은 기록 없음 · 본 뒤 0
  CHECK: cargo test -p an-agent --test requests activity_my_request
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 7 filtered out; finished in 2.26s

- [x] E7: A6 — API: 상대별 요약(건수 · 마지막) · side=both · before 커서 · limit 20 · 옛 kind 400 · 새 kind
  CHECK: cargo test -p coworkagent-app --test ux_api -- peer_pages activity_kinds
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 30 filtered out; finished in 0.44s

- [x] E8: A1·A2·A3·A4·A6·A7 — 문구 · 자리 잠금(개요 · 글자 단추 · 내 활동 탭 · 더 보기 · 행 북마크, 옛 자리 0)
  CHECK: cargo test -p coworkagent-app --test ux_literals
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

- [x] E9: A8 — 문서 규약
  CHECK: python tools/check_ddd_docs.py
  EXPECT: PASS
  EVIDENCE: PASS — DDD 문서 규약 위반 없음

- [x] E10: A8 — 의존 경계
  CHECK: python tools/check_crates.py
  EXPECT: /crates=4 edges=[0-9]+ violations=0/
  EVIDENCE: crates=4 edges=6 violations=0

- [ ] E11: A1·A2·A3·A4·A5·A6·A7 — 실화면 확인(실제 상대 · 두 PC, 다크 · 라이트): 요청 개요 머리 건수 · 탭 토글 · 글자 단추 → 메모 → 상태 · 내 활동 탭 · 상대가 상태를 바꾸면
  EVIDENCE: pending

- [x] E12: A9 — 실제 실행 파일: `/?` · `-h` · `--help` → 종료 0 · 표준 출력에 명령(ask · start · history) · 종료 코드 · `--json` · progress 줄 · 표준 오류 빔, 다른 자리 `/?` 는 2
  CHECK: cargo test -p coworkagent-app --test cli help_exe
  EXPECT: test result: ok. 1 passed; 0 failed
  EVIDENCE: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 16 filtered out; finished in 0.07s

- [x] E13: A10 — README 에 `coworkAgent /?` 연동 안내(3곳 이상 — 기능 표 · 연동 절 설명 · 명령 예) · README 사용자 관점 체커
  CHECK: python -c "t=open('README.md',encoding='utf-8').read();print('help_refs=%d section=%d'%(t.count('coworkAgent /?'),t.count('## ')))" && python tools/check_readme.py
  EXPECT: /help_refs=([3-9]|[1-9][0-9]) section=[0-9]+[\s\S]*readme ok/
  EVIDENCE: help_refs=5 section=6 / readme ok

ABANDON: E11 실제 상대 · 두 PC 실화면 확인(요청 개요 · 글자 단추 → 메모 → 상태 · 내 활동 탭 · 상대가 상태를 바꾸면 맨 위 · 주고받은 메시지 접힘 · 20건 · 더 보기 · 행 북마크) — 수동 테스트는 프로토타입 인수 Task 에서 일괄(masterPlan D28, plan 종결 시 할 일). 19 TC 로 넘김 · 구현 중 mock 확인은 work/stage4.md

## 적대적 평가 — 라운드 1/2

평가자: `codex-review` 세션 · `/codex:adversarial-review --base 4f29654`(job `review-mvd2tazd-uqaa9v`, 실행 HEAD `ea09cbf`) · Verdict needs-attention(high 1 ·
medium 2). 평가자 메모: 대조 기준(intent · plan)은 명령 밖 글머리표라 Codex 에 넘어가지 않았다 — 라운드 2 는 따옴표 focus 로.

| 지적(원문 요지) | 판단 | 조치 |
|---|---|---|
| [high] 요청 개요가 500건 이후의 미처리 요청을 숨긴다 — 쪽마다 `limit=500` 으로 받아 건수 · 거르기를 화면에서 계산(`app.js` 2879-2886) | 사실 — plan 해석 1 의 상한을 건수 · 거르기에까지 걸었다 | core `req_counts` · `GET /api/messages/req_counts`(범위 전체 상태별 건수) → 머리 건수, 상태 거르기는 서버 `req=` 로 옮김(오래된 대기 요청도 나온다). 행은 여전히 쪽(· 상태)마다 최근 500 — `ponytail:` 주석. 시험 core `req_counts_beyond_page`(대기 1 + 완료 501 → 건수 · 대기 거르기) · ux_api `ask_kind_recorded_and_filtered` · node `req_overview_summary`(잘린 목록이어도 서버 건수) |
| [medium] 상태 저장과 안 본 활동 기록이 따로 커밋 — 기록만 실패하면 재전송이 "같은 상태" 로 일찍 돌아가 영영 누락(`agent/src/chat.rs` 2241-2243) | 사실 | `inbound_req_state` 가 `write_tx` 한 트랜잭션으로 상태 · 기록. 시험 agent `activity_my_request_atomic_with_state`(기록 실패 주입 트리거 → 상태 그대로 · `retry` → 트리거 지운 뒤 다시 보낸 상태 + 안 본 1). 고치기 전 코드로 돌리면 `기록이 실패하면 상태도 그대로` 단정 실패 확인 |
| [medium] 주고받은 메시지 다시 그리기 서명이 `id` · `saved` · `text.length` 만 — 요청 상태 · 메모 · 같은 길이 편집이 안 바뀜(`app.js` 3081-3082) | 사실 | `view.peerMsgsSig`(본문 · 요청 상태 · 메모 · 편집 · 상태 · 북마크 · 대화 제목 · 에이전트 · 방향). 시험 node `peer_msgs_sig` |

재검증: `cargo test --workspace --no-fail-fast` → `test result: ok` 75 묶음 · FAILED 0 · clippy `Finished` · node `# pass 120` `# fail 0` · mock(대기 거르기 ·
머리 건수 · 주고받은 메시지) JS 오류 없음.

## 적대적 평가 — 라운드 2/2(상한)

평가자: `codex-review` 세션 · `/codex:adversarial-review --base 4f29654 --scope branch "Task 1q — intent 결정 1~15 ↔ plan 해석 1~10 · E1~E13 …"`(job
`review-mvd3gi8c-vop4cr`, 실행 HEAD `45d7d86` = 요청 일치) · Verdict needs-attention(medium 3). 라운드 1 수정 셋 중 서버 집계 · 원자성은 닫혔다는 판정.

| 지적(원문 요지) | 판단 | 조치 |
|---|---|---|
| [medium] `peerMsgsSig` 가 발화자 표시 변경(`author` · `mine` · `via` · `agent` · `owner`)을 감지하지 못함 — 공유 사본 `upsert` 는 같은 글의 이것만 바꿀 수 있다(`view.js` 424-427) | 사실 | 서명에 `created_at` · `author` · `mine` · `via` · `agent` · `owner` 추가. node `peer_msgs_sig` 에 다섯 경우 추가 |
| [medium] 5초 새로 고침이 "더 보기" 로 받은 페이지를 덮어씀 — 겹치면 늦게 끝난 쪽이 지우고, 500 넘게 보면 매번 잘림(`app.js` 3068-3077) | 사실 | 상대마다 한 줄 차례(`state.peerLoads` — 앞 요청이 끝난 뒤 다음, 그때의 상태로 합침) · 새로 고침 = 첫 페이지(20) + 그보다 앞의 받은 꼬리(`view.mergePage`, `(created_at, id)`) — 받은 꼬리는 지우지 않는다. node `peer_msgs_merge`(새 글 · 꼬리 보존 · 다 안 찬 페이지 · 첫 페이지 안 지운 글 · 같은 시각) |
| [medium] 페이지 · 마지막 메시지가 id 순이라 나중에 들어온 공유 사본의 옛 글이 맨 위 · 마지막 메시지가 된다 — plan 해석 8 의 `created_at DESC, id DESC` 와 다르다(`core/src/chat.rs` 1304-1319) | 사실 — 구현이 plan 해석 8 에서 벗어났다(stage3 기록의 설계 결정) | `messages_page` = `(created_at, id)` 최신순 + 복합 커서(API `before` + `before_at`, 하나만 주면 400) · `peer_message_counts` 마지막 = 같은 순서(창 함수). core `peer_pages_order`(나중에 넣은 옛 사본 글이 맨 아래 · 마지막 메시지 아님 · 한 건씩 넘겨도 빠짐 · 겹침 없음) · ux_api `peer_pages_summary_and_cursor` 커서 · 400 |

라운드 상한 2 도달 — 재리뷰 없음(사용자 전역 지침). 이 라운드 지적은 모두 고쳤고 **미조치 지적은 없다**. 다만 이 수정은 평가자가 다시 보지 않았다.
`peer_pages_order` 는 옛 시그니처(`before: Option<i64>`)로는 컴파일되지 않아 고치기 전 코드로 돌린 RED 확인은 없다(근거: 옛 구현은 `ORDER BY m.id DESC` 라 나중에
넣은 사본 글이 맨 위).

재검증: `cargo test --workspace --no-fail-fast` → `test result: ok` 75 묶음 · FAILED 0 · clippy `Finished` · node `# pass 121` `# fail 0` · mock 주고받은 메시지
(펼침 · 커서) JS 오류 없음.

## 미조치 지적

| 지적 원문(요약) | 판단 | 미조치 사유 |
|---|---|---|
| R1 [high] 요청 목록에 cursor pagination 추가(권고 일부) | 사실(권고) | 건수 · 상태 거르기는 서버로 옮겨 닫았다. 행은 쪽(· 상태)마다 최근 500건까지 — 한 상대와 요청이 500건 넘게 쌓일 일이 드물어 커서 페이지는 다음 Task 후보(`ponytail:` 주석) |

