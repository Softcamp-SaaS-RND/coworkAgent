//! 대화 수명 — 나가기 · 지우기 · 비우기.

use super::*;

/// 도는 실행이 있는 대화를 나가려 했다(실행 중단 기능은 없다).
const LEAVE_BUSY: &str = "실행이 끝난 뒤 나갈 수 있습니다";
/// 기록 삭제 — 실행 중 · 대기 글이 있으면 막는다.
const CLEAR_BUSY: &str = "실행이 끝난 뒤 지울 수 있습니다";
/// 모든 사용자 글 삭제 — 상대에게 보내는 중인 글이 있으면 막는다.
const CLEAR_SENDING: &str = "보내는 중인 글이 있습니다 — 잠시 뒤 다시 지우세요";

impl Chat {
    /// 대화에서 나간다 — 이력은 남고 읽기 전용, 그 뒤 이 대화로 온 요청은 거절(`left`), 상대에게 `Leave`. 공유 사본이면
    /// [`share_leave`](Self::share_leave). 도는 실행이 있으면 막는다(`ResourceBusy`), 아니면 한 트랜잭션: 대기 글 취소 · 답을 기다리던 내 글 닫기 ·
    /// 자동 왕복 끔 · 관람자 공유 끝(`ShareEnd`) · 그 대화의 원 상대 앞 기다리는 보낼 글 버림 · `Leave` 행(대화를 지워도 남게 `conv_id` 없이).
    /// 이미 나갔으면 그대로.
    pub fn leave_conversation(&self, conversation_id: i64) -> io::Result<()> {
        let conv = self.conversation(conversation_id)?;
        if conv.dir == Dir::Shared {
            return self.share_leave(conversation_id);
        }
        let woken = {
            let db = self.db();
            // 실행 표시(`running`)도 같은 잠금으로 쓰인다 — 확인과 나가기 사이에 끼어들지 못한다.
            let tx = an_core::db::write_tx(&db).map_err(db_err)?;
            if chat::running(&tx, conv.id).map_err(db_err)? {
                return Err(io::Error::new(io::ErrorKind::ResourceBusy, LEAVE_BUSY));
            }
            let now = chat::by_id(&tx, conv.id)
                .map_err(db_err)?
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "없는 대화"))?;
            if now.left_at.is_some() {
                return Ok(());
            }
            chat::leave(&tx, conv.id).map_err(db_err)?;
            let mut woken = vec![conv.peer_id];
            let end = Request::ShareEnd {
                conv: conv.uuid.clone(),
            };
            for w in share::watchers(&tx, conv.id).map_err(db_err)? {
                share::remove(&tx, conv.id, w.peer_id).map_err(db_err)?;
                Self::stage_share(&tx, &conv, w.peer_id, &end)?;
                woken.push(w.peer_id);
            }
            // 기다리는 기록 지우기는 남긴다 — 지운 뒤 나가도 상대 쪽은 비워지게.
            tx.execute(
                "DELETE FROM outbox WHERE conv_id = ?1 AND peer_id = ?2 AND state = 'pending' AND kind <> ?3",
                (conv.id, conv.peer_id, CLEAR),
            )
            .map_err(db_err)?;
            let frame = serde_json::to_string(&Request::Leave {
                conv: conv.uuid.clone(),
            })
            .map_err(io::Error::other)?;
            outbox::add(
                &tx,
                conv.peer_id,
                None,
                None,
                LEAVE,
                &frame,
                None,
                "pending",
            )
            .map_err(db_err)?;
            tx.commit().map_err(db_err)?;
            woken
        };
        for p in woken {
            self.kick(p);
        }
        Ok(())
    }

    /// 대화를 이 PC 에서 지운다 — 나간 뒤(아직이면 [`leave_conversation`](Self::leave_conversation) — 같은 막음) 대화 · 글 · 실행 ·
    /// 에이전트 세션 기록 · 딸린 항목을 지운다(`chat::delete_conversation`). 상대 PC 사본은 남는다. 상대가 먼저 나간 대화도 된다.
    pub fn delete_conversation(&self, conversation_id: i64) -> io::Result<()> {
        self.leave_conversation(conversation_id)?;
        let db = self.db();
        let tx = an_core::db::write_tx(&db).map_err(db_err)?;
        if chat::running(&tx, conversation_id).map_err(db_err)? {
            return Err(io::Error::new(io::ErrorKind::ResourceBusy, LEAVE_BUSY));
        }
        chat::delete_conversation(&tx, conversation_id).map_err(db_err)?;
        tx.commit().map_err(db_err)
    }

    /// 대화 기록을 지운다(기록 삭제) — 이 PC 의 그 대화 글 · 실행 · 세션 기록을 비우고 대화는 남긴다(`chat::clear_conversation`). `all` = 모든 사용자 글
    /// 삭제 — 상대 PC 에 `Clear`(그쪽도 비운다), 관람자 사본은 머리(`Share`)를 다시 보내 비운다. 거절: 공유 사본(`READ_ONLY`) · 실행 중 · 대기 글(Busy) ·
    /// `all` 인데 닫힌 대화(`CLOSED`) · `all` 인데 상대 앱이 모름(Unsupported — 아무것도 지우지 않는다). 상대에 닿지 않으면 이 PC 를 비우고 `Clear` 는 줄이 나중에 보낸다.
    pub async fn clear_conversation(&self, conversation_id: i64, all: bool) -> io::Result<()> {
        let conv = self.conversation(conversation_id)?;
        if conv.dir == Dir::Shared {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, READ_ONLY));
        }
        if all {
            if conv.closed() {
                return Err(io::Error::new(io::ErrorKind::InvalidInput, CLOSED));
            }
            self.paired_peer(conv.peer_id)?;
            match self.node.knows_clear(conv.peer_id).await {
                Ok(true) => {}
                Ok(false) => {
                    return Err(io::Error::new(io::ErrorKind::Unsupported, CLEAR_UNKNOWN));
                }
                Err(e) if e.kind() == io::ErrorKind::PermissionDenied => return Err(e),
                // 꺼져 있다 — 이 PC 를 비우고 줄이 나중에 보낸다(그때 모르면 버리고 내 활동에 남긴다).
                Err(_) => {}
            }
        }
        let woken = {
            let db = self.db();
            let head = Self::share_head(&db, &conv)?;
            let tx = an_core::db::write_tx(&db).map_err(db_err)?;
            if chat::busy(&tx, conv.id).map_err(db_err)? {
                return Err(io::Error::new(io::ErrorKind::ResourceBusy, CLEAR_BUSY));
            }
            // 상대에게 보내는 중인 글이 있으면 막는다 — 이미 떠난 옛 글이 지우기보다 늦게 닿아 상대 쪽에 되살아나지 않게.
            if all && outbox::sending(&tx, conv.id, conv.peer_id).map_err(db_err)? {
                return Err(io::Error::new(io::ErrorKind::ResourceBusy, CLEAR_SENDING));
            }
            chat::clear_conversation(&tx, conv.id).map_err(db_err)?;
            let mut woken = Vec::new();
            if all {
                let frame = serde_json::to_string(&Request::Clear {
                    conv: conv.uuid.clone(),
                    id: new_uuid(),
                })
                .map_err(io::Error::other)?;
                // 그 대화의 줄에(`conv_id`) — 뒤에 쓴 글이 지우기보다 먼저 닿지 않게(`dispatch` 의 `has_before`). 나가기 · 삭제는 이 행을 떼어 남긴다.
                outbox::add(&tx, conv.peer_id, Some(conv.id), None, CLEAR, &frame, None, "pending")
                    .map_err(db_err)?;
                woken.push(conv.peer_id);
                woken.extend(Self::reshare(&tx, &conv, &head)?);
            }
            tx.commit().map_err(db_err)?;
            woken
        };
        for p in woken {
            self.kick(p);
        }
        Ok(())
    }

    /// 상대가 그 대화 기록을 모두 지웠다(받은 `Clear`) — 그 상대 소유 대화(`find_own`)의 글을 비우고 관람자 사본도 비운다, 알림. 닫힌 대화는 `handle`
    /// 이 먼저 `left` 로 거절한다. 실행 중 · 대기 글이 있으면 `retry`(보낸 쪽 줄이 다시 보낸다 — 실행이 끝난 뒤 비운다).
    pub(super) fn inbound_clear(&self, peer: &Peer, conv: String, id: String) -> io::Result<Response> {
        let (c, woken) = {
            let db = self.db();
            let Some(c) = chat::find_own(&db, &conv, peer.id).map_err(db_err)? else {
                return Ok(Response::error("unknown"));
            };
            if c.dir == Dir::Shared {
                return Ok(Response::error("unknown"));
            }
            let head = Self::share_head(&db, &c)?;
            let tx = an_core::db::write_tx(&db).map_err(db_err)?;
            // 이미 적용한 지우기(응답이 사라져 다시 왔다) — 그 사이 쓴 글을 지우지 않는다.
            if chat::gone(&tx, &id).map_err(db_err)? {
                return Ok(Response::Ok);
            }
            if chat::busy(&tx, c.id).map_err(db_err)? {
                return Ok(Response::error(RETRY));
            }
            chat::clear_conversation(&tx, c.id).map_err(db_err)?;
            chat::mark_gone(&tx, &id, c.id).map_err(db_err)?;
            let woken = Self::reshare(&tx, &c, &head)?;
            tx.commit().map_err(db_err)?;
            (c, woken)
        };
        for p in woken {
            self.kick(p);
        }
        self.notify(c.id, peer, c.agent.clone(), MessageKind::Clear, String::new());
        Ok(Response::Ok)
    }

    /// 상대가 나갔다 — 그 상대 소유 대화를 읽기 전용으로(대기 글 취소 · 답을 기다리던 내 글 닫기 · 자동 왕복 끔). 없는 대화면 `Ok`(이미 지웠다).
    pub(super) fn inbound_leave(&self, peer: &Peer, conv: String) -> io::Result<Response> {
        let db = self.db();
        let Some(c) = chat::find_own(&db, &conv, peer.id).map_err(db_err)? else {
            return Ok(Response::Ok);
        };
        let tx = an_core::db::write_tx(&db).map_err(db_err)?;
        chat::set_peer_left(&tx, c.id).map_err(db_err)?;
        tx.commit().map_err(db_err)?;
        Ok(Response::Ok)
    }
}
