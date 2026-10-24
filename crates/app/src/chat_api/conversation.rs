//! 대화 API — 목록 · 시작 · 고치기 · 읽음 · 정보 · 자동 왕복 · 컨텍스트 · 관람자 · 나가기 · 지우기 · 비우기.

use super::*;

#[derive(Deserialize)]
pub(super) struct NewConversation {
    peer_id: i64,
    agent: String,
    /// 첫 글 — 비면(웹UI 세션 생성) 글 없이 대화만 연다(`via` · `request` 와 함께는 400).
    #[serde(default)]
    text: String,
    /// `agent` 만(생략 가능) — [`NewMessage::to`] 와 같다(첫 글도 대화 에이전트에게 실행 요청, `human` 은 400).
    #[serde(default)]
    to: Option<String>,
    /// 자동 왕복 — 이 PC 의 내 에이전트. 있으면 두 에이전트 자동 왕복 대화.
    #[serde(default)]
    relay_agent: Option<String>,
    /// 자동 왕복 상한(왕복, 1 ~ 50) — 없으면 설정 기본값(`relay_cap_default`, 설정이 없으면 10).
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
    /// CLI 트랜잭션 키(`via: cli` 일 때만) — 같은 키가 이미 있으면 보내지 않고 그 트랜잭션을 돌려준다.
    #[serde(default)]
    tx_key: Option<String>,
    /// 새 대화를 넣을 레일 그룹(이 PC 에만). 없는 그룹이면 보내지 않고 404.
    #[serde(default)]
    group: Option<i64>,
}

/// `PATCH /api/conversations/{id}` — 이 PC 에서 붙이는 표시만, 준 항목만: 제목(빈 문자열 = 지움 → 첫 사람 글 앞부분) · ★.
/// 상대에 알리지 않는다. 둘 다 없으면 400.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ConversationPatch {
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    starred: Option<bool>,
}

/// `POST /api/conversations/{id}/read` — 화면이 받은 마지막 글 id 까지 읽음.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ReadUpto {
    upto: i64,
}

#[derive(Deserialize)]
pub(super) struct RelaySwitch {
    on: bool,
}

/// 컨텍스트 압축 · 비우기 — `op` = `compact` · `clear`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ContextOp {
    op: String,
}

/// 공유 필드 — 공유 대화(사본) = `share{other, owner_dir, ended}`, 그 밖 = `observers`(내가 넣은 관람자 id) · `watchers`(원 상대가 알려 준 관람자 수).
pub(super) fn share_json(
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

pub(super) async fn list_conversations(State(chat): State<Arc<Chat>>) -> Response {
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

pub(super) async fn start_conversation(
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
    // 같은 키는 확인부터 보내기까지 한 줄로 — 동시에 두 번 불러도 한 번만 보낸다.
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
    let relay_cap = || req.relay_cap.map_or_else(|| default_relay_cap(&chat.db()), Ok);
    // 세션 생성 — 글 없이 대화만(상대에는 첫 글이 갈 때 생긴다). CLI 는 본문 필수.
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
            Some(mine) => match relay_cap() {
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
            let cap = match relay_cap() {
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

pub(super) async fn patch_conversation(
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
pub(super) async fn read_conversation(
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

/// `GET /api/conversations/{id}/info` — 세부정보 '대화 정보': 시작 · 마지막 글 · 압축 · 비우기 수. 받은 대화(내 에이전트가 답함)만 `agent`(실행 도구 · 폴더 —
/// 지웠으면 null) · `cmd`(이 PC 가 띄운 마지막 실행 명령) — 그 밖 대화는 상대 PC 가 정해 null. 명령은 이 응답에만(목록 · 상대 프레임에 싣지 않는다).
pub(super) async fn conversation_info(State(chat): State<Arc<Chat>>, Path(id): Path<i64>) -> Response {
    let db = chat.db();
    let found = chat::by_id(&db, id).and_then(|c| Ok(c.zip(chat::info(&db, id)?)));
    let (conv, info) = match found {
        Ok(Some(x)) => x,
        Ok(None) => return fail(io::Error::new(io::ErrorKind::NotFound, "없는 대화")),
        Err(e) => return db_fail(e),
    };
    let mine = conv.dir == Dir::In;
    let agent = match mine.then(|| agents::by_name(&db, &conv.agent)).transpose() {
        Ok(a) => a.flatten(),
        Err(e) => return db_fail(e),
    };
    Json(json!({
        "created_at": info.created_at, "last_at": info.last_at,
        "compacts": info.compacts, "last_compact_at": info.last_compact_at, "clears": info.clears,
        "agent": agent.map(|a| json!({ "engine": a.engine, "folder": a.folder })),
        "cmd": info.cmd.filter(|_| mine).map(|t| json!({ "text": t, "at": info.cmd_at })),
    }))
    .into_response()
}

/// 자동 왕복 멈춤 · 재개 — 재개는 대화를 연 쪽만.
pub(super) async fn switch_relay(
    State(chat): State<Arc<Chat>>,
    Path(id): Path<i64>,
    Json(req): Json<RelaySwitch>,
) -> Response {
    match chat.relay(id, req.on).await {
        Ok(c) => Json(conversation_json(&c)).into_response(),
        Err(e) => fail(e),
    }
}

/// 대화 컨텍스트 압축 · 비우기 — 구분선 글을 돌려준다. 틀린 `op` 400 · 없는 대화 404 · 진행 중 409.
pub(super) async fn context_op(
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
pub(super) struct NewObserver {
    peer_id: i64,
}

/// 관람자 넣기 — 204. 사본 · 원 상대 · 5명 넘음 400, 없는 대화 · 상대 404, 이미 관람자 · 상대 앱이 이 기능을 모름 409.
pub(super) async fn add_observer(
    State(chat): State<Arc<Chat>>,
    Path(id): Path<i64>,
    Json(req): Json<NewObserver>,
) -> Response {
    match chat.share_add(id, req.peer_id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => fail(e),
    }
}

/// 관람자 빼기 — 204, 관람자가 아니면 404.
pub(super) async fn remove_observer(
    State(chat): State<Arc<Chat>>,
    Path((id, peer)): Path<(i64, i64)>,
) -> Response {
    match chat.share_remove(id, peer) {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => fail(e),
    }
}

/// 대화에서 나가기(모든 대화, 공유 사본은 공유 나가기) — 204. 없는 대화 404 · 도는 실행이 있으면 409.
pub(super) async fn leave_conversation(State(chat): State<Arc<Chat>>, Path(id): Path<i64>) -> Response {
    match chat.leave_conversation(id) {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => fail(e),
    }
}

/// 대화 삭제 — 나가기 + 이 PC 의 대화 · 글 · 딸린 항목. 204. 없는 대화 404 · 도는 실행이 있으면 409.
pub(super) async fn delete_conversation(State(chat): State<Arc<Chat>>, Path(id): Path<i64>) -> Response {
    match chat.delete_conversation(id) {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => fail(e),
    }
}

#[derive(Deserialize)]
pub(super) struct ClearReq {
    /// 참 = 모든 사용자 글 삭제(상대 PC 도 비운다), 거짓 = 내 글만 삭제(이 PC).
    #[serde(default)]
    all: bool,
}

/// 기록 삭제 — 204. 공유 사본 · 닫힌 대화에 `all` 400, 실행 중 · 상대 앱이 모름 409, 없는 대화 404.
pub(super) async fn clear_conversation(
    State(chat): State<Arc<Chat>>,
    Path(id): Path<i64>,
    Json(req): Json<ClearReq>,
) -> Response {
    match chat.clear_conversation(id, req.all).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => fail(e),
    }
}
