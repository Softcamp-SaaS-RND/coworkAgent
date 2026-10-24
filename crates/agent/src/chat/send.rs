//! 보내기 — 질의 · 요청 · 세션 열기.

use super::*;

/// 받은 쪽 실행 요청 사람 글의 미러(내 실행과 무관).
const MIRROR: &str = "mirror";
/// 멘션을 받지 않는 자리(새 대화 · 받은 대화 · 자동 왕복 대화 · 빈 이름)의 오류 문구.
const MENTION_ONLY: &str =
    "멘션은 내가 연 대화(자동 왕복 아님)에 이어 쓸 때만 — 상대 PC 에이전트 이름";
/// 요청은 실행이 없다 — 부를 에이전트(멘션)도, 답을 기다리는 CLI 도 없다.
const REQUEST_NO_MENTION: &str = "요청은 멘션 없이 — 실행 없이 상대 요청 목록에 남는다";
const REQUEST_NO_CLI: &str = "CLI 는 질의만 — 요청은 답이 없다";
/// 요청 규칙 — 멘션(대화 에이전트가 아닌 이름) · CLI 출처와 함께 쓰지 않는다.
fn request_ok(opts: &Compose<'_>, conv_agent: &str) -> io::Result<()> {
    if !opts.request {
        return Ok(());
    }
    if opts.via.is_some() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, REQUEST_NO_CLI));
    }
    if opts.mention.is_some_and(|m| m != conv_agent) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            REQUEST_NO_MENTION,
        ));
    }
    Ok(())
}

impl Chat {
    pub(super) fn paired_peer(&self, peer_id: i64) -> io::Result<Peer> {
        let peer = self
            .node
            .peers()?
            .into_iter()
            .find(|p| p.id == peer_id)
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::NotFound, format!("피어 {peer_id} 없음"))
            })?;
        if !peer.paired() {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "페어링되지 않은 상대",
            ));
        }
        Ok(peer)
    }

    pub async fn remote_agents(&self, peer_id: i64) -> io::Result<Vec<String>> {
        let peer = self.paired_peer(peer_id)?;
        match self
            .node
            .send(addr(&peer)?, Some(peer.spki.clone()), &Request::Agents)
            .await?
        {
            (Response::Agents { names }, _) => Ok(names),
            (other, _) => Err(refused(other)),
        }
    }

    /// 사람 → 원격 에이전트. `conversation` 이 있으면 그 대화에 이어 쓴다. 상대가 받아 대기열에 넣으면 끝(답은 나중에 `reply`).
    pub async fn send(
        &self,
        peer_id: i64,
        agent: &str,
        conversation: Option<i64>,
        text: &str,
    ) -> io::Result<(Conversation, Message)> {
        self.send_via(peer_id, agent, conversation, text, None, None)
            .await
    }

    /// [`send`](Self::send) + 글의 출처(`via` — `cli` 면 사용자 AI 에이전트가 CLI 로 쓴 글, 양쪽 이력에 남는다) · 새 대화의 제목
    /// (`title` — 이 PC 에만, 이어 쓰기면 쓰지 않는다).
    pub async fn send_via(
        &self,
        peer_id: i64,
        agent: &str,
        conversation: Option<i64>,
        text: &str,
        via: Option<&str>,
        title: Option<&str>,
    ) -> io::Result<(Conversation, Message)> {
        let opts = Compose {
            via,
            title,
            ..Compose::default()
        };
        self.send_with(peer_id, agent, conversation, text, opts)
            .await
    }

    /// [`send`](Self::send) + 새 글의 선택([`Compose`]).
    pub async fn send_with(
        &self,
        peer_id: i64,
        agent: &str,
        conversation: Option<i64>,
        text: &str,
        opts: Compose<'_>,
    ) -> io::Result<(Conversation, Message)> {
        if text.trim().is_empty() || text.len() > MAX_TEXT {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("본문은 1 ~ {MAX_TEXT} 바이트"),
            ));
        }
        request_ok(&opts, agent)?;
        // 새 대화의 에이전트는 `agent` — 멘션은 이어 쓰기에서만(대화를 만들기 전에 거른다).
        if conversation.is_none() && opts.mention.is_some_and(|m| m != agent) {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, MENTION_ONLY));
        }
        let peer = self.paired_peer(peer_id)?;
        let conv = {
            let db = self.db();
            match conversation {
                Some(id) => {
                    let c = chat::by_id(&db, id).map_err(db_err)?;
                    if let Some(c) = &c {
                        Self::writable(c)?;
                    }
                    c.filter(|c| c.dir == Dir::Out && c.peer_id == peer_id)
                        .ok_or_else(|| {
                            io::Error::new(io::ErrorKind::NotFound, format!("대화 {id} 없음"))
                        })?
                }
                None => {
                    // 대화와 제목은 한 트랜잭션, 보내기 전에 — 전송이 실패해도 대화는 남으므로 제목도 남는다.
                    let tx = an_core::db::write_tx(&db).map_err(db_err)?;
                    let c = chat::open_conversation(&tx, &new_uuid(), peer_id, agent, Dir::Out)
                        .map_err(db_err)?;
                    chat::set_title(&tx, c.id, opts.title).map_err(db_err)?;
                    // 그룹이 그 사이 지워졌으면 대화도 만들지 않고 보내지 않는다(트랜잭션을 버린다 — 되돌림).
                    if let Some(g) = opts.group
                        && !an_core::groups::join(&tx, g, c.id).map_err(db_err)?
                    {
                        return Err(no_group());
                    }
                    tx.commit().map_err(db_err)?;
                    c
                }
            }
        };
        // 멘션 — 자동 왕복 대화는 아니고(상대 에이전트끼리의 차례에 끼지 않는다), 대화 에이전트 이름이면 멘션이 아니다.
        let mention = opts.mention.filter(|m| *m != conv.agent);
        if mention.is_some_and(|m| conv.relay.is_some() || m.trim().is_empty()) {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, MENTION_ONLY));
        }
        let opts = Compose { mention, ..opts };
        // 세션 생성으로 연 자동 왕복 대화의 첫 글 — 자동 왕복을 연다(`start_relay_with` 와 같은 길).
        if conversation.is_some()
            && let Some(relay) = conv.relay.clone()
            && chat::empty(&self.db(), conv.id).map_err(db_err)?
        {
            if opts.request {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    REQUEST_NO_RELAY,
                ));
            }
            let opts = Compose {
                via: None,
                mention: None,
                ..opts
            };
            return self
                .send_human(conv, &peer, text, Some(relay.agent), opts)
                .await;
        }
        // 사람이 쓰면 자동 왕복은 멈춘다(사람 입력 대기).
        if conversation.is_some() {
            let _ = chat::relay_on(&self.db(), conv.id, false, false);
        }
        self.send_human(conv, &peer, text, None, opts).await
    }

    /// 세션 생성 — 글 없이 이 PC 에 대화만 연다(상대에는 첫 글이 갈 때 생긴다). `relay` = (내 에이전트, 상한) — 있으면 첫 글부터 자동 왕복.
    /// 제목 · 그룹은 대화와 한 트랜잭션.
    pub fn open_session(
        &self,
        peer_id: i64,
        agent: &str,
        relay: Option<(&str, i64)>,
        opts: Compose<'_>,
    ) -> io::Result<Conversation> {
        if agent.trim().is_empty() {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "상대 에이전트를 고른다"));
        }
        if let Some((_, cap)) = relay
            && !(1..=MAX_RELAY_CAP).contains(&cap)
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("자동 왕복 상한은 1 ~ {MAX_RELAY_CAP}"),
            ));
        }
        self.paired_peer(peer_id)?;
        let db = self.db();
        if let Some((mine, _)) = relay
            && agents::by_name(&db, mine).map_err(db_err)?.is_none()
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("내 에이전트 {mine} 가 없다"),
            ));
        }
        let tx = an_core::db::write_tx(&db).map_err(db_err)?;
        let c = chat::open_conversation(&tx, &new_uuid(), peer_id, agent, Dir::Out).map_err(db_err)?;
        if let Some((mine, cap)) = relay {
            chat::set_relay(&tx, c.id, mine, cap).map_err(db_err)?;
        }
        chat::set_title(&tx, c.id, opts.title).map_err(db_err)?;
        if let Some(g) = opts.group
            && !an_core::groups::join(&tx, g, c.id).map_err(db_err)?
        {
            return Err(no_group());
        }
        let c = chat::by_id(&tx, c.id).map_err(db_err)?.unwrap_or(c);
        tx.commit().map_err(db_err)?;
        Ok(c)
    }

    /// 사람 글을 상대 에이전트에게 실행 요청으로. `relay` = 자동 왕복을 여는 첫 요청(내 에이전트 이름).
    pub(super) async fn send_human(
        &self,
        conv: Conversation,
        peer: &Peer,
        text: &str,
        relay: Option<String>,
        opts: Compose<'_>,
    ) -> io::Result<(Conversation, Message)> {
        let via = opts.via;
        // 저장부터 상대 응답까지 대화 단위로 한 번에 하나.
        let lock = self.turn(conv.id);
        let _turn = lock.lock().await;
        let msg = {
            let db = self.db();
            // 글과 출처는 한 트랜잭션 — 출처 없이 글만 남지 않게.
            let tx = an_core::db::write_tx(&db).map_err(db_err)?;
            let mut msg =
                chat::add_message(&tx, conv.id, &new_uuid(), Author::Human, text, State::Sent)
                    .map_err(db_err)?;
            if via.is_some() {
                chat::set_via(&tx, msg.id, via).map_err(db_err)?;
                msg.via = via.map(str::to_string);
            }
            chat::set_request(&tx, msg.id, opts.request).map_err(db_err)?;
            msg.request = opts.request;
            if opts.request {
                // 요청 — 상대 에이전트를 실행하지 않는다: 답을 기다리지 않는 사람 글(닿으면 `done`).
                chat::set_flags(&tx, msg.id, true, false, State::Sent).map_err(db_err)?;
                msg.exec = false;
            }
            if opts.mention.is_some() {
                chat::set_agent(&tx, msg.id, opts.mention).map_err(db_err)?;
                msg.agent = opts.mention.map(str::to_string);
            }
            if via.is_some() {
                cli_tx::create(&tx, opts.tx_key, msg.id).map_err(db_err)?;
            }
            let req = Request::Message {
                conv: conv.uuid.clone(),
                msg: msg.uuid.clone(),
                agent: conv.agent.clone(),
                text: text.to_string(),
                lc: Some(msg.lc),
                by_agent: None,
                relay: relay.clone(),
                via: msg.via.clone(),
                request: msg.request,
                mention: msg.agent.clone(),
            };
            let kind = match (relay.is_some(), msg.request) {
                (true, _) => RELAY_OPEN,
                (false, true) => NOTE,
                (false, false) => MESSAGE,
            };
            // 보내기 행도 같은 트랜잭션 — 글만 남고 보낼 행이 없는 채로 꺼지지 않게.
            let staged = Self::stage(&tx, &conv, Some(msg.id), kind, &req)?;
            tx.commit().map_err(db_err)?;
            (msg, req, staged)
        };
        let (msg, req, staged) = msg;
        let starts_relay = relay.is_some();
        let _ = peer;
        match self.dispatch(&conv, staged, &req).await? {
            // 상대가 받았거나, 닿지 않아 이 PC 에 보관했다(줄이 다시 보낸다, 같은 uuid 라 상대에서 한 번만 실행).
            Shipped::Answered(Response::Ok) if msg.request => {
                let _ = chat::set_state(&self.db(), msg.id, State::Done);
                let msg = chat::message_by_id(&self.db(), msg.id)
                    .map_err(db_err)?
                    .unwrap_or(msg);
                Ok((conv, msg))
            }
            Shipped::Answered(Response::Ok) | Shipped::Kept => Ok((conv, msg)),
            // 명시적 거절만 실패로 확정한다. 자동 왕복을 여는 글이면 켜짐도 거둔다(상대에 그 대화가 없다 — 재개가 갈 곳이 없다).
            Shipped::Answered(other) => {
                self.refused_by_peer(msg.id, conv.peer_id, &other);
                if starts_relay {
                    let _ = chat::relay_on(&self.db(), conv.id, false, false);
                }
                Err(refused(other))
            }
        }
    }

    /// 대화에 사람 글 — `to_agent` 면 대화 에이전트에게 실행 요청, 아니면 실행 없이 양쪽 이력에만.
    /// 보낸 대화(`out`)의 실행 요청은 [`send`](Self::send), 그 밖은 내 이력에 남기고 상대에 `post` 로 미러한다.
    /// 세 번째 값은 경고 — 내 에이전트 실행은 받아들였지만 상대 이력 미러가 실패한 경우(실패로 돌려주면 다시 보내 두 번 실행된다).
    pub async fn post(
        self: &Arc<Self>,
        conversation_id: i64,
        text: &str,
        to_agent: bool,
    ) -> io::Result<(Conversation, Message, Option<String>)> {
        self.post_via(conversation_id, text, to_agent, None).await
    }

    /// [`post`](Self::post) + 글의 출처(`via`).
    pub async fn post_via(
        self: &Arc<Self>,
        conversation_id: i64,
        text: &str,
        to_agent: bool,
        via: Option<&str>,
    ) -> io::Result<(Conversation, Message, Option<String>)> {
        let opts = Compose {
            via,
            ..Compose::default()
        };
        self.post_with(conversation_id, text, to_agent, opts).await
    }

    /// [`post`](Self::post) + 새 글의 선택([`Compose`] — 제목은 쓰지 않는다).
    pub async fn post_with(
        self: &Arc<Self>,
        conversation_id: i64,
        text: &str,
        to_agent: bool,
        opts: Compose<'_>,
    ) -> io::Result<(Conversation, Message, Option<String>)> {
        let via = opts.via;
        if text.trim().is_empty() || text.len() > MAX_TEXT {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("본문은 1 ~ {MAX_TEXT} 바이트"),
            ));
        }
        let conv = chat::by_id(&self.db(), conversation_id)
            .map_err(db_err)?
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("대화 {conversation_id} 없음"),
                )
            })?;
        Self::writable(&conv)?;
        request_ok(&opts, &conv.agent)?;
        // 멘션은 보낸 대화의 실행 요청에만 — 받은 대화의 에이전트는 내 것이다.
        if opts.mention.is_some_and(|m| m != conv.agent) && !(conv.dir == Dir::Out && to_agent) {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, MENTION_ONLY));
        }
        // 요청 — 보낸 대화는 `Message`(상대에 대화가 아직 없을 수 있다), 받은 대화는 실행 없는 미러. 어느 쪽도 에이전트를 실행하지 않는다.
        let to_agent = to_agent && !opts.request;
        if conv.dir == Dir::Out && (to_agent || opts.request) {
            let opts = Compose {
                title: None,
                ..opts
            };
            let (c, m) = self
                .send_with(conv.peer_id, &conv.agent, Some(conv.id), text, opts)
                .await?;
            return Ok((c, m, None));
        }
        let peer = self.paired_peer(conv.peer_id)?;
        // 사람이 쓰면 자동 왕복은 멈춘다 — 연 쪽은 끄고, 받은 쪽은 에이전트발 요청을 거절하고 연 쪽에 알린다.
        if conv.dir == Dir::Out {
            let _ = chat::relay_on(&self.db(), conv.id, false, false);
        } else {
            // 거절은 바로, 연 쪽 알림은 기다리지 않는다(상대가 꺼져 있어도 글쓰기가 늦어지지 않게).
            let _ = chat::set_blocked(&self.db(), conv.id, true);
            let (me, c, p) = (self.clone(), conv.clone(), peer.clone());
            tokio::spawn(async move { me.block_relay(&c, &p).await });
        }
        let lock = self.turn(conv.id);
        let _turn = lock.lock().await;
        // 실행 요청도 먼저 `sent` 로 — 상대 이력에 사람 글이 먼저 닿은 뒤에 실행해야 답 미러가 붙는다.
        let msg = {
            let db = self.db();
            // 글과 출처는 한 트랜잭션 — 출처 없이 글만 남지 않게.
            let tx = an_core::db::write_tx(&db).map_err(db_err)?;
            let mut msg =
                chat::add_human(&tx, conv.id, &new_uuid(), text, State::Sent, true, to_agent)
                    .map_err(db_err)?;
            if via.is_some() {
                chat::set_via(&tx, msg.id, via).map_err(db_err)?;
                msg.via = via.map(str::to_string);
            }
            chat::set_request(&tx, msg.id, opts.request).map_err(db_err)?;
            msg.request = opts.request;
            if via.is_some() {
                cli_tx::create(&tx, opts.tx_key, msg.id).map_err(db_err)?;
            }
            let req = Request::Post {
                conv: conv.uuid.clone(),
                msg: msg.uuid.clone(),
                author: "human".into(),
                text: text.to_string(),
                exec: to_agent,
                reply_to: None,
                ok: None,
                turns: None,
                duration_ms: None,
                lc: Some(msg.lc),
                via: msg.via.clone(),
                model: None,
                effort: None,
                request: msg.request,
                ctx: None,
            };
            let kind = if to_agent { MIRROR } else { NOTE };
            // 보내기 행도 같은 트랜잭션.
            let staged = Self::stage(&tx, &conv, Some(msg.id), kind, &req)?;
            tx.commit().map_err(db_err)?;
            (msg, req, staged)
        };
        let (msg, req, staged) = msg;
        let sent = self.dispatch(&conv, staged, &req).await;
        if to_agent {
            // 내 에이전트는 미러 결과와 무관하게 실행한다.
            let _ = chat::set_state(&self.db(), msg.id, State::Queued);
            self.wake(conv.id);
        }
        let msg = chat::message_by_uuid(&self.db(), &msg.uuid)
            .map_err(db_err)?
            .unwrap_or(msg);
        let mirror_failed = match sent {
            Ok(Shipped::Answered(Response::Ok)) => {
                if !to_agent {
                    let _ = chat::set_state(&self.db(), msg.id, State::Done);
                }
                let msg = chat::message_by_id(&self.db(), msg.id)
                    .map_err(db_err)?
                    .unwrap_or(msg);
                return Ok((conv, msg, None));
            }
            // 닿지 않아 보관 — 줄이 다시 보낸다. 실행 없는 글은 상대에 닿으면 `done`.
            Ok(Shipped::Kept) => return Ok((conv, msg, None)),
            Ok(Shipped::Answered(other)) => {
                if !to_agent {
                    let _ = chat::set_state(&self.db(), msg.id, State::Error);
                    return Err(refused(other));
                }
                refused(other).to_string()
            }
            Err(e) => {
                if !to_agent {
                    return Err(e);
                }
                e.to_string()
            }
        };
        log::warn!("대화 {} 미러 실패: {mirror_failed}", conv.uuid);
        Ok((
            conv,
            msg,
            Some(format!(
                "내 에이전트는 실행한다 — 상대 이력에는 남기지 못했다({mirror_failed}). 다시 보내지 마세요"
            )),
        ))
    }
}
