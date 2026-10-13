# eval.md — 1c.durable-messaging 적대적 평가

## 평가 원장

<!-- tools/check_gates.py 산출 — 손으로 고치지 않는다. 단 CHECK 가 없는 수동 gate 의
     EVIDENCE 와 ABANDON 줄은 사람이 적고, 재실행해도 보존된다.
     WEAK 줄은 기계가 매 실행 다시 단다 — 약한 gate 를 셀 수 있게 하는 표시이며
     판정에는 영향이 없다. -->

측정 커밋: 3127626e82d214505f4ad20b04e281d3bf742f13

- [x] E1: A11 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

- [x] E2: A11 — 우리 코드 경고 0(clippy -D warnings)
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.28s

- [x] E3: A5·A8·A10·A11 — 화면 로직 node 테스트 42건 이상 · 실패 0(기존 39 + outboxText · 마지막 통신 · 단계 문구)
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# pass (4[2-9]|[5-9][0-9])\s+# fail 0/
  EVIDENCE: # skipped 0 / # todo 0 / # duration_ms 81.9429

- [x] E4: A1·A4·A5 — outbox 단위 5건(추가 · 키 합침 · 맨 앞 행 · 재시도 간격 · 실패 보관)
  CHECK: cargo test -p an-core --test outbox
  EXPECT: test result: ok. 5 passed; 0 failed
  EVIDENCE: test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s

- [x] E5: A1·A2·A3·A4·A5 — 두 PC 통합 6건(꺼진 상대에 보낸 요청 → 켜지면 실행 · 답 / 묻는 쪽이 꺼졌을 때 답 → 켜지면 도착 / 보낸 쪽 재시작 뒤 이어 보냄 / 재시도 · 재시작에도 한 번만 실행 / 한 상대가 막혀도 다른 상대 즉시 / 한도 넘으면 실패 보관 · 글 "보내지 못함" · 상태 질의 done 이면 답 다시)
  CHECK: cargo test -p an-agent --test durable
  EXPECT: test result: ok. 6 passed; 0 failed
  EVIDENCE: test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 8.37s

- [x] E6: A6·A7 — CLI 트랜잭션 6건(`--id` 재실행은 새 실행 없음 · 같은 결과 / 같은 `--id` 동시 두 번은 한 번만 보냄(개정 1) / CLI 를 끊고 `wait <tx>` 로 이어 받음 / `status` 한 번 출력 / 대기 · 실행이 확인되는 동안 기다림 / 상대가 꺼지면 정해진 시간 뒤 종료 코드 7)
  CHECK: cargo test -p coworkagent-app --test cli_tx
  EXPECT: test result: ok. 6 passed; 0 failed
  EVIDENCE: test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 8.23s

- [x] E7: A8·A9 — 접속 상태 4건(유휴 동안 피어 연결 0 · 보내기 성공이 접속 시각 갱신 · 받기도 갱신 · announce 로 바뀐 포트를 상대가 따라감 · ping 60초 안 중복 없음)
  CHECK: cargo test -p an-net --test presence
  EXPECT: test result: ok. 4 passed; 0 failed
  EVIDENCE: test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.04s

- [x] E8: A10 — 처리경과 4건(stream-json 줄 → 단계 요약 · MCP 이름 묶기 · 10초 합침 · 보낸 쪽 진행에 단계 · 요약에 입력 · 경로 · 글 없음)
  CHECK: cargo test -p an-agent --test steps
  EXPECT: test result: ok. 4 passed; 0 failed
  EVIDENCE: test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.67s

- [x] E9: A10 — 실물: 실제 claude 2.1.296 stream-json 실행에서 단계 요약이 실행 끝 전에 1회 이상 · 결과 파싱 그대로(`#[ignore]` 시험, 사용량 조금)
  CHECK: cargo test -p an-agent --test steps_live -- --ignored
  EXPECT: test result: ok. 1 passed; 0 failed
  EVIDENCE: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.74s

- [x] E10: A12 — README 상대 링크 전부 존재 · 기술 절(통신 메커니즘 · 피어 프로토콜 · 빌드) 없음 · 목적 4가지 문구 있음
  CHECK: python tools/check_readme.py
  EXPECT: readme ok
  EVIDENCE: readme ok

- [x] E12: A1·A2 — 글과 보내기 행 원자성 4건(보내기 행을 못 적으면 사람 요청 글도 없음 · 답도 없음 — 개정 1 / 자동 왕복 넘김이 `sent` 로 남지 않음 · 포기는 보내는 중인 행을 덮지 않음 — 개정 2)
  CHECK: cargo test -p an-agent --test atomic
  EXPECT: test result: ok. 4 passed; 0 failed
  EVIDENCE: test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s

- [x] E11: A1·A5·A8·A10 — 실화면(헤드리스 Edge · 실행 중 앱): "보내는 중 / 보내지 못함" 메타 · 마지막 통신 문구 · 처리 중 단계 문구 · 콘솔 오류 0(사람 확인은 19 TC)
  EVIDENCE: 2026-10-13 11:35 · 구현 세션(Claude) · `c58f1d1` 릴리스 빌드(`rebuild_run.py` 로 앱 재시작) · 헤드리스 Edge(playwright-core msedge 채널) — 레일 타일 · 대화 머리글 접속 문구 "방금 통신"(시작 알림 뒤) · `GET /api/peers` 모든 피어 `seen_at` 키 / 글 목록 응답을 가로채 마지막 글 3개 주입 → 메타 "보내는 중 — 상대에 닿지 않아 다시 보냄(2회)" · "처리 중 — 2턴 · Grep 1 · Read 3 · 41초" · "오류 · 보내지 못함 — 상대에 오래 닿지 않았다(다시 보내기로 다시 보낼 수 있다)" / 콘솔 · 페이지 오류 0. 상세 [stage5](work/stage5.md)


WEAK: E11 CHECK 없음 — 수동 gate(사람이 적은 EVIDENCE 로 판정)

## 적대적 평가 — 라운드 1/2

- 2026-10-13 · `/codex:adversarial-review --base 5c63e66` · 세션 codex-review · 실행 HEAD `f89a528`(요청과 일치) · Verdict needs-attention(high 2).

### 지적 원문

- [high] 메시지 기록과 outbox 등록 사이 crash가 영구 유실을 만든다 (crates/agent/src/chat.rs:1185-1203) — 발신 메시지는 먼저 commit되고 outbox는
  `ship`에서 별도 등록된다. 이 사이 프로세스가 종료되면 `sent` 메시지만 남고 outbox 행이 없어 `recover`가 재전송하지 못한다. 답도 `record` commit 후
  별도 `ship`하는 동일 구조라 A1/A2의 요청·응답 내구성을 모두 깨뜨린다.
- [high] CLI `--id` 중복 방지가 check-then-act라 동시 요청과 crash에 실패한다 (crates/app/src/chat_api.rs:932-939) — `tx_again` 확인 후 원격 전송을
  완료하고 나서야 `sent_tx`가 `cli_tx`를 생성한다. 같은 키의 동시 요청은 둘 다 사전 확인을 통과해 서로 다른 메시지를 실행하며, 두 번째 UNIQUE 실패는
  삼키고 `201`과 `tx: null`을 반환한다. 전송 후 `tx_create` 전 crash도 같은 키 기록을 남기지 않는다.

### 대조 · 처리

| 지적 | 판단 | 처리 |
|---|---|---|
| high · 기록과 outbox 사이 유실 | 사실 — 사람 요청(`send_human`) · 미러(받은 대화 사람 글) · 답(`record`) 모두 커밋 두 번 | `ship` 을 `stage`(행 적기) + `dispatch`(보내기)로 나누고, 세 곳은 글을 적는 트랜잭션 안에서 `stage` → 커밋 → `dispatch`. 커밋 뒤 꺼지면 행이 `inflight` 로 남아 다시 켤 때 `recover` 가 보낸다. 시험 `atomic` 2건(보내기 행 적기를 트리거로 막으면 글 · 답도 없음) — plan 개정 1 · E12 |
| high · `--id` check-then-act | 사실 | `cli_tx` 를 글과 같은 트랜잭션에서 만들고(`Compose.tx_key`), 키 있는 CLI 보내기는 `Chat::keyed` 잠금 안에서 확인 → 보내기. 트랜잭션이 없으면 201 대신 오류(`tx: null` 성공 없음). 시험 `cli_tx` `same_id_concurrent_sends_once` — plan 개정 1 · E6 6건 |

남는 틈(지적 범위 밖 — 기록): 자동 왕복 넘김(`relay_step` · `relay_last`) · 완료(`done`) · 멈춤 알림(`relay`)은 여전히 기록 커밋 뒤 따로 행을 적는다 —
그 사이 꺼지면 자동 왕복은 멈춘 채 남고(사람이 "재개" 로 넘기지 않은 답을 다시 넘김) 완료 알림은 다시 누르면 간다. 받은 대화 사람 글을 내 에이전트에게
넘기는 `queued` 표시도 커밋 뒤라 그 사이 꺼지면 실행되지 않은 채 `sent` 로 남는다(1b 이전부터).

## 적대적 평가 — 라운드 2/2

- 2026-10-13 · `/codex:adversarial-review --base 5c63e66` · 세션 codex-review(라운드 1 과 같은 세션) · 실행 HEAD `20245ba`(요청과 일치) · Verdict
  needs-attention(high 2). 상한 도달 — 반영 뒤 재리뷰하지 않는다.

### 지적 원문

- [high] 자동 왕복 상태와 outbox가 원자적이지 않아 재시작 시 영구 정지한다 (crates/agent/src/chat.rs:2377-2401) — `State::Sent` 전환을 먼저 커밋한
  뒤 `ship()`이 별도 DB 작업으로 outbox를 생성한다. 두 작업 사이에서 프로세스가 종료되면 실행 요청은 전송되지 않고 outbox도 없다. `recover()`는
  `queued`/`running` 메시지와 기존 outbox만 복구하므로 해당 자동 왕복은 `Sent` 상태로 영구 정지한다.
- [high] 포기 처리가 성공 중인 다른 전송까지 실패로 덮어쓴다 (crates/agent/src/chat.rs:707-719) — 재시도 한도에 도달하면 같은 peer의 모든
  `pending`/`inflight` 행을 실패 처리한다. `dispatch()`는 다른 conversation의 `inflight` 행을 차단하지 않고, `failing()`도 `pending`만 검사한다.
  오래된 전송이 실패하는 동안 새 전송이 성공 중이어도 새 메시지를 `unreachable`로 바꿀 수 있다 — 원격에서는 실행됐지만 로컬에는 실패로 표시되어
  사용자가 다시 보내면 중복 실행된다.

### 대조 · 처리

| 지적 | 판단 | 처리 |
|---|---|---|
| high · 자동 왕복 넘김 원자성 | 사실 — 라운드 1 기록의 "남는 틈" 중 하나 | `relay_forward` 의 `set_flags(sent)` · `set_refused(None)` · 보내기 행(`stage`)을 한 트랜잭션 → 커밋 뒤 `dispatch`. 시험 `atomic` `relay_step_not_marked_sent_without_outbox_row` — plan 개정 2 |
| high · 포기가 보내는 중 행을 덮음 | 사실 | `outbox::queued_for` 를 `pending` 만으로 — 보내는 중(`inflight`) 행은 그 보내기 결과(성공이면 지움 · 실패면 자기 재시도)가 정한다. 상대별 보내기를 한 줄로 강제하는 권고는 하지 않음(바로 보내기가 앞선 행 없는 대화만 보내는 1c 설계 — 다른 대화 지연 방지). 시험 `atomic` `give_up_leaves_inflight_rows` — plan 개정 2 |

## 미조치 지적

| 지적 원문(요약) | 판단 | 미조치 사유 |
|---|---|---|
| 라운드 1 남는 틈 — 완료(`done`) · 멈춤 알림(`relay`) · 자동 왕복 마지막 기록(`relay_last`) 은 기록 커밋 뒤 따로 행을 적는다 | 사실(지적 밖 · 자체 기록) | 그 사이 꺼지면 완료 알림은 다시 누르면 가고, 마지막 기록은 상대 이력에 한 글이 빠질 뿐 실행 · 답은 잃지 않는다. 라운드 상한 |
| 자동 왕복 — 내 에이전트 답 기록(`record`)과 넘김 트랜잭션 사이에 꺼지면 넘기지 않은 답으로 남는다 | 사실(자체 기록) | 연 쪽 "재개" 가 넘기지 않은 답을 다시 넘긴다(1b 이전부터 있는 복구 경로). 라운드 상한 |
| 받은 대화 사람 글의 `queued` 표시가 커밋 뒤 — 그 사이 꺼지면 내 에이전트가 실행하지 않은 채 `sent` | 사실(1b 이전부터 · 자체 기록) | 1c 범위 밖(보내기 아님). 라운드 상한 |
| 페어링을 지운 상대 앞으로 보관된 글도 24시간 다시 보낸다 | 사실(stage1 리뷰 기록) | 보내기는 닿지 않을 뿐 해가 없고 24시간 뒤 "보내지 못함" 으로 끝난다 |

