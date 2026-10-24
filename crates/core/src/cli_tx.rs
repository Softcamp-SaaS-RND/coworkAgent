//! CLI 트랜잭션 — `ask` 하나 = 보낸 실행 요청 글 하나. 앱이 주인이고 CLI 는 상태를 본다: 중복 키(같은 키는 새로 보내지 않음) · 이어 받기
//! (`wait <tx>`) · 닿지 않음 판정(`unreachable_since` → `stalled_at`).
use rusqlite::{Connection, OptionalExtension, Row, params};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tx {
    pub id: i64,
    pub key: Option<String>,
    pub message_id: i64,
    pub created_at: i64,
    /// 마지막으로 상대에 상태를 물은 시각.
    pub asked_at: Option<i64>,
    pub unreachable_since: Option<i64>,
    pub stalled_at: Option<i64>,
}

const COLS: &str = "id, key, message_id, created_at, asked_at, unreachable_since, stalled_at";

fn row(r: &Row) -> rusqlite::Result<Tx> {
    Ok(Tx {
        id: r.get(0)?,
        key: r.get(1)?,
        message_id: r.get(2)?,
        created_at: r.get(3)?,
        asked_at: r.get(4)?,
        unreachable_since: r.get(5)?,
        stalled_at: r.get(6)?,
    })
}

/// 키 규칙 — 1 ~ 64자 `[A-Za-z0-9._-]`.
pub fn key_ok(key: &str) -> bool {
    (1..=64).contains(&key.len())
        && key
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'))
}

pub fn create(conn: &Connection, key: Option<&str>, message_id: i64) -> rusqlite::Result<Tx> {
    conn.execute(
        "INSERT INTO cli_tx (key, message_id, created_at) VALUES (?1, ?2, unixepoch())",
        params![key, message_id],
    )?;
    by_id(conn, conn.last_insert_rowid()).map(|t| t.expect("방금 넣은 행"))
}

pub fn by_id(conn: &Connection, id: i64) -> rusqlite::Result<Option<Tx>> {
    conn.query_row(&format!("SELECT {COLS} FROM cli_tx WHERE id = ?1"), [id], row)
        .optional()
}

pub fn by_key(conn: &Connection, key: &str) -> rusqlite::Result<Option<Tx>> {
    conn.query_row(&format!("SELECT {COLS} FROM cli_tx WHERE key = ?1"), [key], row)
        .optional()
}

pub fn by_message(conn: &Connection, message_id: i64) -> rusqlite::Result<Option<Tx>> {
    conn.query_row(
        &format!("SELECT {COLS} FROM cli_tx WHERE message_id = ?1"),
        [message_id],
        row,
    )
    .optional()
}

/// 상대에 상태를 물을 때인가 — 아직 묻지 않았거나 마지막으로 물은 지 `every` 초가 지났다.
pub fn due(conn: &Connection, id: i64, every: i64) -> rusqlite::Result<bool> {
    conn.prepare(
        "SELECT 1 FROM cli_tx WHERE id = ?1 AND (asked_at IS NULL OR unixepoch() - asked_at >= ?2)",
    )?
    .exists(params![id, every])
}

/// 상태를 물었다 — 시각을 남긴다.
pub fn asked(conn: &Connection, id: i64) -> rusqlite::Result<()> {
    conn.execute("UPDATE cli_tx SET asked_at = unixepoch() WHERE id = ?1", [id])?;
    Ok(())
}

/// 상대에 닿았다 — 닿지 않음 시작을 지운다(닿지 않음 판정 전이면 멈춤도 없다).
pub fn reached(conn: &Connection, id: i64) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE cli_tx SET unreachable_since = NULL WHERE id = ?1 AND stalled_at IS NULL",
        [id],
    )?;
    Ok(())
}

/// 닿지 않았다 — 처음이면 시각을 남기고, `stall_after` 초가 지났으면 멈춤으로. 돌려주는 값: 멈췄나.
pub fn unreachable(conn: &Connection, id: i64, stall_after: i64) -> rusqlite::Result<bool> {
    conn.execute(
        "UPDATE cli_tx SET unreachable_since = coalesce(unreachable_since, unixepoch()) WHERE id = ?1",
        [id],
    )?;
    conn.execute(
        "UPDATE cli_tx SET stalled_at = unixepoch()
         WHERE id = ?1 AND stalled_at IS NULL AND unixepoch() - unreachable_since >= ?2",
        params![id, stall_after],
    )?;
    Ok(by_id(conn, id)?.is_some_and(|t| t.stalled_at.is_some()))
}

/// 이어 받기 — 멈춤을 풀고 다시 판정한다(`wait <tx>`).
pub fn resume(conn: &Connection, id: i64) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE cli_tx SET stalled_at = NULL, unreachable_since = NULL, asked_at = NULL WHERE id = ?1",
        [id],
    )?;
    Ok(())
}
