//! 대화 — 열기 · 찾기 · 목록 · 제목 · 별 · 읽음 · 나가기 · 지우기 · 비우기 · 요약 · 정보.

use super::*;

const CONV_COLS: &str = "id, uuid, peer_id, agent, dir, session, relay_agent, relay_cap, relay_count, relay_on, relay_blocked, left_at, peer_left_at";

/// `CONV_COLS` 열 수 — 그 뒤에 붙는 열의 위치.
const CONV_N: usize = 13;

fn conv_row(r: &Row) -> rusqlite::Result<Conversation> {
    let dir: String = r.get(4)?;
    Ok(Conversation {
        id: r.get(0)?,
        uuid: r.get(1)?,
        peer_id: r.get(2)?,
        agent: r.get(3)?,
        dir: match dir.as_str() {
            "out" => Dir::Out,
            "in" => Dir::In,
            "shared" => Dir::Shared,
            v => return Err(bad(4, v)),
        },
        session: r.get(5)?,
        relay: r
            .get::<_, Option<String>>(6)?
            .map(|agent| -> rusqlite::Result<Relay> {
                Ok(Relay {
                    agent,
                    cap: r.get::<_, Option<i64>>(7)?.unwrap_or(0),
                    count: r.get::<_, Option<i64>>(8)?.unwrap_or(0),
                    on: r.get::<_, Option<bool>>(9)?.unwrap_or(false),
                })
            })
            .transpose()?,
        relay_blocked: r.get::<_, Option<bool>>(10)?.unwrap_or(false),
        left_at: r.get(11)?,
        peer_left_at: r.get(12)?,
    })
}

/// 같은 uuid · 같은 상대의 대화가 있으면 그것을, 없으면 새로. 다른 상대의 uuid 와 겹치면 오류(남의 대화에 끼어들기 차단).
pub fn open_conversation(
    conn: &Connection,
    uuid: &str,
    peer_id: i64,
    agent: &str,
    dir: Dir,
) -> rusqlite::Result<Conversation> {
    if let Some(c) = find(conn, uuid, peer_id)? {
        return Ok(c);
    }
    // 지운 대화(묘비)는 되살리지 않는다 — 받기 앞 확인과 삭제 사이에 끼어든 글도 여기서 막힌다.
    if conn
        .prepare("SELECT 1 FROM left_convs WHERE uuid = ?1 AND peer_id = ?2")?
        .exists(params![uuid, peer_id])?
    {
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }
    conn.execute(
        // 새 대화는 맨 위(손으로 놓은 순서의 앞).
        "INSERT INTO conversations (uuid, peer_id, agent, dir, created_at, pos)
         VALUES (?1, ?2, ?3, ?4, unixepoch(), (SELECT coalesce(min(pos), 0) - 1 FROM conversations))",
        params![uuid, peer_id, agent, dir.as_str()],
    )?;
    by_id(conn, conn.last_insert_rowid()).map(|c| c.expect("방금 쓴 대화"))
}

/// [`find`] 중 공유 사본이 아닌 것 — 상대의 대화 프레임은 이 PC 의 사본을 건드리지 못한다(사본은 `share` 만 쓴다).
pub fn find_own(
    conn: &Connection,
    uuid: &str,
    peer_id: i64,
) -> rusqlite::Result<Option<Conversation>> {
    Ok(find(conn, uuid, peer_id)?.filter(|c| c.dir != Dir::Shared))
}

/// [`find_own`] 중 닫히지 않은 것 — 상대의 대화 프레임이 이력을 바꾸는 받기 처리기가 쓴다(받기 앞 `left` 확인 뒤 나가기가 끼어도 바꾸기와 같은
/// 잠금 안에서 다시 걸러진다). 닫힌 대화는 `None`(= `unknown`).
pub fn find_open(
    conn: &Connection,
    uuid: &str,
    peer_id: i64,
) -> rusqlite::Result<Option<Conversation>> {
    Ok(find_own(conn, uuid, peer_id)?.filter(|c| !c.closed()))
}

pub fn find(conn: &Connection, uuid: &str, peer_id: i64) -> rusqlite::Result<Option<Conversation>> {
    conn.query_row(
        &format!("SELECT {CONV_COLS} FROM conversations WHERE uuid = ?1 AND peer_id = ?2"),
        params![uuid, peer_id],
        conv_row,
    )
    .optional()
}

pub fn by_id(conn: &Connection, id: i64) -> rusqlite::Result<Option<Conversation>> {
    conn.query_row(
        &format!("SELECT {CONV_COLS} FROM conversations WHERE id = ?1"),
        [id],
        conv_row,
    )
    .optional()
}

pub fn list(conn: &Connection) -> rusqlite::Result<Vec<Conversation>> {
    conn.prepare(&format!(
        "SELECT {CONV_COLS} FROM conversations ORDER BY id DESC"
    ))?
    .query_map([], conv_row)?
    .collect()
}

/// 이 PC 에서 붙인 대화 제목(`None` = 지움 → 첫 사람 글 앞부분) — 상대에 알리지 않는다. 값 검증은
/// [`crate::run_opts::clean_title`]. 없는 대화면 `false`.
pub fn set_title(conn: &Connection, id: i64, title: Option<&str>) -> rusqlite::Result<bool> {
    Ok(conn.execute(
        "UPDATE conversations SET title = ?2 WHERE id = ?1",
        params![id, title],
    )? == 1)
}

/// ★ 켜기 · 끄기(이 PC 에만 — 상대에 알리지 않는다). 이미 켜져 있으면 켠 시각은 그대로. 없는 대화면 `false`.
pub fn set_starred(conn: &Connection, id: i64, on: bool) -> rusqlite::Result<bool> {
    Ok(conn.execute(
        "UPDATE conversations SET starred_at = CASE WHEN ?2 THEN coalesce(starred_at, unixepoch()) END WHERE id = ?1",
        params![id, on],
    )? == 1)
}

/// 이 PC 에서 붙인 표시 — (붙인 제목, ★). 없는 대화면 `None`.
pub fn marks(conn: &Connection, id: i64) -> rusqlite::Result<Option<(Option<String>, bool)>> {
    conn.query_row(
        "SELECT title, starred_at IS NOT NULL FROM conversations WHERE id = ?1",
        [id],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )
    .optional()
}

/// 대화를 `upto`(화면이 받은 마지막 글 id)까지 읽음으로(이 PC 에만) — 받은 뒤 온 글은 안 읽음으로 남고, 읽은 위치는 뒤로 가지 않는다.
/// 없는 대화 · `upto` 가 그 대화 글이 아니면 `false`.
pub fn mark_read(conn: &Connection, id: i64, upto: i64) -> rusqlite::Result<bool> {
    Ok(conn.execute(
        "UPDATE conversations SET read_id = max(coalesce(read_id, 0), ?2)
         WHERE id = ?1 AND EXISTS (SELECT 1 FROM messages WHERE id = ?2 AND conversation_id = ?1)",
        [id, upto],
    )? == 1)
}

/// 나간 대화의 거절 코드 — 나간 쪽이 그 대화로 온 요청에 답하고, 나가며 닫은 글에 남긴다.
pub const LEFT: &str = "left";

/// 닫힌 대화 · 지운 대화(묘비)인가 — 이 PC 가 나갔거나 그 상대가 나갔다(`Leave` 를 보낸 상대의 글도 받지 않는다 — 원격 입력은 신뢰하지 않는다).
/// 공유 사본은 아니다(사본은 `share` 규칙).
pub fn left(conn: &Connection, uuid: &str, peer_id: i64) -> rusqlite::Result<bool> {
    conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM conversations WHERE uuid = ?1 AND peer_id = ?2 AND dir <> 'shared'
             AND (left_at IS NOT NULL OR peer_left_at IS NOT NULL))
           OR EXISTS (SELECT 1 FROM left_convs WHERE uuid = ?1 AND peer_id = ?2)",
        params![uuid, peer_id],
        |r| r.get(0),
    )
}

/// 기록 삭제로 지운 글 · 적용한 `Clear` 의 uuid 인가(받은 글 · 지우기의 중복 확인).
pub fn gone(conn: &Connection, uuid: &str) -> rusqlite::Result<bool> {
    conn.prepare("SELECT 1 FROM gone_msgs WHERE uuid = ?1")?.exists([uuid])
}

/// 적용한 `Clear` 를 남긴다 — 같은 지우기가 다시 와도 두 번 지우지 않는다.
pub fn mark_gone(conn: &Connection, uuid: &str, conversation_id: i64) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO gone_msgs (uuid, conversation_id) VALUES (?1, ?2)",
        params![uuid, conversation_id],
    )
    .map(drop)
}

/// 글이 하나도 없는 대화인가(세션 생성으로 연 대화의 첫 글).
pub fn empty(conn: &Connection, id: i64) -> rusqlite::Result<bool> {
    conn.query_row(
        "SELECT NOT EXISTS (SELECT 1 FROM messages WHERE conversation_id = ?1)",
        [id],
        |r| r.get(0),
    )
}

/// 실행 중이거나 실행을 기다리는 글이 있는가(기록 삭제를 막는다: 대기열이 지운 글을 찾지 않게).
pub fn busy(conn: &Connection, id: i64) -> rusqlite::Result<bool> {
    conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM messages WHERE conversation_id = ?1 AND state IN ('running', 'queued'))",
        [id],
        |r| r.get(0),
    )
}

/// 끝나지 않은 글을 닫는다 — 실행 대기(`queued`) · 보낸 뒤 상대를 기다리는 내 글(`sent`)은 실패 + [`LEFT`], 자동 왕복은 끈다.
fn close_pending(conn: &Connection, id: i64) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE messages SET state = 'error', refused = ?2
         WHERE conversation_id = ?1 AND (state = 'queued' OR (mine = 1 AND state = 'sent'))",
        params![id, LEFT],
    )?;
    conn.execute("UPDATE conversations SET relay_on = 0 WHERE id = ?1", [id])
        .map(drop)
}

/// 이 PC 가 나간다 — 끝나지 않은 글 닫기 · 자동 왕복 끔 · 나간 시각(이미 나갔으면 그 시각 그대로). 없는 대화면 거짓.
pub fn leave(conn: &Connection, id: i64) -> rusqlite::Result<bool> {
    close_pending(conn, id)?;
    Ok(conn.execute(
        "UPDATE conversations SET left_at = coalesce(left_at, unixepoch()) WHERE id = ?1",
        [id],
    )? == 1)
}

/// 상대가 나갔다(받은 `Leave`) — [`leave`] 와 같이 닫고 상대 나간 시각. 이미 받았으면 거짓.
pub fn set_peer_left(conn: &Connection, id: i64) -> rusqlite::Result<bool> {
    close_pending(conn, id)?;
    Ok(conn.execute(
        "UPDATE conversations SET peer_left_at = unixepoch() WHERE id = ?1 AND peer_left_at IS NULL",
        [id],
    )? == 1)
}

/// 대화를 이 PC 에서 지운다 — 글 · 실행 · 에이전트 세션 기록 · 요약 · 그룹 소속 · 공유 · 보낼 글 · 기록 · CLI 트랜잭션. 끝 알림 행
/// (`Leave` — `conv_id` 없이 넣는다 · 관람자 앞 `ShareEnd`)은 대화에서 떼어 남긴다(상대 · 관람자가 끝을 받게). 공유 사본이 아니면 묘비를 남긴다
/// — 같은 uuid 로 온 글이 대화를 되살리지 않게(사본은 다시 초대로 새로 받는다). 없는 대화면 거짓.
pub fn delete_conversation(conn: &Connection, id: i64) -> rusqlite::Result<bool> {
    conn.execute(
        "INSERT OR IGNORE INTO left_convs (uuid, peer_id, at)
         SELECT uuid, peer_id, unixepoch() FROM conversations WHERE id = ?1 AND dir <> 'shared'",
        [id],
    )?;
    conn.execute(
        "UPDATE outbox SET conv_id = NULL
         WHERE conv_id = ?1 AND kind = ?2 AND json_extract(frame, '$.type') = 'share_end'",
        params![id, crate::outbox::SHARE],
    )?;
    // 기다리는 기록 지우기도 — 지우기를 고른 뒤 나가도 상대 쪽은 비워지게.
    conn.execute(
        "UPDATE outbox SET conv_id = NULL WHERE conv_id = ?1 AND kind = ?2",
        params![id, crate::outbox::CLEAR],
    )?;
    for sql in [
        "DELETE FROM cli_tx WHERE message_id IN (SELECT id FROM messages WHERE conversation_id = ?1)",
        "DELETE FROM relay_events WHERE conversation_id = ?1
           OR message_id IN (SELECT id FROM messages WHERE conversation_id = ?1)",
        "DELETE FROM outbox WHERE conv_id = ?1
           OR msg_id IN (SELECT id FROM messages WHERE conversation_id = ?1)",
        "DELETE FROM agent_sessions WHERE conversation_id = ?1",
        "DELETE FROM ctx_seeds WHERE conversation_id = ?1",
        "DELETE FROM group_convs WHERE conversation_id = ?1",
        "DELETE FROM shares WHERE conversation_id = ?1",
        "DELETE FROM gone_msgs WHERE conversation_id = ?1",
        "DELETE FROM messages WHERE conversation_id = ?1",
    ] {
        conn.execute(sql, [id])?;
    }
    let gone = conn.execute("DELETE FROM conversations WHERE id = ?1", [id])? == 1;
    lower_marks(conn)?;
    Ok(gone)
}

/// 대화 기록을 이 PC 에서 비운다(기록 삭제) — 글 · 실행 기록 · 그 글의 보낼 행 · 에이전트 세션 기록 · 요약 · CLI 트랜잭션. 대화 행 · 제목 · 그룹 ·
/// 즐겨찾기 · 공유 · 자동 왕복 설정은 남는다(대화는 이어 쓴다). 묘비 없음. 없는 대화면 거짓.
pub fn clear_conversation(conn: &Connection, id: i64) -> rusqlite::Result<bool> {
    if by_id(conn, id)?.is_none() {
        return Ok(false);
    }
    // 지운 글의 uuid 는 남긴다 — 늦게 다시 온 같은 글이 기록을 되살리거나 다시 실행되지 않게.
    conn.execute(
        "INSERT OR IGNORE INTO gone_msgs (uuid, conversation_id) SELECT uuid, conversation_id FROM messages WHERE conversation_id = ?1",
        [id],
    )?;
    for sql in [
        "DELETE FROM cli_tx WHERE message_id IN (SELECT id FROM messages WHERE conversation_id = ?1)",
        "DELETE FROM relay_events WHERE conversation_id = ?1
           OR message_id IN (SELECT id FROM messages WHERE conversation_id = ?1)",
        "DELETE FROM outbox WHERE msg_id IN (SELECT id FROM messages WHERE conversation_id = ?1)",
        "DELETE FROM agent_sessions WHERE conversation_id = ?1",
        "DELETE FROM ctx_seeds WHERE conversation_id = ?1",
        "UPDATE conversations SET session = NULL, relay_on = 0, relay_count = 0 WHERE id = ?1",
        "DELETE FROM messages WHERE conversation_id = ?1",
    ] {
        conn.execute(sql, [id])?;
    }
    lower_marks(conn)?;
    Ok(true)
}

/// 멘션 — 사람 실행 요청이 부른 · 답을 실행한 에이전트(`None` = 대화 에이전트). 글과 같은 트랜잭션에서 부른다.
pub fn set_agent(conn: &Connection, id: i64, agent: Option<&str>) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE messages SET agent = ?2 WHERE id = ?1",
        params![id, agent],
    )
    .map(drop)
}

/// 대화의 마지막 에이전트 실행 명령 — 실행 직전에 덮어쓴다(`at` = 유닉스 초).
pub fn set_cmd(conn: &Connection, id: i64, cmd: &str, at: i64) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE conversations SET cmd = ?2, cmd_at = ?3 WHERE id = ?1",
        params![id, cmd, at],
    )
    .map(drop)
}

/// 세부정보 '대화 정보' — 시작 · 마지막 글 · 끝난 압축(수동 · 자동) · 비우기 수 · 마지막 실행 명령.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConvInfo {
    pub created_at: i64,
    /// 그 대화 글의 가장 늦은 시각 — 글이 없으면 `created_at`.
    pub last_at: i64,
    pub compacts: i64,
    pub last_compact_at: Option<i64>,
    pub clears: i64,
    pub cmd: Option<String>,
    pub cmd_at: Option<i64>,
}

pub fn info(conn: &Connection, id: i64) -> rusqlite::Result<Option<ConvInfo>> {
    conn.query_row(
        "SELECT c.created_at,
                coalesce((SELECT max(created_at) FROM messages WHERE conversation_id = c.id), c.created_at),
                (SELECT count(*) FROM messages WHERE conversation_id = c.id AND state = 'done' AND ctx_op IN ('compact', 'auto_compact')),
                (SELECT max(created_at) FROM messages WHERE conversation_id = c.id AND state = 'done' AND ctx_op IN ('compact', 'auto_compact')),
                (SELECT count(*) FROM messages WHERE conversation_id = c.id AND state = 'done' AND ctx_op = 'clear'),
                c.cmd, c.cmd_at
         FROM conversations c WHERE c.id = ?1",
        [id],
        |r| {
            Ok(ConvInfo {
                created_at: r.get(0)?,
                last_at: r.get(1)?,
                compacts: r.get(2)?,
                last_compact_at: r.get(3)?,
                clears: r.get(4)?,
                cmd: r.get(5)?,
                cmd_at: r.get(6)?,
            })
        },
    )
    .optional()
}

/// 실행이 남은 대화(시작 시 복구).
pub fn pending_conversations(conn: &Connection) -> rusqlite::Result<Vec<i64>> {
    conn.prepare(
        "SELECT DISTINCT conversation_id FROM messages WHERE exec = 1 AND state IN ('queued', 'running')
         ORDER BY conversation_id",
    )?
    .query_map([], |r| r.get(0))?
    .collect()
}

/// 대화 목록 — 마지막 메시지가 최근인 것 먼저(메시지 없으면 대화 생성 시각).
pub fn summaries(conn: &Connection) -> rusqlite::Result<Vec<Summary>> {
    let cols = prefixed(CONV_COLS, "c");
    conn.prepare(&format!(
        "SELECT {cols},
           (SELECT text FROM messages WHERE conversation_id = c.id AND author = 'human' ORDER BY id LIMIT 1),
           (SELECT state FROM messages WHERE conversation_id = c.id ORDER BY lc DESC, uuid DESC LIMIT 1),
           (SELECT created_at FROM messages WHERE conversation_id = c.id ORDER BY lc DESC, uuid DESC LIMIT 1),
           c.title,
           c.starred_at IS NOT NULL,
           (SELECT count(*) FROM messages WHERE conversation_id = c.id AND mine = 0 AND author <> 'context' AND id > coalesce(c.read_id, 0)),
           c.pos,
           (SELECT substr(text, 1, {PREVIEW_CHARS}) FROM messages WHERE conversation_id = c.id AND author <> 'context' ORDER BY lc DESC, uuid DESC LIMIT 1),
           (SELECT author FROM messages WHERE conversation_id = c.id AND author <> 'context' ORDER BY lc DESC, uuid DESC LIMIT 1),
           (SELECT mine FROM messages WHERE conversation_id = c.id AND author <> 'context' ORDER BY lc DESC, uuid DESC LIMIT 1),
           (SELECT agent FROM messages WHERE conversation_id = c.id AND author <> 'context' ORDER BY lc DESC, uuid DESC LIMIT 1),
           (SELECT via FROM messages WHERE conversation_id = c.id AND author <> 'context' ORDER BY lc DESC, uuid DESC LIMIT 1)
         FROM conversations c
         ORDER BY coalesce((SELECT max(created_at) FROM messages WHERE conversation_id = c.id), c.created_at) DESC,
           coalesce((SELECT max(id) FROM messages WHERE conversation_id = c.id), 0) DESC, c.id DESC"
    ))?
    .query_map([], |r| {
        let first: Option<String> = r.get(CONV_N)?;
        let last_state: Option<String> = r.get(CONV_N + 1)?;
        let own: Option<String> = r.get(CONV_N + 3)?;
        Ok(Summary {
            conversation: conv_row(r)?,
            title: own.filter(|t| !t.is_empty()).unwrap_or_else(|| {
                first.unwrap_or_default().chars().take(TITLE_CHARS).collect()
            }),
            last_state: match last_state.as_deref() {
                None => None,
                Some(v) => Some(state_of(v).ok_or_else(|| bad(CONV_N + 1, v))?),
            },
            last_at: r.get(CONV_N + 2)?,
            starred: r.get(CONV_N + 4)?,
            unread: r.get(CONV_N + 5)?,
            pos: r.get(CONV_N + 6)?,
            last: match r.get::<_, Option<String>>(CONV_N + 7)? {
                None => None,
                Some(text) => {
                    let author: String = r.get(CONV_N + 8)?;
                    Some(LastMessage {
                        text,
                        author: match author.as_str() {
                            "human" => Author::Human,
                            "agent" => Author::Agent,
                            "context" => Author::Context,
                            v => return Err(bad(CONV_N + 8, v)),
                        },
                        mine: r.get::<_, Option<bool>>(CONV_N + 9)?.unwrap_or(false),
                        agent: r.get(CONV_N + 10)?,
                        via: r.get(CONV_N + 11)?,
                    })
                }
            },
        })
    })?
    .collect()
}
