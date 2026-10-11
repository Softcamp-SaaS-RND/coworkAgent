pub use rusqlite::{Connection, Error};
use std::path::Path;

pub const SCHEMA_VERSION: i64 = 7;

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
