//! 자동 왕복.

use super::*;

impl Chat {
    /// 두 에이전트 자동 왕복 대화를 연다 — `my_agent` 는 이 PC 프로필, 상한 1 ~ 50 왕복. `title` = 이 PC 에서 붙이는 제목.
    pub async fn start_relay(
        &self,
        peer_id: i64,
        agent: &str,
        my_agent: &str,
        cap: i64,
        text: &str,
        title: Option<&str>,
    ) -> io::Result<(Conversation, Message)> {
        let opts = Compose {
            title,
            ..Compose::default()
        };
        self.start_relay_with(peer_id, agent, my_agent, cap, text, opts)
            .await
    }

    /// [`start_relay`](Self::start_relay) + 첫 글의 선택([`Compose`] — 출처는 쓰지 않는다).
    pub async fn start_relay_with(
        &self,
        peer_id: i64,
        agent: &str,
        my_agent: &str,
        cap: i64,
        text: &str,
        opts: Compose<'_>,
    ) -> io::Result<(Conversation, Message)> {
        if text.trim().is_empty() || text.len() > MAX_TEXT {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("본문은 1 ~ {MAX_TEXT} 바이트"),
            ));
        }
        if !(1..=MAX_RELAY_CAP).contains(&cap) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("자동 왕복 상한은 1 ~ {MAX_RELAY_CAP}"),
            ));
        }
        // 요청은 실행이 없어 넘길 답이 없다 — 대화를 만들기 전에 거른다.
        if opts.request {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                REQUEST_NO_RELAY,
            ));
        }
        let peer = self.paired_peer(peer_id)?;
        let conv = {
            let db = self.db();
            if agents::by_name(&db, my_agent).map_err(db_err)?.is_none() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("내 에이전트 {my_agent} 가 없다"),
                ));
            }
            let tx = an_core::db::write_tx(&db).map_err(db_err)?;
            let c = chat::open_conversation(&tx, &new_uuid(), peer_id, agent, Dir::Out)
                .map_err(db_err)?;
            chat::set_relay(&tx, c.id, my_agent, cap).map_err(db_err)?;
            chat::set_title(&tx, c.id, opts.title).map_err(db_err)?;
            if let Some(g) = opts.group
                && !an_core::groups::join(&tx, g, c.id).map_err(db_err)?
            {
                return Err(no_group());
            }
            let c = chat::by_id(&tx, c.id).map_err(db_err)?.unwrap_or(c);
            tx.commit().map_err(db_err)?;
            c
        };
        let opts = Compose {
            via: None,
            mention: None,
            ..opts
        };
        self.send_human(conv, &peer, text, Some(my_agent.to_string()), opts)
            .await
    }

    /// 자동 왕복이 저절로 멈췄다 — 사유와 함께 알린다(사람이 멈춘 것은 부르지 않는다).
    pub(super) fn relay_end(&self, conv: &Conversation, reason: &str) {
        if let Ok(peer) = self.paired_peer(conv.peer_id) {
            self.notify(
                conv.id,
                &peer,
                conv.agent.clone(),
                MessageKind::RelayEnd,
                reason.to_string(),
            );
        }
    }

    /// 자동 왕복 알림 — 그 상대 소유 대화만. 연 쪽은 `on = false` 만(받은 쪽 멈춤), 받은 쪽은 거절 켜기 · 끄기.
    pub(super) fn inbound_relay(&self, peer: &Peer, conv: String, on: bool) -> io::Result<Response> {
        let db = self.db();
        let Some(c) = chat::find_open(&db, &conv, peer.id).map_err(db_err)? else {
            return Ok(Response::error("unknown"));
        };
        let event = match (c.dir, on) {
            (Dir::Shared, _) => return Ok(Response::error("unknown")),
            (Dir::Out, false) => {
                chat::relay_on(&db, c.id, false, false).map_err(db_err)?;
                // 켜져 있던 자동 왕복을 상대가 멈췄다.
                c.relay
                    .as_ref()
                    .filter(|r| r.on)
                    .map(|_| (MessageKind::RelayEnd, "상대가 멈춤"))
            }
            (Dir::Out, true) => return Ok(Response::error("bad_request")),
            (Dir::In, on) => {
                chat::set_blocked(&db, c.id, !on).map_err(db_err)?;
                on.then_some((MessageKind::RelayStart, "상대가 자동 왕복을 재개"))
            }
        };
        drop(db);
        if let Some((kind, text)) = event {
            self.notify(c.id, peer, c.agent, kind, text.to_string());
        }
        Ok(Response::Ok)
    }

    /// 받은 쪽 멈춤 — 에이전트발 요청을 거절하고 연 쪽에 알린다(알림 실패는 거절로 충분).
    pub(super) async fn block_relay(&self, conv: &Conversation, peer: &Peer) {
        let _ = chat::set_blocked(&self.db(), conv.id, true);
        let req = Request::Relay {
            conv: conv.uuid.clone(),
            on: false,
        };
        let _ = peer;
        let _ = self.ship(conv, None, RELAY, &req).await;
    }

    /// 자동 왕복 멈춤 · 재개. 연 쪽: 끄기 / 켜기(카운트 0, 받은 쪽 거절 해제, 아직 넘기지 않은 상대 답이 있으면 바로 넘김).
    /// 받은 쪽: 끄기만(거절 + 알림).
    pub async fn relay(
        self: &Arc<Self>,
        conversation_id: i64,
        on: bool,
    ) -> io::Result<Conversation> {
        let conv = chat::by_id(&self.db(), conversation_id)
            .map_err(db_err)?
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "없는 대화"))?;
        if conv.closed() {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, CLOSED));
        }
        let peer = self.paired_peer(conv.peer_id)?;
        match (conv.dir, on) {
            (Dir::Shared, _) => {
                return Err(io::Error::new(io::ErrorKind::InvalidInput, READ_ONLY));
            }
            (Dir::In, false) => self.block_relay(&conv, &peer).await,
            (Dir::In, true) => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "재개는 대화를 연 쪽이 한다",
                ));
            }
            (Dir::Out, _) if conv.relay.is_none() => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "자동 왕복 대화가 아니다",
                ));
            }
            (Dir::Out, false) => {
                chat::relay_on(&self.db(), conv.id, false, false).map_err(db_err)?;
            }
            (Dir::Out, true) => {
                let req = Request::Relay {
                    conv: conv.uuid.clone(),
                    on: true,
                };
                match self
                    .node
                    .send(addr(&peer)?, Some(peer.spki.clone()), &req)
                    .await
                {
                    Ok((Response::Ok, _)) => {}
                    Ok((other, _)) => return Err(refused(other)),
                    Err(e) => return Err(e),
                }
                let (queued, resend) = {
                    let db = self.db();
                    let tx = an_core::db::write_tx(&db).map_err(db_err)?;
                    chat::relay_on(&tx, conv.id, true, true).map_err(db_err)?;
                    let last = chat::messages(&tx, conv.id)
                        .map_err(db_err)?
                        .pop()
                        .filter(|m| m.author == Author::Agent);
                    // 아직 내 에이전트에 넘기지 않은 상대 답.
                    let queued = last
                        .as_ref()
                        .filter(|m| !m.mine && !m.exec && m.state == State::Done);
                    if let Some(m) = queued {
                        chat::set_flags(&tx, m.id, false, true, State::Queued).map_err(db_err)?;
                    }
                    tx.commit().map_err(db_err)?;
                    // 상대에 넘기지 못한 내 답 — 멈춤 · 상한으로 기록만(`done` · `undelivered`) 또는 거절 · 전송 오류(`error` · `sent`).
                    let resend = last.clone().filter(|m| {
                        m.mine
                            && if m.exec {
                                matches!(m.state, State::Error | State::Sent)
                            } else {
                                matches!(m.state, State::Done | State::Undelivered)
                            }
                    });
                    (queued.is_some(), resend)
                };
                if queued {
                    self.wake(conv.id);
                }
                if let Some(m) = resend {
                    self.relay_forward(&conv, Some(m.id), true, &m.text, Some(m.lc), None)
                        .await;
                }
            }
        }
        chat::by_id(&self.db(), conversation_id)
            .map_err(db_err)?
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "없는 대화"))
    }

    /// 자동 왕복 — 내 에이전트 답을 상대 에이전트 실행 요청으로 보낸다. 실패 · 멈춤 · 거절이면 자동 왕복을 끈다.
    pub(super) async fn relay_forward(
        &self,
        conv: &Conversation,
        reply_id: Option<i64>,
        ok: bool,
        text: &str,
        lc: Option<i64>,
        reply_to: Option<&str>,
    ) {
        let relay = chat::by_id(&self.db(), conv.id)
            .ok()
            .flatten()
            .and_then(|c| c.relay);
        let still_on = relay.as_ref().is_some_and(|r| r.on);
        let (Some(reply_id), true, true) = (reply_id, ok, still_on) else {
            let _ = chat::relay_on(&self.db(), conv.id, false, false);
            // 사람이 멈춘 것(`still_on` 거짓)은 알리지 않는다.
            if still_on {
                self.relay_end(
                    conv,
                    if ok {
                        "기록 실패"
                    } else {
                        "내 에이전트 실행 실패"
                    },
                );
            }
            return;
        };
        let Some(msg) = chat::messages(&self.db(), conv.id)
            .ok()
            .and_then(|ms| ms.into_iter().find(|m| m.id == reply_id))
        else {
            return;
        };
        // 상한 — 이 답으로 왕복을 채웠다. 상대 에이전트를 더 돌리지 않고 상대 이력에 기록만 남긴 뒤 멈춘다.
        if relay.as_ref().is_some_and(|r| r.count >= r.cap) {
            let _ = chat::relay_on(&self.db(), conv.id, false, false);
            self.relay_end(conv, RELAY_END_CAP);
            let req = Request::Post {
                conv: conv.uuid.clone(),
                msg: msg.uuid.clone(),
                author: "agent".into(),
                text: text.to_string(),
                exec: false,
                reply_to: reply_to.map(str::to_string),
                ok: Some(true),
                turns: msg.turns,
                duration_ms: msg.duration_ms,
                lc,
                via: None,
                model: msg.model.clone(),
                effort: msg.effort.clone(),
                request: false,
                ctx: msg.ctx_tokens,
            };
            let sent = self.ship(conv, Some(msg.id), RELAY_LAST, &req).await;
            if !matches!(sent, Ok(Shipped::Answered(Response::Ok) | Shipped::Kept)) {
                log::warn!("대화 {} 자동 왕복 마지막 답 기록 실패", conv.uuid);
                let _ = chat::set_state(&self.db(), msg.id, State::Undelivered);
            }
            return;
        }
        let req = Request::Message {
            conv: conv.uuid.clone(),
            msg: msg.uuid.clone(),
            agent: conv.agent.clone(),
            text: text.to_string(),
            lc,
            by_agent: conv.relay.as_ref().map(|r| r.agent.clone()),
            relay: None,
            via: None,
            request: false,
            mention: None,
        };
        // 보내기 전에 `sent` 실행 요청으로 — 저장 못 하면 보내지 않는다(늦게 온 답을 받을 근거가 없다). 보내기 행과 한 트랜잭션(`sent`
        // 만 남고 보낼 행이 없는 채로 꺼져 자동 왕복이 멈추지 않게).
        // 결과를 먼저 받는다 — `if let` 조건의 DB 잠금은 블록 끝까지 살아 있어(edition 2024) 안에서 다시 잠그면 멈춘다.
        let flagged = {
            let db = self.db();
            an_core::db::write_tx(&db).map_err(db_err).and_then(|tx| {
                chat::set_flags(&tx, msg.id, true, true, State::Sent)
                    .and_then(|_| chat::set_refused(&tx, msg.id, None))
                    .map_err(db_err)?;
                let staged = Self::stage(&tx, conv, Some(msg.id), RELAY_STEP, &req)?;
                tx.commit().map_err(db_err)?;
                Ok(staged)
            })
        };
        let staged = match flagged {
            Ok(id) => id,
            Err(e) => {
                log::error!("대화 {} 자동 왕복 상태 저장 실패 — 멈춤: {e}", conv.uuid);
                let _ = chat::relay_on(&self.db(), conv.id, false, false);
                self.relay_end(conv, "상태 저장 실패");
                return;
            }
        };
        // 닿지 않으면 보관 — 줄이 다시 보내고 닿으면 이어 간다. 명시적 거절 · 보관 실패만 멈춘다.
        let sent = self.dispatch(conv, staged, &req).await;
        if !matches!(sent, Ok(Shipped::Answered(Response::Ok) | Shipped::Kept)) {
            log::warn!("대화 {} 자동 왕복 전달 실패 — 멈춤", conv.uuid);
            let _ = chat::relay_on(&self.db(), conv.id, false, false);
            match sent {
                Ok(Shipped::Answered(Response::Error { code })) if code == STOPPED => {
                    self.relay_stopped(conv, msg.id);
                }
                Ok(Shipped::Answered(other)) => {
                    let focus = matches!(&other, Response::Error { code } if code == FOCUS);
                    self.refused_by_peer(msg.id, conv.peer_id, &other);
                    self.relay_end(
                        conv,
                        if focus {
                            "상대가 집중 중"
                        } else {
                            "상대가 거절"
                        },
                    );
                }
                _ => self.relay_end(conv, "상태 저장 실패"),
            }
        }
    }

    /// 받은 쪽이 멈춘 뒤 닿은 넘김(`stopped`) — 멈춤 알림(`Relay{on: false}`)이 먼저 닿은 것과 같은 결과: 내 답은 넘기지 않은 답으로
    /// 되돌리고(실패 아님) 자동 왕복을 끄고 "상대가 멈춤". 알림이 뒤늦게 와도 이미 꺼져 있어 다시 알리지 않는다.
    pub(super) fn relay_stopped(&self, conv: &Conversation, msg_id: i64) {
        {
            let db = self.db();
            let _ = chat::set_flags(&db, msg_id, true, false, State::Done);
            let _ = chat::relay_on(&db, conv.id, false, false);
        }
        self.relay_end(conv, "상대가 멈춤");
    }
}
