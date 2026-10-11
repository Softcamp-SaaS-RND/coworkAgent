# plan — 11.two-sided-chat

> 근거: [intent](intent.md)(승인 2026-10-11 — 자동 진행 위임 + 확인 답 3건). **구현자는 이 문서만 보고 일한다.** D·S 는 [masterPlan](../../masterPlan.md).

## 목표

두 PC 양쪽 사람이 대화에 쓰고(에이전트에게 / 사람에게), 양쪽 이력이 같아지며, 에이전트는 그 사이 사람 글을 맥락으로 받는다.

### 성공 기준(intent 사본)

- A1: 받은 쪽 사람이 받은 대화에 "사람에게" 로 쓰면 실행 없이 양쪽 이력에 상대 사람 글로 남고, 보낸 쪽 사람의 "사람에게" 도 마찬가지다.
- A2: 받은 쪽 사람이 "에이전트에게" 로 쓰면 그 PC 의 대화 에이전트가 실행하고, 사람 글과 답이 양쪽 이력에 같은 순서로 남는다.
- A3: 에이전트 실행 입력에 지난 실행 요청 뒤의 사람 글이 발화자와 함께 앞에 붙는다(표준 입력, 가드 규칙 그대로).
- A4: 원격에서 온 미러 글은 그 상대 소유 대화에만 붙고(다른 상대 · 없는 대화 · 미페어링 거부), 같은 글 재전송은 한 번만 남는다.
  에이전트 답 미러는 그 상대가 쓴 사람 글에 대한 답일 때만.
- A5: 이전 이력(스키마 5)은 `mine` · `exec` 가 방향으로 채워져 그대로 보인다.
- A6: 루트 `PROJECT.md` 검증 명령이 통과한다.

### intent 결정 · 제외(사본)

- 양쪽 글마다 대상 선택(기본 에이전트) · 사람 글은 다음 실행 앞에 · 실행 없는 글과 받은 쪽 미러 = 새 요청 `post` · 받은 쪽 실행 답은
  `post`(에이전트) · 미러 실패는 로그 + 안내.
- 제외: 자동 왕복(12) · 다자 · 미러 재전송(Phase 2) · 실화면 · 실제 claude(D28).

## 설계

### 저장(스키마 6, `an-core`)

```sql
ALTER TABLE messages ADD COLUMN mine INTEGER;   -- 1 = 이 PC 에서 쓴 글(사람) 또는 이 PC 에이전트의 답
ALTER TABLE messages ADD COLUMN exec INTEGER;   -- 1 = 에이전트 실행 요청(사람 글), 0 = 실행 없는 글
UPDATE messages SET mine = ... , exec = ...     -- 이전 이력: out 대화의 사람 글 = mine 1, in 대화의 에이전트 답 = mine 1, 사람 글 exec = 1
```

- `Message` 에 `mine: bool` · `exec: bool`. `add_message` 는 지금 시그니처 유지(사람 = 실행 요청, `mine` 은 대화 방향으로) +
  `chat::add_human(conn, conv, uuid, text, state, mine, exec)` · `add_reply` 에 `mine`.
- `next_pending` 은 `exec = 1` 사람 글만. `chat::notes_between(conn, conv, after_id, before_id)` — 그 사이 실행 없는 사람 글.
- 이전(`v < 6`) — 한 트랜잭션 · 있는 열 건너뛰기 · 올리기 전 사본(10 의 규칙 그대로).

### 프로토콜(`an-net`)

- `Request::Post{conv, msg, author: "human"|"agent", text, reply_to: Option<String>, ok: Option<bool>, turns, duration_ms}` — 페어링된 상대만.
  응답 `ok` | `error(unknown | bad_request)`.

### 조율(`an-agent::Chat`)

- `Chat::post(conv_id, text, to_agent) -> (Conversation, Message)` — 대화 방향 · 대상에 따라:
  - `out` + 에이전트에게 → 지금의 `send`(이어 쓰기).
  - `out` + 사람에게 → 사람 글(mine 1, exec 0, `sent`) 저장 · `post`(human) 전송 → `done`(거절이면 `error`, 전송 오류면 `sent` 유지 + 안내).
  - `in` + 에이전트에게 → 사람 글(mine 1, exec 1, `queued`) 저장 · 대화 작업 깨우기 · `post`(human) 미러.
  - `in` + 사람에게 → 사람 글(mine 1, exec 0) 저장 · `post`(human) 미러.
- 받기 `post`: 대화는 `find(conv, peer.id)`(그 상대 소유, 방향 무관), 같은 msg uuid 는 `ok` 만. human → 저장(mine 0, exec 0, `done`).
  agent → `reply_to` 가 이 대화의 `mine = 1` 이 **아닌**(상대가 쓴) 사람 글이어야 함 → `add_reply`(mine 0). 이벤트.
- 작업(`process`): 실행 입력 = 맥락 앞붙임(`[대화 메모 — 실행 요청 아님]` + `- {발화자}: {본문}` 줄 + `[요청]` + 본문). 답 전달 — 그 사람
  글이 상대 것이면 지금처럼 `reply`, 내 것(mine)이면 `post`(agent, reply_to = 그 글 uuid).

### API · 웹UI

- `POST /api/conversations/{id}/messages {text, to: "agent"|"human"}`(기본 agent) — `in` 대화도 허용.
- 메시지 JSON 에 `mine` · `exec`. 입력창: 양쪽 · 대상 라디오. 발화자 = 사람(mine → "나", 아니면 상대 이름) · 에이전트(대화 에이전트 주인).
- `view.js`: `authorLabel(conv, m, peerName)` · 타임라인은 `exec` 사람 글만 요청으로.

## Stage

| stage | 목표 | 산출 | gate |
|---|---|---|---|
| 1 | 스키마 6 · `post` 프로토콜 · `Chat::post` · 맥락 앞붙임 | 테스트(store · paths · roundtrip) | E3 · E4 · E5 |
| 2 | API · 웹UI | 테스트(chat_api · view.test) | E1 · E2 · E6 · E7 · E8 |

## 평가 항목

- [ ] E1: A6 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E2: A6 — 우리 코드 경고 0(clippy -D warnings)
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: pending
- [ ] E3: A5 — 스키마 6 이전(schema6_fills_mine_exec · 기존 이전 테스트 포함 paths 5건)
  CHECK: cargo test -p an-core --test paths
  EXPECT: test result: ok. 5 passed; 0 failed
  EVIDENCE: pending
- [ ] E4: A1·A2·A3·A4 — 양쪽 발화 왕복(two_sided 테스트 7건: human_note_both_ways · receiver_asks_own_agent · notes_prepended_to_run · post_refused_when_foreign · concurrent_notes_same_order_both_sides · own_run_not_failed_by_mirror · late_queued_request_not_skipped)
  CHECK: cargo test -p an-agent --test two_sided
  EXPECT: test result: ok. 7 passed; 0 failed
  EVIDENCE: pending
- [ ] E5: A6 — 기존 왕복 회귀 없음(roundtrip 11건)
  CHECK: cargo test -p an-agent --test roundtrip
  EXPECT: test result: ok. 11 passed; 0 failed
  EVIDENCE: pending
- [ ] E6: A1·A2 — 웹UI API(conversation_post_targets 포함 chat_api 4건)
  CHECK: cargo test -p agentnetwork-app --test chat_api
  EXPECT: test result: ok. 4 passed; 0 failed
  EVIDENCE: pending
- [ ] E7: A1·A2·A5 — 화면 로직 node 테스트(발화자 이름 · 실행 글만 타임라인)
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# fail 0/
  EVIDENCE: pending
- [ ] E8: A1·A2 — 실화면(수동): 양쪽 입력창 · 대상 선택 · 양쪽 이력 일치 · 받은 쪽 실행
  EVIDENCE: pending

## 종결 시 할 일

- 매뉴얼 `an_core`(스키마 6 · `add_human` · `notes_between`) · `an_net`(`post`) · `an_agent`(`Chat::post` · 맥락) · `an_app`(API · 입력창) ·
  PRODUCT(대화 왕복 · 양쪽 발화) · masterPlan `#11`.

## 개정 기록

- 개정 1(2026-10-11, 적대적 평가 라운드 1 반영): 양쪽 공통 순서 = 램포트 시계 `lc`(스키마 6 에 함께 — 쓴 쪽이 대화 최대 `lc` + 1, 받은 쪽은
  그대로 `observe`, 정렬 `(lc, uuid)`, `message` · `reply` · `post` 에 `lc`), 실행 맥락도 같은 순서로. 받은 쪽 실행 요청은 미러가 실패해도
  성공 + 경고(실행을 받아들였으므로 — 실패로 돌려주면 다시 보내 두 번 실행). E4 4 → 6건.
- 개정 2(2026-10-11, 적대적 평가 라운드 2 반영): 실행 대기열은 커서(가장 큰 id) 없이 매번 처음부터 — 작업이 꺼낸 것의 집합만 건너뛴다
  (`chat::pending`, 낮은 id 가 늦게 `queued` 가 돼도 실행). 원격 `lc` 는 1 ~ `MAX_LC`(2^40)만 받고 그 밖은 무시. E4 6 → 7건.
