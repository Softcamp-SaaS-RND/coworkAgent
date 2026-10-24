//! 요청 — 상태 · 메모 · 보관 · 개수.

use super::*;

/// 요청 상태 의견 최대 글자 수.
pub const MAX_REQ_NOTE: usize = 200;

/// 요청 상태 의견 — 여러 줄(CRLF 는 LF, 줄바꿈 말고 제어 문자 없음) [`MAX_REQ_NOTE`] 자까지, 비면 `None`.
pub fn clean_req_note(s: &str) -> Result<Option<String>, String> {
    let s = s.replace("\r\n", "\n");
    let s = s.trim();
    if s.chars().count() > MAX_REQ_NOTE || s.chars().any(|c| c.is_control() && c != '\n') {
        return Err(format!("의견은 {MAX_REQ_NOTE}자까지, 제어 문자 없이"));
    }
    Ok((!s.is_empty()).then(|| s.to_string()))
}

/// 상대에게 보내는 의견 꼴 — 줄바꿈을 U+2028(LINE SEPARATOR)로. U+2028 은 제어 문자(Cc)가 아니라 줄바꿈을 거절하는 이전 버전
/// 검사(`clean_label`)를 통과한다(호환). 받는 쪽은 [`note_from_wire`] 로 되돌린다.
pub fn note_to_wire(s: &str) -> String {
    s.replace('\n', "\u{2028}")
}

pub fn note_from_wire(s: &str) -> String {
    s.replace('\u{2028}', "\n")
}

/// 상대가 내 요청의 상태를 바꿨다 — `relay_events` 의 `req_state` 행(글 id 는 그대로라, 내 활동을 본 뒤 바뀐 상태를 안 본 것으로 센다).
/// 내 사람 요청 글이 아니면 남기지 않는다.
pub fn note_req_state(conn: &Connection, message_id: i64) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO relay_events (conversation_id, kind, detail, created_at, message_id)
         SELECT conversation_id, 'req_state', coalesce(req_state, ''), unixepoch(), id FROM messages
         WHERE id = ?1 AND mine = 1 AND author = 'human' AND coalesce(request, 0) = 1",
        [message_id],
    )
    .map(drop)
}

/// 사람 글의 종류 — 참 = 요청(건의, 상태 '대기'로 시작), 거짓 = 질의(기본). 글과 같은 트랜잭션에서 부른다.
pub fn set_request(conn: &Connection, id: i64, request: bool) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE messages SET request = ?2, req_state = CASE WHEN ?2 THEN coalesce(req_state, 'wait') END WHERE id = ?1",
        params![id, request],
    )
    .map(drop)
}

/// 요청 상태 · 메모를 바꾸고 시각을 남긴다 — 옮김 검사는 부르는 쪽([`ReqState::can_become`]).
pub fn set_req_state(
    conn: &Connection,
    id: i64,
    state: ReqState,
    note: Option<&str>,
) -> rusqlite::Result<()> {
    conn.execute(
        // 상태가 바뀌면 archive 손 표시를 지운다(완료가 아니면 목록에, 다시 완료되면 7일 셈을 새로).
        "UPDATE messages SET req_archived = CASE WHEN req_state IS ?2 THEN req_archived END,
           req_state = ?2, req_note = ?3, req_at = unixepoch() WHERE id = ?1",
        params![id, state.as_str(), note],
    )
    .map(drop)
}

/// 요청 archive 손 표시(이 PC 에만) — 사람 요청 글(공유 사본 아님)만, archive(`on`)는 완료일 때만. 바꿨으면 참.
pub fn set_req_archived(conn: &Connection, id: i64, on: bool) -> rusqlite::Result<bool> {
    conn.execute(
        "UPDATE messages SET req_archived = ?2 WHERE id = ?1 AND author = 'human' AND coalesce(request, 0) = 1
           AND (NOT ?2 OR req_state = 'done')
           AND (SELECT dir FROM conversations c WHERE c.id = messages.conversation_id) <> 'shared'",
        params![id, on],
    )
    .map(|n| n > 0)
}

/// 상대가 거절한 내 글 — 거절 코드(`focus` 등)를 남긴다(상태는 따로 `set_state`). 다시 보내면 `None` 으로 지운다.
pub fn set_refused(conn: &Connection, id: i64, code: Option<&str>) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE messages SET refused = ?2 WHERE id = ?1",
        params![id, code],
    )
    .map(drop)
}

/// 요청 상태별 건수 — 한 쪽(`mine`) 사람 요청 전부(공유 사본 제외), 목록 상한과 무관.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ReqCounts {
    pub wait: i64,
    pub accepted: i64,
    pub rejected: i64,
    /// archive 되지 않은 완료.
    pub done: i64,
    /// archive 된 완료 — [`ReqCounts::total`] 에 들지 않는다.
    pub archived: i64,
}

impl ReqCounts {
    pub fn total(self) -> i64 {
        self.wait + self.accepted + self.rejected + self.done
    }
}

/// [`ReqCounts`] — `scope` 범위, 상태가 없는 요청은 대기.
pub fn req_counts(conn: &Connection, mine: bool, scope: Scope) -> rusqlite::Result<ReqCounts> {
    let (peer, group) = scope.ids();
    let mut out = ReqCounts::default();
    let mut stmt = conn.prepare(&format!(
        "SELECT CASE WHEN {} THEN 'archived' ELSE coalesce(m.req_state, 'wait') END, count(*)
         FROM messages m JOIN conversations c ON c.id = m.conversation_id
         WHERE m.mine = ?3 AND m.author = 'human' AND coalesce(m.request, 0) = 1 AND c.dir <> 'shared' AND {}
         GROUP BY 1",
        archived_sql("m"),
        Scope::sql(1, 2)
    ))?;
    let rows = stmt.query_map(params![peer, group, mine], |r| Ok((r.get::<_, String>(0)?, r.get(1)?)))?;
    for row in rows {
        let (state, n) = row?;
        if state == "archived" {
            out.archived = n;
            continue;
        }
        match ReqState::parse(&state) {
            Some(ReqState::Wait) => out.wait = n,
            Some(ReqState::Accepted) => out.accepted = n,
            Some(ReqState::Rejected) => out.rejected = n,
            Some(ReqState::Done) => out.done = n,
            None => {}
        }
    }
    Ok(out)
}

/// 받은 요청 중 대기(상태 없음 포함)인 것 — 최신부터 `limit` 건(상대 id · 본문)과 대기 전체 수(받은 요청 알림 본문).
pub fn waiting_requests(conn: &Connection, limit: i64) -> rusqlite::Result<(Vec<(i64, String)>, i64)> {
    let total = req_counts(conn, false, Scope::All)?.wait;
    let rows = conn
        .prepare(
            "SELECT c.peer_id, m.text FROM messages m JOIN conversations c ON c.id = m.conversation_id
             WHERE m.mine = 0 AND m.author = 'human' AND coalesce(m.request, 0) = 1 AND c.dir <> 'shared'
               AND coalesce(m.req_state, 'wait') = 'wait'
             ORDER BY m.created_at DESC, m.id DESC LIMIT ?1",
        )?
        .query_map([limit], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<rusqlite::Result<_>>()?;
    Ok((rows, total))
}

/// 에이전트발 실행 요청(받은 쪽 저장 — 상대 에이전트가 보낸 글).
pub fn add_agent_request(
    conn: &Connection,
    conversation_id: i64,
    uuid: &str,
    text: &str,
    state: State,
) -> rusqlite::Result<Message> {
    conn.execute(
        "INSERT INTO messages (uuid, conversation_id, author, text, state, created_at, mine, exec)
         VALUES (?1, ?2, 'agent', ?3, ?4, unixepoch(), 0, 1)",
        params![uuid, conversation_id, text, state.as_str()],
    )?;
    let id = conn.last_insert_rowid();
    stamp(conn, id)?;
    conn.query_row(
        &format!("SELECT {MSG_COLS} FROM messages WHERE id = ?1"),
        [id],
        msg_row,
    )
}
