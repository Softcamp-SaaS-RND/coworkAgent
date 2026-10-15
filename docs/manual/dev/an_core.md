# an_core

## 개요

`an-core`(`crates/core`) — base Layer. 데이터 경로 · SQLite 열기 · 파일 로그 회전 · 단일 인스턴스 잠금 · 핸드오프 정보 · 보내기 보관(outbox) ·
CLI 트랜잭션 표.
OS UI · HTTP · 암호를 모른다(SPKI 는 BLOB 으로만).

## 공개 계약

- `paths::Paths` — `legacy_root()` · `adopt_legacy(legacy)`(개명 전 이름 `LEGACY_APP_DIR` 의 데이터를 첫 실행 때 이어받음 — 옛 `instance.lock` 을 잡고 항목별로 옮김 · 옛 DB · 로그는 새 이름, 옛 앱 실행 중 · 두 루트 모두 데이터면 오류로 시작 멈춤, 다시 불러도 안전) · `default_root()`(Windows `%LOCALAPPDATA%\coworkAgent` · macOS `~/Library/Application Support/coworkAgent`) ·
  `under(root)` · `db_file` · `log_dir` · `log_file` · `lock_file` · `instance_file` · `identity_file` · `ensure()` · `restrict()`(#1a — 루트를 소유자
  전용으로, 주 인스턴스가 잠금 뒤 매 시작). 경로 API 는 루트를 인자로 받는다 — 환경변수 스위치 없음. `paths::protected_roots()` ·
  `agent_folder_problem(folder) -> Option<이유>`(데이터 루트를 품음 · 그 안 · 네트워크 경로 — 등록 · 수정 · 실행 직전) · `folder_blocked(folder, roots)`.
- `owner_only::restrict(dir)`(#1a) — Windows 보호 DACL(토큰 사용자 SID · SYSTEM, `FILE_ALL_ACCESS`, 하위 상속) · Unix 0700.
- `sync::lock(&Mutex<T>)`(#1a) — 오염된 잠금도 안의 값을 꺼내 계속 쓴다(warn). 원시 `.lock().expect(` 는 쓰지 않는다.
- `db::open(path)` — 스키마를 `SCHEMA_VERSION`(16)으로 올린다(→ 2 `peers` 표, → 3 `settings` 표, → 4 `agents` · `conversations` · `messages` 표, → 5
  `messages` 에 `reply_to` · `turns` · `duration_ms` — 한 트랜잭션, 있는 열은 건너뜀). 더 새 스키마면 오류. 올리기 전 `{db}.v{버전}.bak`
  사본(`db::backup_path`, `VACUUM INTO`) — 이전 실행 파일로 롤백할 때 DB 를 이 사본으로 바꿔 쓴다. → 6 `messages` 에 `mine` ·
  `exec` · `lc`(이전 이력은 방향 · id 로 채움). → 7 `conversations` 에 `relay_agent` · `relay_cap` · `relay_count` · `relay_on` ·
  `relay_blocked`(자동 왕복 D24). → 8 `messages` 에 `via`(글의 출처 — `cli` · NULL). → 9 `peers` 에 `grp` · `model` · `effort`, `agents` 에 `grp`. → 10 `conversations` 에
  `title`, `peers` 에 `alias` · `icon`(이 PC 에만 — 상대에 보내지 않는다). → 11 `conversations` 에 `starred_at` · `read_id`, `messages` 에 `saved_at` ·
  `model` · `effort`, `peers` 에 `activity_seen_msg` · `activity_seen_event`(내 활동을 본 마지막 글 · 기록 id), `relay_events` 표(자동 왕복 시작 ·
  끝 + 내 글이 실패로 바뀐 순간 `failed` · `message_id`), 색인 `messages_by_conv` · `relay_events_by_conv` · `relay_events_by_message`(이전 대화는 다
  읽은 것 · 이전 활동은 본 것으로 채움). → 12 `messages` 에 `request` · `done_at` · `agent` · `refused`, `peers` 에 `seen_at` · `focus`,
  `agent_sessions` 표(대화 × 에이전트 세션 — 멘션). → 13 `outbox` 표(보내기 보관) · 색인 `outbox_peer` · `cli_tx` 표(CLI 트랜잭션 — #1c,
  `CREATE TABLE IF NOT EXISTS` 묶음). → 14 `messages` 에 `req_state` · `req_note` · `req_at`(#1d — 요청 글은 `done_at` 이 있으면 `done`, 없으면
  `wait`), 목록 밖 `settings.claude_model` 삭제 · `peers.model` → `NULL`, 못 보낸 옛 완료 알림(outbox `kind = 'done'`)은 같은 자리 · 순서로 `ReqState` 프레임(완료 → `done` ·
  취소 → `wait`)으로 바꾸고 그 요청 글에 잇는다(`done_at` 열은 남기되 읽지 않는다). → 15 `messages.edited_at`(#1f — 편집 시각, 한 트랜잭션).
  `busy_timeout` 5초(여러 연결이 같은 파일).
  `db::user_version(conn)` · `db::write_tx(conn)`(쓰기 트랜잭션 `BEGIN IMMEDIATE` — 읽고 쓰는 지연 트랜잭션이 다른 연결의 쓰기와 겹쳐 기다리지 않고
  `database is locked` 로 실패하지 않게, 앱의 쓰기 트랜잭션은 모두 이것).
  `db::Connection` · `db::Error` 재노출.
- `peers` — `Peer { id, spki, name, host, addr, my_ok, their_ok, round, group, model, effort, alias, icon, seen_at, focus }` · `paired()` ·
  `online(now)`(마지막 통신 `seen_at` 이 `ONLINE_WITHIN` 75초 안 — 받은 시각이 지금보다 뒤면 오프라인) · `set_presence(id, focus)`(시각 = 지금 +
  집중 — `Pong` · 집중 거절 둘 다) · `touch(conn, spki)`(#1c — 통신 성공: 그 키의 `seen_at` = 지금, 집중은 그대로, 없는 키면 아무 일도 없음. 10초
  안에 이미 적었으면 쓰지 않는다 — 읽기로 먼저 걸러 다른 연결의 쓰기를 막지 않게) · `set_addr(id, addr)`. `upsert(…, round)`(있으면 승인 초기화) ·
  `by_spki` · `by_id` · `list` · `count_unapproved` · `set_my_ok` · `set_their_ok(spki, round)`(같은 세대 · 실제 전이만 true) ·
  `remove` · `remove_spki` · `set_meta(id, group, model, effort)` · `set_tile(id, alias, icon)`(통째로 바꿈, `None` = 지움 — 이 PC 만의 표시 ·
  실행 설정).
- `settings::get(conn, key)` · `set(conn, key, value)` — 문자열 설정. `FOCUS_KEY`(`focus`) · `focus(conn)`(값 `1` 만 켜짐 — 읽기 오류는 그대로). `AUTOSTART_KEY`(`autostart`) ·
  `autostart(conn)`(없으면 켜짐 — D13, 값 `0` 만 꺼짐, #1e).
- `agents::READ_ONLY_TOOLS`(`Read` · `Grep` · `Glob` · `LS` · `NotebookRead`, D31) — 모든 실행의 도구(#1d — 프로필마다 고르지 않는다).
- `agents` — `Agent { id, name, folder, prompt, group, uid }`(`uid` = 세대 — 등록 · 폴더 변경 때 새 값) · `add(conn, name, folder, prompt)`(`agents.tools`
  열은 `NOT NULL` 이라 5개를 적어 두고 읽지 않는다) · `by_name` ·
  `by_id` · `list` · `remove` · `update(id, folder, prompt, group)`(이름은 바꾸지 않는다 — 대화 · 상대 PC 가 이름으로 부른다. 폴더가 바뀌면
  그 에이전트가 이어 가던 세션(받은 대화 · 내가 연 자동 왕복 · 멘션 `agent_sessions`)을 비운다 — `forget_sessions(name)`, `remove` 도 같다 — Claude 세션은 폴더에 묶여 있다).
- `chat` — `Dir{Out,In}` · `Author{Human,Agent}` · `State{Sent,Queued,Running,Done,Error,Undelivered}` · `Conversation` · `Message`.
  `open_conversation(uuid, peer_id, agent, dir)`(다른 상대의 같은 uuid 는 오류) · `find(uuid, peer_id)` · `by_id` · `list` · `set_session` ·
  `add_message` · `add_reply(…, reply_to, turns, duration_ms)`(에이전트 답) · `set_state` · `messages` · `message_by_uuid` ·
  `pending(conv)`(DB 대기열 — 커서 없이 매번 처음부터) · `pending_conversations` · `summaries`(`Summary{conversation, title(이 PC 에서 붙인 제목,
  없으면 첫 사람 메시지 앞 40자), last_state, last_at}`, 최근 것 먼저) · `set_title(id, title)`(`None` = 지움) ·
  `messages_across(mine, scope, author, request, q, limit)`(대화를 가로지른 글 — 범위 `Scope::All` · `Peer(id)` · `Group(id)`(#1i) · 종류 · 사람 글
  종류(`request` — 사람 글만) · 본문 일치를 거른 뒤 최신 `limit` 건) · `saved(scope, conversation, limit)` · `Summary.pos`(손으로 놓은 자리 — 목록 자체는 최근
  활동순 그대로). 새 대화 `pos = min - 1`(맨 위). `Message` 에 `created_at` · `reply_to` · `turns` · `duration_ms` · `mine`(이 PC 의 글) · `exec`(실행 요청) ·
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
  #18: `Message` 에 `request`(요청 — 아니면 질의) · `agent`(멘션 — 부른 · 실행한 에이전트) · `refused`(보낸 쪽 — 상대 거절 코드).
  `set_request(id, bool)`(요청이면 `req_state = wait`, 질의면 `NULL`) · `set_agent(id, Option)` · `set_refused(id, Option)`(다시
  보내면 `None`) · `message_by_id` · `agent_session(conv, agent)` · `set_agent_session(conv, agent, session)`.
  #1d: `ReqState`(`Wait` · `Accepted` · `Rejected` · `Done` — `as_str` · `parse` · `can_become`(대기 → 수용 · 반려, 수용 → 완료만)) ·
  `Message.req_state` · `req_note` · `req_at` · `set_req_state(id, state, note)`(옮김 검사는 부르는 쪽) · `MAX_REQ_NOTE`(200) ·
  `clean_req_note`(한 줄 · 제어 문자 없음) · `AskFilter`(`Query` · `Request(Option<ReqState>)` — `messages_across` 의 종류 · 상태 거르기).
  #1f: `Message.edited_at` · `edit_text(id, text)`(본문 · 편집 시각만 — uuid · `lc` · 요청 상태 · 저장 그대로) · `delete_message(id)`(글 + 그 글의
  `cli_tx` · `relay_events` · 실패 보관 outbox 행, 답은 남는다 · 읽음 · 내 활동 본 표시를 남은 최대 id 로 낮춤 — 지운 id 를 다음 글이 다시 받아도
  안 읽음으로 보이게, 부르는 쪽이 트랜잭션) · `RelayMark::SyncFailed(Option<글 id>)`(`sync_failed` — 편집 실패는
  그 글에 이어 글을 지우면 기록도 지워진다) ·
  `ActivityKind::SyncFailed`(내 활동 — 전체 · 실패 거르기에 들고 안 본 수에 든다, 자동 왕복 거르기에는 없다).
- `outbox`(#1c — D7 보내기 보관) — 상대에게 가는 대화 프레임 1개 = 행 1개(프레임 내용은 모른다 — 직렬화된 JSON 그대로). `Item{id, peer_id, conv_id,
  msg_id, kind, frame, key, state, tries, waited, next_at, last_error, due_in}`(`state` = `pending` · `inflight` · `failed`, `due_in` = 다음 시각까지
  남은 초). `add(conn, peer, conv?, msg?, kind, frame, key?, state)`(`key` 가 같은 `pending` 행이 그 상대에 있으면 그 행의 프레임만 바꿈 — 마지막
  값만) · `head(peer)`(줄 맨 앞 — `pending` · `inflight` 중 가장 오래된 것) · `has_before(conv, id, except)`(같은 대화에 앞선 행 — `except` 종류는
  세지 않음) · `failing(peer)`(시도 1번 이상인 `pending` 이 있나 — 닿지 않고 있음) · `get` · `set_state` · `retry(id, delay, cap, error) -> 기다린
  합`(줄로 되돌림 · `tries` +1 · 지난 시도부터 실제로 흐른 시간을 `cap` 까지만 `waited` 에 더함 · 보내는 동안 `hurry` 를 받았으면 지금 다시) ·
  `queued_for(peer)`(기다리는 `pending` 행 — 포기 대상, 보내는 중은 빼고) · `prune_failed(days)` · `hurry(peer)`(지금 보내게 — `inflight` 는 표시만) · `fail(id, error)` · `remove` · `pending_peers()`(남은
  `inflight` 를 `pending` 으로 되돌리고 보낼 것이 남은 상대) · `for_message(msg)`(그 글을 실어 나르는 최근 행 — 화면 표시 · 편집 가능 판정, 반영 행 `EDIT`(`msg_edit`) · `DELETE`(`msg_delete`)는 뺀다 — #1f) · `clear_failed(msg)`.
- `cli_tx`(#1c — CLI 트랜잭션) — `Tx{id, key, message_id, created_at, asked_at, unreachable_since, stalled_at}`. `key_ok(key)`(1 ~ 64자
  `[A-Za-z0-9._-]`) · `create(key?, message_id)`(키 UNIQUE — 같은 키 두 번은 오류) · `by_id` · `by_key` · `by_message(message_id)` · `due(id, every)`(아직 묻지 않았거나 마지막
  질의가 `every` 초 전) · `asked(id)` · `reached(id)`(닿지 않음 시작을 지움 — 멈춘 뒤면 그대로) · `unreachable(id, stall_after) -> 멈췄나`(처음이면
  시작 시각, `stall_after` 초 지나면 `stalled_at`) · `resume(id)`(멈춤 · 닿지 않음 · 질의 시각을 지우고 다시 판정 — `wait`).
- `run_opts` — 실행 옵션 · 화면 설정의 검증과 기본값 단일 지점. `DEFAULT_MODEL`(`sonnet`) · `DEFAULT_EFFORT`(`high`) · `EFFORTS` ·
  설정 키 `MODEL_KEY` · `EFFORT_KEY` · `THEME_KEY` · `SIDE_WIDTH_KEY` · `ME_ALIAS_KEY` · `ME_ICON_KEY` · `MODELS`(`opus` · `sonnet` · `haiku` — #1d) · `valid_model`(목록 안) ·
  `valid_effort` · `clean_group` · `clean_alias` · `clean_title` · `clean_icon`(PNG data URL) · `defaults` · `resolve(conn, peer)`(상대 덮어쓰기 →
  전역 → 기본값) · `theme` · `side_width` · `me_tile`. 읽기는 `rusqlite::Result` — 없거나 틀린 값만 기본값, DB 오류는 그대로 낸다.
- `log_file::FileLogger::install(path, level)` — `LOG_ROTATE_BYTES`(1 MiB)를 넘게 되는 기록 전에 `.1` 로 회전(1개 보관). 패닉 훅도 단다(#1e) —
  `error` "패닉: {메시지} — {파일}:{줄}" 뒤 기본 훅(stderr).
- `groups`(#1i — 이 PC 에만): `Group{id, name, icon, pos, convs}` · `clean_name`(1 ~ `NAME_CHARS` 40자) · `list`(자리 순) · `by_id` · `name_taken(name,
  except)` · `add`(끝에) · `rename` · `set_icon` · `remove`(소속 함께 — 외래 키를 켜지 않는 DB) · `unknown_conversations` · `change(id, add, remove)` ·
  `join(id, conv)`(부르는 쪽 트랜잭션 안 — 새 대화) · `reorder(Order::{Peers, Groups, Conversations}, ids)`(겹침 · 없는 id → `Err(문구)`, 빠진 id 는 제 자리).
  스키마 16: `peers.pos`(새 상대 `max + 1`, `peers::list` = `ORDER BY pos, id`) · `conversations.pos` · `groups`(AUTOINCREMENT) · `group_convs`.
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
