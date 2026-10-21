//! 대화 · 메시지 이력(D19 · S3) — 양쪽 PC 가 같은 대화 UUID 로 각자 남긴다.
use rusqlite::{Connection, OptionalExtension, Row, params};

/// `Out` = 내가 원격 에이전트에게 · `In` = 내 에이전트가 원격에게서 받음 · `Shared` = 상대(대화를 가진 PC)가 공유한 읽기 전용 사본(#1o —
/// 상대 id = 그 PC).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dir {
    Out,
    In,
    Shared,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Author {
    Human,
    Agent,
    /// 컨텍스트 구분선(#1m) — 압축 · 비우기. 본문 = 요약(압축이 끝났으면) · 종류는 `Message.ctx_op`.
    Context,
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

/// 요청 글의 상태(#1d) — 받은 쪽 사람이 바꾸고 쓴 쪽은 받은 대로 남긴다. 대기 → 수용 · 반려, 수용 → 완료.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReqState {
    Wait,
    Accepted,
    Rejected,
    Done,
}

impl ReqState {
    pub const ALL: [ReqState; 4] = [
        ReqState::Wait,
        ReqState::Accepted,
        ReqState::Rejected,
        ReqState::Done,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            ReqState::Wait => "wait",
            ReqState::Accepted => "accepted",
            ReqState::Rejected => "rejected",
            ReqState::Done => "done",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|r| r.as_str() == s)
    }

    /// 받은 쪽이 이 상태에서 `next` 로 옮길 수 있나 — 지금 상태 말고 네 상태 어디로든(되돌리기 포함 — 사용자 2026-10-19).
    pub fn can_become(self, next: Self) -> bool {
        self != next
    }
}

/// 요청 상태 의견 최대 글자 수.
pub const MAX_REQ_NOTE: usize = 200;

/// 요청 상태 의견 — 여러 줄(#1z — CRLF 는 LF, 줄바꿈 말고 제어 문자 없음) [`MAX_REQ_NOTE`] 자까지, 비면 `None`.
pub fn clean_req_note(s: &str) -> Result<Option<String>, String> {
    let s = s.replace("\r\n", "\n");
    let s = s.trim();
    if s.chars().count() > MAX_REQ_NOTE || s.chars().any(|c| c.is_control() && c != '\n') {
        return Err(format!("의견은 {MAX_REQ_NOTE}자까지, 제어 문자 없이"));
    }
    Ok((!s.is_empty()).then(|| s.to_string()))
}

/// 상대에게 보내는 의견 꼴(#1z) — 줄바꿈을 U+2028(LINE SEPARATOR)로. U+2028 은 제어 문자(Cc)가 아니라 줄바꿈을 거절하는 이전 버전
/// 검사(`clean_label`)를 통과한다(호환 — 사용자 2026-10-21). 받는 쪽은 [`note_from_wire`] 로 되돌린다.
pub fn note_to_wire(s: &str) -> String {
    s.replace('\n', "\u{2028}")
}

pub fn note_from_wire(s: &str) -> String {
    s.replace('\u{2028}', "\n")
}

/// 사람 글 종류 거르기 — 질의 · 요청(요청은 상태로 더 거른다, `None` = 모든 상태).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AskFilter {
    Query,
    /// 요청 — 상태 거르기, 둘째 = 참이면 archive 된 것만 · 거짓이면 archive 제외(#1y).
    Request(Option<ReqState>, bool),
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
    /// 이 PC 가 나간 시각(#1w) — 그 뒤 이 대화로 온 요청은 거절(`left`), 읽기 전용.
    pub left_at: Option<i64>,
    /// 상대가 나간 시각(#1w — 상대의 `Leave`) — 읽기 전용.
    pub peer_left_at: Option<i64>,
}

impl Conversation {
    /// 나간 대화(#1w) — 내가 나갔거나 상대가 나갔다. 읽기 전용(글쓰기 · 실행 · 보내기 없음).
    pub fn closed(&self) -> bool {
        self.left_at.is_some() || self.peer_left_at.is_some()
    }
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
    /// 요청 글의 상태 · 메모 · 바꾼 시각(#1d) — 요청이 아니면 `None`.
    pub req_state: Option<ReqState>,
    pub req_note: Option<String>,
    pub req_at: Option<i64>,
    /// 멘션 — 사람 실행 요청이 부른 에이전트 · 그 답을 실행한 에이전트(`None` = 대화 에이전트).
    pub agent: Option<String>,
    /// 보낸 쪽: 상대가 거절한 코드(`focus` 등).
    pub refused: Option<String>,
    /// 편집한 시각(#1f — 내 사람 글 · 상대가 편집해 보낸 글) — 편집하지 않았으면 `None`.
    pub edited_at: Option<i64>,
    /// 답을 낸 세션의 창 사용량(토큰, #1m) — 모르면 `None`.
    pub ctx_tokens: Option<i64>,
    /// 컨텍스트 구분선의 종류(#1m) — `compact` · `auto_compact` · `clear`.
    pub ctx_op: Option<String>,
    /// 공유 사본(#1o)의 글 — 대화를 가진 PC 쪽 글이면 참(그 밖 대화는 거짓).
    pub owner: bool,
    /// 요청 archive 손 표시(#1y) — `None` = 자동(완료 [`ARCHIVE_DAYS`] 일 뒤) · 참 = 손으로 archive · 거짓 = 손으로 해제.
    pub req_archived: Option<bool>,
}

/// 완료 요청이 archive 되기까지 날 수(#1y).
pub const ARCHIVE_DAYS: i64 = 7;

impl Message {
    /// archive 된 요청인가(#1y) — 완료이고 손으로 archive 했거나 · 손 표시 없이 완료 뒤 [`ARCHIVE_DAYS`] 일 지남. [`archived_sql`] 과 같은 규칙.
    pub fn archived(&self, now: i64) -> bool {
        self.req_state == Some(ReqState::Done)
            && match self.req_archived {
                Some(on) => on,
                None => self.req_at.unwrap_or(self.created_at) <= now - ARCHIVE_DAYS * 86400,
            }
    }
}

/// 지금 시각(유닉스 초) — [`Message::archived`] 에 넘긴다.
pub fn unix_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() as i64)
}

/// [`Message::archived`] 의 SQL 식 — `m` = `messages` 표 별칭.
pub fn archived_sql(m: &str) -> String {
    format!(
        "({m}.req_state = 'done' AND coalesce({m}.req_archived, coalesce({m}.req_at, {m}.created_at) <= unixepoch() - {ARCHIVE_DAYS} * 86400))"
    )
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
    /// 손으로 놓은 자리(#1i — 이 PC 에만, 작을수록 위). 목록 순서 자체는 최근 활동순 그대로.
    pub pos: i64,
    /// 마지막 글(#1j — 대화방 목록 미리보기): 본문 앞 [`PREVIEW_CHARS`] 글자 · 쓴 쪽(`human` · `agent`) · 이 PC 쪽 글인가 · 멘션 에이전트. 글이 없으면 `None`.
    pub last: Option<LastMessage>,
}

/// 대화방 목록 미리보기의 마지막 글(#1j) — 대화 화면과 같은 순서(`(lc, uuid)`)의 마지막(라운드 1 — 로컬 `id` 순이 아니다). 요약의 마지막 상태 · 시각도
/// 같은 글에서 읽는다(라운드 2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LastMessage {
    pub text: String,
    pub author: Author,
    pub mine: bool,
    pub agent: Option<String>,
    /// 글 출처(`cli` = AI 에이전트(CLI) 가 쓴 글, 라운드 2 — 말한 이 표시).
    pub via: Option<String>,
}

/// 대화방 미리보기 길이(글자).
pub const PREVIEW_CHARS: usize = 120;

/// 대화 제목 길이(글자).
pub const TITLE_CHARS: usize = 40;

impl Dir {
    pub fn as_str(self) -> &'static str {
        match self {
            Dir::Out => "out",
            Dir::In => "in",
            Dir::Shared => "shared",
        }
    }
}

impl Author {
    pub fn as_str(self) -> &'static str {
        match self {
            Author::Human => "human",
            Author::Agent => "agent",
            Author::Context => "context",
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

const MSG_COLS: &str = "id, uuid, conversation_id, author, text, state, created_at, reply_to, turns, duration_ms, mine, exec, lc, via, saved_at, model, effort, request, req_state, req_note, req_at, agent, refused, edited_at, ctx_tokens, ctx_op, owner, req_archived";
/// `MSG_COLS` 열 수 — 그 뒤에 붙는 열의 위치.
const MSG_N: usize = 28;

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
    let req_state = match r.get::<_, Option<String>>(18)? {
        Some(v) => Some(ReqState::parse(&v).ok_or_else(|| bad(18, &v))?),
        None => None,
    };
    Ok(Message {
        id: r.get(0)?,
        uuid: r.get(1)?,
        conversation_id: r.get(2)?,
        author: match author.as_str() {
            "human" => Author::Human,
            "agent" => Author::Agent,
            "context" => Author::Context,
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
        req_state,
        req_note: r.get(19)?,
        req_at: r.get(20)?,
        agent: r.get(21)?,
        refused: r.get(22)?,
        edited_at: r.get(23)?,
        ctx_tokens: r.get(24)?,
        ctx_op: r.get(25)?,
        owner: r.get::<_, Option<bool>>(26)?.unwrap_or(false),
        req_archived: r.get(27)?,
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
    // 지운 대화(#1w — 묘비)는 되살리지 않는다 — 받기 앞 확인과 삭제 사이에 끼어든 글도 여기서 막힌다.
    if conn
        .prepare("SELECT 1 FROM left_convs WHERE uuid = ?1 AND peer_id = ?2")?
        .exists(params![uuid, peer_id])?
    {
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }
    conn.execute(
        // 새 대화는 맨 위(#1i — 손으로 놓은 순서의 앞).
        "INSERT INTO conversations (uuid, peer_id, agent, dir, created_at, pos)
         VALUES (?1, ?2, ?3, ?4, unixepoch(), (SELECT coalesce(min(pos), 0) - 1 FROM conversations))",
        params![uuid, peer_id, agent, dir.as_str()],
    )?;
    by_id(conn, conn.last_insert_rowid()).map(|c| c.expect("방금 쓴 대화"))
}

/// [`find`] 중 공유 사본(#1o)이 아닌 것 — 상대의 대화 프레임은 이 PC 의 사본을 건드리지 못한다(사본은 `share` 만 쓴다).
pub fn find_own(
    conn: &Connection,
    uuid: &str,
    peer_id: i64,
) -> rusqlite::Result<Option<Conversation>> {
    Ok(find(conn, uuid, peer_id)?.filter(|c| c.dir != Dir::Shared))
}

/// [`find_own`] 중 닫히지 않은 것(#1w) — 상대의 대화 프레임이 이력을 바꾸는 받기 처리기가 쓴다(받기 앞 `left` 확인 뒤 나가기가 끼어도 바꾸기와 같은
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

/// 메시지 저장 · 해제(이 PC 에만). 이미 저장돼 있으면 저장 시각은 그대로. 없는 메시지면 `false`.
pub fn set_saved(conn: &Connection, id: i64, on: bool) -> rusqlite::Result<bool> {
    Ok(conn.execute(
        "UPDATE messages SET saved_at = CASE WHEN ?2 THEN coalesce(saved_at, unixepoch()) END WHERE id = ?1",
        params![id, on],
    )? == 1)
}

/// 본문을 바꾸고 편집 시각을 남긴다(#1f) — uuid · 순서 · 요청 상태 · 저장은 그대로. 바뀐 행이 없으면 거짓.
pub fn edit_text(conn: &Connection, id: i64, text: &str) -> rusqlite::Result<bool> {
    conn.execute(
        "UPDATE messages SET text = ?2, edited_at = unixepoch() WHERE id = ?1",
        params![id, text],
    )
    .map(|n| n > 0)
}

/// 글을 흔적 없이 지운다(#1f) — 그 글의 CLI 트랜잭션 · 실패 기록 · 실패 보관 행도. 답(`reply_to`)은 남는다. 지운 행이 없으면 거짓.
/// 읽음 · 내 활동 본 표시 · 공유 대화의 보낸 커서(#1o)는 남은 마지막 id 로 낮춘다 — 맨 끝 글을 지우면 SQLite 가 그 id 를 다음 글에 다시 주므로,
/// 표시가 그대로면 새 글이 이미 읽은 · 본 · 관람자에게 보낸 것으로 보인다.
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

/// 글 · 기록을 지운 뒤 — 읽음 · 내 활동 본 표시 · 공유 보낸 커서를 남은 마지막 id 로 낮춘다([`delete_message`]).
fn lower_marks(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "UPDATE conversations SET read_id = (SELECT coalesce(max(id), 0) FROM messages)
           WHERE read_id > (SELECT coalesce(max(id), 0) FROM messages);
         UPDATE peers SET activity_seen_msg = (SELECT coalesce(max(id), 0) FROM messages)
           WHERE activity_seen_msg > (SELECT coalesce(max(id), 0) FROM messages);
         UPDATE peers SET activity_seen_event = (SELECT coalesce(max(id), 0) FROM relay_events)
           WHERE activity_seen_event > (SELECT coalesce(max(id), 0) FROM relay_events);
         UPDATE shares SET sent_upto = (SELECT coalesce(max(id), 0) FROM messages)
           WHERE sent_upto > (SELECT coalesce(max(id), 0) FROM messages);",
    )
}

/// 나간 대화의 거절 코드(#1w) — 나간 쪽이 그 대화로 온 요청에 답하고, 나가며 닫은 글에 남긴다.
pub const LEFT: &str = "left";

/// 닫힌 대화 · 지운 대화(묘비)인가(#1w) — 이 PC 가 나갔거나 그 상대가 나갔다(`Leave` 를 보낸 상대의 글도 받지 않는다 — 원격 입력은 신뢰하지 않는다).
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

/// 실행 시작(#1w) — 대기 글(`queued`)이고 대화가 닫히지 않았을 때만 `running` 으로(한 문장 — 확인과 바꾸기 사이에 나가기 · 삭제가 끼지 못한다).
/// 바꿨으면 참 — 거짓이면 실행하지 않는다(그 사이 취소 · 나감 · 지움).
pub fn start_run(conn: &Connection, message_id: i64) -> rusqlite::Result<bool> {
    Ok(conn.execute(
        "UPDATE messages SET state = 'running' WHERE id = ?1 AND state = 'queued'
           AND conversation_id IN (SELECT id FROM conversations WHERE left_at IS NULL AND peer_left_at IS NULL)",
        [message_id],
    )? == 1)
}

/// 도는 실행이 있나(#1w — 실행 중이면 나가기 · 삭제를 막는다). 구분선(압축)도 실행이다.
pub fn running(conn: &Connection, id: i64) -> rusqlite::Result<bool> {
    conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM messages WHERE conversation_id = ?1 AND state = 'running')",
        [id],
        |r| r.get(0),
    )
}

/// 기록 삭제로 지운 글 · 적용한 `Clear` 의 uuid 인가(#1x — 받은 글 · 지우기의 중복 확인).
pub fn gone(conn: &Connection, uuid: &str) -> rusqlite::Result<bool> {
    conn.prepare("SELECT 1 FROM gone_msgs WHERE uuid = ?1")?.exists([uuid])
}

/// 적용한 `Clear` 를 남긴다(#1x) — 같은 지우기가 다시 와도 두 번 지우지 않는다.
pub fn mark_gone(conn: &Connection, uuid: &str, conversation_id: i64) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO gone_msgs (uuid, conversation_id) VALUES (?1, ?2)",
        params![uuid, conversation_id],
    )
    .map(drop)
}

/// 글이 하나도 없는 대화인가(#1x — 세션 생성으로 연 대화의 첫 글).
pub fn empty(conn: &Connection, id: i64) -> rusqlite::Result<bool> {
    conn.query_row(
        "SELECT NOT EXISTS (SELECT 1 FROM messages WHERE conversation_id = ?1)",
        [id],
        |r| r.get(0),
    )
}

/// 실행 중이거나 실행을 기다리는 글이 있는가(#1x — 기록 삭제를 막는다: 대기열이 지운 글을 찾지 않게).
pub fn busy(conn: &Connection, id: i64) -> rusqlite::Result<bool> {
    conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM messages WHERE conversation_id = ?1 AND state IN ('running', 'queued'))",
        [id],
        |r| r.get(0),
    )
}

/// 끝나지 않은 글을 닫는다(#1w) — 실행 대기(`queued`) · 보낸 뒤 상대를 기다리는 내 글(`sent`)은 실패 + [`LEFT`], 자동 왕복은 끈다.
fn close_pending(conn: &Connection, id: i64) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE messages SET state = 'error', refused = ?2
         WHERE conversation_id = ?1 AND (state = 'queued' OR (mine = 1 AND state = 'sent'))",
        params![id, LEFT],
    )?;
    conn.execute("UPDATE conversations SET relay_on = 0 WHERE id = ?1", [id])
        .map(drop)
}

/// 이 PC 가 나간다(#1w) — 끝나지 않은 글 닫기 · 자동 왕복 끔 · 나간 시각(이미 나갔으면 그 시각 그대로). 없는 대화면 거짓.
pub fn leave(conn: &Connection, id: i64) -> rusqlite::Result<bool> {
    close_pending(conn, id)?;
    Ok(conn.execute(
        "UPDATE conversations SET left_at = coalesce(left_at, unixepoch()) WHERE id = ?1",
        [id],
    )? == 1)
}

/// 상대가 나갔다(#1w — 받은 `Leave`) — [`leave`] 와 같이 닫고 상대 나간 시각. 이미 받았으면 거짓.
pub fn set_peer_left(conn: &Connection, id: i64) -> rusqlite::Result<bool> {
    close_pending(conn, id)?;
    Ok(conn.execute(
        "UPDATE conversations SET peer_left_at = unixepoch() WHERE id = ?1 AND peer_left_at IS NULL",
        [id],
    )? == 1)
}

/// 대화를 이 PC 에서 지운다(#1w — D40) — 글 · 실행 · 에이전트 세션 기록 · 요약 · 그룹 소속 · 공유 · 보낼 글 · 기록 · CLI 트랜잭션. 끝 알림 행
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
    // 기다리는 기록 지우기(#1x)도 — 지우기를 고른 뒤 나가도 상대 쪽은 비워지게.
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

/// 대화 기록을 이 PC 에서 비운다(#1x — 기록 삭제) — 글 · 실행 기록 · 그 글의 보낼 행 · 에이전트 세션 기록 · 요약 · CLI 트랜잭션. 대화 행 · 제목 · 그룹 ·
/// 즐겨찾기 · 공유 · 자동 왕복 설정은 남는다(대화는 이어 쓴다). 묘비 없음. 없는 대화면 거짓.
pub fn clear_conversation(conn: &Connection, id: i64) -> rusqlite::Result<bool> {
    if by_id(conn, id)?.is_none() {
        return Ok(false);
    }
    // 지운 글의 uuid 는 남긴다 — 늦게 다시 온 같은 글이 기록을 되살리거나 다시 실행되지 않게(적대적 평가 R1).
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

/// 목록 범위(#1i) — 전부 · 한 상대의 대화 · 한 그룹(이 PC 의 그룹 소속 — 넣은 대화, #1w 그룹 = 대화만)의 대화.
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

/// 자동 왕복 기록 종류 — 같은 표에 상대 반영 실패(#1f)도 남는다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelayMark {
    Start,
    End,
    /// 편집 · 삭제를 상대 창에 반영하지 못했다(상대 거절 · 24시간 포기). 편집이면 그 글 id — 글을 지우면 기록(본문 미리보기)도 함께 지운다.
    SyncFailed(Option<i64>),
}

/// 자동 왕복 시작 · 끝을 남긴다(내 활동) — 알림 설정과 무관하게. `detail` = 끝 사유 · 시작 글 미리보기. 같은 표의 `failed` 행은 내 글이 실패로
/// 바뀐 순간([`set_state`] · [`set_flags`] 가 남긴다 — 목록에는 그 글로 보인다).
pub fn add_relay_event(
    conn: &Connection,
    conversation_id: i64,
    mark: RelayMark,
    detail: &str,
) -> rusqlite::Result<()> {
    let (kind, message) = match mark {
        RelayMark::Start => ("start", None),
        RelayMark::End => ("end", None),
        RelayMark::SyncFailed(message) => ("sync_failed", message),
    };
    conn.execute(
        "INSERT INTO relay_events (conversation_id, kind, detail, created_at, message_id) VALUES (?1, ?2, ?3, unixepoch(), ?4)",
        params![conversation_id, kind, detail, message],
    )
    .map(drop)
}

/// 내 활동 한 줄의 종류(#1q) — 상대 쪽 글(받은 요청 · 받은 답) · 내 쪽 실패 · 내 요청 · 상대 반영 실패(#1f).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivityKind {
    Message,
    Failed,
    MyRequest,
    SyncFailed,
}

impl ActivityKind {
    pub fn as_str(self) -> &'static str {
        match self {
            ActivityKind::Message => "message",
            ActivityKind::Failed => "failed",
            ActivityKind::MyRequest => "my_request",
            ActivityKind::SyncFailed => "sync_failed",
        }
    }
}

/// 내 활동 거르기(#1q) — 전체 · 받은 요청 · 내 요청 · 받은 답 · 실패.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivityFilter {
    All,
    Request,
    MyRequest,
    Answer,
    Failed,
}

/// 내 활동 한 줄.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Activity {
    pub kind: ActivityKind,
    /// 남은 시각(유닉스 초) — 글이면 저장 시각, 내 요청이면 마지막으로 상태가 바뀐 시각.
    pub at: i64,
    pub conversation_id: i64,
    pub peer_id: i64,
    /// 글(받은 요청 · 받은 답 · 내 요청 · 실패한 내 글) — 상대 반영 실패 기록이면 `None`.
    pub message: Option<Message>,
    /// 상대 반영 실패 기록의 사유 · 미리보기.
    pub detail: Option<String>,
    /// 그 상대의 내 활동을 본 마지막 글 · 기록(id) 뒤의 것(지운 상대의 것은 본 것으로 — 본 표시를 둘 곳이 없다).
    pub unseen: bool,
}

/// 받은 요청 — 상대 사람의 요청(공유 사본 글은 빼고 — #1o). `messages m` · `conversations c` 문맥의 SQL 식.
const RECV_REQUEST: &str =
    "(m.mine = 0 AND m.author = 'human' AND coalesce(m.request, 0) = 1 AND c.dir <> 'shared')";
/// 받은 답 — 내 사람 글에 단 상대 에이전트 답(자동 왕복에서 내 에이전트 글에 단 답은 아니다).
const RECV_ANSWER: &str = "(m.mine = 0 AND m.author = 'agent' AND c.dir <> 'shared'
   AND EXISTS (SELECT 1 FROM messages q WHERE q.id = m.reply_to AND q.mine = 1 AND q.author = 'human'))";
/// 실패한 내 글.
const MINE_FAILED: &str = "(m.mine = 1 AND m.state = 'error' AND m.author <> 'context')";
/// 내 요청 — 실패한 것은 '실패' 로만.
const MY_REQUEST: &str =
    "(m.mine = 1 AND m.author = 'human' AND coalesce(m.request, 0) = 1 AND m.state <> 'error')";

/// 내 활동에 드는 글 중 글 id 로 안 본 것을 가르는 것 — 받은 요청 · 받은 답 · 실패한 내 글(#1q — 상대 사람 일반 메시지는 빠진다).
fn activity_messages() -> String {
    format!("({RECV_REQUEST} OR {RECV_ANSWER} OR {MINE_FAILED})")
}

/// 상대가 내 요청의 상태를 바꿨다(#1q) — `relay_events` 의 `req_state` 행(글 id 는 그대로라, 내 활동을 본 뒤 바뀐 상태를 안 본 것으로 센다).
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

/// 내 활동 — 최신 것부터 `limit` 건. `peer` 로 그 상대 대화만, `q` 는 글 본문 · 상대 반영 실패 사유의 부분 일치.
pub fn activity(
    conn: &Connection,
    peer: Option<i64>,
    filter: ActivityFilter,
    q: Option<&str>,
    limit: i64,
) -> rusqlite::Result<Vec<Activity>> {
    let like = q.map(like_pattern);
    let mut out = Vec::new();
    let cols = prefixed(MSG_COLS, "m");
    let msg_where = match filter {
        ActivityFilter::All => Some(activity_messages()),
        ActivityFilter::Request => Some(RECV_REQUEST.to_string()),
        ActivityFilter::Answer => Some(RECV_ANSWER.to_string()),
        ActivityFilter::Failed => Some(MINE_FAILED.to_string()),
        ActivityFilter::MyRequest => None,
    };
    if let Some(w) = msg_where {
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
    // 내 요청 — 요청 하나 = 한 줄, 시각 = 마지막으로 상태가 바뀐 때, 안 본 것 = 본 뒤 상태가 바뀐 기록이 있을 때.
    if matches!(filter, ActivityFilter::All | ActivityFilter::MyRequest) {
        let mut stmt = conn.prepare(&format!(
            "SELECT {cols}, c.peer_id, p.id IS NOT NULL AND EXISTS (SELECT 1 FROM relay_events f WHERE f.message_id = m.id
               AND f.kind = 'req_state' AND f.id > coalesce(p.activity_seen_event, 0)), coalesce(m.req_at, m.created_at) AS at
             FROM messages m JOIN conversations c ON c.id = m.conversation_id LEFT JOIN peers p ON p.id = c.peer_id
             WHERE {MY_REQUEST} AND (?1 IS NULL OR c.peer_id = ?1) AND (?2 IS NULL OR m.text LIKE ?2 ESCAPE '\\')
             ORDER BY at DESC, m.id DESC LIMIT ?3"
        ))?;
        let rows = stmt.query_map(params![peer, like, limit], |r| {
            let m = msg_row(r)?;
            Ok(Activity {
                kind: ActivityKind::MyRequest,
                at: r.get(MSG_N + 2)?,
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
    // 상대 반영 실패 기록(실패 거르기에도).
    if matches!(filter, ActivityFilter::All | ActivityFilter::Failed) {
        let mut stmt = conn.prepare(
            "SELECT e.created_at, e.conversation_id, c.peer_id, e.detail,
               p.id IS NOT NULL AND e.id > coalesce(p.activity_seen_event, 0)
             FROM relay_events e JOIN conversations c ON c.id = e.conversation_id LEFT JOIN peers p ON p.id = c.peer_id
             WHERE e.kind = 'sync_failed' AND (?1 IS NULL OR c.peer_id = ?1) AND (?2 IS NULL OR e.detail LIKE ?2 ESCAPE '\\')
             ORDER BY e.created_at DESC, e.id DESC LIMIT ?3",
        )?;
        let rows = stmt.query_map(params![peer, like, limit], |r| {
            Ok(Activity {
                kind: ActivityKind::SyncFailed,
                at: r.get(0)?,
                conversation_id: r.get(1)?,
                peer_id: r.get(2)?,
                message: None,
                detail: Some(r.get(3)?),
                unseen: r.get(4)?,
            })
        })?;
        for a in rows {
            out.push(a?);
        }
    }
    // 같은 초면 글이 먼저(안정 정렬) — 목록 각각은 이미 최신순.
    out.sort_by_key(|a| std::cmp::Reverse(a.at));
    out.truncate(usize::try_from(limit).unwrap_or(0));
    Ok(out)
}

/// 내 활동 안 본 수 — `peer` 가 있으면 그 상대만(지운 상대의 것은 세지 않는다). 본 뒤 온 받은 요청 · 받은 답 · 실패 글 + 본 뒤 실패로 바뀐 내 글
/// (이미 본 id) + 본 뒤 상대 반영 실패 기록 + 본 뒤 상태가 바뀐 내 요청(요청마다 하나). 가장 앞선 본 표시 뒤의 id 만 훑는다(5초 새로 고침).
pub fn activity_unseen(conn: &Connection, peer: Option<i64>) -> rusqlite::Result<i64> {
    let messages = activity_messages();
    conn.query_row(
        &format!(
            "SELECT
               (SELECT count(*) FROM messages m JOIN conversations c ON c.id = m.conversation_id
                  LEFT JOIN peers p ON p.id = c.peer_id
                WHERE m.id > coalesce((SELECT min(coalesce(activity_seen_msg, 0)) FROM peers WHERE ?1 IS NULL OR id = ?1), 0)
                  AND {messages} AND (?1 IS NULL OR c.peer_id = ?1)
                  AND p.id IS NOT NULL AND m.id > coalesce(p.activity_seen_msg, 0))
             + (SELECT count(DISTINCT m.id) FROM relay_events f JOIN messages m ON m.id = f.message_id
                  JOIN conversations c ON c.id = m.conversation_id LEFT JOIN peers p ON p.id = c.peer_id
                WHERE f.id > coalesce((SELECT min(coalesce(activity_seen_event, 0)) FROM peers WHERE ?1 IS NULL OR id = ?1), 0)
                  AND f.kind = 'failed' AND m.state = 'error' AND (?1 IS NULL OR c.peer_id = ?1)
                  AND p.id IS NOT NULL AND f.id > coalesce(p.activity_seen_event, 0) AND m.id <= coalesce(p.activity_seen_msg, 0))
             + (SELECT count(*) FROM relay_events e JOIN conversations c ON c.id = e.conversation_id
                  LEFT JOIN peers p ON p.id = c.peer_id
                WHERE e.id > coalesce((SELECT min(coalesce(activity_seen_event, 0)) FROM peers WHERE ?1 IS NULL OR id = ?1), 0)
                  AND e.kind = 'sync_failed' AND (?1 IS NULL OR c.peer_id = ?1)
                  AND p.id IS NOT NULL AND e.id > coalesce(p.activity_seen_event, 0))
             + (SELECT count(DISTINCT m.id) FROM relay_events f JOIN messages m ON m.id = f.message_id
                  JOIN conversations c ON c.id = m.conversation_id LEFT JOIN peers p ON p.id = c.peer_id
                WHERE f.id > coalesce((SELECT min(coalesce(activity_seen_event, 0)) FROM peers WHERE ?1 IS NULL OR id = ?1), 0)
                  AND f.kind = 'req_state' AND {MY_REQUEST} AND (?1 IS NULL OR c.peer_id = ?1)
                  AND p.id IS NOT NULL AND f.id > coalesce(p.activity_seen_event, 0))"
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
    let messages = messages_across(conn, None, Scope::All, None, None, Some(q), limit)?;
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

/// 사람 글의 종류 — 참 = 요청(건의, 상태 '대기'로 시작), 거짓 = 질의(기본). 글과 같은 트랜잭션에서 부른다.
pub fn set_request(conn: &Connection, id: i64, request: bool) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE messages SET request = ?2, req_state = CASE WHEN ?2 THEN coalesce(req_state, 'wait') END WHERE id = ?1",
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

/// 요청 상태 · 메모를 바꾸고 시각을 남긴다 — 옮김 검사는 부르는 쪽([`ReqState::can_become`]).
pub fn set_req_state(
    conn: &Connection,
    id: i64,
    state: ReqState,
    note: Option<&str>,
) -> rusqlite::Result<()> {
    conn.execute(
        // 상태가 바뀌면 archive 손 표시를 지운다(#1y — 완료가 아니면 목록에, 다시 완료되면 7일 셈을 새로).
        "UPDATE messages SET req_archived = CASE WHEN req_state IS ?2 THEN req_archived END,
           req_state = ?2, req_note = ?3, req_at = unixepoch() WHERE id = ?1",
        params![id, state.as_str(), note],
    )
    .map(drop)
}

/// 요청 archive 손 표시(#1y — 이 PC 에만) — 사람 요청 글(공유 사본 아님)만, archive(`on`)는 완료일 때만. 바꿨으면 참.
pub fn set_req_archived(conn: &Connection, id: i64, on: bool) -> rusqlite::Result<bool> {
    conn.execute(
        "UPDATE messages SET req_archived = ?2 WHERE id = ?1 AND author = 'human' AND coalesce(request, 0) = 1
           AND (NOT ?2 OR req_state = 'done')
           AND (SELECT dir FROM conversations c WHERE c.id = messages.conversation_id) <> 'shared'",
        params![id, on],
    )
    .map(|n| n > 0)
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

/// 답의 창 사용량(#1m) — `None` 이면 그대로(모름).
pub fn set_ctx_tokens(conn: &Connection, id: i64, tokens: Option<i64>) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE messages SET ctx_tokens = ?2 WHERE id = ?1",
        params![id, tokens],
    )
    .map(drop)
}

/// 컨텍스트 구분선 글(#1m) — `exec` = 이 PC 가 세션을 처리한다(대화 작업자 줄에 든다), `mine` = 이 PC 에서 눌렀다(자동 압축 포함).
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

/// 대화의 Claude 세션 전부(#1m) — (에이전트, 세션). 대화 에이전트는 `None`, 멘션한 에이전트는 이름.
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

/// 대화 세션을 모두 비운다(#1m — 비우기 · 압축 뒤). 남은 요약은 `keep_seeds` 가 거짓이면 함께 지운다.
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

/// 압축 요약(#1m) — 다음 새 세션의 첫 실행 앞에 붙인다. `agent` 빈 글자 = 대화 에이전트.
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

/// 대화의 마지막 `n` 개(오래된 것부터, 개요 미리보기 #1z-4) — [`messages`] 와 같은 순서의 끝을 SQL 에서 자른다(대화 전체를 읽지 않는다).
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
/// `author` 로 그 종류만, `q` 는 본문 부분 일치. 거른 뒤 최신 것부터 `limit` 건. 질의 · 요청 거르기(`ask`)에는 공유 사본 글이 들지 않는다(#1o).
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

/// 요청 상태별 건수(#1q 적대적 평가 R1) — 한 쪽(`mine`) 사람 요청 전부(공유 사본 제외), 목록 상한과 무관.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ReqCounts {
    pub wait: i64,
    pub accepted: i64,
    pub rejected: i64,
    /// archive 되지 않은 완료(#1y).
    pub done: i64,
    /// archive 된 완료(#1y) — [`ReqCounts::total`] 에 들지 않는다.
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

/// 받은 요청 중 대기(상태 없음 포함)인 것 — 최신부터 `limit` 건(상대 id · 본문)과 대기 전체 수(#1x — 받은 요청 알림 본문).
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

/// 주고받은 메시지 한 쪽(#1q) — 양쪽 글(context 글 제외)을 `(created_at, id)` 최신부터 `limit` 건, `before` = 그 `(created_at, id)` 보다 앞의 것만
/// (복합 커서 — 공유 사본은 상대 시각을 남겨 id 순과 다를 수 있다, 적대적 평가 R2).
///
/// `since` = 그 시각 이후(이상)에 쓴 글만(#1y — 보기 기간).
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

/// 주고받은 메시지 상대별 요약(#1q) — (상대 id, 메시지 수, 마지막 메시지 id — [`messages_page`] 와 같은 `(created_at, id)` 순), 모집단도 같다(`since` 포함).
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

/// 실행 대기열 = DB — 대화의 `queued` · `running` 실행 요청(사람 글 · 에이전트발), 오래된 것 먼저. 늦게 `queued` 가 된 낮은 id 도 빠지지 않게
/// 커서 없이 매번 처음부터 본다(작업이 한 번 꺼낸 것은 작업 쪽에서 건너뛴다).
pub fn pending(conn: &Connection, conversation_id: i64) -> rusqlite::Result<Vec<Message>> {
    conn.prepare(&format!(
        "SELECT {MSG_COLS} FROM messages WHERE conversation_id = ?1
           AND exec = 1 AND state IN ('queued', 'running') ORDER BY coalesce(ctx_op = 'auto_compact', 0) DESC, id"
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
