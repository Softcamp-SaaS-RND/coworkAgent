//! 보내는 쪽 보관(D7 실현 — #1c). 상대에게 가는 대화 프레임을 먼저 적고, 상대별 줄이 응답(ack)을 받을 때까지 다시 보낸다. 응답을 받으면 행을
//! 지우고, 한도를 넘으면 `failed` 로 남긴다(실패 보관). 프레임 내용은 모른다 — 직렬화된 JSON 그대로.
use rusqlite::{Connection, OptionalExtension, Row, params};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    pub id: i64,
    pub peer_id: i64,
    pub conv_id: Option<i64>,
    pub msg_id: Option<i64>,
    pub kind: String,
    pub frame: String,
    pub key: Option<String>,
    /// `pending`(기다림) · `inflight`(지금 보내는 중) · `failed`(실패 보관).
    pub state: String,
    pub tries: i64,
    /// 앱이 켜져 있던 동안 다시 보내려고 기다린 초의 합 — 한도 판정.
    pub waited: i64,
    pub next_at: i64,
    pub last_error: Option<String>,
    /// 다음 시각까지 남은 초(0 = 지금 보낼 때).
    pub due_in: i64,
}

const COLS: &str =
    "id, peer_id, conv_id, msg_id, kind, frame, key, state, tries, waited, next_at, last_error, max(next_at - unixepoch(), 0)";

fn row(r: &Row) -> rusqlite::Result<Item> {
    Ok(Item {
        id: r.get(0)?,
        peer_id: r.get(1)?,
        conv_id: r.get(2)?,
        msg_id: r.get(3)?,
        kind: r.get(4)?,
        frame: r.get(5)?,
        key: r.get(6)?,
        state: r.get(7)?,
        tries: r.get(8)?,
        waited: r.get(9)?,
        next_at: r.get(10)?,
        last_error: r.get(11)?,
        due_in: r.get(12)?,
    })
}

/// 행을 넣는다(`state` = `inflight` 면 바로 보내는 중, `pending` 이면 줄에). `key` 가 같은 `pending` 행이 그 상대에 있으면 새 행 대신 그 행의
/// 프레임을 바꾼다(마지막 값만 — 진행 · 완료 표시). 돌려주는 값: 행 id.
#[allow(clippy::too_many_arguments)]
pub fn add(
    conn: &Connection,
    peer_id: i64,
    conv_id: Option<i64>,
    msg_id: Option<i64>,
    kind: &str,
    frame: &str,
    key: Option<&str>,
    state: &str,
) -> rusqlite::Result<i64> {
    if let Some(k) = key {
        let same: Option<i64> = conn
            .query_row(
                "SELECT id FROM outbox WHERE peer_id = ?1 AND key = ?2 AND state = 'pending'",
                params![peer_id, k],
                |r| r.get(0),
            )
            .optional()?;
        if let Some(id) = same {
            conn.execute("UPDATE outbox SET frame = ?2 WHERE id = ?1", params![id, frame])?;
            return Ok(id);
        }
    }
    conn.execute(
        "INSERT INTO outbox (peer_id, conv_id, msg_id, kind, frame, key, state, next_at, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, unixepoch(), unixepoch())",
        params![peer_id, conv_id, msg_id, kind, frame, key, state],
    )?;
    Ok(conn.last_insert_rowid())
}

/// 그 상대 줄의 맨 앞 — 보내는 중이거나 기다리는 가장 오래된 행(실패 보관은 빼고).
pub fn head(conn: &Connection, peer_id: i64) -> rusqlite::Result<Option<Item>> {
    conn.query_row(
        &format!(
            "SELECT {COLS} FROM outbox WHERE peer_id = ?1 AND state IN ('pending', 'inflight') ORDER BY id LIMIT 1"
        ),
        [peer_id],
        row,
    )
    .optional()
}

/// 같은 대화에 `id` 보다 앞선(보내는 중 · 기다리는) 행이 있나 — 대화 안 순서만 지킨다(다른 대화는 앞질러도 된다). 종류가 `except` 인 행은
/// 세지 않는다(앞질러도 되는 알림 — 내부 상수만).
pub fn has_before(
    conn: &Connection,
    conv_id: i64,
    id: i64,
    except: &[&str],
) -> rusqlite::Result<bool> {
    debug_assert!(
        except
            .iter()
            .all(|k| k.bytes().all(|b| b.is_ascii_lowercase() || b == b'_'))
    );
    let skip = except
        .iter()
        .map(|k| format!("'{k}'"))
        .collect::<Vec<_>>()
        .join(", ");
    let skip = if skip.is_empty() { "''".to_string() } else { skip };
    conn.prepare(&format!(
        "SELECT 1 FROM outbox WHERE conv_id = ?1 AND id < ?2 AND state IN ('pending', 'inflight') AND kind NOT IN ({skip})"
    ))?
    .exists(params![conv_id, id])
}

/// 그 상대에 닿지 않고 있나 — 다시 보내기를 기다리는 행(시도 1번 이상)이 있다.
pub fn failing(conn: &Connection, peer_id: i64) -> rusqlite::Result<bool> {
    conn.prepare("SELECT 1 FROM outbox WHERE peer_id = ?1 AND state = 'pending' AND tries > 0")?
        .exists([peer_id])
}

pub fn get(conn: &Connection, id: i64) -> rusqlite::Result<Option<Item>> {
    conn.query_row(&format!("SELECT {COLS} FROM outbox WHERE id = ?1"), [id], row)
        .optional()
}

pub fn set_state(conn: &Connection, id: i64, state: &str) -> rusqlite::Result<()> {
    conn.execute("UPDATE outbox SET state = ?2 WHERE id = ?1", params![id, state])?;
    Ok(())
}

/// 전송 오류 — 다음 시각 · 기다린 합 · 사유를 남기고 줄로 되돌린다. 보내는 동안 [`hurry`] 를 받았으면(`next_at` 0) 지금 다시.
/// 기다린 합에는 지난 시도부터 **실제로 흐른 시간**을 더하되 `cap`(직전 간격) 까지만 — 앞당긴 재시도가 한도를 부풀리지 않고, 앱을 꺼 둔 시간은
/// 세지 않는다. 돌려주는 값: 새 기다린 합(초).
pub fn retry(conn: &Connection, id: i64, delay: i64, cap: i64, error: &str) -> rusqlite::Result<i64> {
    conn.execute(
        "UPDATE outbox SET state = 'pending', tries = tries + 1,
         waited = waited + min(max(unixepoch() - coalesce(last_try, unixepoch()), 0), ?3),
         next_at = CASE WHEN next_at = 0 THEN unixepoch() ELSE unixepoch() + ?2 END,
         last_try = unixepoch(), last_error = ?4 WHERE id = ?1",
        params![id, delay, cap, error],
    )?;
    conn.query_row("SELECT waited FROM outbox WHERE id = ?1", [id], |r| r.get(0))
}

/// 그 상대 줄에 기다리는(`pending`) 행 — 줄 맨 앞을 포기할 때 함께 포기한다. 보내는 중(`inflight`)은 그 결과가 정한다.
pub fn queued_for(conn: &Connection, peer_id: i64) -> rusqlite::Result<Vec<Item>> {
    conn.prepare(&format!(
        "SELECT {COLS} FROM outbox WHERE peer_id = ?1 AND state = 'pending' ORDER BY id"
    ))?
    .query_map([peer_id], row)?
    .collect()
}

/// 오래된 실패 보관을 지운다(`days` 일 지난 것).
pub fn prune_failed(conn: &Connection, days: i64) -> rusqlite::Result<usize> {
    conn.execute(
        "DELETE FROM outbox WHERE state = 'failed' AND created_at < unixepoch() - ?1 * 86400",
        [days],
    )
}

/// 그 상대 줄을 지금 보내게 한다(상대가 돌아온 증거를 받았을 때 · 시작). 보내는 중인 행은 표시(`next_at` 0)만 — 그 시도가 실패하면 바로 다시.
pub fn hurry(conn: &Connection, peer_id: i64) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE outbox SET next_at = CASE state WHEN 'inflight' THEN 0 ELSE unixepoch() END
         WHERE peer_id = ?1 AND state IN ('pending', 'inflight')",
        [peer_id],
    )?;
    Ok(())
}

pub fn fail(conn: &Connection, id: i64, error: &str) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE outbox SET state = 'failed', last_error = ?2 WHERE id = ?1",
        params![id, error],
    )?;
    Ok(())
}

pub fn remove(conn: &Connection, id: i64) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM outbox WHERE id = ?1", [id])?;
    Ok(())
}

/// 보낼 것이 남은 상대(시작 시 줄을 깨운다). `inflight` 는 지난 실행이 보내다 끊긴 것 — `pending` 으로 되돌린다.
pub fn pending_peers(conn: &Connection) -> rusqlite::Result<Vec<i64>> {
    conn.execute("UPDATE outbox SET state = 'pending' WHERE state = 'inflight'", [])?;
    conn.prepare("SELECT DISTINCT peer_id FROM outbox WHERE state = 'pending' ORDER BY peer_id")?
        .query_map([], |r| r.get(0))?
        .collect()
}

/// 그 글의 가장 최근 행(보관 상태 표시용).
pub fn for_message(conn: &Connection, msg_id: i64) -> rusqlite::Result<Option<Item>> {
    conn.query_row(
        &format!("SELECT {COLS} FROM outbox WHERE msg_id = ?1 ORDER BY id DESC LIMIT 1"),
        [msg_id],
        row,
    )
    .optional()
}

/// 그 글의 실패 보관을 지운다(다시 보내기 · 다른 경로로 닿음).
pub fn clear_failed(conn: &Connection, msg_id: i64) -> rusqlite::Result<()> {
    conn.execute(
        "DELETE FROM outbox WHERE msg_id = ?1 AND state = 'failed'",
        [msg_id],
    )?;
    Ok(())
}
