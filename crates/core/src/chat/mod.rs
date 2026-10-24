//! 대화 · 메시지 이력 — 양쪽 PC 가 같은 대화 UUID 로 각자 남긴다.

mod conversation;
mod message;
mod request;
mod activity;
mod search;
mod relay;
mod session;

pub use conversation::*;
pub use message::*;
pub use request::*;
pub use activity::*;
pub use search::*;
pub use relay::*;
pub use session::*;

use rusqlite::{Connection, OptionalExtension, Row, params};

/// `Out` = 내가 원격 에이전트에게 · `In` = 내 에이전트가 원격에게서 받음 · `Shared` = 상대(대화를 가진 PC)가 공유한 읽기 전용 사본(상대 id =
/// 그 PC).
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
    /// 컨텍스트 구분선 — 압축 · 비우기. 본문 = 요약(압축이 끝났으면) · 종류는 `Message.ctx_op`.
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

/// 요청 글의 상태 — 받은 쪽 사람이 지금 상태 말고 네 상태 어디로든 바꾸고(되돌리기 포함 — `can_become`) 쓴 쪽은 받은 대로 남긴다.
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

    /// 받은 쪽이 이 상태에서 `next` 로 옮길 수 있나 — 지금 상태 말고 네 상태 어디로든(되돌리기 포함).
    pub fn can_become(self, next: Self) -> bool {
        self != next
    }
}

/// 사람 글 종류 거르기 — 질의 · 요청(요청은 상태로 더 거른다, `None` = 모든 상태).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AskFilter {
    Query,
    /// 요청 — 상태 거르기, 둘째 = 참이면 archive 된 것만 · 거짓이면 archive 제외.
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
    /// 자동 왕복(연 쪽) — 없으면 자동 왕복 대화가 아니다.
    pub relay: Option<Relay>,
    /// 받은 쪽: 에이전트발 실행 요청을 거절한다(받은 쪽 사람이 멈춤).
    pub relay_blocked: bool,
    /// 이 PC 가 나간 시각 — 그 뒤 이 대화로 온 요청은 거절(`left`), 읽기 전용.
    pub left_at: Option<i64>,
    /// 상대가 나간 시각(상대의 `Leave`) — 읽기 전용.
    pub peer_left_at: Option<i64>,
}

impl Conversation {
    /// 나간 대화 — 내가 나갔거나 상대가 나갔다. 읽기 전용(글쓰기 · 실행 · 보내기 없음).
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
    /// 이 PC 에서 쓴 글(사람) · 이 PC 에이전트의 답이면 참(양쪽 발화).
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
    /// 요청 글의 상태 · 메모 · 바꾼 시각 — 요청이 아니면 `None`.
    pub req_state: Option<ReqState>,
    pub req_note: Option<String>,
    pub req_at: Option<i64>,
    /// 멘션 — 사람 실행 요청이 부른 에이전트 · 그 답을 실행한 에이전트(`None` = 대화 에이전트).
    pub agent: Option<String>,
    /// 보낸 쪽: 상대가 거절한 코드(`focus` 등).
    pub refused: Option<String>,
    /// 편집한 시각(내 사람 글 · 상대가 편집해 보낸 글) — 편집하지 않았으면 `None`.
    pub edited_at: Option<i64>,
    /// 답을 낸 세션의 창 사용량(토큰) — 모르면 `None`.
    pub ctx_tokens: Option<i64>,
    /// 컨텍스트 구분선의 종류 — `compact` · `auto_compact` · `clear`.
    pub ctx_op: Option<String>,
    /// 공유 사본의 글 — 대화를 가진 PC 쪽 글이면 참(그 밖 대화는 거짓).
    pub owner: bool,
    /// 요청 archive 손 표시 — `None` = 자동(완료 [`ARCHIVE_DAYS`] 일 뒤) · 참 = 손으로 archive · 거짓 = 손으로 해제.
    pub req_archived: Option<bool>,
}

/// 완료 요청이 archive 되기까지 날 수.
pub const ARCHIVE_DAYS: i64 = 7;

impl Message {
    /// archive 된 요청인가 — 완료이고 손으로 archive 했거나 · 손 표시 없이 완료 뒤 [`ARCHIVE_DAYS`] 일 지남. [`archived_sql`] 과 같은 규칙.
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
    /// 손으로 놓은 자리(이 PC 에만, 작을수록 위). 목록 순서 자체는 최근 활동순 그대로.
    pub pos: i64,
    /// 마지막 글(대화방 목록 미리보기): 본문 앞 [`PREVIEW_CHARS`] 글자 · 쓴 쪽(`human` · `agent`) · 이 PC 쪽 글인가 · 멘션 에이전트. 글이 없으면 `None`.
    pub last: Option<LastMessage>,
}

/// 대화방 목록 미리보기의 마지막 글 — 대화 화면과 같은 순서(`(lc, uuid)`)의 마지막(로컬 `id` 순이 아니다), 컨텍스트 구분선은 뺀다.
/// 요약의 마지막 상태 · 시각은 구분선까지 센 마지막 글에서 읽는다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LastMessage {
    pub text: String,
    pub author: Author,
    pub mine: bool,
    pub agent: Option<String>,
    /// 글 출처(`cli` = AI 에이전트(CLI) 가 쓴 글 — 말한 이 표시).
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

const MSG_COLS: &str = "id, uuid, conversation_id, author, text, state, created_at, reply_to, turns, duration_ms, mine, exec, lc, via, saved_at, model, effort, request, req_state, req_note, req_at, agent, refused, edited_at, ctx_tokens, ctx_op, owner, req_archived";

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
