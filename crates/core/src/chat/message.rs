//! 글 — 쓰기 · 읽기 · 상태 · 고치기 · 지우기 · 북마크 · 페이지.

use super::*;

/// 메시지 저장 · 해제(이 PC 에만). 이미 저장돼 있으면 저장 시각은 그대로. 없는 메시지면 `false`.
pub fn set_saved(conn: &Connection, id: i64, on: bool) -> rusqlite::Result<bool> {
    Ok(conn.execute(
        "UPDATE messages SET saved_at = CASE WHEN ?2 THEN coalesce(saved_at, unixepoch()) END WHERE id = ?1",
        params![id, on],
    )? == 1)
}

/// 본문을 바꾸고 편집 시각을 남긴다 — uuid · 순서 · 요청 상태 · 저장은 그대로. 바뀐 행이 없으면 거짓.
pub fn edit_text(conn: &Connection, id: i64, text: &str) -> rusqlite::Result<bool> {
    conn.execute(
        "UPDATE messages SET text = ?2, edited_at = unixepoch() WHERE id = ?1",
        params![id, text],
    )
    .map(|n| n > 0)
}

/// 글을 흔적 없이 지운다 — 그 글의 CLI 트랜잭션 · 실패 기록 · 실패 보관 행도. 답(`reply_to`)은 남는다. 지운 행이 없으면 거짓.
/// 읽음 · 내 활동 본 표시 · 공유 대화의 보낸 커서는 남은 마지막 id 로 낮춘다. 글 번호는 다시 쓰지 않지만(스키마 22 `AUTOINCREMENT`) 자동 왕복 표시
/// (`relay_events`) 번호는 다시 쓰므로, 표시가 그대로면 새 표시가 이미 본 것으로 보인다.
pub fn delete_message(conn: &Connection, id: i64) -> rusqlite::Result<bool> {
    conn.execute("DELETE FROM cli_tx WHERE message_id = ?1", [id])?;
    conn.execute("DELETE FROM relay_events WHERE message_id = ?1", [id])?;
    conn.execute(
        "DELETE FROM outbox WHERE msg_id = ?1 AND state = 'failed'",
        [id],
    )?;
    let gone = conn.execute("DELETE FROM messages WHERE id = ?1", [id])? > 0;
    lower_marks(conn)?;
    Ok(gone)
}

/// 목록 범위 — 전부 · 한 상대의 대화 · 한 그룹(이 PC 의 그룹 소속 — 넣은 대화)의 대화.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Scope {
    #[default]
    All,
    Peer(i64),
    Group(i64),
}

impl Scope {
    /// 상대 id 와 그룹 id 중 하나(둘 다 `None` = 전부) — SQL 자리표에 넣는다.
    pub fn ids(self) -> (Option<i64>, Option<i64>) {
        match self {
            Scope::All => (None, None),
            Scope::Peer(p) => (Some(p), None),
            Scope::Group(g) => (None, Some(g)),
        }
    }

    /// 거르는 SQL — 대화 별칭 `c`, 자리표 `?{peer}` · `?{group}`.
    pub fn sql(peer: usize, group: usize) -> String {
        format!(
            "(?{peer} IS NULL OR c.peer_id = ?{peer}) AND (?{group} IS NULL OR c.id IN (SELECT conversation_id FROM group_convs WHERE group_id = ?{group}))"
        )
    }
}

/// 저장된 메시지 — 저장 시각 최신부터(같은 초면 id 큰 것), 범위(`scope`) · `conversation` 으로 거른다.
pub fn saved(
    conn: &Connection,
    scope: Scope,
    conversation: Option<i64>,
    limit: i64,
) -> rusqlite::Result<Vec<Message>> {
    let cols = prefixed(MSG_COLS, "m");
    let (peer, group) = scope.ids();
    conn.prepare(&format!(
        "SELECT {cols} FROM messages m JOIN conversations c ON c.id = m.conversation_id
         WHERE m.saved_at IS NOT NULL AND {} AND (?2 IS NULL OR c.id = ?2)
         ORDER BY m.saved_at DESC, m.id DESC LIMIT ?3",
        Scope::sql(1, 4)
    ))?
    .query_map(params![peer, conversation, limit, group], msg_row)?
    .collect()
}

/// 글의 출처 표시(`cli` 만 남긴다 — 그 밖 값은 표시 없음).
pub fn set_via(conn: &Connection, id: i64, via: Option<&str>) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE messages SET via = ?2 WHERE id = ?1",
        params![id, via.filter(|v| *v == VIA_CLI)],
    )
    .map(drop)
}

/// CLI 로 쓴 글의 `via` 값.
pub const VIA_CLI: &str = "cli";

/// 로컬 id 로 메시지.
pub fn message_by_id(conn: &Connection, id: i64) -> rusqlite::Result<Option<Message>> {
    conn.query_row(
        &format!("SELECT {MSG_COLS} FROM messages WHERE id = ?1"),
        [id],
        msg_row,
    )
    .optional()
}

pub fn add_message(
    conn: &Connection,
    conversation_id: i64,
    uuid: &str,
    author: Author,
    text: &str,
    state: State,
) -> rusqlite::Result<Message> {
    conn.execute(
        "INSERT INTO messages (uuid, conversation_id, author, text, state, created_at) VALUES (?1, ?2, ?3, ?4, ?5, unixepoch())",
        params![uuid, conversation_id, author.as_str(), text, state.as_str()],
    )?;
    fill_mine_exec(conn, conn.last_insert_rowid())
}

/// 방향으로 `mine` 을, 사람 글이면 실행 요청으로 채운다(보낸 쪽 사람 글 · 받은 쪽 실행 요청 · 에이전트 답의 기본).
fn fill_mine_exec(conn: &Connection, id: i64) -> rusqlite::Result<Message> {
    conn.execute(
        &format!(
            "UPDATE messages SET mine = {}, exec = (author = 'human') WHERE id = ?1",
            crate::db::MINE_BY_DIR
        ),
        [id],
    )?;
    stamp(conn, id)?;
    conn.query_row(
        &format!("SELECT {MSG_COLS} FROM messages WHERE id = ?1"),
        [id],
        msg_row,
    )
}

/// 사람 글 — 누가 썼나(`mine`) · 실행 요청인가(`exec`)를 직접 정한다(양쪽 발화 · 미러).
pub fn add_human(
    conn: &Connection,
    conversation_id: i64,
    uuid: &str,
    text: &str,
    state: State,
    mine: bool,
    exec: bool,
) -> rusqlite::Result<Message> {
    conn.execute(
        "INSERT INTO messages (uuid, conversation_id, author, text, state, created_at, mine, exec)
         VALUES (?1, ?2, 'human', ?3, ?4, unixepoch(), ?5, ?6)",
        params![uuid, conversation_id, text, state.as_str(), mine, exec],
    )?;
    let id = conn.last_insert_rowid();
    stamp(conn, id)?;
    conn.query_row(
        &format!("SELECT {MSG_COLS} FROM messages WHERE id = ?1"),
        [id],
        msg_row,
    )
}

/// 원격이 줄 수 있는 순서 값의 상한 — 넘으면 무시하고 이 PC 가 매긴다(`max + 1` 이 정수 범위를 넘지 않게 — 신뢰 경계).
pub const MAX_LC: i64 = 1 << 40;

/// 상대가 정한 순서를 그대로 남긴다(받은 글) — 없거나(이전 버전 상대) 범위 밖이면 이 PC 가 매긴 값.
pub fn observe(conn: &Connection, id: i64, lc: Option<i64>) -> rusqlite::Result<Message> {
    if let Some(lc) = lc.filter(|v| (1..=MAX_LC).contains(v)) {
        conn.execute("UPDATE messages SET lc = ?2 WHERE id = ?1", params![id, lc])?;
    }
    conn.query_row(
        &format!("SELECT {MSG_COLS} FROM messages WHERE id = ?1"),
        [id],
        msg_row,
    )
}

/// `after` 와 `before` 사이(양쪽 공통 순서)의 실행 없는 사람 글(에이전트 실행 맥락). `after` 가 없으면 처음부터.
pub fn notes_between(
    conn: &Connection,
    conversation_id: i64,
    after: Option<&Message>,
    before: &Message,
) -> rusqlite::Result<Vec<Message>> {
    let (alc, auuid) = after.map_or((i64::MIN, String::new()), |m| (m.lc, m.uuid.clone()));
    conn.prepare(&format!(
        "SELECT {MSG_COLS} FROM messages WHERE conversation_id = ?1 AND (lc, uuid) > (?2, ?3) AND (lc, uuid) < (?4, ?5)
           AND author = 'human' AND exec = 0 ORDER BY lc, uuid"
    ))?
    .query_map(
        params![conversation_id, alc, auuid, before.lc, before.uuid],
        msg_row,
    )?
    .collect()
}

/// `before` 앞(양쪽 공통 순서)의 마지막 실행 요청 사람 글.
pub fn last_exec_before(
    conn: &Connection,
    conversation_id: i64,
    before: &Message,
) -> rusqlite::Result<Option<Message>> {
    conn.query_row(
        &format!(
            "SELECT {MSG_COLS} FROM messages WHERE conversation_id = ?1 AND (lc, uuid) < (?2, ?3)
               AND exec = 1 AND author <> 'context' ORDER BY lc DESC, uuid DESC LIMIT 1"
        ),
        params![conversation_id, before.lc, before.uuid],
        msg_row,
    )
    .optional()
}

pub fn set_state(conn: &Connection, message_id: i64, state: State) -> rusqlite::Result<()> {
    if state == State::Error {
        note_failed(conn, message_id, None)?;
    }
    conn.execute(
        "UPDATE messages SET state = ?2 WHERE id = ?1",
        params![message_id, state.as_str()],
    )
    .map(drop)
}

pub fn messages(conn: &Connection, conversation_id: i64) -> rusqlite::Result<Vec<Message>> {
    conn.prepare(&format!(
        "SELECT {MSG_COLS} FROM messages WHERE conversation_id = ?1 ORDER BY lc, uuid"
    ))?
    .query_map([conversation_id], msg_row)?
    .collect()
}

/// 대화의 마지막 `n` 개(오래된 것부터, 개요 미리보기) — [`messages`] 와 같은 순서의 끝을 SQL 에서 자른다(대화 전체를 읽지 않는다).
pub fn messages_tail(conn: &Connection, conversation_id: i64, n: i64) -> rusqlite::Result<Vec<Message>> {
    let mut ms = conn
        .prepare(&format!(
            "SELECT {MSG_COLS} FROM messages WHERE conversation_id = ?1 ORDER BY lc DESC, uuid DESC LIMIT ?2"
        ))?
        .query_map(params![conversation_id, n], msg_row)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    ms.reverse();
    Ok(ms)
}

/// 대화를 가로지른 글 — `mine` 이 참이면 내가 쓴 글, 거짓이면 상대 쪽이 쓴 글, `None` 이면 둘 다. `scope` 로 그 상대 · 그룹 대화만,
/// `author` 로 그 종류만, `q` 는 본문 부분 일치. 거른 뒤 최신 것부터 `limit` 건. 질의 · 요청 거르기(`ask`)에는 공유 사본 글이 들지 않는다.
pub fn messages_across(
    conn: &Connection,
    mine: Option<bool>,
    scope: Scope,
    author: Option<Author>,
    ask: Option<AskFilter>,
    q: Option<&str>,
    limit: i64,
) -> rusqlite::Result<Vec<Message>> {
    let cols = prefixed(MSG_COLS, "m");
    let like = q.map(like_pattern);
    let (request, req_state, archived) = match ask {
        None => (None, None, None),
        Some(AskFilter::Query) => (Some(false), None, None),
        Some(AskFilter::Request(s, a)) => (Some(true), s.map(ReqState::as_str), Some(a)),
    };
    let (peer, group) = scope.ids();
    conn.prepare(&format!(
        "SELECT {cols} FROM messages m JOIN conversations c ON c.id = m.conversation_id
         WHERE (?1 IS NULL OR m.mine = ?1) AND {} AND (?3 IS NULL OR m.author = ?3) AND m.author <> 'context'
           AND (?4 IS NULL OR (m.author = 'human' AND coalesce(m.request, 0) = ?4 AND c.dir <> 'shared'))
           AND (?5 IS NULL OR m.text LIKE ?5 ESCAPE '\\') AND (?7 IS NULL OR m.req_state = ?7)
           AND (?9 IS NULL OR {} = ?9)
         ORDER BY m.created_at DESC, m.id DESC LIMIT ?6",
        Scope::sql(2, 8),
        archived_sql("m")
    ))?
    .query_map(
        params![
            mine,
            peer,
            author.map(Author::as_str),
            request,
            like,
            limit,
            req_state,
            group,
            archived
        ],
        msg_row,
    )?
    .collect()
}

/// 주고받은 메시지 한 쪽 — 양쪽 글(context 글 제외)을 `(created_at, id)` 최신부터 `limit` 건, `before` = 그 `(created_at, id)` 보다 앞의 것만
/// (복합 커서 — 공유 사본은 상대 시각을 남겨 id 순과 다를 수 있다).
///
/// `since` = 그 시각 이후(이상)에 쓴 글만(보기 기간).
pub fn messages_page(
    conn: &Connection,
    scope: Scope,
    before: Option<(i64, i64)>,
    since: Option<i64>,
    limit: i64,
) -> rusqlite::Result<Vec<Message>> {
    let cols = prefixed(MSG_COLS, "m");
    let (peer, group) = scope.ids();
    let (at, id) = before.unzip();
    conn.prepare(&format!(
        "SELECT {cols} FROM messages m JOIN conversations c ON c.id = m.conversation_id
         WHERE m.author <> 'context' AND {} AND (?3 IS NULL OR m.created_at < ?3 OR (m.created_at = ?3 AND m.id < ?4))
           AND (?6 IS NULL OR m.created_at >= ?6)
         ORDER BY m.created_at DESC, m.id DESC LIMIT ?5",
        Scope::sql(1, 2)
    ))?
    .query_map(params![peer, group, at, id, limit, since], msg_row)?
    .collect()
}

/// 주고받은 메시지 상대별 요약 — (상대 id, 메시지 수, 마지막 메시지 id — [`messages_page`] 와 같은 `(created_at, id)` 순), 모집단도 같다(`since` 포함).
pub fn peer_message_counts(conn: &Connection, scope: Scope, since: Option<i64>) -> rusqlite::Result<Vec<(i64, i64, i64)>> {
    let (peer, group) = scope.ids();
    conn.prepare(&format!(
        "SELECT peer_id, n, id FROM (
           SELECT c.peer_id, m.id, count(*) OVER (PARTITION BY c.peer_id) AS n,
             row_number() OVER (PARTITION BY c.peer_id ORDER BY m.created_at DESC, m.id DESC) AS rn
           FROM messages m JOIN conversations c ON c.id = m.conversation_id
           WHERE m.author <> 'context' AND {} AND (?3 IS NULL OR m.created_at >= ?3))
         WHERE rn = 1 ORDER BY peer_id",
        Scope::sql(1, 2)
    ))?
    .query_map(params![peer, group, since], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
    .collect()
}

/// uuid 로 메시지(재전송 · 중복 판별, 답의 원 메시지 찾기).
pub fn message_by_uuid(conn: &Connection, uuid: &str) -> rusqlite::Result<Option<Message>> {
    conn.query_row(
        &format!("SELECT {MSG_COLS} FROM messages WHERE uuid = ?1"),
        [uuid],
        msg_row,
    )
    .optional()
}

/// 실행 대기열 = DB — 대화의 `queued` · `running` 실행 대상(`exec` — 사람 글 · 에이전트발 · 자동 압축 구분선), 자동 압축 먼저 · 그다음 오래된 것 먼저. 늦게 `queued` 가 된 낮은 id 도 빠지지 않게
/// 커서 없이 매번 처음부터 본다(작업이 한 번 꺼낸 것은 작업 쪽에서 건너뛴다).
pub fn pending(conn: &Connection, conversation_id: i64) -> rusqlite::Result<Vec<Message>> {
    conn.prepare(&format!(
        "SELECT {MSG_COLS} FROM messages WHERE conversation_id = ?1
           AND exec = 1 AND state IN ('queued', 'running') ORDER BY coalesce(ctx_op = 'auto_compact', 0) DESC, id"
    ))?
    .query_map([conversation_id], msg_row)?
    .collect()
}

/// 에이전트 답 — 원 요청(`reply_to`) · 실행 정보(턴 · 시간 · model · effort)와 함께.
#[allow(clippy::too_many_arguments)]
pub fn add_reply(
    conn: &Connection,
    conversation_id: i64,
    uuid: &str,
    text: &str,
    state: State,
    reply_to: i64,
    turns: Option<i64>,
    duration_ms: Option<i64>,
    model: Option<&str>,
    effort: Option<&str>,
) -> rusqlite::Result<Message> {
    conn.execute(
        "INSERT INTO messages (uuid, conversation_id, author, text, state, created_at, reply_to, turns, duration_ms, model, effort)
         VALUES (?1, ?2, 'agent', ?3, ?4, unixepoch(), ?5, ?6, ?7, ?8, ?9)",
        params![uuid, conversation_id, text, state.as_str(), reply_to, turns, duration_ms, model, effort],
    )?;
    fill_mine_exec(conn, conn.last_insert_rowid())
}

/// 글의 실행 요청 여부 · 내 것 여부 · 상태를 바꾼다(자동 왕복: 상대 답을 내 에이전트 입력으로, 내 에이전트 답을 상대 실행 요청으로).
pub fn set_flags(
    conn: &Connection,
    id: i64,
    mine: bool,
    exec: bool,
    state: State,
) -> rusqlite::Result<()> {
    if state == State::Error {
        note_failed(conn, id, Some(mine))?;
    }
    conn.execute(
        "UPDATE messages SET mine = ?2, exec = ?3, state = ?4 WHERE id = ?1",
        params![id, mine, exec, state.as_str()],
    )
    .map(drop)
}

/// 내 글이 실패로 바뀌는 순간을 남긴다(`relay_events` 의 `failed` 행) — 글 id 는 그대로라, 내 활동을 본 뒤 실패로 바뀐 글을 다시 안 본 것으로
/// 센다. 이미 실패거나 내 글이 아니면 남기지 않는다. `mine` = 바꾸면서 정하는 값(없으면 지금 값).
fn note_failed(conn: &Connection, id: i64, mine: Option<bool>) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO relay_events (conversation_id, kind, detail, created_at, message_id)
         SELECT conversation_id, 'failed', '', unixepoch(), id FROM messages
         WHERE id = ?1 AND state <> 'error' AND coalesce(?2, mine) = 1",
        params![id, mine],
    )
    .map(drop)
}
