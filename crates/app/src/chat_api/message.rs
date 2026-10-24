//! 글 API — 목록 · 페이지 · 보내기 · 보관 · 지우기.

use super::*;

/// `PATCH /api/messages/{id}` — 준 항목만: 저장된 항목(이 PC 에만) · 받은 요청의 상태 + 메모(쓴 쪽에 간다) · 본문 편집. 모두 없으면
/// 400, 저장 + 상태면 한 트랜잭션(상태를 바꿀 수 없으면 아무것도 쓰지 않는다). 편집은 다른 항목과 함께 쓰지 않는다. 응답도 준 항목만.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct MessagePatch {
    #[serde(default)]
    saved: Option<bool>,
    /// `wait` | `accepted` | `rejected` | `done` — 지금 상태 말고 네 상태 어디로든(되돌리기 포함).
    #[serde(default)]
    req_state: Option<String>,
    /// 한 줄 메모 — `req_state` 와 함께만.
    #[serde(default)]
    note: Option<String>,
    /// 새 본문(끝난 내 사람 글만, 실행 없음).
    #[serde(default)]
    text: Option<String>,
    /// 편집을 상대 창에도 — `text` 와 함께만(없으면 이 PC 에만).
    #[serde(default)]
    mirror: Option<bool>,
}

/// `POST /api/messages/delete` — 끝난 내 사람 글 여러 개를 흔적 없이 지운다(한 대화의 글만 · 하나라도 안 되면 아무것도 안 지움).
/// `mirror` = 상대 창에도.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct MessageDelete {
    ids: Vec<i64>,
    #[serde(default)]
    mirror: bool,
}

/// `GET /api/conversations/{id}/messages` — `limit` 이면 마지막 N 개만(개요 미리보기, 1 ~ [`MESSAGES_MAX`]).
#[derive(Deserialize)]
pub(super) struct TailQuery {
    #[serde(default)]
    limit: Option<i64>,
}

/// `POST /api/messages/{id}/archive` — 요청 archive · 해제(이 PC 에만).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ArchivePatch {
    archived: bool,
}

pub(super) async fn list_messages(State(chat): State<Arc<Chat>>, Path(id): Path<i64>, Query(q): Query<TailQuery>) -> Response {
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
                    // 보관 — 상대에 아직 닿지 않았거나(`pending` · `inflight`) 포기한(`failed`) 내 글.
                    v["outbox"] = chat.outbox_of(m).map_or(
                        Value::Null,
                        |o| json!({ "state": o.state, "tries": o.tries, "error": o.last_error }),
                    );
                    // 편집 · 삭제할 수 있는 글인가 — 끝난 내 사람 글만.
                    v["editable"] = json!(Chat::tidy_ok(&chat.db(), m).unwrap_or(false));
                    v
                })
                .collect::<Vec<_>>(),
        )
        .into_response(),
        Err(e) => db_fail(e),
    }
}

/// 대화를 가로지른 글 — 최신순. 항목 = 메시지 JSON + `conversation{id, peer_id, agent, dir}`.
pub(super) async fn list_across(State(chat): State<Arc<Chat>>, Query(q): Query<MessageQuery>) -> Response {
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

/// 주고받은 메시지 한 쪽(`side=both`) — 양쪽 글 최신부터 · `before` 커서. 거르기(kind · ask · req · q)는 받지 않는다.
pub(super) fn message_page(chat: &Chat, q: &MessageQuery) -> Response {
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

/// 주고받은 메시지 보기 기간(설정 `ui_peer_weeks`)의 시작 시각.
pub(super) fn peer_since(db: &an_core::db::Connection) -> Result<i64, an_core::db::Error> {
    Ok(chat::unix_now() - an_core::run_opts::peer_weeks(db)? * 7 * 86400)
}

/// 주고받은 메시지 상대별 요약 — `[{peer, count, last}]`(`last` = 마지막 메시지 JSON + 대화).
pub(super) async fn message_peers(State(chat): State<Arc<Chat>>, Query(q): Query<SavedQuery>) -> Response {
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

/// 메시지 저장 · 해제(이 PC 에만) · 받은 요청 상태 · 본문 편집.
pub(super) async fn save_message(
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
        // 네 상태 어디로든(대기 되돌리기 포함).
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

/// 요청 archive · 해제(이 PC 에만, 상대에 보내지 않는다) → 메시지 JSON. archive 는 완료 요청만(409).
pub(super) async fn archive_message(
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

/// 끝난 내 사람 글 여러 개 지우기 — `{deleted, warning?}`.
pub(super) async fn delete_messages(
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
