//! 받기 — 상대 프레임 분기 · 받은 글 · 답 · 진행.

use super::*;

/// 받은 창 사용량의 상한(원격 값 검증).
const MAX_CTX_TOKENS: i64 = 100_000_000;

/// 받은 답의 model · effort — `run_opts` 글자 규칙에 맞는 것만 남긴다(원격 값은 신뢰하지 않는 입력).
fn run_meta(model: Option<String>, effort: Option<String>) -> (Option<String>, Option<String>) {
    (
        model.filter(|m| an_core::run_opts::valid_model(m)),
        effort.filter(|e| an_core::run_opts::valid_effort(e)),
    )
}

/// 받은 `message` 의 나머지 — 순서(`lc`) · 자동 왕복(`by_agent` · `relay`) · 출처(`via`) · 종류(`request`) · 멘션(`mention`).
type MessageExtra = (
    Option<i64>,
    Option<String>,
    Option<String>,
    Option<String>,
    bool,
    Option<String>,
);

impl Chat {
    /// 페어링된 상대의 요청 — 막히지 않는다(실행은 대기열로).
    pub(super) fn handle(self: &Arc<Self>, peer: &Peer, req: Request) -> Response {
        self.peer_back(peer.id);
        // 닫힌 대화 · 지운 대화(내가 · 상대가 나감, 묘비) — 그 대화로 온 요청은 거절(나가기 · 공유 끝만 받는다). 공유 사본은 사본 규칙(`live_copy`)
        // 그대로. 대화를 새로 여는 받기(`inbound_message` · `inbound_ask`)는 잠금 안에서 다시 본다(이 확인 뒤 삭제가 끼어들 수 있다).
        if let Some(uuid) = req.conv()
            && !matches!(req, Request::Leave { .. } | Request::ShareEnd { .. })
        {
            let left = chat::left(&self.db(), uuid, peer.id);
            match left {
                Ok(true) => return Response::error(chat::LEFT),
                Ok(false) => {}
                Err(e) => {
                    log::warn!("나간 대화 확인 실패: {e}");
                    return Response::error(RETRY);
                }
            }
        }
        // 글을 남기는 요청 — 끝나면 그 대화의 관람자에게.
        let fan = match &req {
            Request::Message { conv, .. }
            | Request::Reply { conv, .. }
            | Request::Post { conv, .. } => Some(conv.clone()),
            _ => None,
        };
        // 대화가 있는 요청 — 처리 뒤 웹UI 변경 신호.
        let changed = req.conv().map(str::to_owned);
        let result = match req {
            Request::Agents => self.agents(),
            Request::Message {
                conv,
                msg,
                agent,
                text,
                lc,
                by_agent,
                relay,
                via,
                request,
                mention,
            } => self.inbound_message(
                peer,
                conv,
                msg,
                agent,
                text,
                (lc, by_agent, relay, via, request, mention),
            ),
            Request::Relay { conv, on } => self.inbound_relay(peer, conv, on),
            Request::ReqState {
                conv,
                msg,
                state,
                note,
            } => self.inbound_req_state(peer, conv, msg, state, note),
            Request::Progress {
                conv,
                msg,
                state,
                ahead,
                detail,
            } => self.inbound_progress(peer, conv, msg, state, (ahead, detail)),
            Request::Status { conv, msg } => self.inbound_status(peer, conv, msg),
            Request::Edit { conv, msg, text } => self.inbound_edit(peer, conv, msg, text),
            Request::Delete { conv, msg } => self.inbound_delete(peer, conv, msg),
            Request::Context {
                conv,
                msg,
                op,
                text,
            } => self.inbound_context(peer, conv, msg, op, text),
            Request::Share {
                conv,
                title,
                agent,
                owner_dir,
                other,
            } => self.inbound_share(peer, conv, title, agent, (owner_dir, other)),
            Request::SharePost { conv, msg } => self.inbound_share_post(peer, conv, msg),
            Request::ShareDelete { conv, msg } => self.inbound_share_delete(peer, conv, msg),
            Request::ShareEnd { conv } => self.inbound_share_end(peer, conv),
            Request::ShareNotice { conv, n } => self.inbound_share_notice(peer, conv, n),
            Request::Leave { conv } => self.inbound_leave(peer, conv),
            Request::Clear { conv, id } => self.inbound_clear(peer, conv, id),
            reply @ Request::Reply { .. } => self.inbound_reply(peer, reply),
            post @ Request::Post { .. } => self.inbound_post(peer, post),
            _ => Ok(Response::error("bad_request")),
        };
        if let Some(conv) = fan {
            let found = chat::find_own(&self.db(), &conv, peer.id);
            if let Ok(Some(c)) = found {
                self.fan_share(c.id);
            }
        }
        // 결과와 무관하게 보낸다 — 다시 읽기는 해가 없다. 공유 사본도 찾는다.
        if let Some(uuid) = changed {
            let conv = chat::find(&self.db(), &uuid, peer.id)
                .ok()
                .flatten()
                .map(|c| c.id);
            self.node.emit(PeerEvent::Changed { conv });
        }
        // 처리 중 오류(DB 잠금 등)는 거절이 아니다 — 보낸 쪽이 다시 보내게 `retry`.
        result.unwrap_or_else(|e| {
            log::warn!("대화 요청 처리 실패: {e}");
            Response::error(RETRY)
        })
    }

    fn agents(&self) -> io::Result<Response> {
        let names = agents::list(&self.db())
            .map_err(db_err)?
            .into_iter()
            .map(|a| a.name)
            .collect();
        Ok(Response::Agents { names })
    }

    fn inbound_message(
        self: &Arc<Self>,
        peer: &Peer,
        conv: String,
        msg: String,
        agent: String,
        text: String,
        (lc, by_agent, relay, via, request, mention): MessageExtra,
    ) -> io::Result<Response> {
        if text.trim().is_empty() || text.len() > MAX_TEXT || conv.is_empty() || msg.is_empty() {
            return Ok(Response::error("bad_request"));
        }
        // 요청 — 실행이 없어 집중과 무관하게 받는다. 자동 왕복 · 멘션과 함께 오지 않는다.
        if request {
            if relay.is_some()
                || by_agent.is_some()
                || mention.as_ref().is_some_and(|m| *m != agent)
            {
                return Ok(Response::error("bad_request"));
            }
            return self.inbound_ask(peer, &conv, &msg, agent, &text, (lc, via));
        }
        // 집중(이 PC) — 새 실행 요청(자동 왕복 차례 포함)은 실행 · 기록 · 알림 없이 거부. 판정을 먼저 받는다(조건식의 DB 잠금은 블록 끝까지
        // 산다 — edition 2024).
        // 이미 받은 글의 재전송(같은 uuid)은 새 요청이 아니다 — 아래 기록 단계가 `Ok`(한 번만 실행). 자동 왕복 재개의 승격은 새 실행이라 거부.
        let focused = self.focused(&self.db());
        if focused && !self.resent(peer.id, &conv, &msg, by_agent.is_some())? {
            return Ok(Response::error(FOCUS));
        }
        // 자동 왕복을 여는 사람 요청은 "시작" 하나로, 그 뒤 에이전트발 요청은 중간 글(알림 묶기).
        let kind = match (relay.is_some(), by_agent.is_some()) {
            (_, true) => MessageKind::RelayStep,
            (true, false) => MessageKind::RelayStart,
            (false, false) => MessageKind::Request,
        };
        // 멘션 — 대화 에이전트가 아닌 내 에이전트. 자동 왕복 글에는 받지 않는다(대화를 만들기 전에 거른다).
        let mention = mention.filter(|m| *m != agent);
        if mention.is_some() && (relay.is_some() || by_agent.is_some()) {
            return Ok(Response::error("bad_request"));
        }
        let conversation_id = {
            let db = self.db();
            for name in std::iter::once(&agent).chain(mention.as_ref()) {
                if agents::by_name(&db, name).map_err(db_err)?.is_none() {
                    return Ok(Response::error("unknown_agent"));
                }
            }
            // 받기 앞 확인 뒤 나가기 · 삭제가 끼었을 수 있다 — 나가기 · 삭제와 같은 잠금 안에서 다시.
            if chat::left(&db, &conv, peer.id).map_err(db_err)? {
                return Ok(Response::error(chat::LEFT));
            }
            let Ok(c) = chat::open_conversation(&db, &conv, peer.id, &agent, Dir::In) else {
                return Ok(Response::error("bad_request")); // 다른 상대의 대화 uuid
            };
            // 자동 왕복 대화에는 멘션을 받지 않는다.
            if c.dir != Dir::In || c.agent != agent || (mention.is_some() && c.relay.is_some()) {
                return Ok(Response::error("bad_request"));
            }
            // 자동 왕복 대화로 기록(받은 쪽) — 첫 실행 실패의 끝 알림 · 상대 에이전트 이름 표시에 쓴다.
            if let Some(name) = relay.as_deref().or(by_agent.as_deref()) {
                chat::mark_relayed(&db, c.id, &an_net::proto::clean_name(name)).map_err(db_err)?;
            }
            if by_agent.is_some() && c.relay_blocked {
                return Ok(Response::error(STOPPED));
            }
            // 기록 삭제로 지운 글이 늦게 다시 왔다 — 되살리거나 다시 실행하지 않는다.
            if chat::gone(&db, &msg).map_err(db_err)? {
                return Ok(Response::Ok);
            }
            if let Some(old) = chat::message_by_uuid(&db, &msg).map_err(db_err)? {
                // 자동 왕복 재개 — 상한에서 기록만 받은 상대 에이전트 글을 같은 uuid 로 실행 요청으로. 그 밖은 재전송(한 번만 실행).
                let promote = by_agent.is_some()
                    && old.conversation_id == c.id
                    && old.author == Author::Agent
                    && !old.mine
                    && !old.exec;
                if !promote {
                    return Ok(Response::Ok);
                }
                chat::set_flags(&db, old.id, false, true, State::Queued).map_err(db_err)?;
            } else {
                // 저장이 곧 대기열 등록 — 실행 순서는 id 순(저장 순)이다. 글 · 출처 · 순서는 한 트랜잭션.
                let tx = an_core::db::write_tx(&db).map_err(db_err)?;
                let m = if by_agent.is_some() {
                    chat::add_agent_request(&tx, c.id, &msg, &text, State::Queued)
                        .map_err(db_err)?
                } else {
                    let m = chat::add_message(&tx, c.id, &msg, Author::Human, &text, State::Queued)
                        .map_err(db_err)?;
                    chat::set_via(&tx, m.id, via.as_deref()).map_err(db_err)?;
                    chat::set_request(&tx, m.id, request).map_err(db_err)?;
                    if mention.is_some() {
                        chat::set_agent(&tx, m.id, mention.as_deref()).map_err(db_err)?;
                    }
                    m
                };
                chat::observe(&tx, m.id, lc).map_err(db_err)?;
                tx.commit().map_err(db_err)?;
            }
            c.id
        };
        self.wake(conversation_id);
        self.tell_waiting(conversation_id, &msg);
        // 알림의 에이전트 = 실행할 에이전트(멘션이면 부른 이름).
        self.notify(
            conversation_id,
            peer,
            mention.unwrap_or(agent),
            kind,
            preview(&text),
        );
        Ok(Response::Ok)
    }

    /// 상대 사람 글 · 상대 에이전트 답의 미러 — 그 상대 소유 대화에만. 에이전트 답 · 실행 요청 글은 에이전트가 상대 쪽에 있는
    /// 대화(내 `out`)에만 받는다.
    fn inbound_post(&self, peer: &Peer, post: Request) -> io::Result<Response> {
        let Request::Post {
            conv,
            msg,
            author,
            text,
            exec,
            reply_to,
            ok,
            turns,
            duration_ms,
            lc,
            via,
            model,
            effort,
            request,
            ctx,
        } = post
        else {
            return Ok(Response::error("bad_request"));
        };
        let ctx = ctx.filter(|v| (0..=MAX_CTX_TOKENS).contains(v));
        let (model, effort) = run_meta(model, effort);
        // 에이전트 답은 비어 있으면 받지 않는다 — 조용한 빈 성공 · 자동 왕복의 빈 입력.
        if text.len() > MAX_TEXT || msg.is_empty() || (author == "agent" && text.trim().is_empty())
        {
            return Ok(Response::error("bad_request"));
        }
        let mut kind = MessageKind::Request;
        let (cid, agent) = {
            let db = self.db();
            let Some(c) = chat::find_open(&db, &conv, peer.id).map_err(db_err)? else {
                return Ok(Response::error("unknown"));
            };
            if chat::message_by_uuid(&db, &msg).map_err(db_err)?.is_some()
                || chat::gone(&db, &msg).map_err(db_err)?
            {
                return Ok(Response::Ok); // 같은 글 재전송 · 기록 삭제로 지운 글
            }
            match author.as_str() {
                "human" => {
                    if text.trim().is_empty() || (exec && (c.dir != Dir::Out || request)) {
                        return Ok(Response::error("bad_request"));
                    }
                    let tx = an_core::db::write_tx(&db).map_err(db_err)?;
                    let m = chat::add_human(
                        &tx,
                        c.id,
                        &msg,
                        &text,
                        if exec { State::Sent } else { State::Done },
                        false,
                        exec,
                    )
                    .map_err(db_err)?;
                    chat::set_via(&tx, m.id, via.as_deref()).map_err(db_err)?;
                    chat::set_request(&tx, m.id, request).map_err(db_err)?;
                    chat::observe(&tx, m.id, lc).map_err(db_err)?;
                    tx.commit().map_err(db_err)?;
                    if request {
                        kind = MessageKind::Ask;
                    }
                }
                "agent" if c.dir == Dir::In => {
                    // 자동 왕복 상한 — 상대 에이전트가 내 에이전트 답에 단 마지막 답. 실행 없이 기록만.
                    let answered = match reply_to.as_deref() {
                        Some(r) => chat::message_by_uuid(&db, r).map_err(db_err)?,
                        None => None,
                    }
                    .filter(|m| m.conversation_id == c.id && m.author == Author::Agent && m.mine);
                    if exec || text.trim().is_empty() || answered.is_none() {
                        return Ok(Response::error("bad_request"));
                    }
                    let tx = an_core::db::write_tx(&db).map_err(db_err)?;
                    let m = chat::add_agent_request(&tx, c.id, &msg, &text, State::Done)
                        .map_err(db_err)?;
                    chat::set_flags(&tx, m.id, false, false, State::Done).map_err(db_err)?;
                    chat::set_run(
                        &tx,
                        m.id,
                        turns,
                        duration_ms,
                        model.as_deref(),
                        effort.as_deref(),
                    )
                    .map_err(db_err)?;
                    chat::observe(&tx, m.id, lc).map_err(db_err)?;
                    chat::set_ctx_tokens(&tx, m.id, ctx).map_err(db_err)?;
                    tx.commit().map_err(db_err)?;
                    kind = MessageKind::RelayEnd;
                }
                "agent" => {
                    kind = MessageKind::Reply;
                    let asked = match reply_to.as_deref() {
                        Some(r) => chat::message_by_uuid(&db, r).map_err(db_err)?,
                        None => None,
                    }
                    .filter(|m| {
                        m.conversation_id == c.id
                            && m.author == Author::Human
                            && !m.mine
                            && m.exec
                            && m.state == State::Sent
                    });
                    let Some(asked) = asked else {
                        return Ok(Response::error("unknown"));
                    };
                    let state = if ok.unwrap_or(false) {
                        State::Done
                    } else {
                        State::Error
                    };
                    let tx = an_core::db::write_tx(&db).map_err(db_err)?;
                    let r = chat::add_reply(
                        &tx,
                        c.id,
                        &msg,
                        &text,
                        state,
                        asked.id,
                        turns,
                        duration_ms,
                        model.as_deref(),
                        effort.as_deref(),
                    )
                    .map_err(db_err)?;
                    chat::observe(&tx, r.id, lc).map_err(db_err)?;
                    chat::set_ctx_tokens(&tx, r.id, ctx).map_err(db_err)?;
                    chat::set_state(&tx, asked.id, state).map_err(db_err)?;
                    tx.commit().map_err(db_err)?;
                }
                _ => return Ok(Response::error("bad_request")),
            }
            (c.id, c.agent)
        };
        let shown = if kind == MessageKind::RelayEnd {
            RELAY_END_CAP.to_string()
        } else {
            preview(&text)
        };
        self.notify(cid, peer, agent, kind, shown);
        Ok(Response::Ok)
    }

    /// 받은 요청 — 실행 없이 기록만(대기열 · 실행 기록 없음), 상태 '대기'. 같은 uuid 재전송은 `Ok`(한 번만).
    fn inbound_ask(
        &self,
        peer: &Peer,
        conv: &str,
        msg: &str,
        agent: String,
        text: &str,
        (lc, via): (Option<i64>, Option<String>),
    ) -> io::Result<Response> {
        let cid = {
            let db = self.db();
            if agents::by_name(&db, &agent).map_err(db_err)?.is_none() {
                return Ok(Response::error("unknown_agent"));
            }
            if chat::left(&db, conv, peer.id).map_err(db_err)? {
                return Ok(Response::error(chat::LEFT));
            }
            let Ok(c) = chat::open_conversation(&db, conv, peer.id, &agent, Dir::In) else {
                return Ok(Response::error("bad_request")); // 다른 상대의 대화 uuid
            };
            if c.dir != Dir::In || c.agent != agent {
                return Ok(Response::error("bad_request"));
            }
            if chat::message_by_uuid(&db, msg).map_err(db_err)?.is_some()
                || chat::gone(&db, msg).map_err(db_err)?
            {
                return Ok(Response::Ok);
            }
            let tx = an_core::db::write_tx(&db).map_err(db_err)?;
            let m =
                chat::add_human(&tx, c.id, msg, text, State::Done, false, false).map_err(db_err)?;
            chat::set_via(&tx, m.id, via.as_deref()).map_err(db_err)?;
            chat::set_request(&tx, m.id, true).map_err(db_err)?;
            chat::observe(&tx, m.id, lc).map_err(db_err)?;
            tx.commit().map_err(db_err)?;
            c.id
        };
        self.notify(cid, peer, agent, MessageKind::Ask, preview(text));
        Ok(Response::Ok)
    }

    /// 상대가 알려 준 내 실행 요청의 차례 — 그 상대 소유 대화의 내 실행 요청 글 · 아직 답을 기다리는 것만.
    fn inbound_progress(
        &self,
        peer: &Peer,
        conv: String,
        msg: String,
        state: String,
        (ahead, detail): (u32, Option<StepDetail>),
    ) -> io::Result<Response> {
        if !matches!(state.as_str(), "queued" | "running") {
            return Ok(Response::error("bad_request"));
        }
        let db = self.db();
        let Some(c) = chat::find_open(&db, &conv, peer.id).map_err(db_err)? else {
            return Ok(Response::error("unknown"));
        };
        let mine = chat::message_by_uuid(&db, &msg)
            .map_err(db_err)?
            .filter(|m| m.conversation_id == c.id && m.mine && m.exec && m.state == State::Sent);
        let Some(m) = mine else {
            return Ok(Response::error("unknown"));
        };
        let running = state == "running";
        let progress = Progress {
            ahead: if running { 0 } else { ahead },
            detail: detail.filter(|_| running).map(StepDetail::cleaned),
            state,
        };
        an_core::sync::lock(&self.progress).insert(m.id, progress);
        Ok(Response::Ok)
    }

    fn inbound_reply(self: &Arc<Self>, peer: &Peer, reply: Request) -> io::Result<Response> {
        let mut relay_next = None;
        let mut kind = MessageKind::Reply;
        let mut ended = None;
        let Request::Reply {
            conv,
            msg,
            reply_to,
            text,
            ok,
            turns,
            duration_ms,
            lc,
            model,
            effort,
            ctx,
        } = reply
        else {
            return Ok(Response::error("bad_request"));
        };
        let ctx = ctx.filter(|v| (0..=MAX_CTX_TOKENS).contains(v));
        let (model, effort) = run_meta(model, effort);
        if text.len() > MAX_TEXT || msg.is_empty() || text.trim().is_empty() {
            return Ok(Response::error("bad_request"));
        }
        let (cid, agent) = {
            let db = self.db();
            let Some(c) = chat::find_open(&db, &conv, peer.id)
                .map_err(db_err)?
                .filter(|c| c.dir == Dir::Out)
            else {
                return Ok(Response::error("unknown"));
            };
            if chat::message_by_uuid(&db, &msg).map_err(db_err)?.is_some()
                || chat::gone(&db, &msg).map_err(db_err)?
            {
                return Ok(Response::Ok); // 같은 답 재전송 · 기록 삭제로 지운 답
            }
            // 요청 없이 온 답 · 이미 답이 온 메시지에 또 온 답은 받지 않는다.
            let asked = chat::message_by_uuid(&db, &reply_to)
                .map_err(db_err)?
                .filter(|m| m.conversation_id == c.id && m.exec && m.state == State::Sent);
            let Some(asked) = asked else {
                return Ok(Response::error("unknown"));
            };
            if asked.author == Author::Agent {
                kind = MessageKind::RelayStep; // 자동 왕복으로 넘긴 내 에이전트 답에 온 답
            } else if c.relay.as_ref().is_some_and(|r| r.on) {
                kind = MessageKind::RelayStart; // 자동 왕복을 연 사람 요청에 온 첫 답
            }
            let state = if ok { State::Done } else { State::Error };
            let tx = an_core::db::write_tx(&db).map_err(db_err)?;
            let reply = chat::add_reply(
                &tx,
                c.id,
                &msg,
                &text,
                state,
                asked.id,
                turns,
                duration_ms,
                model.as_deref(),
                effort.as_deref(),
            )
            .map_err(db_err)?;
            chat::observe(&tx, reply.id, lc).map_err(db_err)?;
            chat::set_ctx_tokens(&tx, reply.id, ctx).map_err(db_err)?;
            chat::set_state(&tx, asked.id, state).map_err(db_err)?;
            // 멘션한 요청의 답 — 실행한 에이전트는 요청이 부른 이름 그대로.
            if asked.agent.is_some() {
                chat::set_agent(&tx, reply.id, asked.agent.as_deref()).map_err(db_err)?;
            }
            // 자동 왕복 중이고 성공한 답이며 상한 전 · 집중 아님이면 이 답을 내 에이전트 입력으로(연 쪽 구동). 답 기록과 한 트랜잭션 —
            // 사이에 끊겨도 "켜짐인데 대기열 없음" 이 남지 않는다.
            let focused = self.focused(&tx);
            match c.relay.as_ref().filter(|r| r.on) {
                Some(r) if ok && r.count < r.cap && !focused => {
                    chat::set_flags(&tx, reply.id, false, true, State::Queued).map_err(db_err)?;
                    relay_next = Some(c.id);
                }
                // 상대 실행 실패 · 상한 · 집중 — 멈추고 사람 입력을 기다린다.
                Some(r) => {
                    chat::relay_on(&tx, c.id, false, false).map_err(db_err)?;
                    ended = Some(if !ok {
                        "상대 에이전트 실행 실패"
                    } else if r.count >= r.cap {
                        RELAY_END_CAP
                    } else {
                        "집중 중"
                    });
                }
                None => {}
            }
            tx.commit().map_err(db_err)?;
            (c.id, asked.agent.clone().unwrap_or(c.agent))
        };
        if let Some(id) = relay_next {
            self.wake(id);
        }
        self.notify(cid, peer, agent.clone(), kind, preview(&text));
        if let Some(reason) = ended {
            self.notify(cid, peer, agent, MessageKind::RelayEnd, reason.to_string());
        }
        Ok(Response::Ok)
    }
}
