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
    /// 사용자가 붙인 그룹 이름(없으면 `None`).
    pub group: Option<String>,
    /// 이 상대 요청 실행의 model · effort 덮어쓰기(없으면 전역 기본값).
    pub model: Option<String>,
    pub effort: Option<String>,
    /// 이 PC 에서 부르는 표시 이름(별칭) · 타일 아이콘(PNG data URL) — 없으면 `None`.
    pub alias: Option<String>,
    pub icon: Option<String>,
    /// 접속 상태 — 마지막으로 주고받기에 성공한 시각(유닉스 초, 없으면 아직) · 마지막 `Pong` · 거절 때 상대의 집중.
    pub seen_at: Option<i64>,
    pub focus: bool,
}

/// "방금 통신" 으로 보는 마지막 통신 이후 시간(초). 주기 확인은 없다 — 통신할 때만 갱신.
pub const ONLINE_WITHIN: i64 = 75;

impl Peer {
    pub fn paired(&self) -> bool {
        self.my_ok && self.their_ok
    }

    /// 마지막 통신이 `now` 에서 [`ONLINE_WITHIN`] 초 안이면 온라인. 받은 시각이 `now` 보다 뒤(시계를 되돌림)면 오프라인으로 본다.
    pub fn online(&self, now: i64) -> bool {
        self.seen_at
            .is_some_and(|t| (0..=ONLINE_WITHIN).contains(&(now - t)))
    }
}

const COLS: &str = "id, spki, name, host, addr, my_ok, their_ok, round, grp, model, effort, alias, icon, seen_at, focus";

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
        group: r.get(8)?,
        model: r.get(9)?,
        effort: r.get(10)?,
        alias: r.get(11)?,
        icon: r.get(12)?,
        seen_at: r.get(13)?,
        focus: r.get::<_, Option<bool>>(14)?.unwrap_or(false),
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
        "INSERT INTO peers (spki, name, host, addr, round, created_at, pos)
         VALUES (?1, ?2, ?3, ?4, ?5, unixepoch(), (SELECT coalesce(max(pos), -1) + 1 FROM peers))
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
    // 레일 순서(손으로 놓은 자리, 같으면 id).
    conn.prepare(&format!("SELECT {COLS} FROM peers ORDER BY pos, id"))?
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

/// 상대를 지운다 — 그 상대가 관람자인 공유와 그 상대 앞 공유 프레임도(그 상대가 보낸 사본은 남는다).
pub fn remove(conn: &Connection, id: i64) -> rusqlite::Result<bool> {
    crate::share::forget_peer(conn, id)?;
    Ok(conn.execute("DELETE FROM peers WHERE id = ?1", [id])? == 1)
}

/// 그룹 · model · effort 를 통째로 바꾼다(`None` = 지움). 값 검증은 [`crate::run_opts`].
pub fn set_meta(
    conn: &Connection,
    id: i64,
    group: Option<&str>,
    model: Option<&str>,
    effort: Option<&str>,
) -> rusqlite::Result<bool> {
    Ok(conn.execute(
        "UPDATE peers SET grp = ?2, model = ?3, effort = ?4 WHERE id = ?1",
        params![id, group, model, effort],
    )? == 1)
}

/// [`remove`] 를 키로(상대의 삭제 통지).
pub fn remove_spki(conn: &Connection, spki: &[u8]) -> rusqlite::Result<bool> {
    if let Some(p) = by_spki(conn, spki)? {
        crate::share::forget_peer(conn, p.id)?;
    }
    Ok(conn.execute("DELETE FROM peers WHERE spki = ?1", [spki])? == 1)
}

/// 상대 수신 주소를 바꾼다(승인 · 세대 그대로) — 페어링된 키가 다른 출처 · 포트로 `Ping` 했을 때.
pub fn set_addr(conn: &Connection, id: i64, addr: &str) -> rusqlite::Result<bool> {
    Ok(conn.execute(
        "UPDATE peers SET addr = ?2 WHERE id = ?1",
        params![id, addr],
    )? == 1)
}

/// 접속 상태 — 상대가 지금 응답했다(`Pong` · 실행 요청 거절), 그때의 집중과 함께. 없는 상대면 `false`.
pub fn set_presence(conn: &Connection, id: i64, focus: bool) -> rusqlite::Result<bool> {
    Ok(conn.execute(
        "UPDATE peers SET seen_at = unixepoch(), focus = ?2 WHERE id = ?1",
        params![id, focus],
    )? == 1)
}

/// 통신 성공 — 그 키의 상대가 응답했거나 요청을 보냈다(집중은 그대로). 없는 키면 아무 일도 없다. 10초 안에 이미 적었으면 쓰지 않는다
/// (프레임마다 쓰기 잠금을 잡아 다른 연결의 트랜잭션을 막지 않게 — 읽기만으로 거른다).
pub fn touch(conn: &Connection, spki: &[u8]) -> rusqlite::Result<()> {
    let fresh: Option<bool> = conn
        .query_row(
            "SELECT coalesce(seen_at > unixepoch() - 10, 0) FROM peers WHERE spki = ?1",
            [spki],
            |r| r.get(0),
        )
        .optional()?;
    if fresh == Some(false) {
        conn.execute(
            "UPDATE peers SET seen_at = unixepoch() WHERE spki = ?1",
            [spki],
        )?;
    }
    Ok(())
}

/// 표시 이름 · 타일 아이콘을 통째로 바꾼다(`None` = 지움). 값 검증은 [`crate::run_opts`].
pub fn set_tile(
    conn: &Connection,
    id: i64,
    alias: Option<&str>,
    icon: Option<&str>,
) -> rusqlite::Result<bool> {
    Ok(conn.execute(
        "UPDATE peers SET alias = ?2, icon = ?3 WHERE id = ?1",
        params![id, alias, icon],
    )? == 1)
}

#[cfg(test)]
mod tests {
    use super::{ONLINE_WITHIN, Peer};

    #[test]
    fn online_window_edges() {
        let peer = |seen_at| Peer {
            id: 1,
            spki: Vec::new(),
            name: String::new(),
            host: String::new(),
            addr: String::new(),
            my_ok: true,
            their_ok: true,
            round: String::new(),
            group: None,
            model: None,
            effort: None,
            alias: None,
            icon: None,
            seen_at,
            focus: false,
        };
        let now = 1_000;
        assert!(peer(Some(now - ONLINE_WITHIN + 1)).online(now));
        assert!(peer(Some(now - ONLINE_WITHIN)).online(now));
        assert!(!peer(Some(now - ONLINE_WITHIN - 1)).online(now));
        assert!(!peer(Some(now + 1)).online(now), "시계를 되돌림");
        assert!(!peer(None).online(now));
    }
}
