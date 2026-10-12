# stage1 — 스키마 11 · 피어 프레임 model/effort · 자동 왕복 기록 · 로컬 API

> 근거: [plan](../plan.md) Stage 1. 사용자 검증은 진행 목표대로 18 종결 뒤 일괄(plan 개정 1).

## 목표

화면(Stage 2 · 3)이 기댈 데이터 · 프로토콜 · API 를 먼저 만든다 — ★ · 읽음 · 저장 · 답의 실행 옵션 · 내 활동(자동 왕복 기록 · 안 본 수) ·
통합 검색 · 알림 일시 중지. 대상 gate: E6 ~ E10.

## 산출 파일

- `crates/core/src/db.rs` — `SCHEMA_VERSION` 11: `conversations.starred_at` · `read_id`, `messages.saved_at` · `model` · `effort`,
  `peers.activity_seen_at`, `relay_events` 표. 이전 채움 — `read_id` = 대화별 마지막 글 id(다 읽은 것), `activity_seen_at` = 올린 때(본 것).
  기존 규칙(한 트랜잭션 · 있는 열 건너뜀 · `{db}.v10.bak`).
- `crates/core/src/chat.rs` — `Message.saved_at` · `model` · `effort`, `Summary.starred` · `unread`(읽은 뒤 온 상대 쪽 글), `add_reply(…, model,
  effort)`, `messages_across(mine: Option<bool>, …)`(None = 양쪽), `set_starred` · `marks`(제목 · ★) · `mark_read` · `set_saved` · `saved` ·
  `RelayMark` · `add_relay_event` · `Activity` · `ActivityKind` · `ActivityFilter` · `activity` · `activity_unseen` · `mark_activity_seen` ·
  `search`(`Found` · `FoundAgent` · `SEARCH_OTHERS` 10). 열 목록 별칭 · `LIKE` 이스케이프는 `prefixed` · `like_pattern` 한 곳.
- `crates/net/src/proto.rs` — `Request::Reply` · `Request::Post` 에 `model` · `effort`(선택, D32 — 호환 장치 없음).
- `crates/agent/src/chat.rs` — `Ran`(실행 결과 · 실행한 에이전트 · 정한 model · effort): `execute` 가 돌려주고 `record` 가 답에 저장, 답
  (`Reply`) · 미러(`Post`)에 싣는다. 받은 쪽은 `run_meta`(`run_opts` 글자 규칙)를 지난 값만 저장. 자동 왕복 시작 · 끝 기록은 `notify` 안.
- `crates/app/src/chat_api.rs` — `GET /api/search` · `GET /api/activity` · `POST /api/activity/seen` · `PATCH /api/messages/{id}` · `GET /api/saved` ·
  `PATCH /api/conversations/{id}`(`title?` · `starred?`) · `POST /api/conversations/{id}/read`. 메시지 JSON 에 `model` · `effort` · `saved`, 요약
  JSON 에 `starred` · `unread`(필드 추가만 — CLI 가 읽는 필드 그대로).
- `crates/app/src/notify.rs` — `Prefs.notify_paused`(기본 꺼짐) · `SettingsPatch.notify_paused` · `Prefs::allows` 첫 줄에서 거른다.
- `crates/app/src/peers_api.rs` — `PeerView` 를 `pub(crate)`(검색 결과의 사람).
- 테스트 — `core/tests/paths.rs` `schema11_workspace`(+ 버전 리터럴 10 → 11 아홉 곳, "더 새 스키마" 11 → 12) · `core/tests/store.rs`
  `add_reply` 인자 · `agent/tests/roundtrip.rs` `reply_records_model_effort_both_sides` · `agent/tests/relay.rs` `relay_start_end_events_recorded` ·
  `app/tests/notifier.rs` `paused_drops_notices_and_tray` · `resume_after_pause_notifies_again` · `app/tests/ux_api.rs` 8건(plan E6 이름 그대로).
  피어 프레임 리터럴(roundtrip 3 · relay 1 · two_sided 1)에 `model` · `effort`.

## 설계 결정

- **자동 왕복 기록 자리 = `Chat::notify`** — plan 은 "알림 호출(`notify`) 바로 앞" 이고 괄호에 `start_relay` · `mark_relayed` · `relay_end` ·
  `inbound_relay` 를 들었다. 시작 · 끝 알림을 내는 호출부가 6곳이라 공통 함수 한 곳에서 `RelayStart` · `RelayEnd` 일 때 남긴다(알림 설정은 이
  크레이트 밖 — 설정과 무관). 그래서 연 쪽의 시작은 `start_relay` 를 누른 때가 아니라 **알림과 같은 때**(그 요청의 첫 답을 받을 때)다 — 버튼을
  누른 것은 내 활동이 아니라 내 행동. **plan 편차 1**(설계 문구, gate 무관). 사람이 이 PC 에서 멈춘 것은 남기지 않는다(알림과 같다).
  `notify` 를 부르는 6곳은 DB 잠금을 놓은 뒤라 안에서 잠가도 막히지 않는다 — 단 `relay_forward` 의 `if let Err(e) = chat::set_flags(&self.db(), …)`
  한 곳은 조건식의 잠금이 블록 끝까지 살아 있었다(edition 2024, 리뷰 H1 — 고침).
- **알림 일시 중지** = `Prefs::allows` 첫 줄 — 트레이 · SSE · 건수가 한 거르기를 지난다. `notify_inbound` 와 별개 키라 풀면 그 설정대로.
- **실패 항목** = 내 쪽 글 중 `state = 'error'`(plan 해석 12 그대로). 상대 에이전트의 실패 답은 상대 쪽 글로도 들어오므로 "전체" 에서 내 요청의
  실패와 두 줄이 될 수 있다 — 종류 거르기가 다르다(실패 · 에이전트 답).
- **안 본 수** = 상대별 `activity_seen_at` 과 초 단위 비교(`>`) — 본 순간과 같은 초에 온 것은 본 것으로 센다(기존 `created_at` 이 초 단위).
- **검색** — 메시지 = SQL `LIKE`(이스케이프, 양쪽 글, 최신 `limit` 건), 대화 = 화면에 보이는 요약 제목, 사람 = 이름 · 표시 이름 · 호스트,
  에이전트 = 내 것의 이름 · 폴더 + 내가 연 대화의 상대 에이전트 이름(`owner` = 상대 id). 영문 대소문자 무시, `%` · `_` · `\` 는 글자 그대로.
- **받은 model · effort** 는 신뢰하지 않는 입력 — `valid_model` · `valid_effort` 를 지난 것만 저장. 자동 왕복 상한의 마지막 내 답 `Post` 는 그 답에
  저장된 값을 싣는다.
- **PATCH 대화** — 제목 · ★ 를 한 트랜잭션, 둘 다 없으면 400, 모르는 항목은 serde 거절(4xx), 응답 `{id, title, starred}`.
- **`POST /api/activity/seen`** — 빈 몸(Content-Type 없음) · JSON `{}` = 전부, `{peer}` = 그 상대(없는 상대 404). 처음엔 `Json<…>` 이라 빈 몸을
  거절했다(리뷰 M3 — 고침).

## 테스트 결과

RED — 테스트를 먼저 써서 컴파일 실패를 확인했다(`cargo test --workspace --no-run`): `cannot find function activity_unseen / activity /
add_relay_event in module chat` · `cannot find ActivityFilter / ActivityKind / RelayMark` · `no field model / effort / saved_at on Message` ·
`no field starred / unread on Summary`.

GREEN(구현 뒤):

```text
$ cargo test -p an-core --test paths
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s
$ cargo test -p an-agent --test roundtrip
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.36s
$ cargo test -p an-agent --test relay
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.70s
$ cargo test -p coworkagent-app --test notifier
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.22s
$ cargo test -p coworkagent-app --test ux_api
test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.82s
$ cargo test --workspace            # 테스트 실행 파일 43개 전부 ok
$ cargo clippy --workspace --all-targets -- -D warnings   # 경고 0
```

## 리뷰

`465d05e` 를 서브에이전트가 읽기 전용으로 리뷰했다(범위 `291a8d1..465d05e`, gate 테스트 5종 · `cargo test --workspace` green 확인 포함). 지적마다 코드로
대조하고, 사실인 것은 테스트를 먼저 고쳐 실패를 본 뒤(RED) 고쳤다. 새 테스트 함수는 만들지 않았다 — 기존 테스트에 단언을 더해 gate 건수(E6 24 ·
E8 13 · E9 10)를 지킨다.

| # | 지적(요지) | 판단 | 조치 |
|---|---|---|---|
| H1 | `relay_forward` 의 `if let Err(e) = chat::set_flags(&self.db(), …)` — edition 2024 에서 조건식 임시값(DB 잠금)이 블록 끝까지 살아, 안의 `relay_on(&self.db())` · `relay_end → notify → add_relay_event(&self.db())` 가 같은 스레드에서 다시 잠근다(교착, `set_flags` 오류 때만). | 사실(`crates/agent/Cargo.toml` edition 2024 · 코드). 앞 재잠금은 기존, 뒤는 이 단계가 더했다. | 결과를 먼저 받는다(`let flagged = …; if let Err(e) = flagged`). 같은 꼴 다른 자리 없음(`if let`/`match` 조건의 `self.db()` grep). 재현 테스트 없음 — DB 오류를 주입할 수단이 없다. |
| M1 | 지운 상대의 글 · 기록은 열람 시각이 없어(`LEFT JOIN … coalesce(…, 0)`) 영영 안 본 것 — `unseen_all` 이 0 이 되지 않는다. 검색의 상대 에이전트도 지운 상대 id 를 낸다. | 사실(`peers::remove` 는 peers 행만 지운다, 대화 · 글은 남는다). | 안 본 판정 4곳에 `p.id IS NOT NULL AND`(이력은 남기고 본 것으로), 검색 에이전트는 남아 있는 상대만. 단언: `activity_unseen_by_peer_and_mark_seen` · `search_groups_messages_titles_peers_agents`(고치기 전 실패 확인). |
| M2 | 자동 왕복 상한의 마지막 답(`Post`, 받은 쪽 `"agent"` · `In` 갈래)이 model · effort 를 버린다. | 사실 — plan "받은 쪽은 `inbound_post` 에서 저장" 과 어긋남(턴 · 시간도 빠지는 기존 동작). | `chat::set_run`(턴 · 시간 · model · effort)을 같은 트랜잭션에서. 단언: `relay_start_end_events_recorded`(고치기 전 `(None, None, None, None)` 실패 확인). |
| M2-부 | 중간 왕복 `Request::Message{by_agent}` 에는 model · effort 가 없다. | 의도대로 — 받는 쪽에서 그 글은 *요청* 이고, 실행 상세는 그 요청의 답(받는 쪽 실행)의 값을 쓴다. | 없음. |
| M3 | `POST /api/activity/seen` 이 `Json<…>` 이라 빈 몸 거절 — plan "빈 몸 = 전부" 를 stage1 이 `{}` 로 조용히 바꿨다. | 사실. | `Option<Json<…>>` — axum 0.8.9 는 Content-Type 이 없으면 `None`(`axum-0.8.9/src/json.rs` `OptionalFromRequest`). 단언: Content-Type 없는 빈 몸 → 204(고치기 전 실패 확인). |
| L1 | 받은 쪽은 `Relay{on:true}` 마다 시작을 기록 — 자동 왕복이 아닌 대화 · 겹친 프레임도. | 사실이나 영향 작음 — 정상 상대는 자동 왕복 대화의 재개 때만 보낸다(`relay()` 가 그 밖은 거절), 알림도 같은 횟수로 이미 나간다. 권한 `c.relay.is_some()` 은 받은 쪽에서 늘 `None` 이라 맞지 않다. | 미조치. |
| L2 | 실패 항목의 시각 · 안 본 판정이 글 시각 — 열람 뒤에 실패로 바뀐 내 요청은 안 본 것으로 세지 않는다. | 추정 · 영향 작음 — 상대 쪽 실패 답은 새 글로 따로 들어온다. | 미조치. |
| L3 | `notify_paused` 를 켜도 쌓인 트레이 건수는 그대로. | 의도대로 — 일시 중지는 새 알림을 막는 것, 쌓인 표시는 웹UI 를 열면(`/api/seen`) 지워진다. 해석 9 의 "같은 경로" 는 이벤트를 버리는 자리다. | 없음. |
| L4 | 읽음 · 열람 기준이 서버의 지금 — 조회(GET)와 표시(POST) 사이에 온 글은 안 보고 읽음 · 본 것. | 사실 · plan 대로(해석 2 · 3) — 열린 화면은 5초 안에 다시 그린다. | 미조치. |
| T | 테스트 약점 — 같은 초라 순서 단언이 빈약, `run_meta` 무시험, 안 본 수의 기록 · 실패 쪽 무시험, 저장 시각 순서. | 사실. | 단언 추가: 내 활동 시각을 벌려 순서 · 합친 뒤 자르기(`?limit=2`), 안 본 수 = 글 · 기록 · 실패 3, 저장 시각 순(id 순과 반대), 틀린 model · effort 는 버림(roundtrip 원시 `Reply`). |
| N | 검색이 `summaries` 를 두 번 돈다 · LIKE(ASCII) 와 `to_lowercase`(유니코드)의 대소문자 차이 · `execute` 실패 답에 model · effort 없음. | 사실 · 영향 작음. | 미조치. |
| 파생 | (M1 확인 중 발견 — 이 단계 밖 기존 결함) `peers.id` 가 `INTEGER PRIMARY KEY`(AUTOINCREMENT 없음)라 가장 큰 id 의 상대를 지우고 새 상대를 이으면 그 id 를 다시 쓴다 — 지운 상대의 대화가 새 상대의 타일 · 목록에 붙는다. | 사실(측정 — SQLite 메모리 DB: A(1) · B(2) → B 삭제 → C 추가 = id 2). | 이 Task 범위 밖 — eval `## 미조치 지적` → 원장(종결 시). |

리뷰 반영 뒤:

```text
$ cargo test -p coworkagent-app --test ux_api
test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.73s
$ cargo test --no-fail-fast -p an-agent --test roundtrip --test relay
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.70s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.35s
```

## 다음 단계

Stage 2 — 웹UI 뼈대(토큰 · 5영역 · 상단 바 · 메뉴 · 사이드바 내비/섹션 · 경로 · 레일 배지 · '받은 글' → '내 활동' · 타임라인 탭 제거).
