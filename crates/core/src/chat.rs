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
    /// 글의 출처 — `Some("cli")` = 사용자 AI 에이전트가 CLI 로 쓴 글, `None` = 웹UI(사람). 동작은 사람 글과 같다.
    pub via: Option<String>,
    /// 대화 안 양쪽 공통 순서(램포트 시계) — 쓴 쪽이 정하고 받은 쪽은 그대로 남긴다. 정렬은 `(lc, uuid)`.
    pub lc: i64,
    /// 이 PC 에서 저장한 시각(저장된 항목) — 저장하지 않았으면 `None`.
    pub saved_at: Option<i64>,
    /// 답을 실행한 model · effort — 실행한 쪽이 정해 답과 함께 보낸다(없을 수 있다).
    pub model: Option<String>,
    pub effort: Option<String>,
    /// 사람 글의 종류 — 참 = 요청(건의), 거짓 = 질의(기본). 쓴 쪽이 정해 함께 보낸다.
    pub request: bool,
    /// 요청 완료 표시 시각 — 받은 쪽이 바꾸고 쓴 쪽은 받은 대로 남긴다.
    pub done_at: Option<i64>,
    /// 멘션 — 사람 실행 요청이 부른 에이전트 · 그 답을 실행한 에이전트(`None` = 대화 에이전트).
    pub agent: Option<String>,
    /// 보낸 쪽: 상대가 거절한 코드(`focus` 등).
    pub refused: Option<String>,
}

/// 대화 목록 한 줄.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Summary {
    pub conversation: Conversation,
    /// 이 PC 에서 붙인 제목, 없으면 첫 사람 메시지 앞 [`TITLE_CHARS`] 글자, 그것도 없으면 빈 문자열.
    pub title: String,
    pub last_state: Option<State>,
    pub last_at: Option<i64>,
    /// ★ 즐겨찾기(이 PC 에만).
    pub starred: bool,
    /// 읽은 뒤에 온 상대 쪽 글 수(이 PC 의 읽음).
    pub unread: i64,
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

const MSG_COLS: &str = "id, uuid, conversation_id, author, text, state, created_at, reply_to, turns, duration_ms, mine, exec, lc, via, saved_at, model, effort, request, done_at, agent, refused";
/// `MSG_COLS` 열 수 — 그 뒤에 붙는 열의 위치.
const MSG_N: usize = 21;

/// 열 목록에 표 별칭을 붙인다(`id, uuid` → `m.id, m.uuid`).
fn prefixed(cols: &str, alias: &str) -> String {
    cols.split(", ")
        .map(|c| format!("{alias}.{c}"))
        .collect::<Vec<_>>()
        .join(", ")
}

/// 본문 부분 일치 `LIKE` 패턴 — `\` · `%` · `_` 는 글자 그대로(`ESCAPE '\'` 와 함께 쓴다).
fn like_pattern(q: &str) -> String {
    let esc = q
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_");
    format!("%{esc}%")
}

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
        via: r.get(13)?,
        saved_at: r.get(14)?,
        model: r.get(15)?,
        effort: r.get(16)?,
        request: r.get::<_, Option<bool>>(17)?.unwrap_or(false),
        done_at: r.get(18)?,
        agent: r.get(19)?,
        refused: r.get(20)?,
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

/// 메시지 저장 · 해제(이 PC 에만). 이미 저장돼 있으면 저장 시각은 그대로. 없는 메시지면 `false`.
pub fn set_saved(conn: &Connection, id: i64, on: bool) -> rusqlite::Result<bool> {
    Ok(conn.execute(
        "UPDATE messages SET saved_at = CASE WHEN ?2 THEN coalesce(saved_at, unixepoch()) END WHERE id = ?1",
        params![id, on],
    )? == 1)
}

/// 저장된 메시지 — 저장 시각 최신부터(같은 초면 id 큰 것), `peer` · `conversation` 으로 거른다.
pub fn saved(
    conn: &Connection,
    peer: Option<i64>,
    conversation: Option<i64>,
    limit: i64,
) -> rusqlite::Result<Vec<Message>> {
    let cols = prefixed(MSG_COLS, "m");
    conn.prepare(&format!(
        "SELECT {cols} FROM messages m JOIN conversations c ON c.id = m.conversation_id
         WHERE m.saved_at IS NOT NULL AND (?1 IS NULL OR c.peer_id = ?1) AND (?2 IS NULL OR c.id = ?2)
         ORDER BY m.saved_at DESC, m.id DESC LIMIT ?3"
    ))?
    .query_map(params![peer, conversation, limit], msg_row)?
    .collect()
}

/// 자동 왕복 기록 종류.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelayMark {
    Start,
    End,
}

/// 자동 왕복 시작 · 끝을 남긴다(내 활동) — 알림 설정과 무관하게. `detail` = 끝 사유 · 시작 글 미리보기. 같은 표의 `failed` 행은 내 글이 실패로
/// 바뀐 순간([`set_state`] · [`set_flags`] 가 남긴다 — 목록에는 그 글로 보인다).
pub fn add_relay_event(
    conn: &Connection,
    conversation_id: i64,
    mark: RelayMark,
    detail: &str,
) -> rusqlite::Result<()> {
    let kind = match mark {
        RelayMark::Start => "start",
        RelayMark::End => "end",
    };
    conn.execute(
        "INSERT INTO relay_events (conversation_id, kind, detail, created_at) VALUES (?1, ?2, ?3, unixepoch())",
        params![conversation_id, kind, detail],
    )
    .map(drop)
}

/// 내 활동 한 줄의 종류 — 상대 쪽 글 · 내 쪽 실패 · 자동 왕복 시작 · 끝.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivityKind {
    Message,
    Failed,
    RelayStart,
    RelayEnd,
}

impl ActivityKind {
    pub fn as_str(self) -> &'static str {
        match self {
            ActivityKind::Message => "message",
            ActivityKind::Failed => "failed",
            ActivityKind::RelayStart => "relay_start",
            ActivityKind::RelayEnd => "relay_end",
        }
    }
}

/// 내 활동 거르기 — 전체 · 상대 사람 글 · 상대 에이전트 답 · 자동 왕복 · 실패.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivityFilter {
    All,
    Human,
    Agent,
    Relay,
    Failed,
    /// 상대 쪽 사람 글 중 질의 · 요청('내게 요청된 목록').
    Query,
    Request,
}

/// 내 활동 한 줄.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Activity {
    pub kind: ActivityKind,
    /// 남은 시각(유닉스 초) — 글이면 저장 시각.
    pub at: i64,
    pub conversation_id: i64,
    pub peer_id: i64,
    /// 글(상대 쪽 글 · 실패한 내 글) — 자동 왕복 기록이면 `None`.
    pub message: Option<Message>,
    /// 자동 왕복 기록의 사유 · 미리보기.
    pub detail: Option<String>,
    /// 그 상대의 내 활동을 본 마지막 글 · 기록(id) 뒤의 것(지운 상대의 것은 본 것으로 — 본 표시를 둘 곳이 없다).
    pub unseen: bool,
}

/// 내 활동에 드는 글 — 상대 쪽 글 전부 · 실패한 내 쪽 글. `messages m` 문맥의 SQL 식.
const ACTIVITY_MESSAGES: &str = "(m.mine = 0 OR m.state = 'error')";

/// 내 활동 — 최신 것부터 `limit` 건. `peer` 로 그 상대 대화만, `q` 는 글 본문 · 자동 왕복 기록 사유의 부분 일치.
pub fn activity(
    conn: &Connection,
    peer: Option<i64>,
    filter: ActivityFilter,
    q: Option<&str>,
    limit: i64,
) -> rusqlite::Result<Vec<Activity>> {
    let like = q.map(like_pattern);
    let mut out = Vec::new();
    let msg_where = match filter {
        ActivityFilter::All => Some(ACTIVITY_MESSAGES),
        ActivityFilter::Human => Some("m.mine = 0 AND m.author = 'human'"),
        ActivityFilter::Agent => Some("m.mine = 0 AND m.author = 'agent'"),
        ActivityFilter::Failed => Some("m.mine = 1 AND m.state = 'error'"),
        ActivityFilter::Query => {
            Some("m.mine = 0 AND m.author = 'human' AND coalesce(m.request, 0) = 0")
        }
        ActivityFilter::Request => Some("m.mine = 0 AND m.author = 'human' AND m.request = 1"),
        ActivityFilter::Relay => None,
    };
    if let Some(w) = msg_where {
        let cols = prefixed(MSG_COLS, "m");
        let mut stmt = conn.prepare(&format!(
            "SELECT {cols}, c.peer_id, p.id IS NOT NULL AND (m.id > coalesce(p.activity_seen_msg, 0)
               OR EXISTS (SELECT 1 FROM relay_events f WHERE f.message_id = m.id AND f.kind = 'failed'
                 AND f.id > coalesce(p.activity_seen_event, 0)))
             FROM messages m JOIN conversations c ON c.id = m.conversation_id LEFT JOIN peers p ON p.id = c.peer_id
             WHERE {w} AND (?1 IS NULL OR c.peer_id = ?1) AND (?2 IS NULL OR m.text LIKE ?2 ESCAPE '\\')
             ORDER BY m.created_at DESC, m.id DESC LIMIT ?3"
        ))?;
        let rows = stmt.query_map(params![peer, like, limit], |r| {
            let m = msg_row(r)?;
            Ok(Activity {
                kind: if m.mine {
                    ActivityKind::Failed
                } else {
                    ActivityKind::Message
                },
                at: m.created_at,
                conversation_id: m.conversation_id,
                peer_id: r.get(MSG_N)?,
                unseen: r.get(MSG_N + 1)?,
                message: Some(m),
                detail: None,
            })
        })?;
        for a in rows {
            out.push(a?);
        }
    }
    if matches!(filter, ActivityFilter::All | ActivityFilter::Relay) {
        let mut stmt = conn.prepare(
            "SELECT e.kind, e.created_at, e.conversation_id, c.peer_id, e.detail,
               p.id IS NOT NULL AND e.id > coalesce(p.activity_seen_event, 0)
             FROM relay_events e JOIN conversations c ON c.id = e.conversation_id LEFT JOIN peers p ON p.id = c.peer_id
             WHERE e.kind IN ('start', 'end') AND (?1 IS NULL OR c.peer_id = ?1) AND (?2 IS NULL OR e.detail LIKE ?2 ESCAPE '\\')
             ORDER BY e.created_at DESC, e.id DESC LIMIT ?3",
        )?;
        let rows = stmt.query_map(params![peer, like, limit], |r| {
            let kind: String = r.get(0)?;
            Ok(Activity {
                kind: match kind.as_str() {
                    "start" => ActivityKind::RelayStart,
                    "end" => ActivityKind::RelayEnd,
                    v => return Err(bad(0, v)),
                },
                at: r.get(1)?,
                conversation_id: r.get(2)?,
                peer_id: r.get(3)?,
                message: None,
                detail: Some(r.get(4)?),
                unseen: r.get(5)?,
            })
        })?;
        for a in rows {
            out.push(a?);
        }
    }
    // 같은 초면 글이 먼저(안정 정렬) — 두 목록 각각은 이미 최신순.
    out.sort_by_key(|a| std::cmp::Reverse(a.at));
    out.truncate(usize::try_from(limit).unwrap_or(0));
    Ok(out)
}

/// 내 활동 안 본 수 — `peer` 가 있으면 그 상대만(지운 상대의 것은 세지 않는다). 본 뒤 온 글 + 본 뒤 실패로 바뀐 내 글(이미 본 id) + 본 뒤
/// 자동 왕복 기록. 가장 앞선 본 표시 뒤의 id 만 훑는다(5초 새로 고침).
pub fn activity_unseen(conn: &Connection, peer: Option<i64>) -> rusqlite::Result<i64> {
    conn.query_row(
        &format!(
            "SELECT
               (SELECT count(*) FROM messages m JOIN conversations c ON c.id = m.conversation_id
                  LEFT JOIN peers p ON p.id = c.peer_id
                WHERE m.id > coalesce((SELECT min(coalesce(activity_seen_msg, 0)) FROM peers WHERE ?1 IS NULL OR id = ?1), 0)
                  AND {ACTIVITY_MESSAGES} AND (?1 IS NULL OR c.peer_id = ?1)
                  AND p.id IS NOT NULL AND m.id > coalesce(p.activity_seen_msg, 0))
             + (SELECT count(DISTINCT m.id) FROM relay_events f JOIN messages m ON m.id = f.message_id
                  JOIN conversations c ON c.id = m.conversation_id LEFT JOIN peers p ON p.id = c.peer_id
                WHERE f.id > coalesce((SELECT min(coalesce(activity_seen_event, 0)) FROM peers WHERE ?1 IS NULL OR id = ?1), 0)
                  AND f.kind = 'failed' AND m.state = 'error' AND (?1 IS NULL OR c.peer_id = ?1)
                  AND p.id IS NOT NULL AND f.id > coalesce(p.activity_seen_event, 0) AND m.id <= coalesce(p.activity_seen_msg, 0))
             + (SELECT count(*) FROM relay_events e JOIN conversations c ON c.id = e.conversation_id
                  LEFT JOIN peers p ON p.id = c.peer_id
                WHERE e.id > coalesce((SELECT min(coalesce(activity_seen_event, 0)) FROM peers WHERE ?1 IS NULL OR id = ?1), 0)
                  AND e.kind IN ('start', 'end') AND (?1 IS NULL OR c.peer_id = ?1)
                  AND p.id IS NOT NULL AND e.id > coalesce(p.activity_seen_event, 0))"
        ),
        params![peer],
        |r| r.get(0),
    )
}

/// 지금 마지막 글 · 자동 왕복 기록 id — 내 활동 화면이 받은 범위(본 표시는 이 범위까지만 보낸다).
pub fn activity_upto(conn: &Connection) -> rusqlite::Result<(i64, i64)> {
    conn.query_row(
        "SELECT (SELECT coalesce(max(id), 0) FROM messages), (SELECT coalesce(max(id), 0) FROM relay_events)",
        [],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )
}

/// 내 활동을 `(message, event)` 까지 봤다(이 PC, [`activity_upto`] 로 받은 범위) — `peer` 가 있으면 그 상대만, 없으면 전부. 본 표시는
/// 뒤로 가지 않고 지금 마지막 항목을 넘지 않는다(앞으로 올 것을 미리 본 것으로 하지 않게). 돌려주는 값: 바꾼 상대 수.
pub fn mark_activity_seen(
    conn: &Connection,
    peer: Option<i64>,
    (message, event): (i64, i64),
) -> rusqlite::Result<usize> {
    conn.execute(
        "UPDATE peers SET
           activity_seen_msg = max(coalesce(activity_seen_msg, 0), min(?2, (SELECT coalesce(max(id), 0) FROM messages))),
           activity_seen_event = max(coalesce(activity_seen_event, 0), min(?3, (SELECT coalesce(max(id), 0) FROM relay_events)))
         WHERE ?1 IS NULL OR id = ?1",
        params![peer, message, event],
    )
}

/// 통합 검색 결과 — 묶음별.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Found {
    pub messages: Vec<Message>,
    pub conversations: Vec<Summary>,
    pub peers: Vec<crate::peers::Peer>,
    pub agents: Vec<FoundAgent>,
}

/// 검색에 걸린 에이전트 — `owner` 가 `None` 이면 내 에이전트, 아니면 그 상대(대화에 나온 이름).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FoundAgent {
    pub name: String,
    pub owner: Option<i64>,
}

/// 통합 검색의 메시지 밖 묶음 상한.
pub const SEARCH_OTHERS: usize = 10;

/// 통합 검색 — 메시지 본문(양쪽 글, 최신 `limit` 건) · 대화 제목 · 사람(이름 · 표시 이름 · 호스트) · 에이전트(내 것의 이름 ·
/// 폴더, 대화에 나온 상대 것의 이름). 글자 그대로 부분 일치(영문은 대소문자 무시), 메시지 밖은 묶음마다 [`SEARCH_OTHERS`] 건.
pub fn search(conn: &Connection, q: &str, limit: i64) -> rusqlite::Result<Found> {
    let ql = q.to_lowercase();
    let has = |s: &str| s.to_lowercase().contains(&ql);
    let messages = messages_across(conn, None, None, None, None, Some(q), limit)?;
    let conversations = summaries(conn)?
        .into_iter()
        .filter(|s| has(&s.title))
        .take(SEARCH_OTHERS)
        .collect();
    let peers = crate::peers::list(conn)?
        .into_iter()
        .filter(|p| has(&p.name) || has(&p.host) || p.alias.as_deref().is_some_and(has))
        .take(SEARCH_OTHERS)
        .collect();
    let mut agents: Vec<FoundAgent> = crate::agents::list(conn)?
        .into_iter()
        .filter(|a| has(&a.name) || has(&a.folder))
        .map(|a| FoundAgent {
            name: a.name,
            owner: None,
        })
        .collect();
    let mut stmt = conn.prepare(
        "SELECT DISTINCT peer_id, agent FROM conversations
         WHERE dir = 'out' AND peer_id IN (SELECT id FROM peers) ORDER BY peer_id, agent",
    )?;
    for row in stmt.query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))? {
        let (peer, name) = row?;
        if has(&name) {
            agents.push(FoundAgent {
                name,
                owner: Some(peer),
            });
        }
    }
    agents.truncate(SEARCH_OTHERS);
    Ok(Found {
        messages,
        conversations,
        peers,
        agents,
    })
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

/// 사람 글의 종류 — 참 = 요청(건의), 거짓 = 질의(기본). 글과 같은 트랜잭션에서 부른다.
pub fn set_request(conn: &Connection, id: i64, request: bool) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE messages SET request = ?2 WHERE id = ?1",
        params![id, request],
    )
    .map(drop)
}

/// 멘션 — 사람 실행 요청이 부른 · 답을 실행한 에이전트(`None` = 대화 에이전트). 글과 같은 트랜잭션에서 부른다.
pub fn set_agent(conn: &Connection, id: i64, agent: Option<&str>) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE messages SET agent = ?2 WHERE id = ?1",
        params![id, agent],
    )
    .map(drop)
}

/// 요청 완료 표시 · 취소 — 이미 완료면 처음 시각을 둔다.
pub fn set_done(conn: &Connection, id: i64, done: bool) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE messages SET done_at = CASE WHEN ?2 THEN coalesce(done_at, unixepoch()) END WHERE id = ?1",
        params![id, done],
    )
    .map(drop)
}

/// 로컬 id 로 메시지.
pub fn message_by_id(conn: &Connection, id: i64) -> rusqlite::Result<Option<Message>> {
    conn.query_row(
        &format!("SELECT {MSG_COLS} FROM messages WHERE id = ?1"),
        [id],
        msg_row,
    )
    .optional()
}

/// 상대가 거절한 내 글 — 거절 코드(`focus` 등)를 남긴다(상태는 따로 `set_state`). 다시 보내면 `None` 으로 지운다.
pub fn set_refused(conn: &Connection, id: i64, code: Option<&str>) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE messages SET refused = ?2 WHERE id = ?1",
        params![id, code],
    )
    .map(drop)
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

/// 대화를 가로지른 글 — `mine` 이 참이면 내가 쓴 글, 거짓이면 상대 쪽이 쓴 글, `None` 이면 둘 다. `peer` 로 그 상대 대화만,
/// `author` 로 그 종류만, `q` 는 본문 부분 일치. 거른 뒤 최신 것부터 `limit` 건.
pub fn messages_across(
    conn: &Connection,
    mine: Option<bool>,
    peer: Option<i64>,
    author: Option<Author>,
    request: Option<bool>,
    q: Option<&str>,
    limit: i64,
) -> rusqlite::Result<Vec<Message>> {
    let cols = prefixed(MSG_COLS, "m");
    let like = q.map(like_pattern);
    conn.prepare(&format!(
        "SELECT {cols} FROM messages m JOIN conversations c ON c.id = m.conversation_id
         WHERE (?1 IS NULL OR m.mine = ?1) AND (?2 IS NULL OR c.peer_id = ?2) AND (?3 IS NULL OR m.author = ?3)
           AND (?4 IS NULL OR (m.author = 'human' AND coalesce(m.request, 0) = ?4))
           AND (?5 IS NULL OR m.text LIKE ?5 ESCAPE '\\')
         ORDER BY m.created_at DESC, m.id DESC LIMIT ?6"
    ))?
    .query_map(
        params![mine, peer, author.map(Author::as_str), request, like, limit],
        msg_row,
    )?
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

/// 대화 목록 — 마지막 메시지가 최근인 것 먼저(메시지 없으면 대화 생성 시각).
pub fn summaries(conn: &Connection) -> rusqlite::Result<Vec<Summary>> {
    let cols = prefixed(CONV_COLS, "c");
    conn.prepare(&format!(
        "SELECT {cols},
           (SELECT text FROM messages WHERE conversation_id = c.id AND author = 'human' ORDER BY id LIMIT 1),
           (SELECT state FROM messages WHERE conversation_id = c.id ORDER BY id DESC LIMIT 1),
           (SELECT created_at FROM messages WHERE conversation_id = c.id ORDER BY id DESC LIMIT 1),
           c.title,
           c.starred_at IS NOT NULL,
           (SELECT count(*) FROM messages WHERE conversation_id = c.id AND mine = 0 AND id > coalesce(c.read_id, 0))
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
