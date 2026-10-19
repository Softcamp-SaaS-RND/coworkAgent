//! 레일 그룹 · 손으로 놓는 순서 API(#1i) — 이 PC 에만(상대 PC 로 가는 프레임 없음). `chat_api::routes` 가 함께 붙인다.
use an_agent::chat::Chat;
use an_core::groups::{self, Group, Order};
use an_core::run_opts;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, patch, put};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NewGroup {
    name: String,
}

/// `PATCH /api/groups/{id}` — 준 항목만: 이름 · 그림(빈 문자열 = 지움). 둘 다 없으면 400.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GroupPatch {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    icon: Option<String>,
}

/// `PUT /api/groups/{id}/conversations` — 넣고 뺄 대화 id(이미 있음 · 없음은 그대로).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Membership {
    #[serde(default)]
    add: Vec<i64>,
    #[serde(default)]
    remove: Vec<i64>,
}

/// `PUT /api/order` — 그 목록의 새 순서(앞부터).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NewOrder {
    kind: String,
    ids: Vec<i64>,
}

pub fn routes(chat: Arc<Chat>) -> Router {
    Router::new()
        .route("/api/groups", get(list).post(add))
        .route("/api/groups/{id}", patch(update).delete(remove))
        .route("/api/groups/{id}/conversations", put(change))
        .route("/api/order", put(reorder))
        .with_state(chat)
}

fn group_json(g: &Group) -> Value {
    json!({ "id": g.id, "name": g.name, "icon": g.icon, "pos": g.pos, "convs": g.convs })
}

fn err(code: StatusCode, msg: &str) -> Response {
    (code, Json(json!({ "error": msg }))).into_response()
}

fn db_fail(e: an_core::db::Error) -> Response {
    err(StatusCode::BAD_GATEWAY, &e.to_string())
}

const TAKEN: &str = "같은 이름의 그룹이 있습니다";
const MISSING: &str = "없는 그룹";

async fn list(State(chat): State<Arc<Chat>>) -> Response {
    match groups::list(&chat.db()) {
        Ok(gs) => Json(gs.iter().map(group_json).collect::<Vec<_>>()).into_response(),
        Err(e) => db_fail(e),
    }
}

async fn add(State(chat): State<Arc<Chat>>, Json(req): Json<NewGroup>) -> Response {
    let name = match groups::clean_name(&req.name) {
        Ok(n) => n,
        Err(m) => return err(StatusCode::BAD_REQUEST, &m),
    };
    let db = chat.db();
    match groups::name_taken(&db, &name, None) {
        Ok(true) => return err(StatusCode::CONFLICT, TAKEN),
        Ok(false) => {}
        Err(e) => return db_fail(e),
    }
    match groups::add(&db, &name) {
        Ok(g) => (StatusCode::CREATED, Json(group_json(&g))).into_response(),
        Err(e) => db_fail(e),
    }
}

async fn update(
    State(chat): State<Arc<Chat>>,
    Path(id): Path<i64>,
    Json(req): Json<GroupPatch>,
) -> Response {
    if req.name.is_none() && req.icon.is_none() {
        return err(StatusCode::BAD_REQUEST, "name 또는 icon 을 준다");
    }
    // 둘 다 검증한 뒤 쓴다 — 하나라도 틀리면 아무것도 바꾸지 않는다.
    let name = match req.name.as_deref().map(groups::clean_name) {
        None => None,
        Some(Ok(n)) => Some(n),
        Some(Err(m)) => return err(StatusCode::BAD_REQUEST, &m),
    };
    let icon = match req.icon.as_deref().map(run_opts::clean_icon) {
        None => None,
        Some(Ok(i)) => Some(i),
        Some(Err(m)) => return err(StatusCode::BAD_REQUEST, &m),
    };
    let db = chat.db();
    let tx = match an_core::db::write_tx(&db) {
        Ok(t) => t,
        Err(e) => return db_fail(e),
    };
    let done = (|| -> Result<Result<(), Response>, an_core::db::Error> {
        if groups::by_id(&tx, id)?.is_none() {
            return Ok(Err(err(StatusCode::NOT_FOUND, MISSING)));
        }
        if let Some(n) = &name {
            if groups::name_taken(&tx, n, Some(id))? {
                return Ok(Err(err(StatusCode::CONFLICT, TAKEN)));
            }
            groups::rename(&tx, id, n)?;
        }
        if let Some(i) = &icon {
            groups::set_icon(&tx, id, i.as_deref())?;
        }
        Ok(Ok(()))
    })();
    match done {
        Ok(Ok(())) => match tx.commit().and_then(|_| groups::by_id(&db, id)) {
            Ok(Some(g)) => Json(group_json(&g)).into_response(),
            Ok(None) => err(StatusCode::NOT_FOUND, MISSING),
            Err(e) => db_fail(e),
        },
        Ok(Err(r)) => r,
        Err(e) => db_fail(e),
    }
}

async fn remove(State(chat): State<Arc<Chat>>, Path(id): Path<i64>) -> Response {
    match groups::remove(&chat.db(), id) {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => err(StatusCode::NOT_FOUND, MISSING),
        Err(e) => db_fail(e),
    }
}

async fn change(
    State(chat): State<Arc<Chat>>,
    Path(id): Path<i64>,
    Json(req): Json<Membership>,
) -> Response {
    let db = chat.db();
    // 넣기 · 빼기 모두 있는 대화여야 한다 — 그룹에 없는 대화를 빼는 것만 그대로 둔다(라운드 2).
    let all: Vec<i64> = req.add.iter().chain(&req.remove).copied().collect();
    match groups::unknown_conversations(&db, &all) {
        Ok(u) if !u.is_empty() => {
            return err(StatusCode::BAD_REQUEST, &format!("없는 대화 {u:?}"));
        }
        Ok(_) => {}
        Err(e) => return db_fail(e),
    }
    match groups::change(&db, id, &req.add, &req.remove)
        .and_then(|ok| Ok(if ok { groups::by_id(&db, id)? } else { None }))
    {
        Ok(Some(g)) => Json(group_json(&g)).into_response(),
        Ok(None) => err(StatusCode::NOT_FOUND, MISSING),
        Err(e) => db_fail(e),
    }
}

async fn reorder(State(chat): State<Arc<Chat>>, Json(req): Json<NewOrder>) -> Response {
    let Some(kind) = Order::parse(&req.kind) else {
        return err(
            StatusCode::BAD_REQUEST,
            "kind 는 peers · groups · conversations",
        );
    };
    match groups::reorder(&chat.db(), kind, &req.ids) {
        Ok(Ok(())) => StatusCode::NO_CONTENT.into_response(),
        Ok(Err(m)) => err(StatusCode::BAD_REQUEST, &m),
        Err(e) => db_fail(e),
    }
}
