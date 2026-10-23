//! 에이전트 프로필 · 대화 API(S1 · S3 · D21) — `server::router_with` 의 guard 뒤에 붙는다. 원격 답 본문은 웹UI 가 textContent 로만 그린다.
use crate::peers_api::PeerView;
use an_agent::chat::{Chat, Compose, TxView, default_relay_cap};
use an_core::agents::{self, Agent};
use an_core::chat::{
    self, ActivityFilter, AskFilter, Author, Conversation, Dir, Message, ReqState, Summary,
};
use an_core::share;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::io;
use std::sync::Arc;

/// 에이전트 이름 최대 글자 수.
pub const MAX_AGENT_NAME: usize = 64;

/// 도구는 받지 않는다(#1d — 모든 실행이 탐색 도구 전부, 모르는 필드는 422).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NewAgent {
    name: String,
    folder: String,
    #[serde(default)]
    prompt: String,
    /// 상위 한 단계 읽기 허용(#05-2) — 없으면 끔.
    #[serde(default)]
    parent_read: bool,
    /// 셸 명령 실행 허용(#05-3) — 없으면 끔.
    #[serde(default)]
    shell: bool,
}

#[derive(Deserialize)]
struct NewConversation {
    peer_id: i64,
    agent: String,
    /// 첫 글 — 비면(웹UI 세션 생성, #1x) 글 없이 대화만 연다(`via` · `request` 와 함께는 400).
    #[serde(default)]
    text: String,
    /// `agent` 만(생략 가능) — [`NewMessage::to`] 와 같다(첫 글도 대화 에이전트에게 실행 요청, `human` 은 400).
    #[serde(default)]
    to: Option<String>,
    /// 자동 왕복(D24) — 이 PC 의 내 에이전트. 있으면 두 에이전트 자동 왕복 대화.
    #[serde(default)]
    relay_agent: Option<String>,
    /// 자동 왕복 상한(왕복, 1 ~ 50) — 없으면 10.
    #[serde(default)]
    relay_cap: Option<i64>,
    /// 글의 출처 — `cli` 만(사용자 AI 에이전트가 CLI 로 쓴 글). 없으면 웹UI.
    #[serde(default)]
    via: Option<String>,
    /// 이 PC 에서 붙이는 대화 제목 — 비거나 없으면 첫 메시지 앞부분. 상대에 보내지 않는다. 대화와 함께 저장한다(전송이 실패해도 남는 대화에 붙는다).
    #[serde(default)]
    title: Option<String>,
    /// 첫 글의 종류 — 참 = 요청(건의), 없거나 거짓 = 질의.
    #[serde(default)]
    request: bool,
    /// CLI 트랜잭션 키(#1c, `via: cli` 일 때만) — 같은 키가 이미 있으면 보내지 않고 그 트랜잭션을 돌려준다.
    #[serde(default)]
    tx_key: Option<String>,
    /// 새 대화를 넣을 레일 그룹(#1i — 이 PC 에만). 없는 그룹이면 보내지 않고 404.
    #[serde(default)]
    group: Option<i64>,
}

/// `PATCH /api/conversations/{id}` — 이 PC 에서 붙이는 표시만, 준 항목만: 제목(빈 문자열 = 지움 → 첫 사람 글 앞부분) · ★.
/// 상대에 알리지 않는다. 둘 다 없으면 400.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ConversationPatch {
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    starred: Option<bool>,
}

/// `PATCH /api/messages/{id}` — 준 항목만: 저장된 항목(이 PC 에만) · 받은 요청의 상태 + 메모(#1d, 쓴 쪽에 간다) · 본문 편집(#1f). 모두 없으면
/// 400, 저장 + 상태면 한 트랜잭션(상태를 바꿀 수 없으면 아무것도 쓰지 않는다). 편집은 다른 항목과 함께 쓰지 않는다. 응답도 준 항목만.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MessagePatch {
    #[serde(default)]
    saved: Option<bool>,
    /// `accepted` | `rejected` | `done`(대기 → 수용 · 반려, 수용 → 완료).
    #[serde(default)]
    req_state: Option<String>,
    /// 한 줄 메모 — `req_state` 와 함께만.
    #[serde(default)]
    note: Option<String>,
    /// 새 본문(#1f — 끝난 내 사람 글만, 실행 없음).
    #[serde(default)]
    text: Option<String>,
    /// 편집을 상대 창에도 — `text` 와 함께만(없으면 이 PC 에만).
    #[serde(default)]
    mirror: Option<bool>,
}

/// `POST /api/messages/delete` — 끝난 내 사람 글 여러 개를 흔적 없이 지운다(#1f, 한 대화의 글만 · 하나라도 안 되면 아무것도 안 지움).
/// `mirror` = 상대 창에도.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MessageDelete {
    ids: Vec<i64>,
    #[serde(default)]
    mirror: bool,
}

/// 한 번에 지울 수 있는 글 수.
pub const DELETE_MAX: usize = 200;

/// `GET /api/search` — 통합 검색.
#[derive(Deserialize)]
struct SearchQuery {
    q: String,
    /// 메시지 묶음 건수(그 밖 묶음은 [`chat::SEARCH_OTHERS`]).
    #[serde(default)]
    limit: Option<i64>,
    /// 닫힌 대화(나감 · 상대가 나감 · 끝난 공유 사본)도 찾는가(#1z-8 — 웹UI "나간 대화 포함", 기본 뺌).
    #[serde(default)]
    closed: Option<bool>,
}

/// `GET /api/conversations/{id}/messages` — `limit` 이면 마지막 N 개만(개요 미리보기 #1z-4, 1 ~ [`MESSAGES_MAX`]).
#[derive(Deserialize)]
struct TailQuery {
    #[serde(default)]
    limit: Option<i64>,
}

/// 검색어 길이(글자) · 메시지 묶음 기본 · 최대 건수.
pub const SEARCH_MAX_CHARS: usize = 100;
pub const SEARCH_DEFAULT: i64 = 20;
pub const SEARCH_MAX: i64 = 100;

/// `POST /api/conversations/{id}/read` — 화면이 받은 마지막 글 id 까지 읽음.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ReadUpto {
    upto: i64,
}

/// `GET /api/activity` — 내 활동.
#[derive(Deserialize)]
struct ActivityQuery {
    #[serde(default)]
    peer: Option<i64>,
    /// `all`(기본) | `request`(받은 요청) | `mine`(내 요청) | `answer`(받은 답) | `failed`(#1q).
    #[serde(default)]
    kind: Option<String>,
    #[serde(default)]
    q: Option<String>,
    /// 0 = 목록 없이 안 본 수만(5초 새로 고침).
    #[serde(default)]
    limit: Option<i64>,
}

/// `POST /api/activity/seen` — 내 활동을 화면이 받은 범위(`GET` 의 `upto` — 마지막 글 · 자동 왕복 기록 id)까지 봤다. `peer` 가 없으면 전부.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ActivitySeen {
    #[serde(default)]
    peer: Option<i64>,
    message: i64,
    event: i64,
}

/// `GET /api/saved` — 저장된 항목.
#[derive(Deserialize)]
struct SavedQuery {
    #[serde(default)]
    peer: Option<i64>,
    /// 레일 그룹(#1i) — `peer` 와 함께 주면 400.
    #[serde(default)]
    group: Option<i64>,
    #[serde(default)]
    conversation: Option<i64>,
    #[serde(default)]
    limit: Option<i64>,
}

/// `PATCH /api/agents/{id}` — 준 항목만 바꾼다. 이름은 바꾸지 않는다(대화 · 상대 PC 가 이름으로 부른다 — 주면 400).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AgentPatch {
    #[serde(default)]
    name: Option<Value>,
    #[serde(default)]
    folder: Option<String>,
    #[serde(default)]
    prompt: Option<String>,
    /// 빈 문자열 = 그룹 없음.
    #[serde(default)]
    group: Option<String>,
    #[serde(default)]
    parent_read: Option<bool>,
    #[serde(default)]
    shell: Option<bool>,
    /// 아이콘 색(#1z-7) — `agents::ICON_COLORS` 밖이면 400.
    #[serde(default)]
    color: Option<String>,
}

/// `GET /api/messages` — 대화를 가로지른 글 목록.
#[derive(Deserialize)]
struct MessageQuery {
    /// `theirs`(상대 쪽이 쓴 글) | `mine`(내가 쓴 글) | `both`(양쪽 — 주고받은 메시지 페이지, #1q).
    side: String,
    /// 커서(#1q, `side=both` 만) — 받은 마지막 글의 id · 시각(`before_at`), 둘 다 주면 그 `(created_at, id)` 보다 앞의 것만.
    #[serde(default)]
    before: Option<i64>,
    #[serde(default)]
    before_at: Option<i64>,
    #[serde(default)]
    peer: Option<i64>,
    /// 레일 그룹(#1i) — 그 그룹 대화만. `peer` 와 함께 주면 400.
    #[serde(default)]
    group: Option<i64>,
    #[serde(default)]
    q: Option<String>,
    /// `human`(사람 글) | `agent`(에이전트 답) — 건수 제한 전에 거른다. 없으면 둘 다.
    #[serde(default)]
    kind: Option<String>,
    /// `query`(질의) | `request`(요청) — 사람 글의 종류, 건수 제한 전에 거른다. 없으면 모두.
    #[serde(default)]
    ask: Option<String>,
    /// 요청 상태(#1d) `wait` | `accepted` | `rejected` | `done` — `ask=request` 와 함께만.
    #[serde(default)]
    req: Option<String>,
    /// 요청 archive(#1y) — `true` = archive 된 것만, 없거나 `false` = archive 제외. `ask=request` 와 함께만.
    #[serde(default)]
    archived: Option<bool>,
    #[serde(default)]
    limit: Option<i64>,
}

/// `POST /api/messages/{id}/archive` — 요청 archive · 해제(#1y, 이 PC 에만).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ArchivePatch {
    archived: bool,
}

/// 글 목록 기본 · 최대 건수.
pub const MESSAGES_DEFAULT: i64 = 200;
pub const MESSAGES_MAX: i64 = 500;

#[derive(Deserialize)]
struct RelaySwitch {
    on: bool,
}

/// 컨텍스트 압축 · 비우기(#1m) — `op` = `compact` · `clear`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ContextOp {
    op: String,
}

#[derive(Deserialize)]
struct NewMessage {
    text: String,
    /// `agent` 만(생략 가능) — 모든 새 글은 대화 에이전트에게 실행 요청이다(D23 개정, `human` 은 400).
    #[serde(default)]
    to: Option<String>,
    /// 글의 출처 — [`NewConversation::via`] 와 같다.
    #[serde(default)]
    via: Option<String>,
    /// 글의 종류 — [`NewConversation::request`] 와 같다.
    #[serde(default)]
    request: bool,
    /// 멘션 — 보낸 대화(자동 왕복 아님)에서 부를 상대 PC 에이전트 이름. 대화 에이전트면 멘션이 아니고, 받은 대화 · 자동 왕복 · 빈 이름은 400.
    #[serde(default)]
    agent: Option<String>,
    /// [`NewConversation::tx_key`] 와 같다.
    #[serde(default)]
    tx_key: Option<String>,
}

pub fn routes(chat: Arc<Chat>) -> Router {
    Router::new()
        .route("/api/agents", get(list_agents).post(add_agent))
        .route("/api/agents/{id}", delete(remove_agent).patch(update_agent))
        .route("/api/messages", get(list_across))
        .route("/api/messages/{id}", axum::routing::patch(save_message))
        .route("/api/messages/{id}/archive", axum::routing::post(archive_message))
        .route("/api/messages/delete", axum::routing::post(delete_messages))
        .route("/api/messages/peers", get(message_peers))
        .route("/api/messages/req_counts", get(req_counts))
        .route("/api/search", get(search))
        .route("/api/activity", get(activity))
        .route("/api/activity/seen", axum::routing::post(activity_seen))
        .route("/api/saved", get(saved_list))
        .route("/api/peers/{id}/agents", get(remote_agents))
        .route(
            "/api/conversations",
            get(list_conversations).post(start_conversation),
        )
        .route(
            "/api/conversations/{id}",
            axum::routing::patch(patch_conversation).delete(delete_conversation),
        )
        .route(
            "/api/conversations/{id}/clear",
            axum::routing::post(clear_conversation),
        )
        .route(
            "/api/conversations/{id}/read",
            axum::routing::post(read_conversation),
        )
        .route(
            "/api/conversations/{id}/messages",
            get(list_messages).post(continue_conversation),
        )
        .route("/api/runs", get(list_runs).put(set_runs))
        .route("/api/tx/{id}", get(get_tx))
        .route("/api/tx/{id}/resume", axum::routing::post(resume_tx))
        .route(
            "/api/conversations/{id}/relay",
            axum::routing::post(switch_relay),
        )
        .route(
            "/api/conversations/{id}/context",
            axum::routing::post(context_op),
        )
        .route(
            "/api/conversations/{id}/observers",
            axum::routing::post(add_observer),
        )
        .route(
            "/api/conversations/{id}/observers/{peer}",
            delete(remove_observer),
        )
        .route(
            "/api/conversations/{id}/leave",
            axum::routing::post(leave_conversation),
        )
        .with_state(chat.clone())
        .merge(crate::groups_api::routes(chat))
}

fn fail(e: io::Error) -> Response {
    let code = match e.kind() {
        io::ErrorKind::NotFound => StatusCode::NOT_FOUND,
        io::ErrorKind::InvalidInput => StatusCode::BAD_REQUEST,
        io::ErrorKind::AlreadyExists | io::ErrorKind::Unsupported | io::ErrorKind::ResourceBusy => {
            StatusCode::CONFLICT
        }
        io::ErrorKind::PermissionDenied => StatusCode::FORBIDDEN,
        _ => StatusCode::BAD_GATEWAY,
    };
    (code, Json(json!({ "error": e.to_string() }))).into_response()
}

/// 목록 범위(#1i) — 상대 · 그룹 중 하나, 둘 다면 `None`.
fn scope_of(peer: Option<i64>, group: Option<i64>) -> Option<chat::Scope> {
    match (peer, group) {
        (None, None) => Some(chat::Scope::All),
        (Some(p), None) => Some(chat::Scope::Peer(p)),
        (None, Some(g)) => Some(chat::Scope::Group(g)),
        (Some(_), Some(_)) => None,
    }
}

fn db_fail(e: an_core::db::Error) -> Response {
    fail(io::Error::other(e))
}

fn bad(msg: &str) -> Response {
    fail(io::Error::new(io::ErrorKind::InvalidInput, msg.to_string()))
}

/// 요청의 `via` — 없거나 `cli`. 그 밖 값은 400.
fn via_of(via: &Option<String>) -> Result<Option<&'static str>, &'static str> {
    match via.as_deref() {
        None => Ok(None),
        Some(chat::VIA_CLI) => Ok(Some(chat::VIA_CLI)),
        Some(_) => Err("via 는 cli 만"),
    }
}

/// 요청의 `to` — 없거나 `agent`. 새 대화 · 이어 쓰기 둘 다(D23 개정) — 그 밖 값은 400.
fn to_of(to: &Option<String>) -> Result<(), &'static str> {
    match to.as_deref() {
        None | Some("agent") => Ok(()),
        Some(_) => Err("to 는 agent 만 — 모든 글은 대화 에이전트에게 간다"),
    }
}

/// `uid` = 등록 세대(같은 이름 · 같은 id 로 다시 등록해도 다름 — 웹UI 저장 전 값의 주인, #1z-8 R2).
/// `parent_dir` = 상위 읽기로 실제 여는 폴더(끔 · 막힌 곳이면 `null` — 웹UI 가 생략 사유를 보인다).
fn agent_json(a: Agent) -> Value {
    let up = a
        .parent_read
        .then(|| an_core::paths::parent_read_dir(std::path::Path::new(&a.folder)))
        .flatten()
        .map(|p| p.display().to_string());
    json!({ "id": a.id, "name": a.name, "folder": a.folder, "prompt": a.prompt, "group": a.group, "parent_read": a.parent_read, "parent_dir": up, "shell": a.shell, "color": a.color, "uid": a.uid })
}

/// 폴더는 있는 절대 경로만 — 상대 경로는 앱의 현재 폴더 기준이 되어 의도와 달라진다. 데이터 루트를 품거나 네트워크 경로면 거부
/// ([`an_core::paths::agent_folder_problem`], 1a A7).
fn checked_folder(folder: &str) -> Result<String, String> {
    let folder = std::path::Path::new(folder.trim());
    if !folder.is_absolute() || !folder.is_dir() {
        return Err("폴더는 이 PC 에 있는 절대 경로여야 한다".into());
    }
    if let Some(why) = an_core::paths::agent_folder_problem(folder) {
        return Err(why.into());
    }
    Ok(folder.display().to_string())
}

fn conversation_json(c: &Conversation) -> Value {
    let r = c.relay.as_ref();
    json!({
        "id": c.id, "uuid": c.uuid, "peer_id": c.peer_id, "agent": c.agent, "dir": c.dir.as_str(),
        "relay_agent": r.map(|r| &r.agent), "relay_cap": r.map(|r| r.cap), "relay_count": r.map(|r| r.count),
        "relay_on": r.is_some_and(|r| r.on), "relay_blocked": c.relay_blocked,
        "left_at": c.left_at, "peer_left_at": c.peer_left_at,
    })
}

/// 공유 필드(#1o) — 공유 대화(사본) = `share{other, owner_dir, ended}`, 그 밖 = `observers`(내가 넣은 관람자 id) · `watchers`(원 상대가 알려 준 관람자 수).
fn share_json(
    db: &an_core::db::Connection,
    c: &Conversation,
    v: &mut Value,
) -> Result<(), an_core::db::Error> {
    if c.dir == Dir::Shared {
        v["share"] = json!(share::copy(db, c.id)?.map(|s| json!({
            "other": s.other, "owner_dir": s.owner_dir, "ended": s.ended.then_some(true),
        })));
    } else {
        v["observers"] = json!(
            share::watchers(db, c.id)?
                .iter()
                .map(|w| w.peer_id)
                .collect::<Vec<_>>()
        );
        v["watchers"] = json!(share::watched(db, c.id)?);
    }
    Ok(())
}

fn message_json(m: &Message) -> Value {
    json!({
        "id": m.id, "author": m.author.as_str(), "text": m.text, "state": m.state.as_str(), "created_at": m.created_at,
        "reply_to": m.reply_to, "turns": m.turns, "duration_ms": m.duration_ms, "mine": m.mine, "exec": m.exec,
        "via": m.via, "model": m.model, "effort": m.effort, "saved": m.saved_at.is_some(),
        "request": m.request, "req_state": m.req_state.map(ReqState::as_str), "req_note": m.req_note, "req_at": m.req_at,
        "agent": m.agent, "refused": m.refused, "edited_at": m.edited_at,
        "ctx_tokens": m.ctx_tokens, "ctx_op": m.ctx_op, "owner": m.owner, "archived": m.archived(chat::unix_now()),
    })
}

/// 글이 속한 대화 — 목록 · 검색 · 내 활동 항목에 붙는다(제목 포함).
fn conversation_ref(s: &Summary) -> Value {
    let c = &s.conversation;
    json!({
        "id": c.id, "peer_id": c.peer_id, "agent": c.agent, "dir": c.dir.as_str(), "title": s.title,
        "left_at": c.left_at, "peer_left_at": c.peer_left_at,
    })
}

/// 대화 id → 요약(제목 · 대화 정보).
fn summary_map(db: &an_core::db::Connection) -> Result<HashMap<i64, Summary>, an_core::db::Error> {
    Ok(chat::summaries(db)?
        .into_iter()
        .map(|s| (s.conversation.id, s))
        .collect())
}

/// 메시지 JSON + `conversation{id, peer_id, agent, dir, title}`.
fn message_in(m: &Message, sums: &HashMap<i64, Summary>) -> Value {
    let mut v = message_json(m);
    v["conversation"] = sums
        .get(&m.conversation_id)
        .map_or(Value::Null, conversation_ref);
    v
}

fn summary_json(s: &chat::Summary) -> Value {
    let mut v = conversation_json(&s.conversation);
    v["title"] = json!(s.title);
    v["last_state"] = json!(s.last_state.map(|st| st.as_str()));
    v["last_at"] = json!(s.last_at);
    v["starred"] = json!(s.starred);
    v["unread"] = json!(s.unread);
    v["pos"] = json!(s.pos);
    // 대화방 목록 미리보기(#1j) — 마지막 글.
    v["last"] = json!(s.last.as_ref().map(|l| json!({
        "text": l.text,
        "author": l.author.as_str(),
        "mine": l.mine,
        "agent": l.agent,
        "via": l.via,
    })));
    v
}

async fn list_agents(State(chat): State<Arc<Chat>>) -> Response {
    match agents::list(&chat.db()) {
        Ok(list) => Json(list.into_iter().map(agent_json).collect::<Vec<_>>()).into_response(),
        Err(e) => db_fail(e),
    }
}

async fn add_agent(State(chat): State<Arc<Chat>>, Json(req): Json<NewAgent>) -> Response {
    let name = req.name.trim();
    if name.is_empty()
        || name.chars().count() > MAX_AGENT_NAME
        || name.chars().any(char::is_control)
    {
        return bad("이름은 1 ~ 64자, 제어 문자 없이");
    }
    let folder = match checked_folder(&req.folder) {
        Ok(f) => f,
        Err(m) => return bad(&m),
    };
    let db = chat.db();
    if matches!(agents::by_name(&db, name), Ok(Some(_))) {
        return fail(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("같은 이름의 에이전트가 있다: {name}"),
        ));
    }
    match agents::add_with(&db, name, &folder, req.prompt.trim(), req.parent_read, req.shell) {
        Ok(a) => (StatusCode::CREATED, Json(agent_json(a))).into_response(),
        Err(e) => db_fail(e),
    }
}

async fn update_agent(
    State(chat): State<Arc<Chat>>,
    Path(id): Path<i64>,
    Json(req): Json<AgentPatch>,
) -> Response {
    if req.name.is_some() {
        return bad("에이전트 이름은 바꾸지 않는다 — 새 이름이면 새로 등록");
    }
    let db = chat.db();
    let now = match agents::by_id(&db, id) {
        Ok(Some(a)) => a,
        Ok(None) => return fail(io::Error::new(io::ErrorKind::NotFound, "없는 에이전트")),
        Err(e) => return db_fail(e),
    };
    let folder = match req.folder.as_deref().map(checked_folder) {
        None => now.folder,
        Some(Ok(f)) => f,
        Some(Err(m)) => return bad(&m),
    };
    let prompt = req.prompt.map_or(now.prompt, |p| p.trim().to_string());
    let group = match req.group.as_deref().map(an_core::run_opts::clean_group) {
        None => now.group,
        Some(Ok(g)) => g,
        Some(Err(m)) => return bad(&m),
    };
    let parent_read = req.parent_read.unwrap_or(now.parent_read);
    let shell = req.shell.unwrap_or(now.shell);
    if let Some(c) = req.color.as_deref()
        && !agents::ICON_COLORS.contains(&c)
    {
        return bad(&format!("color 는 {}", agents::ICON_COLORS.join(" · ")));
    }
    match agents::update(&db, id, &folder, &prompt, group.as_deref(), parent_read, shell) {
        Ok(Some(mut a)) => {
            if let Some(c) = req.color {
                if let Err(e) = agents::set_color(&db, id, &c) {
                    return db_fail(e);
                }
                a.color = c;
            }
            Json(agent_json(a)).into_response()
        }
        Ok(None) => fail(io::Error::new(io::ErrorKind::NotFound, "없는 에이전트")),
        Err(e) => db_fail(e),
    }
}

async fn remove_agent(State(chat): State<Arc<Chat>>, Path(id): Path<i64>) -> Response {
    match agents::remove(&chat.db(), id) {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => fail(io::Error::new(io::ErrorKind::NotFound, "없는 에이전트")),
        Err(e) => db_fail(e),
    }
}

async fn remote_agents(State(chat): State<Arc<Chat>>, Path(id): Path<i64>) -> Response {
    match chat.remote_agents(id).await {
        Ok(names) => Json(json!({ "names": names })).into_response(),
        Err(e) => fail(e),
    }
}

async fn list_conversations(State(chat): State<Arc<Chat>>) -> Response {
    let db = chat.db();
    let read = chat::summaries(&db).and_then(|list| {
        list.iter()
            .map(|s| {
                let mut v = summary_json(s);
                share_json(&db, &s.conversation, &mut v).map(|()| v)
            })
            .collect::<Result<Vec<_>, _>>()
    });
    match read {
        Ok(list) => Json(list).into_response(),
        Err(e) => db_fail(e),
    }
}

async fn list_messages(State(chat): State<Arc<Chat>>, Path(id): Path<i64>, Query(q): Query<TailQuery>) -> Response {
    if q.limit.is_some_and(|n| !(1..=MESSAGES_MAX).contains(&n)) {
        return bad(&format!("limit 은 1 ~ {MESSAGES_MAX}"));
    }
    // DB 잠금은 글을 읽는 동안만 — 아래 보관 상태 조회가 같은 잠금을 다시 잡는다.
    let read = {
        let db = chat.db();
        match chat::by_id(&db, id) {
            Ok(Some(_)) => {}
            Ok(None) => return fail(io::Error::new(io::ErrorKind::NotFound, "없는 대화")),
            Err(e) => return db_fail(e),
        }
        // 마지막 N 개(오래된 것부터)는 SQL 에서 자른다 — 글마다 진행 · 보관 · 편집 가능도 그 글만 읽는다.
        match q.limit {
            Some(n) => chat::messages_tail(&db, id, n),
            None => chat::messages(&db, id),
        }
    };
    match read {
        Ok(ms) => Json(
            ms.iter()
                .map(|m| {
                    let mut v = message_json(m);
                    v["progress"] = chat.progress_of(m).map_or(
                        Value::Null,
                        |p| json!({ "state": p.state, "ahead": p.ahead, "detail": p.detail }),
                    );
                    // 보관(#1c) — 상대에 아직 닿지 않았거나(`pending` · `inflight`) 포기한(`failed`) 내 글.
                    v["outbox"] = chat.outbox_of(m).map_or(
                        Value::Null,
                        |o| json!({ "state": o.state, "tries": o.tries, "error": o.last_error }),
                    );
                    // 편집 모드 체크박스 · 연필(#1f) — 끝난 내 사람 글만.
                    v["editable"] = json!(Chat::tidy_ok(&chat.db(), m).unwrap_or(false));
                    v
                })
                .collect::<Vec<_>>(),
        )
        .into_response(),
        Err(e) => db_fail(e),
    }
}

/// 실행 목록 항목 — 받은 글 · 대화 · 상대 · 에이전트 · 앞부분 40자. 그새 지워진 글은 id 만.
fn run_json(db: &an_core::db::Connection, id: i64) -> Result<Value, an_core::db::Error> {
    let Some(m) = chat::message_by_id(db, id)? else {
        return Ok(json!({ "message_id": id }));
    };
    let conv = chat::by_id(db, m.conversation_id)?;
    let peer = match &conv {
        Some(c) => an_core::peers::by_id(db, c.peer_id)?,
        None => None,
    };
    Ok(json!({
        "message_id": id, "conversation_id": m.conversation_id, "peer_id": conv.as_ref().map(|c| c.peer_id),
        "peer": peer.map(|p| p.name), "agent": m.agent.or_else(|| conv.map(|c| c.agent)),
        "text": m.text.chars().take(40).collect::<String>(),
    }))
}

/// 이 PC 실행 문 — 상한 · 실행 중 · 기다림(닿은 순서).
async fn list_runs(State(chat): State<Arc<Chat>>) -> Response {
    let (cap, running, waiting) = chat.run_queue();
    let db = chat.db();
    let items = |ids: Vec<i64>| {
        ids.into_iter()
            .map(|id| run_json(&db, id))
            .collect::<Result<Vec<_>, _>>()
    };
    match (items(running), items(waiting)) {
        (Ok(r), Ok(w)) => Json(json!({ "cap": cap, "running": r, "waiting": w })).into_response(),
        (Err(e), _) | (_, Err(e)) => db_fail(e),
    }
}

/// 동시 실행 상한(1 ~ 8) — 이 PC 에 저장하고 문에 바로. 정수가 아니거나 범위 밖이면 400.
async fn set_runs(State(chat): State<Arc<Chat>>, Json(req): Json<Value>) -> Response {
    let Some(cap) = req["cap"].as_u64().and_then(|n| usize::try_from(n).ok()) else {
        return bad("cap 은 1 ~ 8 정수");
    };
    match chat.set_max_runs(cap) {
        Ok(()) => list_runs(State(chat)).await,
        Err(e) => fail(e),
    }
}

/// 대화를 가로지른 글 — 최신순. 항목 = 메시지 JSON + `conversation{id, peer_id, agent, dir}`.
async fn list_across(State(chat): State<Arc<Chat>>, Query(q): Query<MessageQuery>) -> Response {
    let mine = match q.side.as_str() {
        "theirs" => false,
        "mine" => true,
        "both" => return message_page(&chat, &q),
        _ => return bad("side 는 theirs · mine · both"),
    };
    if q.before.is_some() || q.before_at.is_some() {
        return bad("before · before_at 은 side=both 와 함께");
    }
    let author = match q.kind.as_deref() {
        None => None,
        Some("human") => Some(Author::Human),
        Some("agent") => Some(Author::Agent),
        Some(_) => return bad("kind 는 human 또는 agent"),
    };
    let req = match q.req.as_deref().map(|s| ReqState::parse(s).ok_or(s)) {
        None => None,
        Some(Ok(s)) => Some(s),
        Some(Err(_)) => return bad("req 는 wait · accepted · rejected · done"),
    };
    let ask = match (q.ask.as_deref(), req) {
        (None, None) => None,
        (Some("query"), None) => Some(AskFilter::Query),
        (Some("request"), s) => Some(AskFilter::Request(s, q.archived.unwrap_or(false))),
        (Some("query") | None, Some(_)) => return bad("req 는 ask=request 와 함께"),
        (Some(_), _) => return bad("ask 는 query 또는 request"),
    };
    if q.archived.is_some() && !matches!(ask, Some(AskFilter::Request(..))) {
        return bad("archived 는 ask=request 와 함께");
    }
    let limit = q.limit.unwrap_or(MESSAGES_DEFAULT);
    if !(1..=MESSAGES_MAX).contains(&limit) {
        return bad(&format!("limit 은 1 ~ {MESSAGES_MAX}"));
    }
    let Some(scope) = scope_of(q.peer, q.group) else {
        return bad("peer 와 group 은 함께 주지 않는다");
    };
    let text = q.q.as_deref().map(str::trim).filter(|t| !t.is_empty());
    let db = chat.db();
    let ms = match chat::messages_across(&db, Some(mine), scope, author, ask, text, limit) {
        Ok(ms) => ms,
        Err(e) => return db_fail(e),
    };
    let mut convs = HashMap::new();
    let mut out = Vec::with_capacity(ms.len());
    for m in &ms {
        if let Entry::Vacant(slot) = convs.entry(m.conversation_id) {
            match chat::by_id(&db, m.conversation_id) {
                Ok(c) => slot.insert(c),
                Err(e) => return db_fail(e),
            };
        }
        let mut v = message_json(m);
        v["conversation"] = match &convs[&m.conversation_id] {
            Some(c) => {
                json!({
                    "id": c.id, "peer_id": c.peer_id, "agent": c.agent, "dir": c.dir.as_str(),
                    "left_at": c.left_at, "peer_left_at": c.peer_left_at,
                })
            }
            None => Value::Null,
        };
        out.push(v);
    }
    Json(out).into_response()
}

/// 주고받은 메시지 한 쪽(#1q, `side=both`) — 양쪽 글 최신부터 · `before` 커서. 거르기(kind · ask · req · q)는 받지 않는다.
fn message_page(chat: &Chat, q: &MessageQuery) -> Response {
    if q.kind.is_some() || q.ask.is_some() || q.req.is_some() || q.q.is_some() || q.archived.is_some() {
        return bad("side=both 는 peer · group · before · limit 만");
    }
    let limit = q.limit.unwrap_or(MESSAGES_DEFAULT);
    if !(1..=MESSAGES_MAX).contains(&limit) {
        return bad(&format!("limit 은 1 ~ {MESSAGES_MAX}"));
    }
    let Some(scope) = scope_of(q.peer, q.group) else {
        return bad("peer 와 group 은 함께 주지 않는다");
    };
    let before = match (q.before_at, q.before) {
        (Some(at), Some(id)) => Some((at, id)),
        (None, None) => None,
        _ => return bad("before 와 before_at 은 함께"),
    };
    let db = chat.db();
    let page = peer_since(&db).and_then(|since| chat::messages_page(&db, scope, before, Some(since), limit));
    match page.and_then(|ms| Ok((ms, summary_map(&db)?))) {
        Ok((ms, sums)) => Json(ms.iter().map(|m| message_in(m, &sums)).collect::<Vec<_>>()).into_response(),
        Err(e) => db_fail(e),
    }
}

/// 요청 상태별 건수(#1q 적대적 평가 R1) — `side=theirs|mine` · `peer` | `group` 범위 전체 → `{wait, accepted, rejected, done, total}`.
async fn req_counts(State(chat): State<Arc<Chat>>, Query(q): Query<MessageQuery>) -> Response {
    let mine = match q.side.as_str() {
        "theirs" => false,
        "mine" => true,
        _ => return bad("side 는 theirs 또는 mine"),
    };
    let Some(scope) = scope_of(q.peer, q.group) else {
        return bad("peer 와 group 은 함께 주지 않는다");
    };
    match chat::req_counts(&chat.db(), mine, scope) {
        Ok(n) => Json(json!({
            "wait": n.wait, "accepted": n.accepted, "rejected": n.rejected, "done": n.done, "archived": n.archived, "total": n.total(),
        }))
        .into_response(),
        Err(e) => db_fail(e),
    }
}

/// 주고받은 메시지 보기 기간(#1y — 설정 `ui_peer_weeks`)의 시작 시각.
fn peer_since(db: &an_core::db::Connection) -> Result<i64, an_core::db::Error> {
    Ok(chat::unix_now() - an_core::run_opts::peer_weeks(db)? * 7 * 86400)
}

/// 주고받은 메시지 상대별 요약(#1q) — `[{peer, count, last}]`(`last` = 마지막 메시지 JSON + 대화).
async fn message_peers(State(chat): State<Arc<Chat>>, Query(q): Query<SavedQuery>) -> Response {
    let Some(scope) = scope_of(q.peer, q.group) else {
        return bad("peer 와 group 은 함께 주지 않는다");
    };
    let db = chat.db();
    let rows = (|| {
        let sums = summary_map(&db)?;
        chat::peer_message_counts(&db, scope, Some(peer_since(&db)?))?
            .into_iter()
            .map(|(peer, count, last)| {
                let m = chat::message_by_id(&db, last)?;
                Ok(json!({ "peer": peer, "count": count, "last": m.map_or(Value::Null, |m| message_in(&m, &sums)) }))
            })
            .collect::<Result<Vec<Value>, an_core::db::Error>>()
    })();
    match rows {
        Ok(v) => Json(v).into_response(),
        Err(e) => db_fail(e),
    }
}

async fn start_conversation(
    State(chat): State<Arc<Chat>>,
    Json(req): Json<NewConversation>,
) -> Response {
    if let Err(m) = to_of(&req.to) {
        return bad(m);
    }
    let via = match via_of(&req.via) {
        Ok(v) => v,
        Err(m) => return bad(m),
    };
    // 보내기 전에 검증 — 틀린 제목 때문에 글만 가고 실패로 끝나지 않게.
    let title = match req.title.as_deref().map(an_core::run_opts::clean_title) {
        None => None,
        Some(Ok(t)) => t,
        Some(Err(m)) => return bad(&m),
    };
    let key = match tx_key_of(&req.tx_key, via) {
        Ok(k) => k,
        Err(m) => return bad(m),
    };
    // 같은 키는 확인부터 보내기까지 한 줄로 — 동시에 두 번 불러도 한 번만 보낸다(#1c).
    let _keyed = match key {
        Some(_) => Some(chat.keyed().await),
        None => None,
    };
    if let Some(again) = tx_again(&chat, key) {
        return again;
    }
    if let Some(g) = req.group {
        match an_core::groups::by_id(&chat.db(), g) {
            Ok(Some(_)) => {}
            Ok(None) => return fail(io::Error::new(io::ErrorKind::NotFound, "없는 그룹")),
            Err(e) => return db_fail(e),
        }
    }
    // 세션 생성(#1x) — 글 없이 대화만(상대에는 첫 글이 갈 때 생긴다). CLI 는 지금처럼 본문 필수.
    if req.text.trim().is_empty() {
        if via.is_some() || req.request {
            return bad("본문이 비었다");
        }
        let relay = req
            .relay_agent
            .as_deref()
            .map(str::trim)
            .filter(|m| !m.is_empty());
        let relay = match relay {
            Some(mine) => match req
                .relay_cap
                .map_or_else(|| default_relay_cap(&chat.db()), Ok)
            {
                Ok(cap) => Some((mine, cap)),
                Err(e) => return db_fail(e),
            },
            None => None,
        };
        let opts = Compose {
            title: title.as_deref(),
            group: req.group,
            ..Compose::default()
        };
        return match chat.open_session(req.peer_id, &req.agent, relay, opts) {
            Ok(c) => (
                StatusCode::CREATED,
                Json(json!({ "conversation": conversation_json(&c) })),
            )
                .into_response(),
            Err(e) => fail(e),
        };
    }
    let started = match req.relay_agent.as_deref().map(str::trim) {
        Some(mine) if !mine.is_empty() && via.is_some() => {
            return bad("자동 왕복 시작에는 via 를 쓰지 않는다");
        }
        Some(mine) if !mine.is_empty() => {
            let cap = match req
                .relay_cap
                .map_or_else(|| default_relay_cap(&chat.db()), Ok)
            {
                Ok(c) => c,
                Err(e) => return db_fail(e),
            };
            let opts = Compose {
                title: title.as_deref(),
                request: req.request,
                group: req.group,
                ..Compose::default()
            };
            chat.start_relay_with(req.peer_id, &req.agent, mine, cap, &req.text, opts)
                .await
        }
        _ => {
            let opts = Compose {
                via,
                tx_key: key,
                title: title.as_deref(),
                request: req.request,
                group: req.group,
                ..Compose::default()
            };
            chat.send_with(req.peer_id, &req.agent, None, &req.text, opts)
                .await
        }
    };
    match started {
        Ok((c, m)) => sent_tx(&chat, c, m, None, via),
        Err(e) => fail(e),
    }
}

async fn patch_conversation(
    State(chat): State<Arc<Chat>>,
    Path(id): Path<i64>,
    Json(req): Json<ConversationPatch>,
) -> Response {
    if req.title.is_none() && req.starred.is_none() {
        return bad("title 또는 starred 를 준다");
    }
    let title = match req.title.as_deref().map(an_core::run_opts::clean_title) {
        None => None,
        Some(Ok(t)) => Some(t),
        Some(Err(m)) => return bad(&m),
    };
    let db = chat.db();
    // 둘 다 한 트랜잭션 — 하나만 바뀌지 않는다.
    let marks = (|| {
        let tx = an_core::db::write_tx(&db)?;
        if chat::marks(&tx, id)?.is_none() {
            return Ok(None);
        }
        if let Some(t) = &title {
            chat::set_title(&tx, id, t.as_deref())?;
        }
        if let Some(on) = req.starred {
            chat::set_starred(&tx, id, on)?;
        }
        let marks = chat::marks(&tx, id)?;
        tx.commit()?;
        Ok(marks)
    })();
    match marks {
        Ok(Some((title, starred))) => {
            Json(json!({ "id": id, "title": title, "starred": starred })).into_response()
        }
        Ok(None) => fail(io::Error::new(io::ErrorKind::NotFound, "없는 대화")),
        Err(e) => db_fail(e),
    }
}

/// 대화를 열었다 — 화면이 받은 마지막 글(`upto`)까지 읽음(이 PC 에만). 그 대화 글이 아니면 400.
async fn read_conversation(
    State(chat): State<Arc<Chat>>,
    Path(id): Path<i64>,
    Json(req): Json<ReadUpto>,
) -> Response {
    let db = chat.db();
    match chat::by_id(&db, id)
        .and_then(|c| c.map(|_| chat::mark_read(&db, id, req.upto)).transpose())
    {
        Ok(Some(true)) => StatusCode::NO_CONTENT.into_response(),
        Ok(Some(false)) => bad("upto 는 이 대화의 글 id"),
        Ok(None) => fail(io::Error::new(io::ErrorKind::NotFound, "없는 대화")),
        Err(e) => db_fail(e),
    }
}

/// 메시지 저장 · 해제(이 PC 에만) · 받은 요청 상태 · 본문 편집(#1f).
async fn save_message(
    State(chat): State<Arc<Chat>>,
    Path(id): Path<i64>,
    Json(req): Json<MessagePatch>,
) -> Response {
    if req.mirror.is_some() && req.text.is_none() {
        return bad("mirror 는 text 와 함께");
    }
    if let Some(text) = req.text.as_deref() {
        if req.saved.is_some() || req.req_state.is_some() || req.note.is_some() {
            return bad("text 는 다른 항목과 함께 쓰지 않는다");
        }
        return match chat.edit_message(id, text, req.mirror == Some(true)).await {
            Ok((m, warning)) => {
                let mut out = json!({ "id": id, "text": m.text, "edited_at": m.edited_at });
                if let Some(w) = warning {
                    out["warning"] = json!(w);
                }
                Json(out).into_response()
            }
            Err(e) => fail(e),
        };
    }
    if req.saved.is_none() && req.req_state.is_none() {
        return bad("saved · req_state · text 중 하나가 필요하다");
    }
    if req.note.is_some() && req.req_state.is_none() {
        return bad("note 는 req_state 와 함께");
    }
    let mut out = json!({ "id": id });
    if let Some(state) = req.req_state.as_deref() {
        // 네 상태 어디로든(대기 되돌리기 포함 — 사용자 2026-10-19 "4 상태 자유 이동", #1y 보완).
        let Some(state) = ReqState::parse(state) else {
            return bad("req_state 는 wait · accepted · rejected · done");
        };
        match chat
            .set_req_state(id, state, req.note.as_deref().unwrap_or(""), req.saved)
            .await
        {
            Ok((m, warning)) => {
                out["req_state"] = json!(m.req_state.map(ReqState::as_str));
                out["req_note"] = json!(m.req_note);
                if req.saved.is_some() {
                    out["saved"] = json!(m.saved_at.is_some());
                }
                if let Some(w) = warning {
                    out["warning"] = json!(w);
                }
            }
            Err(e) => return fail(e),
        }
    } else if let Some(saved) = req.saved {
        match chat::set_saved(&chat.db(), id, saved) {
            Ok(true) => out["saved"] = json!(saved),
            Ok(false) => return fail(io::Error::new(io::ErrorKind::NotFound, "없는 메시지")),
            Err(e) => return db_fail(e),
        }
    }
    Json(out).into_response()
}

/// 요청 archive · 해제(#1y — 이 PC 에만, 상대에 보내지 않는다) → 메시지 JSON. archive 는 완료 요청만(409).
async fn archive_message(
    State(chat): State<Arc<Chat>>,
    Path(id): Path<i64>,
    Json(req): Json<ArchivePatch>,
) -> Response {
    let db = chat.db();
    let m = match chat::message_by_id(&db, id) {
        Ok(Some(m)) => m,
        Ok(None) => return fail(io::Error::new(io::ErrorKind::NotFound, "없는 메시지")),
        Err(e) => return db_fail(e),
    };
    match chat::set_req_archived(&db, id, req.archived) {
        Ok(true) => {}
        Ok(false) if m.request && m.author == Author::Human && req.archived && m.req_state != Some(ReqState::Done) => {
            return fail(io::Error::new(io::ErrorKind::Unsupported, "완료된 요청만 archive 할 수 있습니다"));
        }
        Ok(false) => return bad("요청 글만 archive 한다(공유 사본 제외)"),
        Err(e) => return db_fail(e),
    }
    match chat::message_by_id(&db, id) {
        Ok(Some(m)) => Json(message_json(&m)).into_response(),
        Ok(None) => fail(io::Error::new(io::ErrorKind::NotFound, "없는 메시지")),
        Err(e) => db_fail(e),
    }
}

/// 끝난 내 사람 글 여러 개 지우기(#1f) — `{deleted, warning?}`.
async fn delete_messages(
    State(chat): State<Arc<Chat>>,
    Json(req): Json<MessageDelete>,
) -> Response {
    if req.ids.is_empty() || req.ids.len() > DELETE_MAX {
        return bad(&format!("ids 는 1 ~ {DELETE_MAX}개"));
    }
    match chat.delete_messages(&req.ids, req.mirror).await {
        Ok((n, warning)) => {
            let mut out = json!({ "deleted": n });
            if let Some(w) = warning {
                out["warning"] = json!(w);
            }
            Json(out).into_response()
        }
        Err(e) => fail(e),
    }
}

/// 통합 검색 — 묶음별 `{messages, conversations, peers, agents}`. 에이전트 `owner` = `"me"` 또는 그 상대 id.
async fn search(State(chat): State<Arc<Chat>>, Query(q): Query<SearchQuery>) -> Response {
    let text = q.q.trim();
    if text.is_empty() || text.chars().count() > SEARCH_MAX_CHARS {
        return bad(&format!("q 는 1 ~ {SEARCH_MAX_CHARS}자"));
    }
    let limit = q.limit.unwrap_or(SEARCH_DEFAULT);
    if !(1..=SEARCH_MAX).contains(&limit) {
        return bad(&format!("limit 은 1 ~ {SEARCH_MAX}"));
    }
    let db = chat.db();
    let closed = q.closed.unwrap_or(false);
    let (found, sums) =
        match chat::search(&db, text, limit, closed).and_then(|f| Ok((f, summary_map(&db)?))) {
            Ok(v) => v,
            Err(e) => return db_fail(e),
        };
    let agents: Vec<Value> = found
        .agents
        .iter()
        .map(|a| json!({ "name": a.name, "owner": a.owner.map_or(json!("me"), |p| json!(p)) }))
        .collect();
    Json(json!({
        "messages": found.messages.iter().map(|m| message_in(m, &sums)).collect::<Vec<_>>(),
        "conversations": found.conversations.iter().map(summary_json).collect::<Vec<_>>(),
        "peers": found.peers.into_iter().map(PeerView::from).collect::<Vec<_>>(),
        "agents": agents,
    }))
    .into_response()
}

/// 내 활동 — `{items, unseen(그 범위), unseen_all, upto: {message, event}}`. 항목 = `{kind, at, conversation, message?, detail?, unseen}`.
/// `upto` = 목록과 같은 잠금에서 읽은 마지막 글 · 자동 왕복 기록 id(본 표시의 범위). `limit=0` 이면 목록 없이 수만.
async fn activity(State(chat): State<Arc<Chat>>, Query(q): Query<ActivityQuery>) -> Response {
    let filter = match q.kind.as_deref() {
        None | Some("all") => ActivityFilter::All,
        Some("request") => ActivityFilter::Request,
        Some("mine") => ActivityFilter::MyRequest,
        Some("answer") => ActivityFilter::Answer,
        Some("failed") => ActivityFilter::Failed,
        Some(_) => return bad("kind 는 all · request · mine · answer · failed"),
    };
    let limit = q.limit.unwrap_or(MESSAGES_DEFAULT);
    if !(0..=MESSAGES_MAX).contains(&limit) {
        return bad(&format!("limit 은 0 ~ {MESSAGES_MAX}"));
    }
    let text = q.q.as_deref().map(str::trim).filter(|t| !t.is_empty());
    let db = chat.db();
    let page = (|| {
        let items = if limit == 0 {
            Vec::new()
        } else {
            chat::activity(&db, q.peer, filter, text, limit)?
        };
        let sums = if items.is_empty() {
            HashMap::new()
        } else {
            summary_map(&db)?
        };
        let items: Vec<Value> = items
            .iter()
            .map(|a| {
                let mut v = json!({
                    "kind": a.kind.as_str(), "at": a.at, "unseen": a.unseen,
                    "conversation": sums.get(&a.conversation_id).map_or(Value::Null, conversation_ref),
                });
                if let Some(m) = &a.message {
                    v["message"] = message_json(m);
                }
                if let Some(d) = &a.detail {
                    v["detail"] = json!(d);
                }
                v
            })
            .collect();
        let unseen_all = chat::activity_unseen(&db, None)?;
        let unseen = match q.peer {
            Some(p) => chat::activity_unseen(&db, Some(p))?,
            None => unseen_all,
        };
        let (message, event) = chat::activity_upto(&db)?;
        Ok::<_, an_core::db::Error>(json!({
            "items": items,
            "unseen": unseen,
            "unseen_all": unseen_all,
            "upto": { "message": message, "event": event },
        }))
    })();
    match page {
        Ok(v) => Json(v).into_response(),
        Err(e) => db_fail(e),
    }
}

/// 내 활동을 봤다(이 PC) — 그 상대 또는 전부를 화면이 받은 범위까지. 범위 없는 몸은 거절(본 범위를 모른 채 전부를 본 것으로 하지 않게).
async fn activity_seen(State(chat): State<Arc<Chat>>, Json(req): Json<ActivitySeen>) -> Response {
    let peer = req.peer;
    match chat::mark_activity_seen(&chat.db(), peer, (req.message, req.event)) {
        Ok(0) if peer.is_some() => fail(io::Error::new(io::ErrorKind::NotFound, "없는 상대")),
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => db_fail(e),
    }
}

/// 저장된 항목 — 저장 순 최신부터, 메시지 JSON + 대화.
async fn saved_list(State(chat): State<Arc<Chat>>, Query(q): Query<SavedQuery>) -> Response {
    let limit = q.limit.unwrap_or(MESSAGES_DEFAULT);
    if !(1..=MESSAGES_MAX).contains(&limit) {
        return bad(&format!("limit 은 1 ~ {MESSAGES_MAX}"));
    }
    let Some(scope) = scope_of(q.peer, q.group) else {
        return bad("peer 와 group 은 함께 주지 않는다");
    };
    let db = chat.db();
    let list =
        chat::saved(&db, scope, q.conversation, limit).and_then(|ms| Ok((ms, summary_map(&db)?)));
    match list {
        Ok((ms, sums)) => {
            Json(ms.iter().map(|m| message_in(m, &sums)).collect::<Vec<_>>()).into_response()
        }
        Err(e) => db_fail(e),
    }
}

/// 자동 왕복 멈춤 · 재개(D24) — 재개는 대화를 연 쪽만.
async fn switch_relay(
    State(chat): State<Arc<Chat>>,
    Path(id): Path<i64>,
    Json(req): Json<RelaySwitch>,
) -> Response {
    match chat.relay(id, req.on).await {
        Ok(c) => Json(conversation_json(&c)).into_response(),
        Err(e) => fail(e),
    }
}

/// 대화 컨텍스트 압축 · 비우기(#1m) — 구분선 글을 돌려준다. 틀린 `op` 400 · 없는 대화 404 · 진행 중 409.
async fn context_op(
    State(chat): State<Arc<Chat>>,
    Path(id): Path<i64>,
    Json(req): Json<ContextOp>,
) -> Response {
    match chat.context(id, &req.op) {
        Ok(m) => Json(message_json(&m)).into_response(),
        Err(e) => fail(e),
    }
}

#[derive(Deserialize)]
struct NewObserver {
    peer_id: i64,
}

/// 관람자 넣기(#1o) — 204. 사본 · 원 상대 · 5명 넘음 400, 없는 대화 · 상대 404, 이미 관람자 · 상대 앱이 이 기능을 모름 409.
async fn add_observer(
    State(chat): State<Arc<Chat>>,
    Path(id): Path<i64>,
    Json(req): Json<NewObserver>,
) -> Response {
    match chat.share_add(id, req.peer_id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => fail(e),
    }
}

/// 관람자 빼기(#1o) — 204, 관람자가 아니면 404.
async fn remove_observer(
    State(chat): State<Arc<Chat>>,
    Path((id, peer)): Path<(i64, i64)>,
) -> Response {
    match chat.share_remove(id, peer) {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => fail(e),
    }
}

/// 대화에서 나가기(#1w — 모든 대화, 공유 사본은 #1o 공유 나가기) — 204. 없는 대화 404 · 도는 실행이 있으면 409.
async fn leave_conversation(State(chat): State<Arc<Chat>>, Path(id): Path<i64>) -> Response {
    match chat.leave_conversation(id) {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => fail(e),
    }
}

/// 대화 삭제(#1w) — 나가기 + 이 PC 의 대화 · 글 · 딸린 항목. 204. 없는 대화 404 · 도는 실행이 있으면 409.
async fn delete_conversation(State(chat): State<Arc<Chat>>, Path(id): Path<i64>) -> Response {
    match chat.delete_conversation(id) {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => fail(e),
    }
}

#[derive(Deserialize)]
struct ClearReq {
    /// 참 = 모든 사용자 글 삭제(상대 PC 도 비운다), 거짓 = 내 글만 삭제(이 PC).
    #[serde(default)]
    all: bool,
}

/// 기록 삭제(#1x) — 204. 공유 사본 · 닫힌 대화에 `all` 400, 실행 중 · 상대 앱이 모름 409, 없는 대화 404.
async fn clear_conversation(
    State(chat): State<Arc<Chat>>,
    Path(id): Path<i64>,
    Json(req): Json<ClearReq>,
) -> Response {
    match chat.clear_conversation(id, req.all).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => fail(e),
    }
}

/// 대화에 이어 쓴다(D18 · D23 개정) — 양쪽 다, 늘 대화 에이전트에게 실행 요청.
async fn continue_conversation(
    State(chat): State<Arc<Chat>>,
    Path(id): Path<i64>,
    Json(req): Json<NewMessage>,
) -> Response {
    if let Err(m) = to_of(&req.to) {
        return bad(m);
    }
    let via = match via_of(&req.via) {
        Ok(v) => v,
        Err(m) => return bad(m),
    };
    let key = match tx_key_of(&req.tx_key, via) {
        Ok(k) => k,
        Err(m) => return bad(m),
    };
    let _keyed = match key {
        Some(_) => Some(chat.keyed().await),
        None => None,
    };
    if let Some(again) = tx_again(&chat, key) {
        return again;
    }
    let opts = Compose {
        via,
        tx_key: key,
        request: req.request,
        mention: req.agent.as_deref(),
        ..Compose::default()
    };
    match chat.post_with(id, &req.text, true, opts).await {
        Ok((c, m, warning)) => sent_tx(&chat, c, m, warning, via),
        Err(e) => fail(e),
    }
}

/// 트랜잭션 키 — CLI 글(`via: cli`)에만, 규칙은 [`an_core::cli_tx::key_ok`].
fn tx_key_of<'a>(
    key: &'a Option<String>,
    via: Option<&str>,
) -> Result<Option<&'a str>, &'static str> {
    match key.as_deref() {
        None => Ok(None),
        Some(_) if via.is_none() => Err("tx_key 는 via: cli 에만"),
        Some(k) if an_core::cli_tx::key_ok(k) => Ok(Some(k)),
        Some(_) => Err("tx_key 는 1 ~ 64자 영문 · 숫자 · . _ -"),
    }
}

/// 같은 키의 트랜잭션이 있으면 보내지 않고 그것을(200).
fn tx_again(chat: &Chat, key: Option<&str>) -> Option<Response> {
    let found = chat.tx_by_key(key?).and_then(|t| match t {
        Some(t) => chat.tx_view(t.id),
        None => Ok(None),
    });
    match found {
        Ok(Some(view)) => Some(Json(tx_json(&chat_conv(chat, &view), &view)).into_response()),
        Ok(None) => None,
        Err(e) => Some(fail(e)),
    }
}

fn chat_conv(chat: &Chat, view: &TxView) -> Value {
    chat::by_id(&chat.db(), view.message.conversation_id)
        .ok()
        .flatten()
        .map_or(Value::Null, |c| conversation_json(&c))
}

/// 보냄(201) — 보냈지만 덧붙은 일이 어긋났으면 `warning`, CLI 글이면 트랜잭션(`tx`, #1c).
fn sent_tx(
    chat: &Chat,
    conv: Conversation,
    msg: Message,
    warning: Option<String>,
    via: Option<&str>,
) -> Response {
    // CLI 글의 트랜잭션은 글과 같은 DB 트랜잭션에서 만들었다(#1c) — 없으면 그 사실을 감추지 않는다.
    let tx = match via.map(|_| chat.tx_of_message(msg.id)) {
        None => None,
        Some(Ok(Some(t))) => Some(t.id),
        Some(Ok(None)) => return fail(io::Error::other("CLI 트랜잭션이 없다")),
        Some(Err(e)) => return fail(e),
    };
    let mut v = json!({ "conversation": conversation_json(&conv), "message": message_json(&msg), "tx": tx });
    if let Some(w) = warning {
        v["warning"] = json!(w);
    }
    (StatusCode::CREATED, Json(v)).into_response()
}

/// 트랜잭션 JSON — `{tx, state, conversation, message, reply, progress, outbox, again}`.
fn tx_json(conv: &Value, view: &TxView) -> Value {
    json!({
        "tx": view.tx.id, "key": view.tx.key, "state": view.state, "conversation": conv,
        "message": message_json(&view.message),
        "reply": view.reply.as_ref().map_or(Value::Null, message_json),
        "progress": view.progress.as_ref().map_or(Value::Null, |p| json!({ "state": p.state, "ahead": p.ahead, "detail": p.detail })),
        "outbox": view.outbox.as_ref().map_or(Value::Null, |o| json!({ "state": o.state, "tries": o.tries, "error": o.last_error })),
        "stalled": view.tx.stalled_at.is_some(),
    })
}

async fn get_tx(State(chat): State<Arc<Chat>>, Path(id): Path<i64>) -> Response {
    match chat.tx_view(id) {
        Ok(Some(view)) => Json(tx_json(&chat_conv(&chat, &view), &view)).into_response(),
        Ok(None) => fail(io::Error::new(io::ErrorKind::NotFound, "없는 트랜잭션")),
        Err(e) => fail(e),
    }
}

/// 이어 받기 — 멈춤을 풀고 지금 상태.
async fn resume_tx(State(chat): State<Arc<Chat>>, Path(id): Path<i64>) -> Response {
    if let Err(e) = chat.tx_resume(id) {
        return fail(e);
    }
    get_tx(State(chat), Path(id)).await
}
