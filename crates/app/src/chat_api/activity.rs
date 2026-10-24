//! 내 활동 · 검색 · 북마크 목록 · 요청 개수 API.

use super::*;

/// `GET /api/search` — 통합 검색.
#[derive(Deserialize)]
pub(super) struct SearchQuery {
    q: String,
    /// 메시지 묶음 건수(그 밖 묶음은 [`chat::SEARCH_OTHERS`]).
    #[serde(default)]
    limit: Option<i64>,
    /// 닫힌 대화(나감 · 상대가 나감 · 끝난 공유 사본)도 찾는가(웹UI "나간 대화 포함", 기본 뺌).
    #[serde(default)]
    closed: Option<bool>,
}

/// `GET /api/activity` — 내 활동.
#[derive(Deserialize)]
pub(super) struct ActivityQuery {
    #[serde(default)]
    peer: Option<i64>,
    /// `all`(기본) | `request`(받은 요청) | `mine`(내 요청) | `answer`(받은 답) | `failed`(실패).
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
pub(super) struct ActivitySeen {
    #[serde(default)]
    peer: Option<i64>,
    message: i64,
    event: i64,
}

/// 요청 상태별 건수 — `side=theirs|mine` · `peer` | `group` 범위 전체 → `{wait, accepted, rejected, done, archived, total}`.
pub(super) async fn req_counts(State(chat): State<Arc<Chat>>, Query(q): Query<MessageQuery>) -> Response {
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

/// 통합 검색 — 묶음별 `{messages, conversations, peers, agents}`. 에이전트 `owner` = `"me"` 또는 그 상대 id.
pub(super) async fn search(State(chat): State<Arc<Chat>>, Query(q): Query<SearchQuery>) -> Response {
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
pub(super) async fn activity(State(chat): State<Arc<Chat>>, Query(q): Query<ActivityQuery>) -> Response {
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
pub(super) async fn activity_seen(State(chat): State<Arc<Chat>>, Json(req): Json<ActivitySeen>) -> Response {
    let peer = req.peer;
    match chat::mark_activity_seen(&chat.db(), peer, (req.message, req.event)) {
        Ok(0) if peer.is_some() => fail(io::Error::new(io::ErrorKind::NotFound, "없는 상대")),
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => db_fail(e),
    }
}

/// 저장된 항목 — 저장 순 최신부터, 메시지 JSON + 대화.
pub(super) async fn saved_list(State(chat): State<Arc<Chat>>, Query(q): Query<SavedQuery>) -> Response {
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
