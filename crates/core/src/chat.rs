//! 대화 · 메시지 이력(D19 · S3) — 양쪽 PC 가 같은 대화 UUID 로 각자 남긴다.
use rusqlite::{Connection, OptionalExtension, Row, params};

/// `Out` = 내가 원격 에이전트에게 · `In` = 내 에이전트가 원격에게서 받음.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dir {
    Out,
    In,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Author {
    Human,
    Agent,
}

/// `Sent` 보냄 · `Queued` 실행 대기 · `Running` 실행 중 · `Done` 끝 · `Error` 실패 · `Undelivered` 답을 상대에 못 보냄.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Sent,
    Queued,
    Running,
    Done,
    Error,
    Undelivered,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Conversation {
    pub id: i64,
    pub uuid: String,
    pub peer_id: i64,
    pub agent: String,
    pub dir: Dir,
    /// Claude 세션 ID — 에이전트가 있는 쪽(`In`), 자동 왕복이면 연 쪽(`Out`)의 내 에이전트 세션.
    pub session: Option<String>,
    /// 자동 왕복(D24, 연 쪽) — 없으면 자동 왕복 대화가 아니다.
    pub relay: Option<Relay>,
    /// 받은 쪽: 에이전트발 실행 요청을 거절한다(받은 쪽 사람이 멈춤).
    pub relay_blocked: bool,
}

/// 자동 왕복 상태 — 1왕복 = 내 에이전트가 상대 답에 한 번 답함.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Relay {
    pub agent: String,
    pub cap: i64,
    pub count: i64,
    pub on: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    pub id: i64,
    pub uuid: String,
    pub conversation_id: i64,
    pub author: Author,
    pub text: String,
    pub state: State,
    /// 저장 시각(유닉스 초).
    pub created_at: i64,
    /// 답(agent)이 대응하는 사람 메시지의 로컬 id.
    pub reply_to: Option<i64>,
    /// 실행 턴 수 · 시간(`claude` 출력 `num_turns` · `duration_ms`) — 답에만, 없을 수 있다.
    pub turns: Option<i64>,
    pub duration_ms: Option<i64>,
    /// 이 PC 에서 쓴 글(사람) · 이 PC 에이전트의 답이면 참(D23 — 양쪽 발화).
    pub mine: bool,
    /// 에이전트 실행 요청인 사람 글이면 참(실행 없는 사람 글 · 답은 거짓).
    pub exec: bool,
    /// 대화 안 양쪽 공통 순서(램포트 시계) — 쓴 쪽이 정하고 받은 쪽은 그대로 남긴다. 정렬은 `(lc, uuid)`.
    pub lc: i64,
}

/// 대화 목록 한 줄.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Summary {
    pub conversation: Conversation,
    /// 첫 사람 메시지 앞 [`TITLE_CHARS`] 글자, 없으면 빈 문자열.
    pub title: String,
    pub last_state: Option<State>,
    pub last_at: Option<i64>,
}

/// 대화 제목 길이(글자).
pub const TITLE_CHARS: usize = 40;

impl Dir {
    pub fn as_str(self) -> &'static str {
        match self {
            Dir::Out => "out",
            Dir::In => "in",
        }
    }
}

impl Author {
    pub fn as_str(self) -> &'static str {
        match self {
            Author::Human => "human",
            Author::Agent => "agent",
        }
    }
}

impl State {
    pub fn as_str(self) -> &'static str {
        match self {
            State::Sent => "sent",
            State::Queued => "queued",
            State::Running => "running",
            State::Done => "done",
            State::Error => "error",
            State::Undelivered => "undelivered",
        }
    }
}

fn state_of(v: &str) -> Option<State> {
    Some(match v {
        "sent" => State::Sent,
        "queued" => State::Queued,
        "running" => State::Running,
        "done" => State::Done,
        "error" => State::Error,
        "undelivered" => State::Undelivered,
        _ => return None,
    })
}

fn bad(col: usize, v: &str) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(
        col,
        rusqlite::types::Type::Text,
        format!("알 수 없는 값 {v}").into(),
    )
}

const CONV_COLS: &str = "id, uuid, peer_id, agent, dir, session, relay_agent, relay_cap, relay_count, relay_on, relay_blocked";
/// `CONV_COLS` 열 수 — 그 뒤에 붙는 열의 위치.
const CONV_N: usize = 11;

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
    })
}

const MSG_COLS: &str = "id, uuid, conversation_id, author, text, state, created_at, reply_to, turns, duration_ms, mine, exec, lc";

fn msg_row(r: &Row) -> rusqlite::Result<Message> {
    let (author, state): (String, String) = (r.get(3)?, r.get(5)?);
    Ok(Message {
        id: r.get(0)?,
        uuid: r.get(1)?,
        conversation_id: r.get(2)?,
        author: match author.as_str() {
            "human" => Author::Human,
            "agent" => Author::Agent,
            v => return Err(bad(3, v)),
        },
        text: r.get(4)?,
        created_at: r.get(6)?,
        reply_to: r.get(7)?,
        turns: r.get(8)?,
        duration_ms: r.get(9)?,
        mine: r.get::<_, Option<bool>>(10)?.unwrap_or(false),
        exec: r.get::<_, Option<bool>>(11)?.unwrap_or(false),
        lc: r.get::<_, Option<i64>>(12)?.unwrap_or(0),
        state: state_of(&state).ok_or_else(|| bad(5, &state))?,
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
    conn.execute(
        "INSERT INTO conversations (uuid, peer_id, agent, dir, created_at) VALUES (?1, ?2, ?3, ?4, unixepoch())",
        params![uuid, peer_id, agent, dir.as_str()],
    )?;
    by_id(conn, conn.last_insert_rowid()).map(|c| c.expect("방금 쓴 대화"))
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

pub fn set_session(conn: &Connection, id: i64, session: &str) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE conversations SET session = ?2 WHERE id = ?1",
        params![id, session],
    )
    .map(drop)
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

/// 이 PC 에서 새로 쓴 글의 순서 — 대화의 가장 큰 `lc` + 1(램포트 시계: 받은 글의 `lc` 도 세므로 인과 순서가 지켜진다).
fn stamp(conn: &Connection, id: i64) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE messages SET lc = (SELECT coalesce(max(m.lc), 0) + 1 FROM messages m
           WHERE m.conversation_id = messages.conversation_id AND m.id <> messages.id)
         WHERE id = ?1 AND lc IS NULL",
        [id],
    )
    .map(drop)
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
               AND exec = 1 ORDER BY lc DESC, uuid DESC LIMIT 1"
        ),
        params![conversation_id, before.lc, before.uuid],
        msg_row,
    )
    .optional()
}

pub fn set_state(conn: &Connection, message_id: i64, state: State) -> rusqlite::Result<()> {
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

/// uuid 로 메시지(재전송 · 중복 판별, 답의 원 메시지 찾기).
pub fn message_by_uuid(conn: &Connection, uuid: &str) -> rusqlite::Result<Option<Message>> {
    conn.query_row(
        &format!("SELECT {MSG_COLS} FROM messages WHERE uuid = ?1"),
        [uuid],
        msg_row,
    )
    .optional()
}

/// 실행 대기열 = DB — 대화의 `queued` · `running` 실행 요청(사람 글 · 에이전트발), 오래된 것 먼저. 늦게 `queued` 가 된 낮은 id 도 빠지지 않게
/// 커서 없이 매번 처음부터 본다(작업이 한 번 꺼낸 것은 작업 쪽에서 건너뛴다).
pub fn pending(conn: &Connection, conversation_id: i64) -> rusqlite::Result<Vec<Message>> {
    conn.prepare(&format!(
        "SELECT {MSG_COLS} FROM messages WHERE conversation_id = ?1
           AND exec = 1 AND state IN ('queued', 'running') ORDER BY id"
    ))?
    .query_map([conversation_id], msg_row)?
    .collect()
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

/// 에이전트 답 — 원 요청(`reply_to`) · 실행 정보와 함께.
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
) -> rusqlite::Result<Message> {
    conn.execute(
        "INSERT INTO messages (uuid, conversation_id, author, text, state, created_at, reply_to, turns, duration_ms)
         VALUES (?1, ?2, 'agent', ?3, ?4, unixepoch(), ?5, ?6, ?7)",
        params![uuid, conversation_id, text, state.as_str(), reply_to, turns, duration_ms],
    )?;
    fill_mine_exec(conn, conn.last_insert_rowid())
}

/// 대화 목록 — 마지막 메시지가 최근인 것 먼저(메시지 없으면 대화 생성 시각).
pub fn summaries(conn: &Connection) -> rusqlite::Result<Vec<Summary>> {
    let cols = CONV_COLS
        .split(", ")
        .map(|c| format!("c.{c}"))
        .collect::<Vec<_>>()
        .join(", ");
    conn.prepare(&format!(
        "SELECT {cols},
           (SELECT text FROM messages WHERE conversation_id = c.id AND author = 'human' ORDER BY id LIMIT 1),
           (SELECT state FROM messages WHERE conversation_id = c.id ORDER BY id DESC LIMIT 1),
           (SELECT created_at FROM messages WHERE conversation_id = c.id ORDER BY id DESC LIMIT 1)
         FROM conversations c
         ORDER BY coalesce((SELECT max(created_at) FROM messages WHERE conversation_id = c.id), c.created_at) DESC,
           coalesce((SELECT max(id) FROM messages WHERE conversation_id = c.id), 0) DESC, c.id DESC"
    ))?
    .query_map([], |r| {
        let first: Option<String> = r.get(CONV_N)?;
        let last_state: Option<String> = r.get(CONV_N + 1)?;
        Ok(Summary {
            conversation: conv_row(r)?,
            title: first.unwrap_or_default().chars().take(TITLE_CHARS).collect(),
            last_state: match last_state.as_deref() {
                None => None,
                Some(v) => Some(state_of(v).ok_or_else(|| bad(CONV_N + 1, v))?),
            },
            last_at: r.get(CONV_N + 2)?,
        })
    })?
    .collect()
}

/// 자동 왕복을 켠다(연 쪽, 새 대화) — 카운트 0.
pub fn set_relay(conn: &Connection, id: i64, agent: &str, cap: i64) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE conversations SET relay_agent = ?2, relay_cap = ?3, relay_count = 0, relay_on = 1 WHERE id = ?1",
        params![id, agent, cap],
    )
    .map(drop)
}

/// 받은 쪽: 자동 왕복 대화로 기록 — 상대(연 쪽) 에이전트 이름만(상한 · 켜짐은 연 쪽 것).
pub fn mark_relayed(conn: &Connection, id: i64, agent: &str) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE conversations SET relay_agent = ?2 WHERE id = ?1 AND dir = 'in' AND relay_agent IS NULL",
        params![id, agent],
    )
    .map(drop)
}

/// 자동 왕복 켜기 · 끄기(연 쪽). `reset` 이면 카운트 0(재개).
pub fn relay_on(conn: &Connection, id: i64, on: bool, reset: bool) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE conversations SET relay_on = ?2, relay_count = CASE WHEN ?3 THEN 0 ELSE relay_count END
         WHERE id = ?1 AND relay_agent IS NOT NULL",
        params![id, on, reset],
    )
    .map(drop)
}

/// 왕복 하나를 센다 — 돌려주는 값: 센 뒤 카운트.
pub fn relay_step(conn: &Connection, id: i64) -> rusqlite::Result<i64> {
    conn.execute(
        "UPDATE conversations SET relay_count = coalesce(relay_count, 0) + 1 WHERE id = ?1",
        [id],
    )?;
    conn.query_row(
        "SELECT relay_count FROM conversations WHERE id = ?1",
        [id],
        |r| r.get(0),
    )
}

/// 받은 쪽: 에이전트발 실행 요청 거절 켜기 · 끄기.
pub fn set_blocked(conn: &Connection, id: i64, blocked: bool) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE conversations SET relay_blocked = ?2 WHERE id = ?1",
        params![id, blocked],
    )
    .map(drop)
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

/// 글의 실행 요청 여부 · 내 것 여부 · 상태를 바꾼다(자동 왕복: 상대 답을 내 에이전트 입력으로, 내 에이전트 답을 상대 실행 요청으로).
pub fn set_flags(
    conn: &Connection,
    id: i64,
    mine: bool,
    exec: bool,
    state: State,
) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE messages SET mine = ?2, exec = ?3, state = ?4 WHERE id = ?1",
        params![id, mine, exec, state.as_str()],
    )
    .map(drop)
}
