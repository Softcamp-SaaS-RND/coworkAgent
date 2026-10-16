use rusqlite::OptionalExtension;
pub use rusqlite::{Connection, Error};
use std::path::Path;

pub const SCHEMA_VERSION: i64 = 18;

/// 쓰기 트랜잭션 — `BEGIN IMMEDIATE`. 읽고 나서 쓰는 지연(deferred) 트랜잭션은 다른 연결(피어 노드 · 알림)이 쓰는 중이면 잠금을 올리지 못해
/// `busy_timeout` 을 기다리지 않고 바로 `database is locked` 로 실패한다(SQLite 교착 회피) — 처음부터 쓰기 잠금을 잡아 기다린다.
pub fn write_tx(conn: &Connection) -> rusqlite::Result<rusqlite::Transaction<'_>> {
    rusqlite::Transaction::new_unchecked(conn, rusqlite::TransactionBehavior::Immediate)
}

/// DB 를 열고 스키마를 `SCHEMA_VERSION` 으로 올린다. 이 실행 파일보다 새 스키마면 열지 않는다.
pub fn open(path: &Path) -> rusqlite::Result<Connection> {
    let conn = Connection::open(path)?;
    // 같은 파일을 여러 연결(피어 노드 · 알림 · 대화)이 쓴다 — 잠금이 풀릴 때까지 잠깐 기다린다.
    conn.busy_timeout(std::time::Duration::from_secs(5))?;
    let v = user_version(&conn)?;
    if v > SCHEMA_VERSION {
        return Err(rusqlite::Error::InvalidParameterName(format!(
            "DB 스키마 {v} 가 이 실행 파일({SCHEMA_VERSION})보다 새롭다"
        )));
    }
    // 스키마를 올리기 전에 지금 DB 를 떠 둔다 — 새 버전에 문제가 있으면 이전 실행 파일로 돌아가 이 파일로 바꿔 쓴다
    // (이전 실행 파일은 더 새 스키마를 열지 않는다). 같은 버전 사본이 이미 있으면 그대로 둔다.
    if (1..SCHEMA_VERSION).contains(&v) {
        backup_before_migration(&conn, path, v)?;
    }
    if v < 2 {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS peers (
                id INTEGER PRIMARY KEY,
                spki BLOB NOT NULL UNIQUE,
                name TEXT NOT NULL,
                host TEXT NOT NULL,
                addr TEXT NOT NULL,
                my_ok INTEGER NOT NULL DEFAULT 0,
                their_ok INTEGER NOT NULL DEFAULT 0,
                round TEXT NOT NULL DEFAULT '',
                created_at INTEGER NOT NULL
            );
            PRAGMA user_version = 2;",
        )?;
    }
    if v < 3 {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS settings (key TEXT PRIMARY KEY, value TEXT NOT NULL);
            PRAGMA user_version = 3;",
        )?;
    }
    if v < 4 {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS agents (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                folder TEXT NOT NULL,
                tools TEXT NOT NULL,
                prompt TEXT NOT NULL DEFAULT '',
                created_at INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS conversations (
                id INTEGER PRIMARY KEY,
                uuid TEXT NOT NULL UNIQUE,
                peer_id INTEGER NOT NULL,
                agent TEXT NOT NULL,
                dir TEXT NOT NULL,
                session TEXT,
                created_at INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS messages (
                id INTEGER PRIMARY KEY,
                uuid TEXT NOT NULL UNIQUE,
                conversation_id INTEGER NOT NULL,
                author TEXT NOT NULL,
                text TEXT NOT NULL,
                state TEXT NOT NULL,
                created_at INTEGER NOT NULL
            );
            PRAGMA user_version = 4;",
        )?;
    }
    if v < 5 {
        // 열 추가 · 버전 갱신을 한 트랜잭션으로 — 중간에 끊겨도 반쯤 오른 스키마가 남지 않는다. 이미 있는 열은 건너뛴다.
        let tx = conn.unchecked_transaction()?;
        for col in ["reply_to", "turns", "duration_ms"] {
            if !has_column(&tx, "messages", col)? {
                tx.execute_batch(&format!("ALTER TABLE messages ADD COLUMN {col} INTEGER;"))?;
            }
        }
        tx.execute_batch("PRAGMA user_version = 5;")?;
        tx.commit()?;
    }
    if v < 6 {
        // 양쪽 발화(D18 · D23) — 이 PC 에서 쓴 글인가(`mine`) · 실행 요청인가(`exec`) · 양쪽 공통 순서(`lc`, 램포트 시계).
        // 이전 이력은 방향 · id 로 채운다.
        let tx = conn.unchecked_transaction()?;
        for col in ["mine", "exec", "lc"] {
            if !has_column(&tx, "messages", col)? {
                tx.execute_batch(&format!("ALTER TABLE messages ADD COLUMN {col} INTEGER;"))?;
            }
        }
        tx.execute_batch(&format!(
            "UPDATE messages SET mine = {MINE_BY_DIR}, exec = (author = 'human') WHERE mine IS NULL OR exec IS NULL;
             UPDATE messages SET lc = id WHERE lc IS NULL;
             PRAGMA user_version = 6;"
        ))?;
        tx.commit()?;
    }
    if v < 7 {
        // 자동 왕복(D24) — 연 쪽: 내 에이전트 · 상한 · 지금 왕복 · 켜짐, 받은 쪽: 에이전트발 실행 요청 거절.
        let tx = conn.unchecked_transaction()?;
        for (col, ty) in [
            ("relay_agent", "TEXT"),
            ("relay_cap", "INTEGER"),
            ("relay_count", "INTEGER"),
            ("relay_on", "INTEGER"),
            ("relay_blocked", "INTEGER"),
        ] {
            if !has_column(&tx, "conversations", col)? {
                tx.execute_batch(&format!("ALTER TABLE conversations ADD COLUMN {col} {ty};"))?;
            }
        }
        tx.execute_batch("PRAGMA user_version = 7;")?;
        tx.commit()?;
    }
    if v < 8 {
        // 글의 출처 — `cli` = 사용자 AI 에이전트가 CLI 로 쓴 글, NULL = 웹UI(사람).
        let tx = conn.unchecked_transaction()?;
        if !has_column(&tx, "messages", "via")? {
            tx.execute_batch("ALTER TABLE messages ADD COLUMN via TEXT;")?;
        }
        tx.execute_batch("PRAGMA user_version = 8;")?;
        tx.commit()?;
    }
    if v < 9 {
        // 그룹(피어 · 에이전트) · 상대별 실행 덮어쓰기(model · effort, NULL = 전역 기본값).
        let tx = conn.unchecked_transaction()?;
        for (table, col) in [
            ("peers", "grp"),
            ("peers", "model"),
            ("peers", "effort"),
            ("agents", "grp"),
        ] {
            if !has_column(&tx, table, col)? {
                tx.execute_batch(&format!("ALTER TABLE {table} ADD COLUMN {col} TEXT;"))?;
            }
        }
        tx.execute_batch("PRAGMA user_version = 9;")?;
        tx.commit()?;
    }
    if v < 10 {
        // 이 PC 에서 붙이는 표시 — 대화 제목(NULL = 첫 사람 글 앞부분) · 상대 표시 이름(별칭) · 타일 아이콘(PNG data URL).
        let tx = conn.unchecked_transaction()?;
        for (table, col) in [
            ("conversations", "title"),
            ("peers", "alias"),
            ("peers", "icon"),
        ] {
            if !has_column(&tx, table, col)? {
                tx.execute_batch(&format!("ALTER TABLE {table} ADD COLUMN {col} TEXT;"))?;
            }
        }
        tx.execute_batch("PRAGMA user_version = 10;")?;
        tx.commit()?;
    }
    if v < 11 {
        // 이 PC 의 작업 화면 — ★ · 읽은 마지막 글 · 저장 · 답의 실행 옵션 · 내 활동을 본 마지막 글 · 기록 id · 자동 왕복 시작/끝 기록
        // (+ 내 글이 실패로 바뀐 순간 — `message_id`) · 대화별 글 · 기록 색인(5초 새로 고침의 대화 목록 · 안 읽음 · 내 활동이 표 전체를 훑지 않게).
        // 이전 대화는 다 읽은 것, 이전 활동은 본 것으로 둔다.
        let tx = conn.unchecked_transaction()?;
        for (table, col, ty) in [
            ("conversations", "starred_at", "INTEGER"),
            ("conversations", "read_id", "INTEGER"),
            ("messages", "saved_at", "INTEGER"),
            ("messages", "model", "TEXT"),
            ("messages", "effort", "TEXT"),
            ("peers", "activity_seen_msg", "INTEGER"),
            ("peers", "activity_seen_event", "INTEGER"),
        ] {
            if !has_column(&tx, table, col)? {
                tx.execute_batch(&format!("ALTER TABLE {table} ADD COLUMN {col} {ty};"))?;
            }
        }
        tx.execute_batch(
            "CREATE TABLE IF NOT EXISTS relay_events (
               id INTEGER PRIMARY KEY,
               conversation_id INTEGER NOT NULL,
               kind TEXT NOT NULL,
               detail TEXT NOT NULL DEFAULT '',
               created_at INTEGER NOT NULL,
               message_id INTEGER
             );
             CREATE INDEX IF NOT EXISTS messages_by_conv ON messages(conversation_id);
             CREATE INDEX IF NOT EXISTS relay_events_by_conv ON relay_events(conversation_id);
             CREATE INDEX IF NOT EXISTS relay_events_by_message ON relay_events(message_id);
             UPDATE conversations SET read_id = (SELECT max(m.id) FROM messages m WHERE m.conversation_id = conversations.id)
               WHERE read_id IS NULL;
             UPDATE peers SET activity_seen_msg = (SELECT coalesce(max(id), 0) FROM messages),
               activity_seen_event = (SELECT coalesce(max(id), 0) FROM relay_events)
               WHERE activity_seen_msg IS NULL;
             PRAGMA user_version = 11;",
        )?;
        tx.commit()?;
    }
    if v < 12 {
        // 상대에게 보이는 동작(집중 · 질의/요청 · 멘션) — 글: 요청(건의, NULL · 0 = 질의) · 완료 표시 시각 · 멘션한 에이전트(NULL = 대화
        // 에이전트) · 상대가 거절한 코드, 상대: 마지막 `Pong` 시각 · 그때의 집중, 멘션한 에이전트의 Claude 세션(대화 에이전트는
        // `conversations.session` 그대로).
        let tx = conn.unchecked_transaction()?;
        for (table, col, ty) in [
            ("messages", "request", "INTEGER"),
            ("messages", "done_at", "INTEGER"),
            ("messages", "agent", "TEXT"),
            ("messages", "refused", "TEXT"),
            ("peers", "seen_at", "INTEGER"),
            ("peers", "focus", "INTEGER"),
            ("agents", "uid", "TEXT"),
        ] {
            if !has_column(&tx, table, col)? {
                tx.execute_batch(&format!("ALTER TABLE {table} ADD COLUMN {col} {ty};"))?;
            }
        }
        // 에이전트 세대 — 끝난 실행이 지워졌다 다시 등록된(또는 폴더가 바뀐) 에이전트에 옛 세션을 남기지 않게.
        tx.execute_batch(
            "UPDATE agents SET uid = lower(hex(randomblob(16))) WHERE uid IS NULL;
             CREATE TABLE IF NOT EXISTS agent_sessions (
               conversation_id INTEGER NOT NULL,
               agent TEXT NOT NULL,
               session TEXT NOT NULL,
               PRIMARY KEY (conversation_id, agent)
             );
             PRAGMA user_version = 12;",
        )?;
        tx.commit()?;
    }
    if v < 13 {
        // 보내는 쪽 보관(D7 — #1c): 상대에게 가는 대화 프레임을 먼저 적고 상대별 줄이 ack 까지 다시 보낸다. CLI 트랜잭션(중복 키 · 이어 받기 ·
        // 닿지 않음 판정).
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS outbox (
                id INTEGER PRIMARY KEY,
                peer_id INTEGER NOT NULL,
                conv_id INTEGER,
                msg_id INTEGER,
                kind TEXT NOT NULL,
                frame TEXT NOT NULL,
                key TEXT,
                state TEXT NOT NULL DEFAULT 'pending',
                tries INTEGER NOT NULL DEFAULT 0,
                waited INTEGER NOT NULL DEFAULT 0,
                next_at INTEGER NOT NULL DEFAULT 0,
                last_try INTEGER,
                last_error TEXT,
                created_at INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS outbox_peer ON outbox (peer_id, state, id);
            CREATE TABLE IF NOT EXISTS cli_tx (
                id INTEGER PRIMARY KEY,
                key TEXT UNIQUE,
                message_id INTEGER NOT NULL,
                created_at INTEGER NOT NULL,
                asked_at INTEGER,
                unreachable_since INTEGER,
                stalled_at INTEGER
            );
            PRAGMA user_version = 13;",
        )?;
    }
    if v < 14 {
        // 요청 목록(#1d): 요청 글의 상태(`wait` · `accepted` · `rejected` · `done`) · 메모 · 바꾼 시각 — 완료 표시(`done_at`)는 `done` 으로 옮기고
        // 열은 읽지 않는다. 모델은 목록(`run_opts::MODELS`) 밖 값을 지운다(기본값). 아직 못 보낸 옛 완료 알림(`Request::Done`) 행은 같은 자리 · 순서 그대로
        // `ReqState` 프레임(완료 → `done`, 취소 → `wait`)으로 바꾸고 그 요청 글에 잇는다 — 요청자가 꺼져 있는 동안 바꾼 상태를 잃지 않게(#1d R1).
        let tx = conn.unchecked_transaction()?;
        for (col, ty) in [
            ("req_state", "TEXT"),
            ("req_note", "TEXT"),
            ("req_at", "INTEGER"),
        ] {
            if !has_column(&tx, "messages", col)? {
                tx.execute_batch(&format!("ALTER TABLE messages ADD COLUMN {col} {ty};"))?;
            }
        }
        tx.execute_batch(
            "UPDATE messages SET req_state = CASE WHEN done_at IS NOT NULL THEN 'done' ELSE 'wait' END, req_at = done_at
             WHERE request = 1 AND req_state IS NULL;
             DELETE FROM settings WHERE key = 'claude_model' AND value NOT IN ('opus', 'sonnet', 'haiku');
             UPDATE peers SET model = NULL WHERE model NOT IN ('opus', 'sonnet', 'haiku');
             UPDATE outbox SET kind = 'req_state',
               msg_id = (SELECT m.id FROM messages m WHERE m.uuid = json_extract(outbox.frame, '$.msg')),
               frame = json_object('type', 'req_state', 'conv', json_extract(frame, '$.conv'), 'msg', json_extract(frame, '$.msg'),
                 'state', CASE WHEN json_extract(frame, '$.done') THEN 'done' ELSE 'wait' END, 'note', NULL)
             WHERE kind = 'done';
             PRAGMA user_version = 14;",
        )?;
        tx.commit()?;
    }
    if v < 15 {
        // 이력 정리(#1f): 내 사람 글을 편집한 시각 — "(편집됨)" 표시.
        let tx = conn.unchecked_transaction()?;
        if !has_column(&tx, "messages", "edited_at")? {
            tx.execute_batch("ALTER TABLE messages ADD COLUMN edited_at INTEGER;")?;
        }
        tx.execute_batch("PRAGMA user_version = 15;")?;
        tx.commit()?;
    }
    if v < 16 {
        // 레일 그룹 · 손으로 놓는 순서(#1i): 그룹(이 PC 에만) · 소속, 상대 · 대화 자리. 첫 자리 = 지금 보이는 순서(상대 = id 순, 대화 = 최근 활동순).
        // 외래 키는 켜지 않는다(이 DB 기본) — 그룹을 지울 때 소속은 `groups::remove` 가 함께 지운다.
        let tx = conn.unchecked_transaction()?;
        for table in ["peers", "conversations"] {
            if !has_column(&tx, table, "pos")? {
                tx.execute_batch(&format!(
                    "ALTER TABLE {table} ADD COLUMN pos INTEGER NOT NULL DEFAULT 0;"
                ))?;
            }
        }
        tx.execute_batch(
            "UPDATE peers SET pos = (SELECT count(*) FROM peers p WHERE p.id < peers.id);
             UPDATE conversations SET pos = (SELECT r.n FROM (
               SELECT c.id, row_number() OVER (
                 ORDER BY coalesce((SELECT max(created_at) FROM messages WHERE conversation_id = c.id), c.created_at) DESC,
                   coalesce((SELECT max(id) FROM messages WHERE conversation_id = c.id), 0) DESC, c.id DESC) - 1 AS n
               FROM conversations c) r WHERE r.id = conversations.id);
             CREATE TABLE IF NOT EXISTS groups (
               id INTEGER PRIMARY KEY AUTOINCREMENT,
               name TEXT NOT NULL UNIQUE,
               icon TEXT,
               pos INTEGER NOT NULL
             );
             CREATE TABLE IF NOT EXISTS group_convs (
               group_id INTEGER NOT NULL,
               conversation_id INTEGER NOT NULL,
               PRIMARY KEY (group_id, conversation_id)
             );
             CREATE INDEX IF NOT EXISTS group_convs_conv ON group_convs(conversation_id);
             PRAGMA user_version = 16;",
        )?;
        tx.commit()?;
    }
    if v < 17 {
        // 상대 번호를 다시 쓰지 않는다(#1h — 원장 #17 S1-P): 가장 큰 번호의 상대를 지우면 다음 상대가 그 번호를 받아 남은 대화 · 보낼 글이 새 상대에
        // 붙었다. SQLite 는 AUTOINCREMENT 를 제자리에서 못 붙인다 — 지금 정의(뒤에 붙인 열 포함)로 새 표 → 번호째 복사 → 옛 표 지움 → 이름 바꿈.
        // 순번 시작 = 지운 상대가 남긴 대화 · 보낼 글 번호까지(이미 지운 상대의 번호도 다시 쓰지 않는다). 외래 키는 꺼져 있다(이 DB 기본).
        let tx = conn.unchecked_transaction()?;
        let sql: String = tx.query_row(
            "SELECT sql FROM sqlite_master WHERE type = 'table' AND name = 'peers'",
            [],
            |r| r.get(0),
        )?;
        if !sql.contains("AUTOINCREMENT") {
            // 이름 바꾸기를 거친 표의 정의는 `CREATE TABLE "peers"` 로 남는다 — 두 꼴 모두.
            let new = sql
                .replacen("CREATE TABLE \"peers\"", "CREATE TABLE peers", 1)
                .replacen("CREATE TABLE peers", "CREATE TABLE peers_new", 1)
                .replacen(
                    "id INTEGER PRIMARY KEY",
                    "id INTEGER PRIMARY KEY AUTOINCREMENT",
                    1,
                );
            if !new.contains("CREATE TABLE peers_new") || !new.contains("AUTOINCREMENT") {
                return Err(rusqlite::Error::InvalidParameterName(format!(
                    "스키마 17 — peers 정의를 알 수 없다: {sql}"
                )));
            }
            tx.execute_batch(&format!(
                "{new};
                 INSERT INTO peers_new SELECT * FROM peers;
                 DROP TABLE peers;
                 ALTER TABLE peers_new RENAME TO peers;"
            ))?;
        }
        // 접힘 설정 열쇠(`{화면}:{상대 번호}`)의 번호도 — 대화 없이 지운 상대의 남은 흔적이다(라운드 2, 남은 접힘이 새 상대에 붙지 않게).
        // 흔적이 하나도 없는 지운 번호는 되살릴 수 없다 — 붙을 데이터도 없다(원장 #1h R1-M1).
        let folded = crate::run_opts::folded(&tx)?
            .iter()
            .filter_map(|k| k.split_once(':')?.1.parse::<i64>().ok())
            .max()
            .unwrap_or(0);
        tx.execute("DELETE FROM sqlite_sequence WHERE name = 'peers'", [])?;
        tx.execute(
            "INSERT INTO sqlite_sequence (name, seq) VALUES ('peers', max(
               (SELECT coalesce(max(id), 0) FROM peers),
               (SELECT coalesce(max(peer_id), 0) FROM conversations),
               (SELECT coalesce(max(peer_id), 0) FROM outbox),
               ?1))",
            [folded],
        )?;
        tx.execute_batch("PRAGMA user_version = 17;")?;
        tx.commit()?;
    }
    if v < 18 {
        // 상대 단위 그룹(#1j): 그룹에 상대를 넣으면 그 상대의 대화(뒤에 생긴 것도)가 그 그룹에 보인다. 옛 자유 입력 그룹(`peers.grp`)은 같은 이름
        // 레일 그룹으로 옮긴다(없으면 만든다 — 이름은 `groups::fit_name`). `agents.grp` 는 읽지 않는다(열은 그대로 둔다).
        let tx = conn.unchecked_transaction()?;
        tx.execute_batch(
            "CREATE TABLE IF NOT EXISTS group_peers (
               group_id INTEGER NOT NULL,
               peer_id INTEGER NOT NULL,
               PRIMARY KEY (group_id, peer_id)
             );
             CREATE INDEX IF NOT EXISTS group_peers_peer ON group_peers(peer_id);",
        )?;
        let old: Vec<(i64, String)> = tx
            .prepare("SELECT id, grp FROM peers WHERE grp IS NOT NULL ORDER BY pos, id")?
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
            .collect::<rusqlite::Result<_>>()?;
        for (peer, grp) in old {
            let Some(name) = crate::groups::fit_name(&grp) else {
                continue;
            };
            let found: Option<i64> = tx
                .query_row("SELECT id FROM groups WHERE name = ?1", [&name], |r| {
                    r.get(0)
                })
                .optional()?;
            let id = match found {
                Some(id) => id,
                None => crate::groups::add(&tx, &name)?.id,
            };
            tx.execute(
                "INSERT OR IGNORE INTO group_peers (group_id, peer_id) VALUES (?1, ?2)",
                rusqlite::params![id, peer],
            )?;
        }
        tx.execute_batch("PRAGMA user_version = 18;")?;
        tx.commit()?;
    }
    Ok(conn)
}

/// 방향으로 정한 "이 PC 의 글" — 보낸 대화(`out`)의 사람 글 · 받은 대화(`in`)의 에이전트 답. `messages` 행 문맥의 SQL 식.
pub const MINE_BY_DIR: &str =
    "(CASE (SELECT dir FROM conversations c WHERE c.id = messages.conversation_id)
    WHEN 'out' THEN author = 'human' ELSE author = 'agent' END)";

/// `{db}.v{버전}.bak` — 스키마를 올리기 전 사본.
pub fn backup_path(path: &Path, version: i64) -> std::path::PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(format!(".v{version}.bak"));
    path.with_file_name(name)
}

/// `VACUUM INTO` 는 열린 연결에서도 일관된 사본을 만든다.
fn backup_before_migration(conn: &Connection, path: &Path, version: i64) -> rusqlite::Result<()> {
    let to = backup_path(path, version);
    if to.exists() {
        return Ok(());
    }
    conn.execute("VACUUM INTO ?1", [to.to_string_lossy()])?;
    Ok(())
}

fn has_column(conn: &Connection, table: &str, col: &str) -> rusqlite::Result<bool> {
    conn.prepare(&format!(
        "SELECT 1 FROM pragma_table_info('{table}') WHERE name = ?1"
    ))?
    .exists([col])
}

pub fn user_version(conn: &Connection) -> rusqlite::Result<i64> {
    conn.query_row("PRAGMA user_version", [], |r| r.get(0))
}
