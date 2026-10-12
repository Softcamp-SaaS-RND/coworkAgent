# plan — 18.focus-mention

> 근거: [intent](intent.md)(승인 2026-10-12 — 라운드 3, 위임). **구현자는 이 문서만 보고 일한다.** D·S 는 [masterPlan](../../masterPlan.md).
> 기준 커밋: `0fba45b`(main — Merge Task #17, `task/18-focus-mention` 분기점). plan 승인: 2026-10-12 — 위임(masterPlan "자동 진행 위임" · 진행 목표 원문
> "Goal set: task 15,17,18 까지 완료. 사용자 테스트는 task 18 완료 후 일괄 검증.").

## 목표

상대 PC 에게 보이는 동작 셋을 더한다 — **집중**(사용자가 켜면 상대의 새 실행 요청을 즉시 거부 · 알림 멈춤 · 상대에게 상태가 보임) · **질의 · 요청 구분**
(메시지마다 종류, 받은 쪽만 바꾸는 완료 표시) · **@멘션**(내가 연 대화에서 상대 PC 의 다른 에이전트를 불러 그 에이전트 맥락으로 답). 접속 상태(온라인)도
집중과 같은 `Ping` · `Pong` 으로 잰다.

### 성공 기준(intent 사본)

- A1: 상단 바 · 프로필 메뉴에서 상태를 받기 가능 ↔ 집중으로 바꾸고, 값은 이 PC 에 저장돼 다시 실행해도 유지된다.
- A2: 집중 동안 상대 PC 의 새 요청은 에이전트를 실행하지 않고 새 오류 코드로 거부된다. 집중을 끄면 다시 받는다.
- A3: 거부된 요청을 보낸 쪽 글은 실패 상태와 "상대가 집중 중" 사유로 보이고, 다시 보내기로 재시도할 수 있다.
- A4: 집중을 켜면 진행 중이던 자동 왕복이 다음 차례에서 멈춘다(재개는 지금 방식).
- A5: 상대의 집중 상태가 레일 타일과 대화 머리글에 보인다.
- A6: 입력창에 @ 를 치면 대화 상대 PC 의 에이전트가 후보로 뜨고, 고른 에이전트가 그 글을 실행해 답하며 답에 실행한 에이전트 이름이 보인다.
- A7: 한 대화에서 같은 에이전트를 다시 부르면 그 에이전트의 이전 맥락으로 답한다(대화 × 에이전트 세션).
- A8: 15 · 17 에서 정한 결정이 유지된다(기존 테스트 green).
- A9: 입력창에서 메시지마다 질의 · 요청(건의)을 사용자가 고르고(기본 질의), 고른 종류가 양쪽 이력에 남아 보낸 · 받은 메시지 목록을 질의 · 요청으로
  거를 수 있으며, 요청을 그 대화 안과 '내게 요청된 목록' 같은 메뉴에서 모아 볼 수 있다.
- A10: 받은 쪽만 받은 요청에 완료 표시를 할 수 있다(보낸 쪽 표시 · '확인' 단계는 미해소 11 결론대로).

### intent 결정(사본)

- 집중 = 들어오는 요청까지 받지 않음 · 즉시 거부 + 표시(새 오류 코드 · 보낸 쪽 "상대가 집중 중" · 다시 보내기 · 진행 중 자동 왕복 멈춤 · 상대 타일
  표시) · 이 PC 저장. @멘션 = 상대 PC 에이전트만. 질의 · 요청 = 메시지 단위(기본 질의) · 사용자가 고름 · 완료는 받은 쪽만. 호환성 없음(D32).
- 라운드 3(위임): 거부 대상 `Request::Message` 만 · 코드 `focus` · 받은 쪽 기록 없음 · 집중 = 거부 + 알림 일시 중지 · `Pong` 에 집중 · 멘션 없는 글 =
  대화 에이전트 · 자동 왕복 대화 멘션 없음 · CLI 같은 규칙 · 온라인 = 같은 주기 Ping · 실행은 종류 무관 · 완료는 상태로 쓴 쪽에(완료 하나) · 요청
  모아 보기 = 내 활동 거르기 + 대화 안 · 입력창 = 두 값 고르기(라디오).
- 제외: 제3 PC · 내 PC 에이전트 멘션 · 집중 중 요청 큐 · 오프라인 보관 재전송(D7) · 화면 배치(17) · macOS 실동작(D25) · 수동 gate 실행(D28).

## intent 대비 해석 (plan 승인 대상)

코드 실측(기준 커밋)으로 결정의 전제를 확인한 결과 — 결정이 정하지 않은 자리를 정한다.

1. **멘션은 내가 연 대화(보낸 대화 `out`, 자동 왕복 아님)에서만.** 후보 = 그 상대 PC 의 에이전트 목록(`GET /api/peers/{id}/agents`). 받은 대화(`in`)의
   대화 에이전트는 내 것이라 '상대 PC 에이전트' 를 부르면 상대가 연 대화에서 상대 에이전트가 도는 새 실행 경로가 된다 — 받은 대화에는 후보를 띄우지
   않는다(intent "같은 상대의 다른 에이전트에게 물으려면 새 대화를 열어야 한다" 의 경로만).
2. **멘션 대상 = 글 안 첫 `@이름`** — 그 상대 PC 에이전트 이름과 글자 그대로 같고 앞이 글 머리 · 공백, 뒤가 글 끝 · 공백 · 문장 부호인 것(긴 이름 먼저
   — 한 이름이 다른 이름의 앞부분이어도).
   후보를 고르면 입력 중인 `@…` 를 `@이름 ` 으로 바꾼다. 손으로 친 `@이름` 도 같다. 한 글 = 대상 하나(나머지는 글자 그대로). 대상이 대화 에이전트면
   멘션이 아니다. 글 본문은 고치지 않고 보낸다(`@이름` 이 실행 입력에 남는다).
3. **세션** — 대화 에이전트는 지금 그대로 `conversations.session`, 멘션한 에이전트는 새 표 `agent_sessions(conversation_id, agent, session)`.
   폴더가 바뀌면 그 에이전트의 두 곳을 모두 지운다(`agents::update`, 15 R2 결정 그대로 — 실행 중 폴더가 바뀌면 남기지 않음). 에이전트를 지울 때도
   같다(`agents::remove` — 개정 4).
4. **멘션 기록** — `messages.agent`: 사람 실행 요청 = 부른 에이전트, 그 답 = 실행한 에이전트(NULL = 대화 에이전트). 양쪽이 같은 값을 남긴다(보낸 쪽은
   답을 받을 때 요청의 값을 그대로). 답 발화자 이름 = `{상대}/{m.agent ?? 대화 에이전트}`.
5. **실행 입력 메모**(`with_notes`)는 그대로 — 모든 새 글이 실행 요청이라(D23 개정) 메모가 생기지 않는다. 다른 에이전트의 주고받음은 맥락에 넣지 않는다
   (A7 = 그 에이전트 세션).
6. **집중 설정** — `settings` 키 `focus`(`1`/`0`, 기본 꺼짐), `GET/PUT /api/settings` 의 `focus`. 거부는 받는 순간 설정을 읽어 판정(재시작 불필요).
7. **거부 위치** — `inbound_message` 의 형식 검사 바로 뒤(에이전트 · 대화 조회 · 저장 · 알림보다 앞). 자동 왕복 재개(같은 uuid 승격)도 `Message` 라 거부된다.
   이미 받은 같은 글의 재전송(같은 uuid · 승격 아님)은 새 요청이 아니라 거부하지 않는다(개정 2 — 판정용 uuid 조회만 앞선다).
8. **집중과 자동 왕복** — 받은 쪽: 상대의 다음 차례(`Message` by_agent)가 `focus` 로 거부 → 연 쪽이 "상대가 집중 중" 으로 끝(`relay_end`). 연 쪽:
   `inbound_reply` 가 집중이면 이 답을 내 에이전트에 넘기지 않고 끈다(끝 사유 "집중 중"). 이미 대기열에 든 차례는 끝까지 돈다(다음 차례에서 멈춤 — A4).
9. **거절 사유 기록** — 보낸 쪽 `messages.refused`(상대가 준 코드). 화면은 `focus` 면 "상대가 집중 중", 그 밖은 "상대가 거절({code})". 다시 보내기는
   지금 메시지 작업 그대로(실패한 내 실행 요청 — 17).
10. **접속 상태** — 30초마다 페어링된 상대에 `Ping`(앱 시작 직후 한 번), `Pong { focus }` 을 받으면 `peers.seen_at`(받은 시각) · `peers.focus` 를 남긴다.
    온라인 = 마지막 `Pong` 이 75초 안. 집중 표시는 온라인일 때만(오프라인이면 "오프라인"). 수동 "연결 확인"(`POST /api/peers/{id}/ping`)도 같은
    기록. 보낸 요청이 `focus` 로 거부되면 그 상대를 바로 접속 · 집중으로 남긴다(`seen_at` = 지금 — 거절도 접속 증거, 다음 Ping 을 기다리지 않게).
    `GET /api/peers` 는 `online` · `focus` 두 참거짓만 낸다(시각을 내면 30초마다 레일 서명이 바뀌어 다시 그린다 — 17 Stage 2 리뷰 M3).
11. **질의 · 요청 저장** — `messages.request`(1 = 요청, NULL · 0 = 질의), 사람 글에만. `Message` · `Post` 에 `request` 를 실어 상대 이력에도 남긴다.
12. **완료 표시** — `messages.done_at`. 바꾸는 쪽 = 그 글을 받은 쪽(사람 글 · 요청 · `mine = false`). `PATCH /api/messages/{id} {done}` 이 이 PC 에 쓰고
    새 프레임 `Request::Done { conv, msg, done }` 으로 쓴 쪽에 보낸다(쓴 쪽은 자기 요청 글에만 받는다). 전송 실패는 경고(이 PC 표시는 남고, 누를 때마다
    그때 값을 보내므로 상대 쪽은 다음에 바꿀 때 맞춰진다 — 개정 1). 내 글 · 질의 · 답에 `done` 은 400.
13. **거르기** — 받은 목록(내 활동) 종류에 "질의" · "요청"(상대 쪽 사람 글), 보낸 목록(대화 '보낸 메시지' 탭)에 종류 칸 "전체" · "질의" · "요청"
    (`GET /api/messages` 의 `ask`, 제한 전에 거름 — 15 R2 결정 그대로). '내게 요청된 목록' = 내 활동 종류 "요청". 대화 안 = 실행 상세 '요청' 절.
14. **입력창 종류** — 서식 버튼 줄에 라디오 두 개 "질의"(기본) · "요청", 글을 보내면 "질의" 로. 요청 글은 머리에 태그 "요청"(완료면 "요청 · 완료").
15. **집중 칩 · 메뉴** — 상단 바 칩(누르면 바뀜) "받기 가능" ↔ "집중", 프로필 메뉴 "집중 — 요청 받지 않음"(켜짐 표시). 집중을 켜도 알림 일시 중지
    설정 값은 바꾸지 않는다(집중이 알림을 막는다).

## 설계

### 저장(스키마 12, `an-core`)

```sql
ALTER TABLE messages ADD COLUMN request INTEGER;  -- 사람 글: 1 = 요청(건의), NULL · 0 = 질의
ALTER TABLE messages ADD COLUMN done_at INTEGER;  -- 요청 완료 표시 시각(받은 쪽이 바꾸고 쓴 쪽은 받은 대로), NULL = 아님
ALTER TABLE messages ADD COLUMN agent TEXT;       -- 멘션: 사람 실행 요청이 부른 · 답을 실행한 에이전트, NULL = 대화 에이전트
ALTER TABLE messages ADD COLUMN refused TEXT;     -- 보낸 쪽: 상대가 거절한 코드(`focus` …)
ALTER TABLE peers    ADD COLUMN seen_at INTEGER;  -- 마지막 Pong 받은 시각
ALTER TABLE peers    ADD COLUMN focus INTEGER;    -- 마지막 Pong 의 상대 집중
ALTER TABLE agents   ADD COLUMN uid TEXT;         -- 에이전트 세대(개정 5): 등록 · 폴더 변경 때마다 새 값, 이전 행은 채움
CREATE TABLE IF NOT EXISTS agent_sessions (
    conversation_id INTEGER NOT NULL, agent TEXT NOT NULL, session TEXT NOT NULL,
    PRIMARY KEY (conversation_id, agent)
);
```

- 기존 규칙: `if v < 12` 한 트랜잭션 · 있는 열 건너뛰기 · `{db}.v11.bak` · `SCHEMA_VERSION = 12`. `core/tests/paths.rs` 의 "지금 버전" 리터럴은 12 로
  (`schema11_workspace` 의 `user_version == 11` 은 `db::SCHEMA_VERSION` 으로 — 이름은 그대로, E13). 스키마는 Stage 1 에서 한 번에 올리고 `agents::update`
  의 `agent_sessions` 지우기도 표를 만드는 Stage 1 에서.
- `an_core::settings` — `FOCUS_KEY = "focus"` · `focus(conn) -> Result<bool>`(값 `1` 만 참, 읽기 오류는 그대로).
- `an_core::peers` — `Peer.seen_at` · `Peer.focus` · `set_presence(id, focus)`(받은 시각 = 지금 — `Pong` · 집중 거절 둘 다).
- `an_core::chat` — `Message.request` · `done_at` · `agent` · `refused`. `add_message` · `add_human` 뒤 `set_request(id, bool)` · `set_agent(id, Option<&str>)` ·
  `set_refused(id, code)` · `set_done(id, bool)`(이미 완료면 처음 시각을 둔다 — 개정 1). `agent_session(conv, agent)` · `set_agent_session(conv, agent, session)`.
  `activity` 거르기 `Query` · `Request`(상대 쪽 사람 글) · `messages_across(…, request: Option<bool>, …)`(사람 글만 — 개정 1).
- `an_core::agents::update` — 폴더가 바뀌면 `agent_sessions` 에서 그 에이전트 행도 지운다.

### 피어 프레임(`an-net`)

- `Request::Message` · `Request::Post` 에 `#[serde(default)] request: bool`. 새 `Request::Done { conv, msg, done: bool }`(페어링된 상대만 — 처리기로).
  `Request::Message` 에 `#[serde(default)] mention: Option<String>` — `agent` 는 늘 대화 에이전트(개정 4).
- `Response::Pong { focus: bool }` · 오류 코드 `focus`(문서 주석 목록에 추가).
- `Node::answer` 의 `Ping` → `Pong { focus: settings::focus(db)? }`. `Node::ping(id) -> io::Result<bool>`(상대 집중) — 받으면 `peers::set_presence(id, 지금, focus)`.
- `Node::watch_presence(self: Arc<Self>)` — 시작 직후와 `PRESENCE_EVERY`(30초)마다 페어링된 상대 각각에 `ping`(상대별 작업, 실패는 기록 안 함). `main.rs` 가 띄운다.

### 실행(`an-agent`)

- 집중: `inbound_message` 가 형식 검사 뒤 `settings::focus` 면 `Response::error("focus")`(기록 · 알림 없음 — 판정 값을 먼저 받고 `if`, 조건식의 DB
  잠금이 블록 끝까지 사는 edition 2024 함정을 피한다). `inbound_reply` — 자동 왕복 켜짐 + 집중이면
  넘기지 않고 끔 · `relay_end` 사유 "집중 중".
- 보낸 쪽 거절: `send_human` · `relay_forward` 가 `Response::Error { code }` 를 받으면 `set_refused(msg, code)`(+ 지금의 `error` 상태). `focus` 면
  그 상대 `peers::set_presence(id, true)`(개정 1) · 오류 문구 "상대가 집중 중 — 요청을 받지 않습니다. 나중에 다시 보내세요" · 자동 왕복 끝 사유 "상대가 집중 중".
- 질의 · 요청: `Compose { via, title, request, mention }` — `send_with(peer, agent, conversation, text, Compose)` · `post_with(conversation, text, to_agent,
  Compose)`. 기존 `send` · `send_via` · `post` · `post_via` 는 그대로(기본 `Compose`). 보낼 때 `request` 를 싣고, 받은 쪽(`inbound_message` · `inbound_post`
  사람 글)이 남긴다.
- 완료: `Chat::mark_done(id, done, saved) -> io::Result<(Message, Option<String>)>`(경고 · `saved` 는 같은 트랜잭션 — 개정 3) · `inbound_done` — 그 상대 대화 · 그 uuid · 내 요청 사람 글에만.
- 멘션: 보낸 쪽 `send_human` 이 `Message.mention` 에 부른 이름(`agent` 는 대화 에이전트 그대로) · 글 `agent = mention`. 받은 쪽 `inbound_message` —
  `mention` 이 대화 에이전트가 아니면 멘션: `relay` · `by_agent` 와 함께면(대화를 만들기 전) · 자동 왕복 대화면 `bad_request`, 없는 에이전트면
  `unknown_agent`, 그 밖은 글 `agent` 를 남긴다. 대화가 아직 없으면(첫 글이 닿지 않음) 대화 에이전트로 연다(개정 4). `process` 대상 = `asked.agent ?? 대화 에이전트`, `execute` 세션 = 멘션이면 `agent_session` 아니면 `conversations.session`,
  `record` 가 같은 자리에 남기고 답 `agent` = 요청의 값. `inbound_reply` 는 답 `agent` = 요청의 값.

### 로컬 API(`an-app`)

- 설정 `focus`(bool) — `GET/PUT /api/settings`. `Prefs.focus` — `allows()` 는 `notify_paused || focus` 면 거짓.
- `GET /api/peers` 항목에 `online`(마지막 Pong 75초 안) · `focus`(마지막 Pong, 오프라인이면 거짓). `POST /api/peers/{id}/ping` 은 지금처럼 204(받은
  `Pong` 은 접속 상태로 남는다 — 15 `direct_add_with_code_pairs_both_sides` 의 204 단정 유지).
- `POST /api/conversations` · `POST /api/conversations/{id}/messages` 에 `request: bool`(기본 거짓), 이어 쓰기에 `agent`(멘션 — 보낸 대화 · 자동 왕복 아님 ·
  상대 PC 에이전트 이름, 대화 에이전트면 멘션 아님, 받은 대화 · 자동 왕복이면 400). 거절(`focus` 등)은 지금처럼 오류 + 글 `refused`.
- `PATCH /api/messages/{id}` — `saved` · `done` 중 하나 이상(둘 다 없으면 400). `done` 은 받은 요청만(그 밖 400) → 준 항목만 `{id, saved?, done?, warning?}`
  (둘 다면 한 트랜잭션 — 완료 자격이 없으면 아무것도 쓰지 않는다, 개정 3).
- `GET /api/messages` 에 `ask=query|request`(그 밖 400) · `GET /api/activity` 종류에 `query` · `request`.
- message_json 에 `request` · `done`(bool) · `agent` · `refused`.
- CLI `ask --conv N --agent X` — X 가 그 대화 에이전트가 아니면 멘션(`agent` 를 실어 보냄). 자동 왕복 · 받은 대화면 지금처럼 오류 문구.

### 웹UI(`web/`, 빌드 없는 ES 모듈 — `createElement` · `textContent` 만)

- **상단 바** — 상태 칩 `#focus_chip`(검색 칸과 + 사이) "받기 가능" | "집중", `aria-pressed`, 누르면 `PUT /api/settings {focus}`. 프로필 메뉴 `view.js`
  `profileMenu(paused, focus)` 에 "집중 — 요청 받지 않음"(켜짐 표시). 아바타 title 에 "집중" 을 더한다.
- **상대 상태** — `view.js` `presence(p)` → `{key: "online"|"focus"|"offline", text: "온라인"|"집중 중"|"오프라인"}`. 레일 타일 점(`.rail-dot.{key}`) ·
  aria-label 끝에 text, 사이드바 머리(고른 타일) · 대화 머리글 칩 `#conv_presence`(상대 PC 상태).
- **입력창** — 라디오 `name="ask"` "질의"(기본) · "요청"(묶음 라벨 "글 종류", 새 대화 칸에도 — 개정 3), 보내면 "질의" 로. 멘션: `view.js` `mentionQuery(text, caret)`(입력 중인
  `@…` 범위) · `mentionTarget(text, names)`(해석 2). 후보 목록 `#mention_box`(listbox, "에이전트 멘션") — ↑ ↓ 고르기 · Enter · Tab 넣기 · Esc 닫기, 열린
  동안 Enter 는 보내지 않는다. 힌트에 "@ 로 상대 에이전트 부르기"(보낸 대화 · 자동 왕복 아님일 때만 보임).
- **메시지** — 요청 태그 "요청" · "요청 · 완료"(`view.js` `askTag(m)`), 실패 사유 `view.js` `refusedText(code)`("상대가 집중 중" · "상대가 거절({code})").
  `messageActions(m)` 에 받은 요청 "완료 표시" | "완료 취소". 답 발화자 = 실행한 에이전트(`authorLabel` 이 `m.agent`).
- **실행 상세** — 절 "요청"(개수) — 이 대화의 요청 글(최신부터, 완료 표시), 누르면 그 글로.
- **내 활동** — 종류에 "질의" · "요청", 행 라벨에 요청 태그. **보낸 메시지 탭** — 종류 칸 `#sent_ask` "전체" · "질의" · "요청".
- **문구 literal**(`ux_literals` 가 index.html · app.js · view.js 에서 찾는다): "받기 가능" · "집중" · "집중 — 요청 받지 않음" · "온라인" · "집중 중" ·
  "오프라인" · "상대가 집중 중" · "글 종류" · "질의" · "요청" · "요청 · 완료" · "완료 표시" · "완료 취소" · "에이전트 멘션" · "@ 로 상대 에이전트 부르기".

## Stage

| stage | 목표 | 산출 | gate |
|---|---|---|---|
| 1 | 집중 · 접속 상태 — 스키마 12 · `focus` 설정 · 거부 · 거절 사유 · 자동 왕복 멈춤 · `Pong` · 주기 Ping · 알림 멈춤 · 칩 · 메뉴 · 타일 점 · 머리글 | 테스트(paths · pairing · roundtrip · relay · notifier · ux_api · view · ux_literals) | E3 ~ E11 |
| 2 | 질의 · 요청 · 완료 — 프레임 `request` · `Done` · API 거르기 · 입력창 종류 · 태그 · 완료 작업 · 실행 상세 '요청' · 내 활동 · 보낸 메시지 거르기 | 테스트(roundtrip · ux_api · view · ux_literals) | E3 · E4 · E6 · E8 |
| 3 | @멘션 — 글 `agent` · `agent_sessions` · 받은 쪽 수락 · 실행 세션 · 답 이름 · CLI · 입력창 후보 | 테스트(roundtrip · relay · ux_api · cli · view) | E3 · E6 · E8 · E9 · E12 |
| 공통 | 검증 명령 · 실화면 | — | E1 · E2 · E13 · E14 |

## 평가 항목

- [ ] E1: 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E2: 우리 코드 경고 0(clippy -D warnings)
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: pending
- [ ] E3: A1·A3·A5·A6·A9·A10 — 화면 로직 node 테스트 34건(기존 31 + presence · mention · ask_marks — 받은 요청 작업 · 내 활동 · 보낸 메시지 라벨은 ask_marks 에, 기존 profile_menu 보강)
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# pass 34\s+# fail 0/
  EVIDENCE: pending
- [ ] E4: A1·A3·A5·A6·A9·A10 — 화면 문구 literal(ux_literals 9건: 기존 8 + focus_mention_labels — 설계 "문구 literal" 목록 전부)
  CHECK: cargo test -p coworkagent-app --test ux_literals
  EXPECT: test result: ok. 9 passed; 0 failed
  EVIDENCE: pending
- [ ] E5: A1·A5 — 색 · 글꼴 크기 리터럴은 `app.css` 첫 `:root` 안에만이고, 다크 토큰이 Nocturne 값 · 글꼴 첫 이름이 Noto Sans KR
  CHECK: python -c "import re;c=open('crates/app/web/app.css',encoding='utf-8').read();j=open('crates/app/web/app.js',encoding='utf-8').read();m=re.search(r':root\s*\{[^}]*\}',c);r=m.group(0) if m else '';rest=c.replace(r,'');v=len(re.findall(r'#[0-9a-fA-F]{3,8}\b|rgba?\(',rest))+len(re.findall(r'font-size:\s*[0-9.]+(?:px|rem|em)',rest))+len(re.findall(r'#[0-9a-fA-F]{6}\b',j));need=['--dark-bg: #0E1117','--dark-surface: #171C28','--dark-border: #262D3D','--dark-text: #E8EBF2','--dark-accent: #7C83FF','--dark-agent: #3DD6B5','--dark-run: #F5B83D','--dark-ok: #3DD68C','--dark-danger: #FF6B6B','--font: '+chr(39)+'Noto Sans KR'+chr(39)];n=sum(x in r for x in need);print(f'root={int(bool(m))} violations={v} nocturne={n}')"
  EXPECT: root=1 violations=0 nocturne=10
  EVIDENCE: pending
- [ ] E6: A1·A3·A5·A6·A9·A10 — 로컬 API(ux_api 28건: 기존 24 + focus_setting_and_peer_presence · ask_kind_recorded_and_filtered · done_only_by_receiver · mention_api_rules)
  CHECK: cargo test -p coworkagent-app --test ux_api
  EXPECT: test result: ok. 28 passed; 0 failed
  EVIDENCE: pending
- [ ] E7: A1·A5·A6·A7·A9·A10 — 스키마 12 이전(paths 14건: 기존 13 + schema12_focus_mention — 새 열 · `agent_sessions` · `v11.bak`)
  CHECK: cargo test -p an-core --test paths
  EXPECT: test result: ok. 14 passed; 0 failed
  EVIDENCE: pending
- [ ] E8: A2·A3·A6·A7·A9·A10 — 두 PC 실행(roundtrip 16건: 기존 13 + focus_refuses_new_request_unrecorded · ask_kind_and_done_sync_both_sides · mention_runs_named_agent_own_session)
  CHECK: cargo test -p an-agent --test roundtrip
  EXPECT: test result: ok. 16 passed; 0 failed
  EVIDENCE: pending
- [ ] E9: A4·A6 — 자동 왕복(relay 12건: 기존 10 + focus_stops_relay_both_sides · mention_rejected_in_relay)
  CHECK: cargo test -p an-agent --test relay
  EXPECT: test result: ok. 12 passed; 0 failed
  EVIDENCE: pending
- [ ] E10: A1 — 집중 동안 알림이 나가지 않는다(notifier 12건: 기존 11 + focus_pauses_notices)
  CHECK: cargo test -p coworkagent-app --test notifier
  EXPECT: test result: ok. 12 passed; 0 failed
  EVIDENCE: pending
- [ ] E11: A5 — `Pong` 의 집중 · 접속 기록(pairing 6건: 기존 5 + pong_carries_focus_and_presence)
  CHECK: cargo test -p an-net --test pairing
  EXPECT: test result: ok. 6 passed; 0 failed
  EVIDENCE: pending
- [ ] E12: A6·A7 — CLI `ask --conv --agent` 멘션(cli 16건: 기존 15 + ask_mention_in_conversation)
  CHECK: cargo test -p coworkagent-app --test cli
  EXPECT: test result: ok. 16 passed; 0 failed
  EVIDENCE: pending
- [ ] E13: A8 — 15 · 17 결정을 지키는 테스트 55건이 이름 그대로 남아 있다(15 의 29 + 17 의 26)
  CHECK: python -c "fs={'crates/app/tests/ux_api.rs': ['favicon_served', 'peer_patch_group_model_effort', 'agent_patch_edits_but_not_name', 'settings_claude_defaults', 'inbox_lists_their_messages_with_search', 'outbox_lists_my_messages_to_peer', 'theme_setting_persists', 'to_human_rejected', 'conversation_title_set_and_edit', 'peer_tile_alias_icon', 'settings_me_tile_side_width', 'direct_add_with_code_pairs_both_sides', 'direct_readd_known_key_waits_for_approval', 'new_conversation_to_human_rejected', 'inbox_kind_filter_applies_before_limit', 'settings_read_error_is_500', 'search_groups_messages_titles_peers_agents', 'search_treats_like_wildcards_literally', 'activity_merges_theirs_failures_relay_events', 'activity_unseen_by_peer_and_mark_seen', 'star_toggles_local_only', 'saved_toggle_and_list_scoped', 'conversation_read_clears_unread', 'settings_notify_paused_roundtrip'], 'crates/app/tests/ux_literals.rs': ['turn_label_literal', 'invite_labels_without_messenger', 'favicon_link_present', 'title_search_label', 'tray_menu_open_quit_only', 'workspace_labels', 'no_received_label', 'no_timeline_tab'], 'crates/app/web/view.test.mjs': ['settings_tree', 'markdown_blocks_safe', 'html_doc_detect', 'group_filter_search', 'inbox_rows', 'peer_messages', 'direction_tag', 'tile_text', 'side_nav', 'new_menu', 'profile_menu', 'insert_markdown', 'enter_sends', 'message_actions', 'run_steps', 'search_groups', 'activity_rows', 'conv_sections'], 'crates/core/tests/paths.rs': ['schema11_workspace'], 'crates/agent/tests/roundtrip.rs': ['reply_records_model_effort_both_sides'], 'crates/agent/tests/relay.rs': ['relay_start_end_events_recorded'], 'crates/app/tests/notifier.rs': ['paused_drops_notices_and_tray', 'resume_after_pause_notifies_again']};n=sum(len(v) for v in fs.values());m=sum(((chr(34)+t+' ') if f.endswith('.mjs') else ('fn '+t+'(')) not in open(f,encoding='utf-8').read() for f,v in fs.items() for t in v);print(f'targets={n} missing={m}')"
  EXPECT: targets=55 missing=0
  EVIDENCE: pending
- [ ] E14: A1·A2·A3·A4·A5·A6·A7·A9·A10 — 실화면 · 두 PC(수동): 집중 칩 · 메뉴 · 거부 · "상대가 집중 중" · 다시 보내기 · 자동 왕복 멈춤 · 타일 점 · 머리글 · @ 후보 · 멘션 답 이름 · 같은 에이전트 맥락 · 질의 · 요청 · 거르기 · 완료 표시 양쪽
  EVIDENCE: pending

## 종결 시 할 일

- E14 는 D28 에 따라 `ABANDON`(사유: D28) + 19.prototype-acceptance 에 TC 추가(위 실화면 · 두 PC 항목).
- masterPlan D17 개정(사용자가 켠 집중 중에는 새 요청 거부) · D19 보강(대화 × 에이전트 세션 — 멘션) · S1 보강(질의 · 요청 · 완료 · @멘션 · 접속 상태) ·
  D32 적용 기록(`Pong` · `Done` · `request`).
- PRODUCT 도메인 규칙 "피어 프레임" · "웹UI 구조" · "스키마 올리기" 갱신, 매뉴얼 `an_core` · `an_net` · `an_agent` · `an_app`.

## 개정 기록

- 개정 1(2026-10-12, Stage 1 · 2 구현 — **설계 문구 정합 · 평가 항목 변경 없음 · 사용자 승인 — 위임**(masterPlan "자동 진행 위임")): 구현이 정한
  내부 이름 · 문구를 본문에 맞춘다. `peers::set_focus` 없이 `set_presence(id, focus)` 하나(거절도 접속 증거 — Stage 1) · `POST /api/peers/{id}/ping`
  은 204 그대로(Stage 1, 15 결정) · `chat::set_done(id, bool)`(이미 완료면 처음 시각을 둔다) · `messages_across` 의 종류 인자 이름 `request` ·
  완료 알림 실패 경고 "… 상대 쪽은 다음에 바꿀 때 맞춰진다"(단추가 표시 ↔ 취소로 바뀌므로 "다시 누르면 다시 보낸다" 는 취소를 부르는 안내가 된다).
- 개정 2(2026-10-12, Stage 1 리뷰 — **설계 보강 · 평가 항목 변경 없음 · 사용자 승인 — 위임**(masterPlan "자동 진행 위임")): 해석 7 — 이미 받은 글의
  재전송은 집중이어도 거부하지 않는다(연 쪽이 그 글을 실패로 확정해 늦게 온 답을 버리던 어긋남, 리뷰 L3). 그 밖 리뷰 고침은 결정 안의 정확성 —
  다시 보내면 거절 사유를 지운다 · 자동 왕복 시작이 거절되면 켜짐을 거둔다 · 상대가 준 거절 코드는 `[a-z_]` 1 ~ 32 자만(그 밖 `unknown`) · 온라인
  판정은 `Peer::online(now)`(`ONLINE_WITHIN` 은 `an_core::peers`, 받은 시각이 지금보다 뒤면 오프라인) · `Ping` 답의 설정 읽기 오류는 꺼짐으로 답한다 ·
  놓친 Ping 주기는 몰아 보내지 않는다.
- 개정 3(2026-10-12, Stage 2 리뷰 — **설계 문구 정합 · 평가 항목 수 변경 없음 · 사용자 승인 — 위임**(masterPlan "자동 진행 위임")): `PATCH
  /api/messages/{id}` 응답은 준 항목만(`done?`), `saved` · `done` 을 함께 주면 한 트랜잭션(`mark_done(id, done, saved)` — 완료 자격이 없으면 아무것도
  쓰지 않는다) · 완료 쓰기와 알림은 대화 차례 안에서(두 창의 반대 값 경합) · 새 대화 칸에도 글 종류 라디오(Stage 2 구현이 넣은 것을 본문에) · E3
  제목 문구(받은 요청 작업 · 라벨 단정은 새 `ask_marks` 에 — CHECK · EXPECT 그대로).
- 개정 4(2026-10-12, Stage 3 리뷰 — **설계 변경 · 평가 항목 변경 없음 · 사용자 승인 — 위임**(masterPlan "자동 진행 위임")): 멘션을 `Message.agent`
  대신 새 필드 `Message.mention` 으로 싣는다 — 첫 글이 받은 쪽에 닿지 않은 대화(집중 · 없는 에이전트 거절 · 전송 오류)에 멘션을 쓰면 받은 쪽이 부른
  에이전트를 대화 에이전트로 삼아 대화를 열어 양쪽 기록이 어긋났다(리뷰 1, 재현). `agent` 는 늘 대화 에이전트라 받은 쪽은 대화를 그 에이전트로 연다
  (D32 — 이전 버전 대비 없음). 에이전트를 지울 때도 그 에이전트 세션(받은 대화 · 자동 왕복 · 멘션)을 비운다(`agents::remove` — 같은 이름을 다른
  폴더로 다시 등록하면 옛 세션을 잇던 구멍, #15 이전부터).
- 개정 5(2026-10-12, 적대적 평가 라운드 1 — **설계 보강 · 평가 항목 변경 없음 · 사용자 승인 — 위임**(masterPlan "자동 진행 위임")): 해석 2 보강 —
  멘션을 쓰는 대화(내가 연 대화 · 자동 왕복 아님)에서 후보 목록(상대 PC 에이전트)을 받지 못했으면(받는 중 · 실패) 멘션처럼 보이는 글(글 머리 ·
  공백 뒤 `@이름`)은 보내지 않고 목록을 다시 묻는다(대화 에이전트로 조용히 가지 않게 — R1 high). 에이전트 세대 `agents.uid`(스키마 12 안 —
  D32): 등록 · 폴더 변경 때 새 값, 끝난 실행은 지금 세대가 실행 때와 같을 때만 세션을 남긴다(실행 중 지우고 같은 이름 · 같은 폴더로 다시 등록해도
  옛 세션을 되살리지 않게 — R1 high). 이 Task 개발 중 빌드로 12 가 된 DB 는 지우고 다시 만든다(D32).
- 개정 6(2026-10-12, 적대적 평가 라운드 2 — **설계 보강 · 평가 항목 변경 없음 · 사용자 승인 — 위임**(masterPlan "자동 진행 위임")): 해석 2 보강 —
  멘션 꼴 `@이름` 이 있는데 후보와 하나도 맞지 않으면(오타 · 낡은 목록) 한 번 막고 알린다("보내지 않음 — {@이름}: 상대 에이전트 목록에 없는
  이름입니다(목록을 다시 받습니다). 멘션이 아니면 한 번 더 보내세요") — 목록을 다시 받고, 같은 글을 다시 보내면 멘션 없이 보낸다(코드의
  `@Override` 같은 글도 보낼 수 있게). 대화 에이전트로 조용히 가지 않는다(R2 high).
