# eval.md — 1d.request-list 적대적 평가

## 평가 원장

<!-- tools/check_gates.py 산출 — 손으로 고치지 않는다. 단 CHECK 가 없는 수동 gate 의
     EVIDENCE 와 ABANDON 줄은 사람이 적고, 재실행해도 보존된다.
     WEAK 줄은 기계가 매 실행 다시 단다 — 약한 gate 를 셀 수 있게 하는 표시이며
     판정에는 영향이 없다. -->

측정 커밋: 1de91916bbd9663bafa6dd885fce8a3c34d54517

- [x] E1: A7 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

- [x] E2: A7 — 우리 코드 경고 0(clippy -D warnings)
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.22s

- [x] E3: A1·A4·A5·A7 — 화면 로직 node 테스트 47건 이상 · 실패 0(기존 42 + 모델 목록 · 요청 상태 문구 · 작업 막대 옮김 · 사이드바 '요청' 배지 · `#requests` 경로)
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# pass (4[7-9]|[5-9][0-9])\s+# fail 0/
  EVIDENCE: # skipped 0 / # todo 0 / # duration_ms 85.4618

- [x] E4: A4·A6 — 요청 저장 단위 4건(스키마 14 가 완료 표시 → `done` · 나머지 → `wait` · 목록 밖 모델 지움 / 옮김 규칙 넷(대기→수용·반려, 수용→완료, 그 밖 거부) / 메모 규칙(200자 · 줄바꿈 거부) / 목록 상태 거르기)
  CHECK: cargo test -p an-core --test requests
  EXPECT: test result: ok. 4 passed; 0 failed
  EVIDENCE: test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s

- [x] E5: A3·A4 — 두 PC 통합 6건(새 대화 요청은 받는 쪽 실행 · 대기열 없음 · '대기' / 이어 쓰기 · 받은 대화(`in`)의 요청도 실행 없음 / 집중 중에도 요청은 받고 질의는 거부 / 수용 · 메모가 요청자 글에 도착 / 요청자가 꺼져 있을 때 바꾼 상태가 켜지면 도착(outbox) / 쓴 쪽 · 다른 상대 · 위조 `ReqState` 는 거부 · 멘션 · CLI 요청은 400 / 요청자에 오래 닿지 않아 포기하면 그 글에 표시 · 같은 상태로 다시 알리기 — 개정 3)
  CHECK: cargo test -p an-agent --test requests
  EXPECT: test result: ok. 7 passed; 0 failed
  EVIDENCE: test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 9.52s

- [x] E6: A1·A2 — 앱 API 4건(전역 · 상대별 모델 목록 밖 400 · 목록 안 · 빈 값(상대별) 저장 / 에이전트 JSON 에 `tools` 없음 · 등록 · 수정에 `tools` 를 보내면 422 / `GET /api/messages?ask=request&req=wait` 거르기 / `GET /api/activity?kind=request` 400)
  CHECK: cargo test -p coworkagent-app --test models_tools
  EXPECT: test result: ok. 4 passed; 0 failed
  EVIDENCE: test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s

- [x] E7: A2 — 실행 인자: 어떤 프로필 값이든 `--tools` · `--allowedTools` = `Read,Grep,Glob,LS,NotebookRead`(runner 시험 8건)
  CHECK: cargo test -p an-agent --test runner
  EXPECT: test result: ok. 8 passed; 0 failed
  EVIDENCE: test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.02s

- [x] E8: A1·A2·A3·A4·A5 — 실화면(헤드리스 Edge · 실행 중 앱 2개): 설정 · 연결 목록 모델 드롭다운 · 에이전트 화면 도구 칸 없음 · 요청 보내기 → 받는 쪽 '요청' 배지 · 목록 '대기' → 수용(메모) → 보낸 쪽 '수용' · 메모 · 콘솔 오류 0(사람 확인은 19 TC)
  EVIDENCE: 2026-10-13 · 구현 세션(Claude) · `3dc6d18` 소스(라운드 2 반영 뒤 `d60cda8` 에서 같은 확인 다시 — 같은 값 · 오류 0) · 임시 두 PC 하네스(`tests/zz_e8.rs` — 커밋 안 함, 가상 `user-a` · `user-b`, 운영과 같은 `router_with` · 웹 정적 파일, 실행 중인 사용자 앱 · 실제 DB 는 건드리지 않음) · 헤드리스 Edge(playwright-core msedge) — 설정 모델 `SELECT` [opus, sonnet, haiku] · 연결 목록 상대별 ["", opus, sonnet, haiku] · 에이전트 체크박스 0 · 안내 "탐색 도구(읽기 전용) 항상 사용 — Read · Grep · Glob · LS · NotebookRead" / A 입력창 '요청' → 안내 보임 · 태그 "요청 · 대기" / B 사이드바 '요청' 배지 1 · 목록 "대기" → 수용(메모 "이번 주 안에 반영") → "수용" · 남은 단추 [완료] / A 보낸 요청 행 "수용 … 메모: 이번 주 안에 반영" · 대화 태그 "요청 · 수용" / 콘솔 · 페이지 오류 0. 상세 [stage3](work/stage3.md)


WEAK: E8 CHECK 없음 — 수동 gate(사람이 적은 EVIDENCE 로 판정)

## 적대적 평가 — 라운드 1/2

- 2026-10-13 · `/codex:adversarial-review --base 31ee236` · 세션 codex-review · 실행 HEAD `d0f18f2`(요청과 일치) · Verdict needs-attention(high 1 ·
  medium 1).

### 지적 원문

- [high] 스키마 마이그레이션이 미전달 완료 상태를 삭제한다 (crates/core/src/db.rs:280) — 스키마 14 마이그레이션은 `kind = 'done'`인 outbox 행을
  상태와 무관하게 삭제한다. 요청자가 오프라인인 동안 생성된 pending 완료 알림도 제거되므로, 로컬 메시지는 `done`으로 마이그레이션되지만 요청자
  복사본은 계속 `wait`로 남는다. 이는 D32의 SQLite 사용자 데이터 보존 및 오프라인 전달 계약을 깨는 영구 상태 분기다.
- [medium] ReqState 재시도 포기가 사용자에게 드러나지 않는다 (crates/agent/src/chat.rs:1857) — 상태 outbox 행을 `msg_id=None`으로 생성한다. 초기
  전송이 `Shipped::Kept`이면 API는 성공을 반환하지만, 상대가 계속 닿지 않아 재시도 한도를 넘으면 `give_up`은 `msg_id`가 없어 즉시 반환한다. 따라서
  받은 쪽은 수용/완료로 표시한 채 요청자는 계속 이전 상태를 보며, UI에 동기화 실패나 재전송 수단도 남지 않는다.

### 대조 · 처리

| 지적 | 판단 | 처리 |
|---|---|---|
| high · 마이그레이션이 못 보낸 완료 알림을 지움 | 사실 — plan 해석 2 ③ 이 "지운다" 로 정했으나 D32 의 사용자 데이터 보존과 어긋난다 | 같은 자리 · 순서로 `ReqState` 프레임(`done: true` → `done` · `false` → `wait`, `note: null`)으로 바꾸고 `msg_id` 를 그 요청 글에(SQLite JSON 함수, 한 트랜잭션). 시험 `core requests` `schema14_moves_done_and_models`(바뀐 행 2 · 다른 행 그대로) · `agent requests` 바뀐 프레임 꼴 읽기 — plan 개정 3 |
| medium · 상태 알림 포기가 안 보임 | 사실 | 상태 보내기 행을 그 요청 글에 잇고(`stage(…, Some(id), REQ_STATE, …)`), `give_up` 의 `REQ_STATE` → 그 글 `refused = unreachable`. 화면 "요청자에게 상태를 알리지 못함" · 작업 "다시 알리기"(같은 상태 · 메모 — 이때만 같은 상태 옮김 허용, 표시를 지움). 시험 `agent requests` `state_give_up_marked_and_sent_again` · `view.test` `req_moves` — plan 개정 3 · E5 7건 |

## 적대적 평가 — 라운드 2/2

- 2026-10-13 · `/codex:adversarial-review --base 31ee236` · 세션 codex-review(라운드 1 과 같은 세션) · 실행 HEAD `97cab8b`(요청과 일치) · Verdict
  needs-attention(high 1 · medium 1). 상한 도달 — 반영 뒤 재리뷰하지 않는다.

### 지적 원문

- [high] 명시적 전송 거절 후 상태 동기화를 복구할 수 없다 (crates/agent/src/chat.rs:1870-1875) — `dispatch`는 `Response::Error`도 응답 수신
  성공으로 보고 outbox 행을 삭제한다. 여기서는 non-`Ok`를 일회성 warning으로만 반환하며 실패 표시는 남기지 않는다. 요청자 DB rollback 등으로
  `unknown`이 반환되면 로컬 `rejected`/`done`은 유지되지만 원격은 이전 상태에 머문다. 같은 상태 재전송은 `refused == unreachable`일 때만 허용되므로
  새로 고침 후 복구 경로도 없다.
- [medium] 동일 ReqState 재전송이 idempotent하지 않다 (crates/agent/src/chat.rs:1900-1918) — 상태 프레임은 응답 유실 시 동일 내용으로 재전송될 수
  있지만, 수신부는 기존 상태·메모 비교 없이 매번 `req_at`을 갱신하고 `AskState` 알림을 다시 발생시킨다. 한 번 적용된 변경이 최신 요청처럼 계속
  위로 올라오고 브라우저·트레이 알림이 반복될 수 있다.

### 대조 · 처리

| 지적 | 판단 | 처리 |
|---|---|---|
| high · 거절 뒤 복구 경로 없음 | 사실 | 바로 보낼 때의 거절 · 줄이 보낼 때의 거절(`settle` 의 `REQ_STATE`) 모두 그 받은 요청 글에 거절 코드(`refused`) — 화면 "요청자에게 상태를 알리지 못함 — 상대가 받지 않았다(code)" · "다시 알리기"(같은 상태 옮김을 `refused` 가 있으면 허용). 시험 `agent requests` `state_give_up_marked_and_sent_again`(요청자 글이 없어 `unknown` → 표시 · 경고 → 되살린 뒤 다시 알리기로 도착) · `view.test` `req_moves` — plan 개정 4 |
| medium · 같은 상태 재전송이 시각 · 알림을 다시 만듦 | 사실 | `inbound_req_state` 가 지금 상태 · 메모와 같으면 바로 `Ok`(시각 · 알림 그대로). 시험 같은 시험에 같은 프레임 재전송 — `req_at` · `AskState` 수 그대로 — plan 개정 4 |

관찰(지적 아님): 라운드 1 반영 뒤 gate 체커 실행 중 E1(`cargo test --workspace`)이 rc 101 로 3회 실패했다(약 12회 중). 같은 명령을 체커 밖에서
14회 · 체커를 감싸 4회 돌렸을 때는 모두 통과했고 실패한 시험 이름은 잡지 못했다 — 같은 시각 다른 cargo 작업(리뷰 세션 · 반복 실행)과 겹친 부하로
본다(확인 필요). 종결 측정은 단독 실행으로 한다. → 종결 뒤 원인 식별 · 조치: [부하에서 간헐 실패](../../troubleshootings/2026-10-13-flaky-tests-under-load.md)
(제품 결함 2 — 지연 트랜잭션 잠금 · `stopped` 넘김 표시, 시험 시간 창 5).

## 미조치 지적

없음
