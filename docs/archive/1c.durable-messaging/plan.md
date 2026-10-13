# plan — 1c.durable-messaging

> 근거: [intent](intent.md)(승인 2026-10-13 — 라운드 2). **구현자는 이 문서만 보고 일한다.** D·S 는 [masterPlan](../../masterPlan.md).
> 기준 커밋: `5c63e66`(main — Merge Task #1b, `task/1c-durable-messaging` 분기점). plan 승인: 2026-10-13 — 위임(사용자 원문 "승인. 개발 진행").

## 목표

상대에게 가는 대화 프레임을 SQLite outbox 에 먼저 적고 상대별 줄이 ack 까지 다시 보낸다(D7 의 "보관했다가 재전송" 실현). CLI `ask` 는 앱 DB 의
트랜잭션을 보며 중복 키 · 이어 받기 · 정해진 종료(`stalled`)를 갖는다. 30초 접속 확인을 없애고 통신할 때만 접속 시각을 갱신한다. 받는 쪽은
`claude -p` 를 실시간 이벤트로 돌려 단계 요약을 진행으로 보낸다. 끝으로 README 를 사용자 관점으로 바꾸고 기술 내용은 매뉴얼로 옮긴다.

### 성공 기준(intent 사본)

- A1: 상대가 꺼져 있을 때 보낸 실행 요청은 이 PC 에 남고, 상대가 다시 켜지면(이 PC 를 다시 켠 경우 포함) 사람이 다시 보내지 않아도 전달 · 실행 · 답이 온다.
- A2: 답 · 완료 표시 · 진행도 같은 보장 — 답을 보낼 때 묻는 쪽이 꺼져 있었어도 다시 켜지면 답이 도착한다.
- A3: 같은 글은 몇 번 다시 보내도(재시도 · 앱 재시작 · CLI 재실행) 상대에서 한 번만 실행된다.
- A4: 한 상대가 닿지 않아도 다른 상대로 가는 글 · 진행이 늦어지지 않는다.
- A5: 재시도 한도를 넘은 글은 실패 보관으로 옮겨 화면에 "보내지 못함" 과 다시 보내기가 보인다.
- A6: CLI `ask --id <키>` 를 다시 실행하면 새 실행 없이 같은 트랜잭션의 결과를 받는다. CLI 가 중간에 죽어도 `wait <tx>` 로 이어 받는다.
- A7: CLI 는 상대가 대기 · 실행 중임이 확인되는 동안 계속 기다리고, 상대에 정해진 시간 동안 닿지 않으면 정해진 종료 코드로 끝난다.
- A8: 주기적 접속 확인이 없다 — 유휴 상태에서 피어 연결이 생기지 않는다. 주고받기에 성공하면 접속 시각이 갱신되고 화면은 "마지막 통신 n분 전".
- A9: 1b 의 바뀐 포트 · IP 따라가기는 그대로 된다(주기 확인 없이).
- A10: 실행 중 단계 요약이 보낸 쪽 웹UI · CLI(평문 · `--json`)에 실행이 끝나기 전에 보인다. 상대에게 가는 요약에 도구 입력 · 파일 경로 · 모델 글은 없다.
- A11: 기존 테스트 · 웹UI 테스트 green, 경고 0.
- A12: README 는 목적 · 주요 기능 요약 · 매뉴얼 링크 중심, 기술 내용은 매뉴얼 — README 의 링크가 모두 살아 있다.

## intent 대비 해석 (plan 승인 대상)

1. **outbox 표(스키마 13)** — `outbox(id, peer_id, msg_id?, kind, frame JSON, key?, state 'pending'|'inflight'|'failed', tries, waited, next_at,
   last_error, created_at)`. ack(상대 응답 수신 — `Ok` 든 거절이든) 받으면 **행을 지운다**, 실패 보관은 `failed` 로 남긴다. `key` 가 같은 `pending` 행이
   있으면 새 행 대신 그 행의 프레임을 바꾼다(진행 · 완료 표시 — 마지막 값만).
2. **보내기 = `Chat::ship`** — 행을 `inflight` 로 넣고, 그 상대에 앞선 행(`pending` · `inflight`)이 없으면 **바로 한 번** 보낸다(지금처럼 화면이 거절을 바로
   받는다). 응답이면 행을 지우고 응답을 돌려준다. 전송 오류거나 앞선 행이 있으면 `pending` 으로 두고 상대 줄을 깨운다 → 호출자는 "보관됨".
   진행 알림은 바로 보내지 않고 줄에만(`enqueue`).
3. **상대 줄 = 상대마다 tokio 작업 하나** — `pending` 을 id 순으로 하나씩(앞 행이 못 가면 뒤 행도 기다린다 — 대화 순서 보존). 전송 오류면 간격 5초부터 두
   배 · 최대 5분, `waited` 에 더해 24시간이면 `failed`. 깨우는 때: 새 행 · 그 상대에서 무엇이든 받음(받으면 맨 앞 행의 `next_at` 을 지금으로) · 시작
   (`recover`) · 다음 `next_at`. 시험용 간격은 `Chat::set_retry(base, max, give_up)`.
4. **ack 뒤 처리(`settle`) = 지금 그 자리의 응답 처리 그대로** — 종류(`kind`): `message`(사람 실행 요청) · `relay_open` · `relay_step`(자동 왕복 넘김) ·
   `mirror`(사람 글 미러) · `reply` · `reply_post`(내 에이전트 답 미러) · `relay_last`(상한 기록) · `done` · `relay`(멈춤 알림) · `progress`. 거절이면
   지금과 같이(`refused_by_peer` · 자동 왕복 끔 · `relay_end` · 미러 `error`), `Ok` 면 미러 `done`. **실패 보관**: `message` · `relay_open` · `relay_step` ·
   `mirror` → 글 `error` + 거절 코드 `unreachable`("보내지 못함 — 상대에 24시간 닿지 않았다", 다시 보내기), `reply` · `reply_post` · `relay_last` →
   `undelivered`, 나머지 → 로그.
5. **바뀌는 사용자 동작** — 사람 실행 요청 · 미러가 전송 오류여도 **오류가 아니다**: 글은 `sent` 로 남고 메시지 JSON `outbox {state, tries, next_at,
   error}` 로 웹UI 메타 줄에 "보내는 중 — 상대에 닿지 않아 다시 보냄(n회)" · 실패 보관이면 "보내지 못함". "전달 확인 안 됨 … 다시 보내세요" 오류는
   없어진다(다시 보내도 uuid 로 한 번만 실행 — A3). 완료 표시 경고("상대에 알리지 못했다")도 없어진다. 자동 왕복 넘김이 전송 오류여도 멈추지 않는다
   (닿으면 이어 간다). 자동 왕복 **재개**(`Relay{on:true}`)와 `Hello` · `Approve` · `Remove` · `Agents` · `Status` 는 지금처럼 바로(사람이 결과를 기다림).
6. **상태 질의 `Request::Status{conv, msg}` → `Response::Status{state, ahead}`** — 받는 쪽: 그 상대 대화의 상대 실행 요청 글이면 `queued`(실행 목록 위치 =
   앞 사람 수) · `running` · `done`(답이 있음 — 그 답 행이 outbox 에 남아 있으면 그 상대 줄을 지금 깨운다) · 그 밖 `unknown`. 페어링된 상대만 · id 는 UUID 꼴.
7. **CLI 트랜잭션(스키마 13 `cli_tx`)** — `cli_tx(id, key UNIQUE?, message_id, created_at, unreachable_since?, stalled_at?)`. 로컬 API 새 글 요청
   (`POST /api/conversations` · `/api/conversations/{id}/messages`)에 `tx_key?`(via `cli` 일 때만) — 같은 키가 있으면 **보내지 않고** 그 트랜잭션의
   대화 · 글을 200 으로, 없으면 보내고 트랜잭션을 만든다. 응답에 `tx`. `GET /api/tx/{id}` → `{tx, state, conversation, message, reply?, progress?, outbox?}`,
   `state` = `sending`(outbox 에 남음) · `sent` · `queued` · `running` · `answered` · `failed` · `stalled`. 이 조회가 감시도 한다 — 끝나지 않았고 마지막
   상태 질의가 60초 전이면 `Status` 를 하나 보낸다(같은 글에 하나만 진행 중), 닿으면 `unreachable_since` 를 지우고 진행을 갱신, 전송 오류면
   `unreachable_since` 를 남기고 그로부터 10분이면 `stalled_at`. 시험용 간격 `Chat::set_watch(status_every, stall_after)`.
8. **CLI 명령** — `ask … [--id <키>]`(키는 1 ~ 64자 `[A-Za-z0-9._-]`), `wait <tx>` · `status <tx>`(한 번 출력). `ask` · `wait` 은 `GET /api/tx` 를 1초마다
   보고 진행이 바뀌면 표준 오류에 한 줄(1b 문구 + 단계 요약), `answered` → 답 출력 · 0, `failed` → 5, `stalled` → 7(`EXIT_STALLED`, "상대에 10분 닿지
   않음 — 나중에: coworkAgent wait <tx>"). CLI 자체 시계는 안전판 `WAIT_LIMIT` 24시간(넘으면 6). `--json` 결과에 `tx`. 1b 의 "queued 동안 멈춤" 시계는
   없앤다.
9. **접속 상태** — `watch_presence` · `PRESENCE_EVERY` 삭제. `Node::announce()`(시작 때 페어링된 상대 모두에 `Ping` 1회 — 주소 알림 · 집중) ·
   `peers::touch(spki)`(보내기 성공 · 페어링된 상대의 요청 받기마다 `seen_at`). 피어 JSON `seen_ago`(초 · 없으면 null) · `online`(75초 안) 그대로.
   `POST /api/peers/{id}/ping` 은 `seen_at` 이 60초 안이면 보내지 않고 지금 값. 웹UI: 시작 때 페어링된 상대 모두 · 레일 타일을 고를 때 그 상대 ping,
   표시 문구 "방금 통신" · "n분 전 통신" · "n시간 전 통신" · "통신 기록 없음"(집중이면 앞에 "집중 중 ·").
10. **처리경과** — `claude_args` 는 `--output-format stream-json --verbose`. `run_process_lines(…, on_line)`(표준 출력을 줄 단위로 읽으며 부름 —
    `run_process` 는 빈 콜백). `RunInput.steps: StepSink`(`Option<Arc<dyn Fn(&Step) + Send + Sync>>` 감싼 새 형 — 비교 · 출력에서 무시). `Step{turns,
    tools: BTreeMap<이름, 횟수>, secs}` — `assistant` 의 `tool_use` 이름을 세고(`mcp__` 로 시작하면 `MCP`), 턴 = 받은 `user` 이벤트 + 1. 받는 쪽 `Chat`
    은 10초에 한 번 이하로 `Progress{state: running, detail: Step}` 을 outbox(키 = 그 글)에. 보낸 쪽은 받은 이름을 `clean_name` · 16개 · 24자로 자르고
    저장(메모리). 문구 "처리 중 — 2턴 · Read 3 · Grep 1 · 41초". `parse_output` 은 지금처럼 마지막 JSON 줄(`result`).
11. **README** — 사용자 관점: 무엇 · 왜(웹서버 없음 — 각 PC 앱끼리 직접 · AI API 비용 없음 — 각자의 Claude 구독 · 페어링 코워크 · 대화가 서버에 쌓이지
    않음) · 주요 기능 한 줄씩 + 매뉴얼 링크 · 설치 · 빠른 시작. 통신 메커니즘 · 피어 프로토콜 · 빌드 · 개발은 [developer-guide](../../manual/dev/developer-guide.md) 로,
    사용 방법 상세는 [user-guide](../../manual/user/user-guide.md) 로. `tools/check_readme.py` — README 상대 링크가 모두 있고 금지 절(통신 메커니즘 · 피어 프로토콜 ·
    빌드)이 없는지.

## 설계

### `an-core`
- `db.rs` 스키마 13 — `outbox` · `cli_tx` 표.
- `outbox.rs`(새) — `add` · `head(peer)` · `set_inflight` · `retry` · `fail` · `remove` · `pending_peers` · `for_message` · `failed_for`.
- `cli_tx.rs`(새) — `create(key?, message_id)` · `by_key` · `by_id` · `set_unreachable` · `clear_unreachable` · `set_stalled`.
- `peers.rs` — `touch(spki)` · `Peer.seen_ago(now)`.

### `an-net`
- `proto.rs` — `Request::Status{conv, msg}` · `Response::Status{state, ahead}` · `Progress.detail: Option<StepDetail>` · `ids_ok` 에 `Status`.
- `node.rs` — `watch_presence` 삭제 · `announce()` · 보내기 성공 · 페어링된 상대 요청마다 `touch` · `Status` 를 처리기로.

### `an-agent`
- `chat.rs` — `ship` · `enqueue` · 상대 줄 · `settle` · `give_up` · `set_retry` · `inbound_status` · `status_of`(보낸 쪽 질의) · 트랜잭션 조회 · 감시 ·
  `set_watch` · 진행 단계 저장 · 10초 합침. 7곳 `node.send` → `ship` / `enqueue`(재개 `Relay{on:true}` 는 그대로).
- `runner.rs` — stream-json · `run_process_lines` · `Step` · `StepSink`.

### `an-app`
- `chat_api.rs` — `tx_key` · `GET /api/tx/{id}` · 메시지 JSON `outbox` · 진행 `detail`. `peers_api.rs` — `seen_ago` · ping 60초. `main.rs` — `announce`.
- `cli.rs` — `--id` · `wait` · `status` · `EXIT_STALLED` · `/api/tx` 감시 · `WAIT_LIMIT` 24시간.
- 웹UI — `view.js` `progressText`(단계) · `outboxText` · `presence`(마지막 통신), `app.js` 메타 줄 · ping 시점.

### 문서 · 도구
- README 개편 · `tools/check_readme.py` · 매뉴얼(user-guide · developer-guide · 모듈 4).

## Stage

| stage | 목표 | 산출 | gate |
|---|---|---|---|
| 1 | outbox · 상대 줄 · settle · 상태 질의 · 화면 "보내는 중 / 보내지 못함" | outbox 단위 · `durable` 통합 | E1 ~ E5 |
| 2 | CLI 트랜잭션 · `--id` · `wait` · `status` · `stalled` | `cli_tx` | E1 · E2 · E6 |
| 3 | 통신 때만 접속 상태 · announce · 마지막 통신 | `presence` | E1 ~ E3 · E7 |
| 4 | 처리경과(stream-json 단계 요약) | `steps` · runner 단위 | E1 ~ E3 · E8 · E9 |
| 5 | README 개편 · 매뉴얼 · 실화면 | `check_readme.py` | E10 · E11 |

## 평가 항목

- [ ] E1: A11 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E2: A11 — 우리 코드 경고 0(clippy -D warnings)
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: pending
- [ ] E3: A5·A8·A10·A11 — 화면 로직 node 테스트 42건 이상 · 실패 0(기존 39 + outboxText · 마지막 통신 · 단계 문구)
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# pass (4[2-9]|[5-9][0-9])\s+# fail 0/
  EVIDENCE: pending
- [ ] E4: A1·A4·A5 — outbox 단위 5건(추가 · 키 합침 · 맨 앞 행 · 재시도 간격 · 실패 보관)
  CHECK: cargo test -p an-core --test outbox
  EXPECT: test result: ok. 5 passed; 0 failed
  EVIDENCE: pending
- [ ] E5: A1·A2·A3·A4·A5 — 두 PC 통합 6건(꺼진 상대에 보낸 요청 → 켜지면 실행 · 답 / 묻는 쪽이 꺼졌을 때 답 → 켜지면 도착 / 보낸 쪽 재시작 뒤 이어 보냄 / 재시도 · 재시작에도 한 번만 실행 / 한 상대가 막혀도 다른 상대 즉시 / 한도 넘으면 실패 보관 · 글 "보내지 못함" · 상태 질의 done 이면 답 다시)
  CHECK: cargo test -p an-agent --test durable
  EXPECT: test result: ok. 6 passed; 0 failed
  EVIDENCE: pending
- [ ] E6: A6·A7 — CLI 트랜잭션 6건(`--id` 재실행은 새 실행 없음 · 같은 결과 / 같은 `--id` 동시 두 번은 한 번만 보냄(개정 1) / CLI 를 끊고 `wait <tx>` 로 이어 받음 / `status` 한 번 출력 / 대기 · 실행이 확인되는 동안 기다림 / 상대가 꺼지면 정해진 시간 뒤 종료 코드 7)
  CHECK: cargo test -p coworkagent-app --test cli_tx
  EXPECT: test result: ok. 6 passed; 0 failed
  EVIDENCE: pending
- [ ] E7: A8·A9 — 접속 상태 4건(유휴 동안 피어 연결 0 · 보내기 성공이 접속 시각 갱신 · 받기도 갱신 · announce 로 바뀐 포트를 상대가 따라감 · ping 60초 안 중복 없음)
  CHECK: cargo test -p an-net --test presence
  EXPECT: test result: ok. 4 passed; 0 failed
  EVIDENCE: pending
- [ ] E8: A10 — 처리경과 4건(stream-json 줄 → 단계 요약 · MCP 이름 묶기 · 10초 합침 · 보낸 쪽 진행에 단계 · 요약에 입력 · 경로 · 글 없음)
  CHECK: cargo test -p an-agent --test steps
  EXPECT: test result: ok. 4 passed; 0 failed
  EVIDENCE: pending
- [ ] E9: A10 — 실물: 실제 claude 2.1.296 stream-json 실행에서 단계 요약이 실행 끝 전에 1회 이상 · 결과 파싱 그대로(`#[ignore]` 시험, 사용량 조금)
  CHECK: cargo test -p an-agent --test steps_live -- --ignored
  EXPECT: test result: ok. 1 passed; 0 failed
  EVIDENCE: pending
- [ ] E10: A12 — README 상대 링크 전부 존재 · 기술 절(통신 메커니즘 · 피어 프로토콜 · 빌드) 없음 · 목적 4가지 문구 있음
  CHECK: python tools/check_readme.py
  EXPECT: readme ok
  EVIDENCE: pending
- [ ] E12: A1·A2 — 글과 보내기 행 원자성 4건(보내기 행을 못 적으면 사람 요청 글도 없음 · 답도 없음 — 개정 1 / 자동 왕복 넘김이 `sent` 로 남지 않음 · 포기는 보내는 중인 행을 덮지 않음 — 개정 2)
  CHECK: cargo test -p an-agent --test atomic
  EXPECT: test result: ok. 4 passed; 0 failed
  EVIDENCE: pending
- [ ] E11: A1·A5·A8·A10 — 실화면(헤드리스 Edge · 실행 중 앱): "보내는 중 / 보내지 못함" 메타 · 마지막 통신 문구 · 처리 중 단계 문구 · 콘솔 오류 0(사람 확인은 19 TC)
  EVIDENCE: pending

## 종결 시 할 일

- masterPlan D7 실현(보관 · 재전송 = SQLite outbox) · D14 보강(CLI 트랜잭션 · `wait` · `status` · 종료 코드 7) · D24 개정 보강(처리경과 요약 · 1b CLI 시계
  대체) · D32 적용(`Status` · `Progress.detail`) · 30초 접속 확인 삭제 기록.
- PRODUCT 도메인 규칙(보내기 보장 · 접속 상태 · 처리경과), 매뉴얼 모듈 4 · user-guide · developer-guide.
- 19 acceptance TC — 꺼진 상대 · 보내지 못함 · CLI `wait` · 마지막 통신 · 처리 중 단계.
- GitHub 반영(push)은 사용자 확인 뒤.

## 개정 기록

- 개정 1(2026-10-13, 적대적 평가 라운드 1/2 — high 2건 사실): 사람 요청 · 미러 · 답은 **글과 보내기 행(`outbox`)을 한 DB 트랜잭션으로** 적고 커밋 뒤
  보낸다(`Chat::stage` · `dispatch`, `record` 가 답 행까지). CLI 트랜잭션(`cli_tx`)도 그 트랜잭션에서 만들고, 키 있는 CLI 보내기는 확인부터 보내기까지
  한 줄(`Chat::keyed`) — 같은 키 동시 요청은 한 번만 보내고 둘 다 같은 트랜잭션, CLI 글에 `tx: null` 성공 응답 없음. E6 5 → 6건 · E12 추가.
- 개정 2(2026-10-13, 적대적 평가 라운드 2/2 — high 2건 사실): 자동 왕복 넘김의 `sent` 표시 · 거절 코드 지우기 · 보내기 행을 한 트랜잭션으로. 24시간 포기는
  그 상대 줄의 **기다리는(`pending`) 행만** — 지금 보내는 중(`inflight`)인 행은 그 결과가 정한다(닿은 글을 "보내지 못함" 으로 덮지 않음). E12 2 → 4건.
