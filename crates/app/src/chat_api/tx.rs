//! CLI 이어 묻기 · 트랜잭션 API.

use super::*;

#[derive(Deserialize)]
pub(super) struct NewMessage {
    text: String,
    /// `agent` 만(생략 가능) — 보낸 대화의 새 글은 대화 에이전트에게 실행 요청이다(`human` 은 400). 받은 대화의 글은 실행 없이 양쪽 창에만.
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

/// 대화에 이어 쓴다 — 보낸 대화는 상대 에이전트에게 실행 요청, 받은 대화(내 에이전트가 답함)는 실행 없이 양쪽 창에만(내 에이전트를 거치지 않는다).
pub(super) async fn continue_conversation(
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
    let to_agent = match chat::by_id(&chat.db(), id) {
        Ok(c) => c.is_none_or(|c| c.dir != Dir::In),
        Err(e) => return db_fail(e),
    };
    match chat.post_with(id, &req.text, to_agent, opts).await {
        Ok((c, m, warning)) => sent_tx(&chat, c, m, warning, via),
        Err(e) => fail(e),
    }
}

/// 트랜잭션 키 — CLI 글(`via: cli`)에만, 규칙은 [`an_core::cli_tx::key_ok`].
pub(super) fn tx_key_of<'a>(
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
pub(super) fn tx_again(chat: &Chat, key: Option<&str>) -> Option<Response> {
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

pub(super) fn chat_conv(chat: &Chat, view: &TxView) -> Value {
    chat::by_id(&chat.db(), view.message.conversation_id)
        .ok()
        .flatten()
        .map_or(Value::Null, |c| conversation_json(&c))
}

/// 보냄(201) — 보냈지만 덧붙은 일이 어긋났으면 `warning`, CLI 글이면 트랜잭션(`tx`).
pub(super) fn sent_tx(
    chat: &Chat,
    conv: Conversation,
    msg: Message,
    warning: Option<String>,
    via: Option<&str>,
) -> Response {
    // CLI 글의 트랜잭션은 글과 같은 DB 트랜잭션에서 만들었다 — 없으면 그 사실을 감추지 않는다.
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

/// 트랜잭션 JSON — `{tx, key, state, conversation, message, reply, progress, outbox, stalled}`.
pub(super) fn tx_json(conv: &Value, view: &TxView) -> Value {
    json!({
        "tx": view.tx.id, "key": view.tx.key, "state": view.state, "conversation": conv,
        "message": message_json(&view.message),
        "reply": view.reply.as_ref().map_or(Value::Null, message_json),
        "progress": view.progress.as_ref().map_or(Value::Null, |p| json!({ "state": p.state, "ahead": p.ahead, "detail": p.detail })),
        "outbox": view.outbox.as_ref().map_or(Value::Null, |o| json!({ "state": o.state, "tries": o.tries, "error": o.last_error })),
        "stalled": view.tx.stalled_at.is_some(),
    })
}

pub(super) async fn get_tx(State(chat): State<Arc<Chat>>, Path(id): Path<i64>) -> Response {
    match chat.tx_view(id) {
        Ok(Some(view)) => Json(tx_json(&chat_conv(&chat, &view), &view)).into_response(),
        Ok(None) => fail(io::Error::new(io::ErrorKind::NotFound, "없는 트랜잭션")),
        Err(e) => fail(e),
    }
}

/// 이어 받기 — 멈춤을 풀고 지금 상태.
pub(super) async fn resume_tx(State(chat): State<Arc<Chat>>, Path(id): Path<i64>) -> Response {
    if let Err(e) = chat.tx_resume(id) {
        return fail(e);
    }
    get_tx(State(chat), Path(id)).await
}
