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

/// 로그인 때 자동 시작(S7 · D13) — 없으면 켜짐(D13 기본 ON), 값 `0` 만 꺼짐.
pub const AUTOSTART_KEY: &str = "autostart";

pub fn autostart(conn: &Connection) -> rusqlite::Result<bool> {
    Ok(get(conn, AUTOSTART_KEY)?.as_deref() != Some("0"))
}

/// 자동 압축(#1m) — 이 PC 에서 돈 실행의 창 사용량이 [`AUTO_COMPACT_AT`] 이상이면 그 대화 세션을 요약 후 새 세션으로. 없으면 켜짐, 값 `0` 만 꺼짐.
pub const AUTO_COMPACT_KEY: &str = "auto_compact";

/// 컨텍스트 상한(토큰) · 자동 압축 문턱(80%) — 상한은 1M 고정(#1m 결정).
pub const CONTEXT_LIMIT: i64 = 1_000_000;
pub const AUTO_COMPACT_AT: i64 = CONTEXT_LIMIT / 10 * 8;

pub fn auto_compact(conn: &Connection) -> rusqlite::Result<bool> {
    Ok(get(conn, AUTO_COMPACT_KEY)?.as_deref() != Some("0"))
}

pub fn set(conn: &Connection, key: &str, value: &str) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = ?2",
        params![key, value],
    )
    .map(drop)
}
