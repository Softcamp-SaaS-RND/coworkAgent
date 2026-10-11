//! 피어 기록 — 상대 키(SPKI DER)로 식별한다. 페어링 = 내 승인(`my_ok`) ∧ 상대 승인(`their_ok`).
use rusqlite::{Connection, OptionalExtension, Row, params};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Peer {
    pub id: i64,
    pub spki: Vec<u8>,
    pub name: String,
    pub host: String,
    pub addr: String,
    pub my_ok: bool,
    pub their_ok: bool,
    /// 이번 페어링 세대 — 마지막 hello 의 `round`. approve 는 같은 세대에만 적용된다.
    pub round: String,
}

impl Peer {
    pub fn paired(&self) -> bool {
        self.my_ok && self.their_ok
    }
}

const COLS: &str = "id, spki, name, host, addr, my_ok, their_ok, round";

fn row(r: &Row) -> rusqlite::Result<Peer> {
    Ok(Peer {
        id: r.get(0)?,
        spki: r.get(1)?,
        name: r.get(2)?,
        host: r.get(3)?,
        addr: r.get(4)?,
        my_ok: r.get(5)?,
        their_ok: r.get(6)?,
        round: r.get(7)?,
    })
}

/// 없으면 만들고, 있으면 이름 · 호스트 · 주소를 바꾸고 **승인을 둘 다 지운다** — hello(주소 추가)는 페어링을 처음부터
/// 다시 하는 것이다. 삭제 통지가 유실된 뒤 다시 추가해도 양쪽이 같은 "승인 대기" 에서 출발한다.
pub fn upsert(
    conn: &Connection,
    spki: &[u8],
    name: &str,
    host: &str,
    addr: &str,
    round: &str,
) -> rusqlite::Result<Peer> {
    conn.execute(
        "INSERT INTO peers (spki, name, host, addr, round, created_at) VALUES (?1, ?2, ?3, ?4, ?5, unixepoch())
         ON CONFLICT(spki) DO UPDATE SET name = ?2, host = ?3, addr = ?4, round = ?5, my_ok = 0, their_ok = 0",
        params![spki, name, host, addr, round],
    )?;
    by_spki(conn, spki).map(|p| p.expect("방금 쓴 기록"))
}

pub fn by_spki(conn: &Connection, spki: &[u8]) -> rusqlite::Result<Option<Peer>> {
    conn.query_row(
        &format!("SELECT {COLS} FROM peers WHERE spki = ?1"),
        [spki],
        row,
    )
    .optional()
}

pub fn by_id(conn: &Connection, id: i64) -> rusqlite::Result<Option<Peer>> {
    conn.query_row(
        &format!("SELECT {COLS} FROM peers WHERE id = ?1"),
        [id],
        row,
    )
    .optional()
}

pub fn list(conn: &Connection) -> rusqlite::Result<Vec<Peer>> {
    conn.prepare(&format!("SELECT {COLS} FROM peers ORDER BY id"))?
        .query_map([], row)?
        .collect()
}

/// 내가 승인하지 않은 피어 수 — 원격 hello 로 쌓이는 기록의 상한 판정용.
pub fn count_unapproved(conn: &Connection) -> rusqlite::Result<i64> {
    conn.query_row("SELECT count(*) FROM peers WHERE my_ok = 0", [], |r| {
        r.get(0)
    })
}

pub fn set_my_ok(conn: &Connection, id: i64) -> rusqlite::Result<bool> {
    Ok(conn.execute("UPDATE peers SET my_ok = 1 WHERE id = ?1", [id])? == 1)
}

/// 같은 세대(`round`)의 기록에 상대 승인을 남긴다. 실제로 false → true 로 바뀐 경우만 `true`(재통지는 `false`).
pub fn set_their_ok(conn: &Connection, spki: &[u8], round: &str) -> rusqlite::Result<bool> {
    Ok(conn.execute(
        "UPDATE peers SET their_ok = 1 WHERE spki = ?1 AND round = ?2 AND their_ok = 0",
        params![spki, round],
    )? == 1)
}

pub fn remove(conn: &Connection, id: i64) -> rusqlite::Result<bool> {
    Ok(conn.execute("DELETE FROM peers WHERE id = ?1", [id])? == 1)
}

pub fn remove_spki(conn: &Connection, spki: &[u8]) -> rusqlite::Result<bool> {
    Ok(conn.execute("DELETE FROM peers WHERE spki = ?1", [spki])? == 1)
}
