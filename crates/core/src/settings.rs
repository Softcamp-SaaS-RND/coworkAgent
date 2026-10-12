//! 앱 설정 — 키 · 문자열 값(스키마 3 `settings` 표).
use rusqlite::{Connection, OptionalExtension, params};

pub fn get(conn: &Connection, key: &str) -> rusqlite::Result<Option<String>> {
    conn.query_row("SELECT value FROM settings WHERE key = ?1", [key], |r| {
        r.get(0)
    })
    .optional()
}

/// 집중(이 PC) — 켜 있는 동안 상대 PC 의 새 실행 요청을 거부하고 알림을 멈춘다. 값 `1` 만 켜짐.
pub const FOCUS_KEY: &str = "focus";

pub fn focus(conn: &Connection) -> rusqlite::Result<bool> {
    Ok(get(conn, FOCUS_KEY)?.as_deref() == Some("1"))
}

pub fn set(conn: &Connection, key: &str, value: &str) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = ?2",
        params![key, value],
    )
    .map(drop)
}
