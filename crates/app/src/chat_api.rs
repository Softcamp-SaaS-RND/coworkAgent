//! 에이전트 프로필 · 대화 API(S1 · S3 · D21) — `server::router_with` 의 guard 뒤에 붙는다. 원격 답 본문은 웹UI 가 textContent 로만 그린다.
use an_agent::chat::{Chat, default_relay_cap};
use an_core::agents::{self, Agent};
use an_core::chat::{self, Conversation, Message};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{Value, json};
use std::io;
use std::sync::Arc;

/// 에이전트 이름 최대 글자 수.
pub const MAX_AGENT_NAME: usize = 64;

#[derive(Deserialize)]
struct NewAgent {
    name: String,
    folder: String,
    /// 비면 읽기 전용 기본값.
    #[serde(default)]
    tools: Option<String>,
    #[serde(default)]
    prompt: String,
}

#[derive(Deserialize)]
struct NewConversation {
    peer_id: i64,
    agent: String,
    text: String,
    /// 자동 왕복(D24) — 이 PC 의 내 에이전트. 있으면 두 에이전트 자동 왕복 대화.
    #[serde(default)]
    relay_agent: Option<String>,
    /// 자동 왕복 상한(왕복, 1 ~ 50) — 없으면 10.
    #[serde(default)]
    relay_cap: Option<i64>,
}

#[derive(Deserialize)]
struct RelaySwitch {
    on: bool,
}

#[derive(Deserialize)]
struct NewMessage {
    text: String,
    /// `agent`(기본 — 대화 에이전트에게 실행 요청) | `human`(실행 없이 상대 사람과 양쪽 이력에).
    #[serde(default)]
    to: Option<String>,
}

pub fn routes(chat: Arc<Chat>) -> Router {
    Router::new()
        .route("/api/agents", get(list_agents).post(add_agent))
        .route("/api/agents/{id}", delete(remove_agent))
        .route("/api/peers/{id}/agents", get(remote_agents))
        .route(
            "/api/conversations",
            get(list_conversations).post(start_conversation),
        )
        .route(
            "/api/conversations/{id}/messages",
            get(list_messages).post(continue_conversation),
        )
        .route(
            "/api/conversations/{id}/relay",
            axum::routing::post(switch_relay),
        )
        .with_state(chat)
}

fn fail(e: io::Error) -> Response {
    let code = match e.kind() {
        io::ErrorKind::NotFound => StatusCode::NOT_FOUND,
        io::ErrorKind::InvalidInput => StatusCode::BAD_REQUEST,
        io::ErrorKind::AlreadyExists => StatusCode::CONFLICT,
        io::ErrorKind::PermissionDenied => StatusCode::FORBIDDEN,
        _ => StatusCode::BAD_GATEWAY,
    };
    (code, Json(json!({ "error": e.to_string() }))).into_response()
}

fn db_fail(e: an_core::db::Error) -> Response {
    fail(io::Error::other(e))
}

fn bad(msg: &str) -> Response {
    fail(io::Error::new(io::ErrorKind::InvalidInput, msg.to_string()))
}

fn agent_json(a: Agent) -> Value {
    json!({ "id": a.id, "name": a.name, "folder": a.folder, "tools": a.tools, "prompt": a.prompt })
}

fn conversation_json(c: &Conversation) -> Value {
    let r = c.relay.as_ref();
    json!({
        "id": c.id, "uuid": c.uuid, "peer_id": c.peer_id, "agent": c.agent, "dir": c.dir.as_str(),
        "relay_agent": r.map(|r| &r.agent), "relay_cap": r.map(|r| r.cap), "relay_count": r.map(|r| r.count),
        "relay_on": r.is_some_and(|r| r.on), "relay_blocked": c.relay_blocked,
    })
}

fn message_json(m: &Message) -> Value {
    json!({
        "id": m.id, "author": m.author.as_str(), "text": m.text, "state": m.state.as_str(), "created_at": m.created_at,
        "reply_to": m.reply_to, "turns": m.turns, "duration_ms": m.duration_ms, "mine": m.mine, "exec": m.exec,
    })
}

fn summary_json(s: &chat::Summary) -> Value {
    let mut v = conversation_json(&s.conversation);
    v["title"] = json!(s.title);
    v["last_state"] = json!(s.last_state.map(|st| st.as_str()));
    v["last_at"] = json!(s.last_at);
    v
}

async fn list_agents(State(chat): State<Arc<Chat>>) -> Response {
    match agents::list(&chat.db()) {
        Ok(list) => Json(list.into_iter().map(agent_json).collect::<Vec<_>>()).into_response(),
        Err(e) => db_fail(e),
    }
}

/// 폴더는 있는 절대 경로만 — 상대 경로는 앱의 현재 폴더 기준이 되어 의도와 달라진다.
async fn add_agent(State(chat): State<Arc<Chat>>, Json(req): Json<NewAgent>) -> Response {
    let name = req.name.trim();
    if name.is_empty()
        || name.chars().count() > MAX_AGENT_NAME
        || name.chars().any(char::is_control)
    {
        return bad("이름은 1 ~ 64자, 제어 문자 없이");
    }
    let folder = std::path::Path::new(req.folder.trim());
    if !folder.is_absolute() || !folder.is_dir() {
        return bad("폴더는 이 PC 에 있는 절대 경로여야 한다");
    }
    let tools = req
        .tools
        .as_deref()
        .map(str::trim)
        .filter(|t| !t.is_empty());
    // 원격 요청은 문의 응답 — 쓰기 · 실행 도구는 등록하지 않는다(D31).
    let unknown = tools.map(agents::unknown_tools).unwrap_or_default();
    if !unknown.is_empty() {
        return bad(&format!(
            "읽기 전용 도구만({}) — 받을 수 없는 도구: {}",
            agents::READ_ONLY_TOOLS.join(","),
            unknown.join(",")
        ));
    }
    let db = chat.db();
    if matches!(agents::by_name(&db, name), Ok(Some(_))) {
        return fail(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("같은 이름의 에이전트가 있다: {name}"),
        ));
    }
    match agents::add(
        &db,
        name,
        &folder.display().to_string(),
        tools,
        req.prompt.trim(),
    ) {
        Ok(a) => (StatusCode::CREATED, Json(agent_json(a))).into_response(),
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
    match chat::summaries(&chat.db()) {
        Ok(list) => Json(list.iter().map(summary_json).collect::<Vec<_>>()).into_response(),
        Err(e) => db_fail(e),
    }
}

async fn list_messages(State(chat): State<Arc<Chat>>, Path(id): Path<i64>) -> Response {
    let db = chat.db();
    match chat::by_id(&db, id) {
        Ok(Some(_)) => {}
        Ok(None) => return fail(io::Error::new(io::ErrorKind::NotFound, "없는 대화")),
        Err(e) => return db_fail(e),
    }
    match chat::messages(&db, id) {
        Ok(ms) => Json(ms.iter().map(message_json).collect::<Vec<_>>()).into_response(),
        Err(e) => db_fail(e),
    }
}

fn sent(conv: Conversation, msg: Message) -> Response {
    (
        StatusCode::CREATED,
        Json(json!({ "conversation": conversation_json(&conv), "message": message_json(&msg) })),
    )
        .into_response()
}

async fn start_conversation(
    State(chat): State<Arc<Chat>>,
    Json(req): Json<NewConversation>,
) -> Response {
    let started = match req.relay_agent.as_deref().map(str::trim) {
        Some(mine) if !mine.is_empty() => {
            let cap = req
                .relay_cap
                .unwrap_or_else(|| default_relay_cap(&chat.db()));
            chat.start_relay(req.peer_id, &req.agent, mine, cap, &req.text)
                .await
        }
        _ => chat.send(req.peer_id, &req.agent, None, &req.text).await,
    };
    match started {
        Ok((c, m)) => sent(c, m),
        Err(e) => fail(e),
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

/// 대화에 이어 쓴다(D18 · D23) — 양쪽 다, 글마다 대상(`to`)을 고른다.
async fn continue_conversation(
    State(chat): State<Arc<Chat>>,
    Path(id): Path<i64>,
    Json(req): Json<NewMessage>,
) -> Response {
    let to_agent = match req.to.as_deref() {
        None | Some("agent") => true,
        Some("human") => false,
        Some(_) => return bad("to 는 agent 또는 human"),
    };
    match chat.post(id, &req.text, to_agent).await {
        Ok((c, m, None)) => sent(c, m),
        Ok((c, m, Some(warning))) => (
            StatusCode::CREATED,
            Json(json!({ "conversation": conversation_json(&c), "message": message_json(&m), "warning": warning })),
        )
            .into_response(),
        Err(e) => fail(e),
    }
}
