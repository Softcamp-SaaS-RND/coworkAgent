//! 실행 큐 API.

use super::*;

/// 실행 목록 항목 — 받은 글 · 대화 · 상대 · 에이전트 · 앞부분 40자. 그새 지워진 글은 id 만.
pub(super) fn run_json(db: &an_core::db::Connection, id: i64) -> Result<Value, an_core::db::Error> {
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
pub(super) async fn list_runs(State(chat): State<Arc<Chat>>) -> Response {
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
pub(super) async fn set_runs(State(chat): State<Arc<Chat>>, Json(req): Json<Value>) -> Response {
    let Some(cap) = req["cap"].as_u64().and_then(|n| usize::try_from(n).ok()) else {
        return bad("cap 은 1 ~ 8 정수");
    };
    match chat.set_max_runs(cap) {
        Ok(()) => list_runs(State(chat)).await,
        Err(e) => fail(e),
    }
}
