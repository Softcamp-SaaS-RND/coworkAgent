# an_core

## 개요

`an-core`(`crates/core`) — base Layer. 데이터 경로 · SQLite 열기 · 파일 로그 회전 · 단일 인스턴스 잠금 · 핸드오프 정보.
OS UI · HTTP · 암호를 모른다(SPKI 는 BLOB 으로만).

## 공개 계약

- `paths::Paths` — `legacy_root()` · `adopt_legacy(legacy)`(개명 전 이름 `LEGACY_APP_DIR` 의 데이터를 첫 실행 때 이어받음 — 옛 `instance.lock` 을 잡고 항목별로 옮김 · 옛 DB · 로그는 새 이름, 옛 앱 실행 중 · 두 루트 모두 데이터면 오류로 시작 멈춤, 다시 불러도 안전) · `default_root()`(Windows `%LOCALAPPDATA%\coworkAgent` · macOS `~/Library/Application Support/coworkAgent`) ·
  `under(root)` · `db_file` · `log_dir` · `log_file` · `lock_file` · `instance_file` · `identity_file` · `ensure()`. 경로 API 는 루트를 인자로
  받는다 — 환경변수 스위치 없음.
- `db::open(path)` — 스키마를 `SCHEMA_VERSION`(12)으로 올린다(→ 2 `peers` 표, → 3 `settings` 표, → 4 `agents` · `conversations` · `messages` 표, → 5
  `messages` 에 `reply_to` · `turns` · `duration_ms` — 한 트랜잭션, 있는 열은 건너뜀). 더 새 스키마면 오류. 올리기 전 `{db}.v{버전}.bak`
  사본(`db::backup_path`, `VACUUM INTO`) — 이전 실행 파일로 롤백할 때 DB 를 이 사본으로 바꿔 쓴다. → 6 `messages` 에 `mine` ·
  `exec` · `lc`(이전 이력은 방향 · id 로 채움). → 7 `conversations` 에 `relay_agent` · `relay_cap` · `relay_count` · `relay_on` ·
  `relay_blocked`(자동 왕복 D24). → 8 `messages` 에 `via`(글의 출처 — `cli` · NULL). → 9 `peers` 에 `grp` · `model` · `effort`, `agents` 에 `grp`. → 10 `conversations` 에
  `title`, `peers` 에 `alias` · `icon`(이 PC 에만 — 상대에 보내지 않는다). → 11 `conversations` 에 `starred_at` · `read_id`, `messages` 에 `saved_at` ·
  `model` · `effort`, `peers` 에 `activity_seen_msg` · `activity_seen_event`(내 활동을 본 마지막 글 · 기록 id), `relay_events` 표(자동 왕복 시작 ·
  끝 + 내 글이 실패로 바뀐 순간 `failed` · `message_id`), 색인 `messages_by_conv` · `relay_events_by_conv` · `relay_events_by_message`(이전 대화는 다
  읽은 것 · 이전 활동은 본 것으로 채움). → 12 `messages` 에 `request` · `done_at` · `agent` · `refused`, `peers` 에 `seen_at` · `focus`,
  `agent_sessions` 표(대화 × 에이전트 세션 — 멘션). `busy_timeout` 5초(여러 연결이 같은 파일).
  `db::user_version(conn)`.
  `db::Connection` · `db::Error` 재노출.
- `peers` — `Peer { id, spki, name, host, addr, my_ok, their_ok, round, group, model, effort, alias, icon, seen_at, focus }` · `paired()` ·
  `online(now)`(마지막 `Pong` 이 `ONLINE_WITHIN` 75초 안 — 받은 시각이 지금보다 뒤면 오프라인) · `set_presence(id, focus)`(받은 시각 = 지금 — `Pong` ·
  집중 거절 둘 다). `upsert(…, round)`(있으면 승인 초기화) ·
  `by_spki` · `by_id` · `list` · `count_unapproved` · `set_my_ok` · `set_their_ok(spki, round)`(같은 세대 · 실제 전이만 true) ·
  `remove` · `remove_spki` · `set_meta(id, group, model, effort)` · `set_tile(id, alias, icon)`(통째로 바꿈, `None` = 지움 — 이 PC 만의 표시 ·
  실행 설정).
- `settings::get(conn, key)` · `set(conn, key, value)` — 문자열 설정. `FOCUS_KEY`(`focus`) · `focus(conn)`(값 `1` 만 켜짐 — 읽기 오류는 그대로).
- `agents::READ_ONLY_TOOLS`(`Read` · `Grep` · `Glob` · `LS` · `NotebookRead`, D31) · `read_only(tools)`(목록 안 것만, 비면 기본) ·
  `unknown_tools(tools)`(목록 밖 — 등록 거부용).
- `agents` — `Agent { id, name, folder, tools, prompt, group, uid }`(`uid` = 세대 — 등록 · 폴더 변경 때 새 값) · `DEFAULT_TOOLS`(`Read,Grep,Glob`) · `add(…, Option<tools>, prompt)` · `by_name` ·
  `by_id` · `list` · `remove` · `update(id, folder, tools, prompt, group)`(이름은 바꾸지 않는다 — 대화 · 상대 PC 가 이름으로 부른다. 폴더가 바뀌면
  그 에이전트가 이어 가던 세션(받은 대화 · 내가 연 자동 왕복 · 멘션 `agent_sessions`)을 비운다 — `forget_sessions(name)`, `remove` 도 같다 — Claude 세션은 폴더에 묶여 있다).
- `chat` — `Dir{Out,In}` · `Author{Human,Agent}` · `State{Sent,Queued,Running,Done,Error,Undelivered}` · `Conversation` · `Message`.
  `open_conversation(uuid, peer_id, agent, dir)`(다른 상대의 같은 uuid 는 오류) · `find(uuid, peer_id)` · `by_id` · `list` · `set_session` ·
  `add_message` · `add_reply(…, reply_to, turns, duration_ms)`(에이전트 답) · `set_state` · `messages` · `message_by_uuid` ·
  `pending(conv)`(DB 대기열 — 커서 없이 매번 처음부터) · `pending_conversations` · `summaries`(`Summary{conversation, title(이 PC 에서 붙인 제목,
  없으면 첫 사람 메시지 앞 40자), last_state, last_at}`, 최근 것 먼저) · `set_title(id, title)`(`None` = 지움) ·
  `messages_across(mine, peer, author, request, q, limit)`(대화를 가로지른 글 — 상대 · 종류 · 사람 글 종류(`request` — 사람 글만) · 본문 일치를 거른
  뒤 최신 `limit` 건). `Message` 에 `created_at` · `reply_to` · `turns` · `duration_ms` · `mine`(이 PC 의 글) · `exec`(실행 요청) ·
  `lc`(양쪽 공통 순서 — 램포트 시계, `messages` 는 `(lc, uuid)` 순) · `via`(`Some("cli")` = 사용자 AI 에이전트가 CLI 로 쓴 글, 동작은 사람 글과 같다). `set_via(id, via)`(`VIA_CLI` 만 남기고 그 밖 값은 NULL — 삽입과 같은 트랜잭션에서 부른다). `add_human(…, mine, exec)` · `observe(id, lc)`(받은 글의 순서 — 1 ~ `MAX_LC` 만, 밖이면 무시) ·
  `last_exec_before(conv, &msg)` · `notes_between(conv, after, &before)`(실행 맥락) · 대기열은 실행 요청만(작성자 무관 — 에이전트발 포함).
  작업 화면(이 PC 에만): `set_starred(id, on)` · `marks(id)`(제목 · ★) · `mark_read(id, upto)`(화면이 받은 마지막 글까지 — 뒤로 가지 않음, 그 대화
  글이 아니면 거짓) · `set_saved(msg, on)` · `saved(peer?, conversation?, limit)`(저장 순) · `add_relay_event(conv, RelayMark, detail)` ·
  `activity(peer?, ActivityFilter, q?, limit)`(상대 쪽 글 · 실패한 내 글 · 자동 왕복 기록, 최신순) · `activity_unseen(peer?)` · `activity_upto()` ·
  `mark_activity_seen(peer?, (message, event))`(받은 범위까지 — 뒤로 가지 않고 지금 마지막 항목을 넘지 않음) · `search(q, limit)`(메시지 · 대화 ·
  사람 · 에이전트 묶음) · `set_run(id, turns, duration_ms, model, effort)`. `set_state` · `set_flags` 는 내 글이 `error` 로 바뀌는 순간을 `relay_events`
  `failed` 로 남긴다 — 본 뒤 실패로 바뀐 글도 다시 안 본 것으로 센다(글 id 는 그대로라). `Summary` 에 `starred` · `unread`, `Message` 에 `saved_at` · `model` · `effort`.
  자동 왕복: `Conversation.relay: Option<Relay{agent, cap, count, on}>` · `relay_blocked`, `set_relay(id, agent, cap)` ·
  `relay_on(id, on, reset)` · `relay_step(id) -> count` · `set_blocked(id, bool)` · `add_agent_request`(에이전트발 실행 요청) ·
  `set_flags(id, mine, exec, state)` · `mark_relayed(id, agent)`(받은 쪽: 자동 왕복 대화 · 상대 에이전트 이름 — `relay` 의 cap · on 은 없음).
  #18: `Message` 에 `request`(요청 — 아니면 질의) · `done_at`(받은 쪽 완료 표시) · `agent`(멘션 — 부른 · 실행한 에이전트) · `refused`(보낸 쪽 —
  상대 거절 코드). `set_request(id, bool)` · `set_done(id, bool)`(이미 완료면 처음 시각) · `set_agent(id, Option)` · `set_refused(id, Option)`(다시
  보내면 `None`) · `message_by_id` · `agent_session(conv, agent)` · `set_agent_session(conv, agent, session)` · `ActivityFilter::Query` · `Request`
  (상대 쪽 사람 글 — '내게 요청된 목록').
- `run_opts` — 실행 옵션 · 화면 설정의 검증과 기본값 단일 지점. `DEFAULT_MODEL`(`sonnet`) · `DEFAULT_EFFORT`(`high`) · `EFFORTS` ·
  설정 키 `MODEL_KEY` · `EFFORT_KEY` · `THEME_KEY` · `SIDE_WIDTH_KEY` · `ME_ALIAS_KEY` · `ME_ICON_KEY` · `valid_model`(1 ~ 64자, `-` 로 시작 금지) ·
  `valid_effort` · `clean_group` · `clean_alias` · `clean_title` · `clean_icon`(PNG data URL) · `defaults` · `resolve(conn, peer)`(상대 덮어쓰기 →
  전역 → 기본값) · `theme` · `side_width` · `me_tile`. 읽기는 `rusqlite::Result` — 없거나 틀린 값만 기본값, DB 오류는 그대로 낸다.
- `log_file::FileLogger::install(path, level)` — `LOG_ROTATE_BYTES`(1 MiB)를 넘게 되는 기록 전에 `.1` 로 회전(1개 보관).
- `instance::InstanceLock::acquire(path)` → `Acquired::Primary(lock)` | `Secondary`. `InstanceInfo { port, token }` 의
  `write` · `read`(hex 토큰만 받음).

## 사용 예

```rust
let paths = Paths::under(Paths::default_root().unwrap());
paths.ensure()?;
FileLogger::install(&paths.log_file(), log::LevelFilter::Info)?;
match InstanceLock::acquire(&paths.lock_file())? {
    Acquired::Primary(lock) => { /* 서버 시작, lock 은 종료 직전까지 보관 */ }
    Acquired::Secondary => { /* instance.json 으로 핸드오프 */ }
}
```

## 금지 사항

- 데이터 루트 밖(실행 파일 폴더 · 현재 폴더)에 쓰지 않는다.
