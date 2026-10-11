# an_core

## 개요

`an-core`(`crates/core`) — base Layer. 데이터 경로 · SQLite 열기 · 파일 로그 회전 · 단일 인스턴스 잠금 · 핸드오프 정보.
OS UI · HTTP · 암호를 모른다(SPKI 는 BLOB 으로만).

## 공개 계약

- `paths::Paths` — `legacy_root()` · `adopt_legacy(legacy)`(개명 전 이름 `LEGACY_APP_DIR` 의 데이터를 첫 실행 때 이어받음 — 옛 `instance.lock` 을 잡고 항목별로 옮김 · 옛 DB · 로그는 새 이름, 옛 앱 실행 중 · 두 루트 모두 데이터면 오류로 시작 멈춤, 다시 불러도 안전) · `default_root()`(Windows `%LOCALAPPDATA%\coworkAgent` · macOS `~/Library/Application Support/coworkAgent`) ·
  `under(root)` · `db_file` · `log_dir` · `log_file` · `lock_file` · `instance_file` · `identity_file` · `ensure()`. 경로 API 는 루트를 인자로
  받는다 — 환경변수 스위치 없음.
- `db::open(path)` — 스키마를 `SCHEMA_VERSION`(7)으로 올린다(→ 2 `peers` 표, → 3 `settings` 표, → 4 `agents` · `conversations` · `messages` 표, → 5
  `messages` 에 `reply_to` · `turns` · `duration_ms` — 한 트랜잭션, 있는 열은 건너뜀). 더 새 스키마면 오류. 올리기 전 `{db}.v{버전}.bak`
  사본(`db::backup_path`, `VACUUM INTO`) — 이전 실행 파일로 롤백할 때 DB 를 이 사본으로 바꿔 쓴다. → 6 `messages` 에 `mine` ·
  `exec` · `lc`(이전 이력은 방향 · id 로 채움). → 7 `conversations` 에 `relay_agent` · `relay_cap` · `relay_count` · `relay_on` ·
  `relay_blocked`(자동 왕복 D24). `busy_timeout` 5초(여러 연결이 같은 파일). `db::user_version(conn)`.
  `db::Connection` · `db::Error` 재노출.
- `peers` — `Peer { id, spki, name, host, addr, my_ok, their_ok, round }` · `paired()`. `upsert(…, round)`(있으면 승인 초기화) ·
  `by_spki` · `by_id` · `list` · `count_unapproved` · `set_my_ok` · `set_their_ok(spki, round)`(같은 세대 · 실제 전이만 true) ·
  `remove` · `remove_spki`.
- `settings::get(conn, key)` · `set(conn, key, value)` — 문자열 설정.
- `agents::READ_ONLY_TOOLS`(`Read` · `Grep` · `Glob` · `LS` · `NotebookRead`, D31) · `read_only(tools)`(목록 안 것만, 비면 기본) ·
  `unknown_tools(tools)`(목록 밖 — 등록 거부용).
- `agents` — `Agent { id, name, folder, tools, prompt }` · `DEFAULT_TOOLS`(`Read,Grep,Glob`) · `add(…, Option<tools>, prompt)` · `by_name` ·
  `list` · `remove`.
- `chat` — `Dir{Out,In}` · `Author{Human,Agent}` · `State{Sent,Queued,Running,Done,Error,Undelivered}` · `Conversation` · `Message`.
  `open_conversation(uuid, peer_id, agent, dir)`(다른 상대의 같은 uuid 는 오류) · `find(uuid, peer_id)` · `by_id` · `list` · `set_session` ·
  `add_message` · `add_reply(…, reply_to, turns, duration_ms)`(에이전트 답) · `set_state` · `messages` · `message_by_uuid` ·
  `pending(conv)`(DB 대기열 — 커서 없이 매번 처음부터) · `pending_conversations` · `summaries`(`Summary{conversation, title(첫 사람 메시지 앞 40자),
  last_state, last_at}`, 최근 것 먼저). `Message` 에 `created_at` · `reply_to` · `turns` · `duration_ms` · `mine`(이 PC 의 글) · `exec`(실행 요청) ·
  `lc`(양쪽 공통 순서 — 램포트 시계, `messages` 는 `(lc, uuid)` 순). `add_human(…, mine, exec)` · `observe(id, lc)`(받은 글의 순서 — 1 ~ `MAX_LC` 만, 밖이면 무시) ·
  `last_exec_before(conv, &msg)` · `notes_between(conv, after, &before)`(실행 맥락) · 대기열은 실행 요청만(작성자 무관 — 에이전트발 포함).
  자동 왕복: `Conversation.relay: Option<Relay{agent, cap, count, on}>` · `relay_blocked`, `set_relay(id, agent, cap)` ·
  `relay_on(id, on, reset)` · `relay_step(id) -> count` · `set_blocked(id, bool)` · `add_agent_request`(에이전트발 실행 요청) ·
  `set_flags(id, mine, exec, state)` · `mark_relayed(id, agent)`(받은 쪽: 자동 왕복 대화 · 상대 에이전트 이름 — `relay` 의 cap · on 은 없음).
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
