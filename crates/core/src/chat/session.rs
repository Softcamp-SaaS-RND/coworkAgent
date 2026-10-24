//! 에이전트 실행 · 세션 · 컨텍스트.

use super::*;

/// 실행 시작 — 대기 글(`queued`)이고 대화가 닫히지 않았을 때만 `running` 으로(한 문장 — 확인과 바꾸기 사이에 나가기 · 삭제가 끼지 못한다).
/// 바꿨으면 참 — 거짓이면 실행하지 않는다(그 사이 취소 · 나감 · 지움).
pub fn start_run(conn: &Connection, message_id: i64) -> rusqlite::Result<bool> {
    Ok(conn.execute(
        "UPDATE messages SET state = 'running' WHERE id = ?1 AND state = 'queued'
           AND conversation_id IN (SELECT id FROM conversations WHERE left_at IS NULL AND peer_left_at IS NULL)",
        [message_id],
    )? == 1)
}

/// 도는 실행이 있나(실행 중이면 나가기 · 삭제를 막는다). 구분선(압축)도 실행이다.
pub fn running(conn: &Connection, id: i64) -> rusqlite::Result<bool> {
    conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM messages WHERE conversation_id = ?1 AND state = 'running')",
        [id],
        |r| r.get(0),
    )
}

/// 멘션한 에이전트의 Claude 세션(대화 × 에이전트) — 대화 에이전트는 `Conversation.session`.
pub fn agent_session(
    conn: &Connection,
    conversation_id: i64,
    agent: &str,
) -> rusqlite::Result<Option<String>> {
    conn.query_row(
        "SELECT session FROM agent_sessions WHERE conversation_id = ?1 AND agent = ?2",
        params![conversation_id, agent],
        |r| r.get(0),
    )
    .optional()
}

pub fn set_agent_session(
    conn: &Connection,
    conversation_id: i64,
    agent: &str,
    session: &str,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO agent_sessions (conversation_id, agent, session) VALUES (?1, ?2, ?3)
         ON CONFLICT(conversation_id, agent) DO UPDATE SET session = ?3",
        params![conversation_id, agent, session],
    )
    .map(drop)
}

/// 답의 창 사용량 — `None` 이면 그대로(모름).
pub fn set_ctx_tokens(conn: &Connection, id: i64, tokens: Option<i64>) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE messages SET ctx_tokens = ?2 WHERE id = ?1",
        params![id, tokens],
    )
    .map(drop)
}

/// 컨텍스트 구분선 글 — `exec` = 이 PC 가 세션을 처리한다(대화 작업자 줄에 든다), `mine` = 이 PC 에서 눌렀다(자동 압축 포함).
pub fn add_context(
    conn: &Connection,
    conversation_id: i64,
    uuid: &str,
    op: &str,
    state: State,
    mine: bool,
    exec: bool,
) -> rusqlite::Result<Message> {
    conn.execute(
        "INSERT INTO messages (uuid, conversation_id, author, text, state, created_at, mine, exec, ctx_op)
         VALUES (?1, ?2, 'context', '', ?3, unixepoch(), ?4, ?5, ?6)",
        params![uuid, conversation_id, state.as_str(), mine, exec, op],
    )?;
    let id = conn.last_insert_rowid();
    stamp(conn, id)?;
    conn.query_row(
        &format!("SELECT {MSG_COLS} FROM messages WHERE id = ?1"),
        [id],
        msg_row,
    )
}

/// 구분선 글 끝 — 요약(압축) · 상태. 구분선이 아니면 아무것도 바꾸지 않는다(거짓).
pub fn finish_context(
    conn: &Connection,
    id: i64,
    state: State,
    text: &str,
) -> rusqlite::Result<bool> {
    Ok(conn.execute(
        "UPDATE messages SET state = ?2, text = ?3 WHERE id = ?1 AND author = 'context'",
        params![id, state.as_str(), text],
    )? > 0)
}

/// 대화의 Claude 세션 전부 — (에이전트, 세션). 대화 에이전트는 `None`, 멘션한 에이전트는 이름.
pub fn sessions(
    conn: &Connection,
    conv: &Conversation,
) -> rusqlite::Result<Vec<(Option<String>, String)>> {
    let mut out: Vec<(Option<String>, String)> =
        conv.session.iter().map(|s| (None, s.clone())).collect();
    let rows = conn
        .prepare(
            "SELECT agent, session FROM agent_sessions WHERE conversation_id = ?1 ORDER BY agent",
        )?
        .query_map([conv.id], |r| Ok((Some(r.get(0)?), r.get(1)?)))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    out.extend(rows);
    Ok(out)
}

/// 대화 세션을 모두 비운다(비우기 · 압축 뒤). 남은 요약은 `keep_seeds` 가 거짓이면 함께 지운다.
pub fn clear_sessions(
    conn: &Connection,
    conversation_id: i64,
    keep_seeds: bool,
) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE conversations SET session = NULL WHERE id = ?1",
        [conversation_id],
    )?;
    conn.execute(
        "DELETE FROM agent_sessions WHERE conversation_id = ?1",
        [conversation_id],
    )?;
    if !keep_seeds {
        conn.execute(
            "DELETE FROM ctx_seeds WHERE conversation_id = ?1",
            [conversation_id],
        )?;
    }
    Ok(())
}

/// 압축 요약 — 다음 새 세션의 첫 실행 앞에 붙인다. `agent` 빈 글자 = 대화 에이전트.
pub fn set_ctx_seed(
    conn: &Connection,
    conversation_id: i64,
    agent: &str,
    text: &str,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO ctx_seeds (conversation_id, agent, text) VALUES (?1, ?2, ?3)
         ON CONFLICT(conversation_id, agent) DO UPDATE SET text = ?3",
        params![conversation_id, agent, text],
    )
    .map(drop)
}

pub fn ctx_seed(
    conn: &Connection,
    conversation_id: i64,
    agent: &str,
) -> rusqlite::Result<Option<String>> {
    conn.query_row(
        "SELECT text FROM ctx_seeds WHERE conversation_id = ?1 AND agent = ?2",
        params![conversation_id, agent],
        |r| r.get(0),
    )
    .optional()
}

/// 요약을 쓴 실행이 세션을 남겼다 — 그 요약은 지운다(한 번만).
pub fn drop_ctx_seed(conn: &Connection, conversation_id: i64, agent: &str) -> rusqlite::Result<()> {
    conn.execute(
        "DELETE FROM ctx_seeds WHERE conversation_id = ?1 AND agent = ?2",
        params![conversation_id, agent],
    )
    .map(drop)
}

pub fn set_session(conn: &Connection, id: i64, session: &str) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE conversations SET session = ?2 WHERE id = ?1",
        params![id, session],
    )
    .map(drop)
}

/// 답 밖 경로로 기록한 에이전트 글(자동 왕복 상한의 마지막 답)에 실행 정보(턴 · 시간 · model · effort)를 남긴다.
pub fn set_run(
    conn: &Connection,
    id: i64,
    turns: Option<i64>,
    duration_ms: Option<i64>,
    model: Option<&str>,
    effort: Option<&str>,
) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE messages SET turns = ?2, duration_ms = ?3, model = ?4, effort = ?5 WHERE id = ?1",
        params![id, turns, duration_ms, model, effort],
    )?;
    Ok(())
}
