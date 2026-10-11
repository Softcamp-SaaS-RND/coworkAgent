# plan — 12.auto-relay

> 근거: [intent](intent.md)(승인 2026-10-11 — 자동 진행 위임 + 확인 답 2건). **구현자는 이 문서만 보고 일한다.** D·S 는 [masterPlan](../../masterPlan.md).

## 목표

두 에이전트 자동 왕복(D24) — 연 쪽이 구동, 상한에서 멈춤, 양쪽 사람이 멈춤, 연 쪽이 재개.

### 성공 기준(intent 사본)

- A1: 자동 왕복 대화에서 상대 에이전트 답 → 내 에이전트 실행 → 그 답이 상대 에이전트 실행 요청으로, 상한(n 왕복)에 닿을 때까지 반복되고
  멈춘다. 양쪽 이력에 같은 순서로 남는다.
- A2: 어느 쪽 사람이든 멈춤을 누르거나 그 대화에 글을 쓰면 자동 전달이 멈춘다. 받은 쪽이 멈추면 그 PC 는 에이전트발 실행 요청을
  `stopped` 로 거절한다.
- A3: 대화를 연 쪽이 재개하면 카운트 0 에서 다시 자동 전달하고, 받은 쪽 거절도 풀린다.
- A4: 에이전트발 실행 요청은 페어링 · 대화 소유 · 가드(D31) 규칙을 그대로 따른다.
- A5: 루트 `PROJECT.md` 검증 명령이 통과한다.

### intent 결정 · 제외(사본)

- 1왕복 = 내 에이전트가 상대 답에 한 번 답함 · 멈춤 = 양쪽 · 사람이 쓰면 멈춤 · 재개 = 연 쪽(받은 쪽 거절 해제) · 상한 기본 10(1 ~ 50) ·
  연 쪽이 구동. 제외: 에이전트 자발 호출 · 다자 · 실화면 · 실제 claude(D28).

## 설계

### 저장(스키마 7, `an-core`)

```sql
ALTER TABLE conversations ADD COLUMN relay_agent TEXT;     -- 연 쪽: 내 에이전트(자동 왕복), 없으면 자동 왕복 아님
ALTER TABLE conversations ADD COLUMN relay_cap INTEGER;     -- 상한(왕복)
ALTER TABLE conversations ADD COLUMN relay_count INTEGER;   -- 지금까지 왕복
ALTER TABLE conversations ADD COLUMN relay_on INTEGER;      -- 연 쪽: 자동 전달 중
ALTER TABLE conversations ADD COLUMN relay_blocked INTEGER; -- 받은 쪽: 에이전트발 실행 요청 거절
```

- `Conversation` 에 `relay: Option<Relay{agent, cap, count, on}>` · `relay_blocked`. `chat::set_relay(conn, id, agent, cap)` ·
  `relay_on(conn, id, on, reset)` · `relay_step(conn, id) -> count` · `set_blocked(conn, id, bool)`.
- 실행 대기열은 작성자와 무관하게 `exec = 1`(에이전트발 실행 요청 포함) — `next_pending` · `pending_conversations` 의 `author = 'human'` 조건 제거.
- 이전(v < 7) — 10 의 규칙 그대로(한 트랜잭션 · 있는 열 건너뛰기 · 사본).

### 프로토콜

- `Request::Message` 에 `#[serde(default)] by_agent: Option<String>` — 있으면 그 이름의 에이전트가 보낸 실행 요청. 받은 쪽은 에이전트 글
  (mine 0 · exec 1)로 저장, `relay_blocked` 면 `error(stopped)`.
- `Request::Relay{conv, on}` — 받은 쪽 → 연 쪽: `on = false`(멈춤), 연 쪽 → 받은 쪽: `on`(재개 = 거절 해제 · 멈춤 알림). 그 상대 소유 대화만.
- 답 받기(`reply`)의 원 요청은 사람 글 또는 에이전트발 실행 요청(`exec` · `sent`).

### 조율(연 쪽 구동)

- 상대 답 수신(`inbound_reply`) 때 그 대화가 자동 왕복 중이고 `count < cap` 이면 그 답을 실행 요청으로(`exec = 1` · `queued`) 표시하고 깨운다.
- `process` — 원 요청이 상대 에이전트 답이면 실행 에이전트 = `relay_agent`, 실행 후 `count + 1`, 내 에이전트 답을 `exec = 1` · `sent` 로 두고
  `message{by_agent}` 로 상대에 보낸다(거절 `stopped` · 오류면 자동 왕복 멈춤). 실행 실패면 멈춤.
- 사람 글(어느 쪽이든, 실행 요청 · 메모 모두): 연 쪽이면 `relay_on = 0`, 받은 쪽이면 `relay_blocked = 1` + `relay{on:false}` 를 연 쪽에.
- `Chat::relay(conv_id, on)` — 연 쪽: 멈춤 / 재개(카운트 0, 받은 쪽에 `relay{on:true}`, 마지막 글이 아직 넘기지 않은 상대 답이면 바로 실행).
  받은 쪽: 멈춤만(`relay_blocked = 1` + 알림).
- `Chat::send` 에 자동 왕복 시작 인자(`relay: Option<(agent, cap)>`) — 새 대화일 때만.

### API · 웹UI

- `POST /api/conversations {peer_id, agent, text, relay_agent?, relay_cap?}` — `relay_agent` 는 이 PC 프로필(없으면 400), 상한 1 ~ 50(기본 10).
- `POST /api/conversations/{id}/relay {on}` · 대화 JSON 에 `relay_agent` · `relay_cap` · `relay_count` · `relay_on` · `relay_blocked`.
- 새 대화 폼에 "내 에이전트로 자동 왕복" · 상한, 대화 머리글 "자동 왕복 n / 상한" · 멈춤 · 재개(연 쪽), 받은 쪽은 멈춤. `view.js` `relayText`.

## Stage

| stage | 목표 | 산출 | gate |
|---|---|---|---|
| 1 | 스키마 7 · 프로토콜 · 연 쪽 구동 · 멈춤 · 재개 | 테스트(relay · paths) | E3 · E4 |
| 2 | API · 웹UI | 테스트(chat_api · view.test) | E1 · E2 · E5 · E6 · E7 |

## 평가 항목

- [ ] E1: A5 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E2: A5 — 우리 코드 경고 0(clippy -D warnings)
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: pending
- [ ] E3: A1·A2·A3·A4 — 자동 왕복(relay 테스트 7건: relay_runs_until_cap · human_write_stops_relay · receiver_stop_refuses_agent_requests · resume_restarts_count · remote_failure_stops_relay · late_reply_after_transport_error_accepted · stop_during_run_then_resume)
  CHECK: cargo test -p an-agent --test relay
  EXPECT: test result: ok. 7 passed; 0 failed
  EVIDENCE: pending
- [ ] E4: A5 — 스키마 7 이전(paths 6건)
  CHECK: cargo test -p an-core --test paths
  EXPECT: test result: ok. 6 passed; 0 failed
  EVIDENCE: pending
- [ ] E5: A1·A2·A3 — 웹UI API(relay_api 포함 chat_api 5건)
  CHECK: cargo test -p agentnetwork-app --test chat_api
  EXPECT: test result: ok. 5 passed; 0 failed
  EVIDENCE: pending
- [ ] E6: A1·A2 — 화면 로직 node 테스트(자동 왕복 문구)
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# fail 0/
  EVIDENCE: pending
- [ ] E7: A1·A2·A3 — 실화면 · 실제 claude(수동): 두 에이전트 자동 왕복 · 상한 · 양쪽 멈춤 · 재개
  EVIDENCE: pending

## 종결 시 할 일

- 매뉴얼 `an_core` · `an_net` · `an_agent` · `an_app` · PRODUCT(자동 왕복) · masterPlan `#12`.

## 개정 기록

- 개정 1(2026-10-11, Stage 1 구현): 자동 왕복 시작은 `Chat::send` 인자 대신 `Chat::start_relay(peer, agent, my_agent, cap, text)` — 새
  대화 전용. 받은 쪽 사람 글의 연 쪽 알림은 기다리지 않는다(거절은 바로).
- 개정 2(2026-10-11, 적대적 평가 라운드 1 반영): 상대 실행 실패 답(`ok = false`)은 넘기지 않고 멈춤. 넘기기 전송 오류는 `sent` 유지(늦은
  답을 받는다 — 명시적 거절만 `error`), 넘기기 전 상태 저장 실패면 보내지 않음. 답 기록과 자동 전달 등록 · 멈춤은 한 트랜잭션. E3 4 → 6건.
- 개정 3(2026-10-11, 적대적 평가 라운드 2 반영): 상한 n = 양쪽 에이전트 n 번씩 — 내 에이전트의 n 번째 답은 실행 요청 대신 상대 이력에
  기록만(`post` agent · 실행 없음, 받은 쪽은 내 에이전트 답에 단 답일 때만 받음)하고 멈춤. 재개는 마지막 글이 넘기지 못한 내 답이면 같은
  uuid 로 다시 넘긴다(받은 쪽은 기록만 받았던 그 글을 실행 요청으로). 켜기와 대기열 등록은 한 트랜잭션. E3 6 → 7건.
