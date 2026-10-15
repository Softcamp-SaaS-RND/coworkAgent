//! 레일 그룹(#1i) — 이 PC 에만 있는 대화 묶음(한 대화는 여러 그룹에) · 상대 · 그룹 · 대화의 손으로 놓은 자리(`pos`).
//! 상대 PC 에 알리지 않는다. 외래 키를 켜지 않는 DB 라 그룹을 지우면 소속도 여기서 함께 지운다.
use rusqlite::{Connection, OptionalExtension, params};
use std::collections::HashSet;

/// 그룹 이름 길이(글자).
pub const NAME_CHARS: usize = 40;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Group {
    pub id: i64,
    pub name: String,
    /// 타일 그림 — PNG data URL(`run_opts::clean_icon`), 없으면 머리글자.
    pub icon: Option<String>,
    pub pos: i64,
    /// 소속 대화 id(작은 것부터).
    pub convs: Vec<i64>,
}

/// 손으로 놓는 자리가 있는 목록.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Order {
    Peers,
    Groups,
    Conversations,
}

impl Order {
    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "peers" => Order::Peers,
            "groups" => Order::Groups,
            "conversations" => Order::Conversations,
            _ => return None,
        })
    }

    pub fn table(self) -> &'static str {
        match self {
            Order::Peers => "peers",
            Order::Groups => "groups",
            Order::Conversations => "conversations",
        }
    }
}

/// 이름 — 앞뒤 공백을 빼고 1 ~ [`NAME_CHARS`]자, 제어 문자 없이.
pub fn clean_name(name: &str) -> Result<String, String> {
    let n = name.trim();
    if n.is_empty() || n.chars().count() > NAME_CHARS || n.chars().any(char::is_control) {
        return Err(format!("그룹 이름은 1 ~ {NAME_CHARS}자, 제어 문자 없이"));
    }
    Ok(n.to_string())
}

/// 레일 순서(자리, 같으면 id).
pub fn list(conn: &Connection) -> rusqlite::Result<Vec<Group>> {
    let mut groups: Vec<Group> = conn
        .prepare("SELECT id, name, icon, pos FROM groups ORDER BY pos, id")?
        .query_map([], |r| {
            Ok(Group {
                id: r.get(0)?,
                name: r.get(1)?,
                icon: r.get(2)?,
                pos: r.get(3)?,
                convs: Vec::new(),
            })
        })?
        .collect::<rusqlite::Result<_>>()?;
    let pairs: Vec<(i64, i64)> = conn
        .prepare("SELECT group_id, conversation_id FROM group_convs ORDER BY conversation_id")?
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<rusqlite::Result<_>>()?;
    for g in &mut groups {
        g.convs = pairs
            .iter()
            .filter(|(gid, _)| *gid == g.id)
            .map(|(_, c)| *c)
            .collect();
    }
    Ok(groups)
}

pub fn by_id(conn: &Connection, id: i64) -> rusqlite::Result<Option<Group>> {
    Ok(list(conn)?.into_iter().find(|g| g.id == id))
}

/// 같은 이름(글자 그대로)의 그룹 id — `except` 는 빼고(이름 바꾸기).
pub fn name_taken(conn: &Connection, name: &str, except: Option<i64>) -> rusqlite::Result<bool> {
    conn.query_row(
        "SELECT 1 FROM groups WHERE name = ?1 AND (?2 IS NULL OR id <> ?2)",
        params![name, except],
        |_| Ok(()),
    )
    .optional()
    .map(|o| o.is_some())
}

/// 새 그룹은 레일 그룹 끝에. 이름은 [`clean_name`] 을 거친 값.
pub fn add(conn: &Connection, name: &str) -> rusqlite::Result<Group> {
    conn.execute(
        "INSERT INTO groups (name, pos) VALUES (?1, (SELECT coalesce(max(pos), -1) + 1 FROM groups))",
        [name],
    )?;
    by_id(conn, conn.last_insert_rowid()).map(|g| g.expect("방금 쓴 그룹"))
}

/// 없는 그룹이면 `false`.
pub fn rename(conn: &Connection, id: i64, name: &str) -> rusqlite::Result<bool> {
    Ok(conn.execute(
        "UPDATE groups SET name = ?2 WHERE id = ?1",
        params![id, name],
    )? == 1)
}

/// 그림(`None` = 지움). 없는 그룹이면 `false`.
pub fn set_icon(conn: &Connection, id: i64, icon: Option<&str>) -> rusqlite::Result<bool> {
    Ok(conn.execute(
        "UPDATE groups SET icon = ?2 WHERE id = ?1",
        params![id, icon],
    )? == 1)
}

/// 그룹과 그 소속을 지운다(대화는 그대로). 없는 그룹이면 `false`.
pub fn remove(conn: &Connection, id: i64) -> rusqlite::Result<bool> {
    let tx = crate::db::write_tx(conn)?;
    tx.execute("DELETE FROM group_convs WHERE group_id = ?1", [id])?;
    let gone = tx.execute("DELETE FROM groups WHERE id = ?1", [id])? == 1;
    tx.commit()?;
    Ok(gone)
}

/// 없는 대화 id(준 순서대로, 겹치면 한 번).
pub fn unknown_conversations(conn: &Connection, ids: &[i64]) -> rusqlite::Result<Vec<i64>> {
    let mut seen = HashSet::new();
    let mut stmt = conn.prepare("SELECT 1 FROM conversations WHERE id = ?1")?;
    let mut out = Vec::new();
    for &id in ids {
        if seen.insert(id) && !stmt.exists([id])? {
            out.push(id);
        }
    }
    Ok(out)
}

/// 소속 넣고 빼기 — 이미 있는 것 · 없는 것은 그대로 둔다. 대화 id 검증은 부르는 쪽([`unknown_conversations`]). 없는 그룹이면 `false`.
pub fn change(conn: &Connection, id: i64, add: &[i64], remove: &[i64]) -> rusqlite::Result<bool> {
    let tx = crate::db::write_tx(conn)?;
    let exists = tx
        .prepare("SELECT 1 FROM groups WHERE id = ?1")?
        .exists([id])?;
    if exists {
        for c in add {
            tx.execute(
                "INSERT OR IGNORE INTO group_convs (group_id, conversation_id) VALUES (?1, ?2)",
                params![id, c],
            )?;
        }
        for c in remove {
            tx.execute(
                "DELETE FROM group_convs WHERE group_id = ?1 AND conversation_id = ?2",
                params![id, c],
            )?;
        }
    }
    tx.commit()?;
    Ok(exists)
}

/// 새 대화를 그룹에 — 부르는 쪽 트랜잭션 안에서(대화와 함께). 없는 그룹이면 `false`(아무것도 하지 않음).
pub fn join(conn: &Connection, id: i64, conversation: i64) -> rusqlite::Result<bool> {
    Ok(conn.execute(
        "INSERT OR IGNORE INTO group_convs (group_id, conversation_id) SELECT id, ?2 FROM groups WHERE id = ?1",
        params![id, conversation],
    )? == 1)
}

/// 준 순서대로 자리 `0..n`(빠진 id 는 제 자리 그대로 — 같은 자리면 id 순). 겹친 id · 없는 id 면 아무것도 바꾸지 않고 `Err(문구)`.
pub fn reorder(
    conn: &Connection,
    kind: Order,
    ids: &[i64],
) -> rusqlite::Result<Result<(), String>> {
    // 개수 상한은 두지 않는다 — 웹UI 는 전체 순서를 보내고 대화는 지워지지 않아 늘기만 한다(라운드 2). 요청 크기는 서버 본문 상한(2 MiB)이 막는다.
    let mut seen = HashSet::new();
    if let Some(d) = ids.iter().find(|id| !seen.insert(**id)) {
        return Ok(Err(format!("겹친 id {d}")));
    }
    let table = kind.table();
    let tx = crate::db::write_tx(conn)?;
    {
        let mut has = tx.prepare(&format!("SELECT 1 FROM {table} WHERE id = ?1"))?;
        for &id in ids {
            if !has.exists([id])? {
                return Ok(Err(format!("없는 id {id}")));
            }
        }
        let mut set = tx.prepare(&format!("UPDATE {table} SET pos = ?2 WHERE id = ?1"))?;
        for (i, &id) in ids.iter().enumerate() {
            set.execute(params![id, i as i64])?;
        }
    }
    tx.commit()?;
    Ok(Ok(()))
}
