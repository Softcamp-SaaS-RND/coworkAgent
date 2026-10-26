//! 에이전트 프로필 · 대화 API — `server::router_with` 의 guard 뒤에 붙는다. 원격 답 본문은 웹UI 가 textContent 로만 그린다.

mod agent;
mod conversation;
mod message;
mod run;
mod activity;
mod tx;

use agent::*;
use conversation::*;
use message::*;
use run::*;
use activity::*;
use tx::*;

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

/// 한 번에 지울 수 있는 글 수.
pub const DELETE_MAX: usize = 200;

/// 검색어 길이(글자) · 메시지 묶음 기본 · 최대 건수.
pub const SEARCH_MAX_CHARS: usize = 100;
pub const SEARCH_DEFAULT: i64 = 20;
pub const SEARCH_MAX: i64 = 100;

/// `GET /api/saved` — 저장된 항목.
#[derive(Deserialize)]
struct SavedQuery {
    #[serde(default)]
    peer: Option<i64>,
    /// 레일 그룹 — `peer` 와 함께 주면 400.
    #[serde(default)]
    group: Option<i64>,
    #[serde(default)]
    conversation: Option<i64>,
    #[serde(default)]
    limit: Option<i64>,
}

/// `GET /api/messages` — 대화를 가로지른 글 목록.
#[derive(Deserialize)]
struct MessageQuery {
    /// `theirs`(상대 쪽이 쓴 글) | `mine`(내가 쓴 글) | `both`(양쪽 — 주고받은 메시지 페이지).
    side: String,
    /// 커서(`side=both` 만) — 받은 마지막 글의 id · 시각(`before_at`), 둘 다 주면 그 `(created_at, id)` 보다 앞의 것만.
    #[serde(default)]
    before: Option<i64>,
    #[serde(default)]
    before_at: Option<i64>,
    #[serde(default)]
    peer: Option<i64>,
    /// 레일 그룹 — 그 그룹 대화만. `peer` 와 함께 주면 400.
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
    /// 요청 상태 `wait` | `accepted` | `rejected` | `done` — `ask=request` 와 함께만.
    #[serde(default)]
    req: Option<String>,
    /// 요청 archive — `true` = archive 된 것만, 없거나 `false` = archive 제외. `ask=request` 와 함께만.
    #[serde(default)]
    archived: Option<bool>,
    #[serde(default)]
    limit: Option<i64>,
}

/// 글 목록 기본 · 최대 건수.
pub const MESSAGES_DEFAULT: i64 = 200;
pub const MESSAGES_MAX: i64 = 500;

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
        .route("/api/conversations/{id}/info", get(conversation_info))
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

/// 목록 범위 — 상대 · 그룹 중 하나, 둘 다면 `None`.
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

/// 요청의 `to` — 없거나 `agent`. 새 대화 · 이어 쓰기 둘 다 — 그 밖 값은 400.
fn to_of(to: &Option<String>) -> Result<(), &'static str> {
    match to.as_deref() {
        None | Some("agent") => Ok(()),
        Some(_) => Err("to 는 agent 만 — 사람에게 보내기는 없다"),
    }
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
    // 대화방 목록 미리보기 — 마지막 글.
    v["last"] = json!(s.last.as_ref().map(|l| json!({
        "text": l.text,
        "author": l.author.as_str(),
        "mine": l.mine,
        "agent": l.agent,
        "via": l.via,
    })));
    v
}
