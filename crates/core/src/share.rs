//! 공유 대화(#1o) — 대화를 가진 PC(A)의 관람자 · 보낸 커서, 관람자 PC(C)의 읽기 전용 사본(`dir = 'shared'`, 상대 = A), 원 상대 PC(B)의
//! 관람자 수. 사본 글은 실행 · 요청 · 내 글이 아니다(`mine = 0 · exec = 0 · request = 0`).
use crate::chat::{self, Author, Conversation, Dir};
use rusqlite::{Connection, OptionalExtension, params};

/// 대화당 관람자 상한.
pub const MAX_WATCHERS: usize = 5;

/// 관람자 한 명(A) — 보낸 마지막 글 id(`sent_upto`, 그 뒤 글을 보낸다).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Watcher {
    pub peer_id: i64,
    pub sent_upto: i64,
    pub created_at: i64,
}

/// 대화의 관람자 — 넣은 순서.
pub fn watchers(conn: &Connection, conversation_id: i64) -> rusqlite::Result<Vec<Watcher>> {
    conn.prepare(
        "SELECT peer_id, sent_upto, created_at FROM shares WHERE conversation_id = ?1 ORDER BY created_at, peer_id",
    )?
    .query_map([conversation_id], |r| {
        Ok(Watcher {
            peer_id: r.get(0)?,
            sent_upto: r.get(1)?,
            created_at: r.get(2)?,
        })
    })?
    .collect()
}

/// 관람자를 넣는다(커서 0 — 이력 전체를 보낸다). 이미 있으면 거짓.
pub fn add(conn: &Connection, conversation_id: i64, peer_id: i64) -> rusqlite::Result<bool> {
    Ok(conn.execute(
        "INSERT OR IGNORE INTO shares (conversation_id, peer_id, sent_upto, created_at) VALUES (?1, ?2, 0, unixepoch())",
        params![conversation_id, peer_id],
    )? == 1)
}

/// 관람자를 뺀다 — 그 관람자 앞 기다리는 공유 프레임(그 대화)도. 없었으면 거짓.
pub fn remove(conn: &Connection, conversation_id: i64, peer_id: i64) -> rusqlite::Result<bool> {
    conn.execute(
        "DELETE FROM outbox WHERE peer_id = ?1 AND conv_id = ?2 AND kind = ?3 AND state = 'pending'",
        params![peer_id, conversation_id, crate::outbox::SHARE],
    )?;
    Ok(conn.execute(
        "DELETE FROM shares WHERE conversation_id = ?1 AND peer_id = ?2",
        params![conversation_id, peer_id],
    )? == 1)
}

/// 보낸 커서를 올린다(뒤로 가지 않는다).
pub fn set_sent(
    conn: &Connection,
    conversation_id: i64,
    peer_id: i64,
    upto: i64,
) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE shares SET sent_upto = max(sent_upto, ?3) WHERE conversation_id = ?1 AND peer_id = ?2",
        params![conversation_id, peer_id, upto],
    )
    .map(drop)
}

/// 상대를 지울 때(페어링 해제) — 그 상대가 관람자인 공유 · 그 상대 앞 공유 · 나가기(#1w) 프레임. 그 상대가 보낸 사본은 남긴다.
pub fn forget_peer(conn: &Connection, peer_id: i64) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM shares WHERE peer_id = ?1", [peer_id])?;
    conn.execute(
        "DELETE FROM outbox WHERE peer_id = ?1 AND kind IN (?2, ?3, ?4)",
        params![
            peer_id,
            crate::outbox::SHARE,
            crate::outbox::SHARE_NOTICE,
            crate::outbox::LEAVE
        ],
    )
    .map(drop)
}

/// 공유 사본이 아닌 대화를 uuid 로(A — 관람자가 보낸 나가기). uuid 는 이 PC 에 하나뿐이다.
pub fn own_by_uuid(conn: &Connection, uuid: &str) -> rusqlite::Result<Option<Conversation>> {
    let id: Option<i64> = conn
        .query_row(
            "SELECT id FROM conversations WHERE uuid = ?1 AND dir <> 'shared'",
            [uuid],
            |r| r.get(0),
        )
        .optional()?;
    match id {
        Some(id) => chat::by_id(conn, id),
        None => Ok(None),
    }
}

/// 사본의 공유 정보(C) — 원 상대 이름 · 원 방향(`out` · `in`) · 빠짐. 사본이 아니면 `None`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Copy {
    pub other: String,
    pub owner_dir: String,
    pub ended: bool,
}

pub fn copy(conn: &Connection, conversation_id: i64) -> rusqlite::Result<Option<Copy>> {
    conn.query_row(
        "SELECT coalesce(share_other, ''), coalesce(share_dir, ''), coalesce(share_ended, 0) FROM conversations
         WHERE id = ?1 AND dir = 'shared'",
        [conversation_id],
        |r| {
            Ok(Copy {
                other: r.get(0)?,
                owner_dir: r.get(1)?,
                ended: r.get(2)?,
            })
        },
    )
    .optional()
}

/// 사본 머리(C) — 대화를 가진 상대(`owner`)의 사본이 있으면 글을 비우고 머리를 새로(다시 초대), 없으면 만든다. 같은 uuid 가 이 PC 에 다른
/// 대화로 있으면(내 대화 · 다른 상대의 것) `None` — 남의 대화에 끼어들지 못하게.
pub fn open_copy(
    conn: &Connection,
    uuid: &str,
    owner: i64,
    agent: &str,
    title: Option<&str>,
    (other, owner_dir): (&str, &str),
) -> rusqlite::Result<Option<Conversation>> {
    let conv = match chat::find(conn, uuid, owner)? {
        Some(c) if c.dir == Dir::Shared => {
            for m in chat::messages(conn, c.id)? {
                chat::delete_message(conn, m.id)?;
            }
            c
        }
        Some(_) => return Ok(None),
        None => {
            let taken = conn
                .prepare("SELECT 1 FROM conversations WHERE uuid = ?1")?
                .exists([uuid])?;
            if taken {
                return Ok(None);
            }
            chat::open_conversation(conn, uuid, owner, agent, Dir::Shared)?
        }
    };
    conn.execute(
        "UPDATE conversations SET agent = ?2, title = ?3, share_other = ?4, share_dir = ?5, share_ended = 0 WHERE id = ?1",
        params![conv.id, agent, title, other, owner_dir],
    )?;
    chat::by_id(conn, conv.id)
}

/// 사본 글 하나(C) — 받은 값은 부르는 쪽이 이미 거른 것(본문 크기 · 이름).
#[derive(Debug, Clone, Copy)]
pub struct Post<'a> {
    pub uuid: &'a str,
    pub author: Author,
    pub text: &'a str,
    pub lc: Option<i64>,
    pub at: i64,
    pub owner: bool,
    pub via: Option<&'a str>,
    pub agent: Option<&'a str>,
    pub ok: bool,
    pub edited: bool,
}

/// 사본에 글을 넣거나(같은 uuid 가 이 사본 것이면) 고친다. 같은 uuid 가 다른 대화의 글이면 거짓(넣지 않는다). 순서 · 시각이 범위 밖이면
/// 이 PC 가 매긴다(`chat::observe` 와 같은 상한).
pub fn upsert(conn: &Connection, copy_id: i64, p: &Post<'_>) -> rusqlite::Result<bool> {
    let state = if p.ok { "done" } else { "error" };
    let via = p.via.filter(|v| *v == chat::VIA_CLI);
    let lc = p.lc.filter(|v| (1..=chat::MAX_LC).contains(v));
    match chat::message_by_uuid(conn, p.uuid)? {
        Some(m) if m.conversation_id == copy_id => {
            conn.execute(
                "UPDATE messages SET author = ?2, text = ?3, state = ?4, via = ?5, agent = ?6, owner = ?7,
                   edited_at = CASE WHEN ?8 THEN coalesce(edited_at, unixepoch()) END, lc = coalesce(?9, lc)
                 WHERE id = ?1",
                params![
                    m.id,
                    p.author.as_str(),
                    p.text,
                    state,
                    via,
                    p.agent,
                    p.owner,
                    p.edited,
                    lc
                ],
            )?;
            Ok(true)
        }
        Some(_) => Ok(false),
        None => {
            conn.execute(
                "INSERT INTO messages (uuid, conversation_id, author, text, state, created_at, mine, exec, request, via, agent, owner,
                   edited_at, lc)
                 VALUES (?1, ?2, ?3, ?4, ?5, CASE WHEN ?6 BETWEEN 1 AND unixepoch() + 86400 THEN ?6 ELSE unixepoch() END, 0, 0, 0, ?7, ?8, ?9,
                   CASE WHEN ?10 THEN unixepoch() END,
                   coalesce(?11, (SELECT coalesce(max(lc), 0) + 1 FROM messages WHERE conversation_id = ?2)))",
                params![
                    p.uuid,
                    copy_id,
                    p.author.as_str(),
                    p.text,
                    state,
                    p.at,
                    via,
                    p.agent,
                    p.owner,
                    p.edited,
                    lc
                ],
            )?;
            Ok(true)
        }
    }
}

/// 사본 글을 지운다(C) — 이 사본의 글만(없으면 그대로).
pub fn delete(conn: &Connection, copy_id: i64, uuid: &str) -> rusqlite::Result<()> {
    if let Some(m) = chat::message_by_uuid(conn, uuid)?.filter(|m| m.conversation_id == copy_id) {
        chat::delete_message(conn, m.id)?;
    }
    Ok(())
}

/// 사본의 공유 끝(C — 빠짐 · 나감). 사본은 남는다. 이미 끝났으면 거짓.
pub fn end(conn: &Connection, copy_id: i64) -> rusqlite::Result<bool> {
    Ok(conn.execute(
        "UPDATE conversations SET share_ended = 1 WHERE id = ?1 AND dir = 'shared' AND coalesce(share_ended, 0) = 0",
        [copy_id],
    )? == 1)
}

/// 원 상대(B)가 받은 그 대화의 관람자 수. 바뀌었으면 참.
pub fn set_watched(conn: &Connection, conversation_id: i64, n: i64) -> rusqlite::Result<bool> {
    Ok(conn.execute(
        "UPDATE conversations SET watchers = ?2 WHERE id = ?1 AND coalesce(watchers, 0) <> ?2",
        params![conversation_id, n],
    )? == 1)
}

/// 원 상대(B) 쪽 관람자 수(모르면 0).
pub fn watched(conn: &Connection, conversation_id: i64) -> rusqlite::Result<i64> {
    conn.query_row(
        "SELECT coalesce(watchers, 0) FROM conversations WHERE id = ?1",
        [conversation_id],
        |r| r.get(0),
    )
    .optional()
    .map(Option::unwrap_or_default)
}
