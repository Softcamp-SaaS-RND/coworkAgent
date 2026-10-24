//! 공유 사본 · 관람자.

use super::*;

impl Chat {
    /// 대화에 관람자를 넣는다(A — 대화를 가진 PC). 거절: 공유 사본(`InvalidInput`) · 원 상대(`InvalidInput`) · 페어링 안 됨 · 이미 관람자
    /// (`AlreadyExists`) · 상한 [`share::MAX_WATCHERS`](`InvalidInput`) · 이 기능을 모르는 상대(`Unsupported`, Ping 으로 묻는다 — 닿지 않으면 그 오류).
    /// 관람자 행 · 머리(`Share`) · 이력(`SharePost` — 글 하나 = 프레임 하나)을 한 트랜잭션으로 그 관람자 줄에 넣고, 원 상대에게 관람자 수를 알린다.
    pub async fn share_add(&self, conversation_id: i64, peer_id: i64) -> io::Result<()> {
        let conv = self.conversation(conversation_id)?;
        if conv.dir == Dir::Shared {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "공유 대화는 다시 공유하지 않는다",
            ));
        }
        Self::writable(&conv)?;
        if peer_id == conv.peer_id {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "대화 상대는 관람자로 넣지 않는다",
            ));
        }
        self.paired_peer(peer_id)?;
        Self::share_room(&self.db(), conv.id, peer_id)?;
        if !self.node.knows_share(peer_id).await? {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "상대 앱 업데이트 필요 — 공유 대화를 모르는 버전",
            ));
        }
        {
            let db = self.db();
            let head = Self::share_head(&db, &conv)?;
            let tx = an_core::db::write_tx(&db).map_err(db_err)?;
            // Ping 하는 사이 다른 창이 넣었을 수 있다 — 트랜잭션 안에서 다시.
            Self::share_room(&tx, conv.id, peer_id)?;
            share::add(&tx, conv.id, peer_id).map_err(db_err)?;
            Self::stage_share(&tx, &conv, peer_id, &head)?;
            // 이력도 같은 트랜잭션 — 머리만 가고 글이 빠진 채로 꺼지지 않게.
            Self::fan_share_in(&tx, conv.id)?;
            tx.commit().map_err(db_err)?;
        }
        self.kick(peer_id);
        self.notice_watchers(&conv);
        Ok(())
    }

    /// 공유 머리(`Share`) — 이 PC 의 대화 제목 · 원 상대 이름. 관람자 사본은 이것을 받으면 글을 비우고 다시 받는다(기록 삭제 뒤에도).
    pub(super) fn share_head(conn: &Connection, conv: &Conversation) -> io::Result<Request> {
        let title = chat::summaries(conn)
            .map_err(db_err)?
            .into_iter()
            .find(|s| s.conversation.id == conv.id)
            .map(|s| s.title)
            .unwrap_or_default();
        let other = an_core::peers::by_id(conn, conv.peer_id)
            .map_err(db_err)?
            .map(|p| p.name)
            .unwrap_or_default();
        Ok(Request::Share {
            conv: conv.uuid.clone(),
            title,
            agent: conv.agent.clone(),
            owner_dir: conv.dir.as_str().into(),
            other,
        })
    }

    /// 관람자 자리 — 이미 관람자 · 상한이면 거절.
    fn share_room(conn: &Connection, conversation_id: i64, peer_id: i64) -> io::Result<()> {
        let ws = share::watchers(conn, conversation_id).map_err(db_err)?;
        if ws.iter().any(|w| w.peer_id == peer_id) {
            return Err(io::Error::new(io::ErrorKind::AlreadyExists, "이미 관람자"));
        }
        if ws.len() >= share::MAX_WATCHERS {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("관람자는 {}명까지", share::MAX_WATCHERS),
            ));
        }
        Ok(())
    }

    /// 관람자를 뺀다(A — 대화를 가진 PC). 이후 글은 가지 않고 관람자 사본은 읽기 전용으로 남는다(`ShareEnd`). 원 상대에게 관람자 수. 관람자가 아니면
    /// `NotFound`.
    pub fn share_remove(&self, conversation_id: i64, peer_id: i64) -> io::Result<()> {
        let conv = self.conversation(conversation_id)?;
        if !self.unshare(&conv, peer_id, true)? {
            return Err(io::Error::new(io::ErrorKind::NotFound, "관람자가 아니다"));
        }
        Ok(())
    }

    /// 공유 대화에서 나간다(C — 관람자 PC) — 사본은 읽기 전용으로 남고(빠짐 표시) 대화를 가진 PC 에 `ShareEnd`. 이미 빠졌으면 그대로.
    pub fn share_leave(&self, conversation_id: i64) -> io::Result<()> {
        let conv = self.conversation(conversation_id)?;
        if conv.dir != Dir::Shared {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "나가기는 공유 대화만",
            ));
        }
        let ended = {
            let db = self.db();
            let tx = an_core::db::write_tx(&db).map_err(db_err)?;
            let ended = share::end(&tx, conv.id).map_err(db_err)?;
            if ended {
                let end = Request::ShareEnd {
                    conv: conv.uuid.clone(),
                };
                Self::stage_share(&tx, &conv, conv.peer_id, &end)?;
            }
            tx.commit().map_err(db_err)?;
            ended
        };
        if ended {
            self.kick(conv.peer_id);
        }
        Ok(())
    }

    /// 관람자 사본을 비운다 — 기다리는 그 대화 공유 글(`SharePost` · `ShareDelete`)을 버리고 관람자마다 머리(`Share`)를 다시 넣는다. 깨울 관람자.
    pub(super) fn reshare(conn: &Connection, conv: &Conversation, head: &Request) -> io::Result<Vec<i64>> {
        conn.execute(
            "DELETE FROM outbox WHERE conv_id = ?1 AND kind = ?2
               AND json_extract(frame, '$.type') IN ('share_post', 'share_delete')",
            (conv.id, SHARE),
        )
        .map_err(db_err)?;
        let mut woken = Vec::new();
        for w in share::watchers(conn, conv.id).map_err(db_err)? {
            Self::stage_share(conn, conv, w.peer_id, head)?;
            woken.push(w.peer_id);
        }
        Ok(woken)
    }

    /// 공유 줄 행의 관람자가 아직 그 대화의 관람자인가. 읽지 못하면 참(보내던 대로 — 다음 시도에 다시 본다).
    pub(super) fn watching(&self, item: &outbox::Item) -> bool {
        let Some(conv) = item.conv_id else {
            return true;
        };
        share::watchers(&self.db(), conv)
            .map_or(true, |ws| ws.iter().any(|w| w.peer_id == item.peer_id))
    }

    /// 관람자 행 · 그 관람자 앞 기다리는 공유 프레임을 지우고, `end` 면 관람자에게 `ShareEnd`, 원 상대에게 바뀐 관람자 수. 없던 관람자면 거짓.
    pub(super) fn unshare(&self, conv: &Conversation, peer_id: i64, end: bool) -> io::Result<bool> {
        let gone = {
            let db = self.db();
            let tx = an_core::db::write_tx(&db).map_err(db_err)?;
            let gone = share::remove(&tx, conv.id, peer_id).map_err(db_err)?;
            if gone && end {
                let req = Request::ShareEnd {
                    conv: conv.uuid.clone(),
                };
                Self::stage_share(&tx, conv, peer_id, &req)?;
            }
            tx.commit().map_err(db_err)?;
            gone
        };
        if gone {
            if end {
                self.kick(peer_id);
            }
            self.notice_watchers(conv);
        }
        Ok(gone)
    }

    /// 원 상대(B)에게 그 대화의 관람자 수(이름 없음). 기다리는 알림은 마지막 값으로 바꾼다.
    fn notice_watchers(&self, conv: &Conversation) {
        let found = share::watchers(&self.db(), conv.id);
        let n = match found {
            Ok(ws) => ws.len(),
            Err(e) => return log::warn!("대화 {} 관람자 읽기 실패: {e}", conv.uuid),
        };
        let req = Request::ShareNotice {
            conv: conv.uuid.clone(),
            n: u32::try_from(n).unwrap_or(u32::MAX),
        };
        let key = format!("share_notice:{}", conv.uuid);
        self.enqueue(conv, None, SHARE_NOTICE, &req, Some(&key));
    }

    /// 공유 대화 — 관람자마다 보낸 커서 뒤 글(구분선 제외)을 `SharePost` 행으로 넣고 커서를 올린다(한 트랜잭션 — 몇 번 불러도 같은 글을
    /// 두 번 넣지 않는다). 글을 남기는 길목(`dispatch` · `enqueue` · `handle` 끝)이 부른다. 실패는 로그만 — 다음 부름이 이어 보낸다.
    // ponytail: 글 하나 = 프레임 하나 = 연결 하나 — 이력이 길면 느리다, 느리면 크기를 재서 묶는다.
    pub(super) fn fan_share(&self, conversation_id: i64) {
        let any = share::watchers(&self.db(), conversation_id);
        if any.as_ref().is_ok_and(Vec::is_empty) {
            return;
        }
        let fanned = (|| -> io::Result<Vec<i64>> {
            let db = self.db();
            let tx = an_core::db::write_tx(&db).map_err(db_err)?;
            let peers = Self::fan_share_in(&tx, conversation_id)?;
            tx.commit().map_err(db_err)?;
            Ok(peers)
        })();
        match fanned {
            Ok(peers) => {
                for p in peers {
                    self.kick(p);
                }
            }
            Err(e) => log::warn!("대화 {conversation_id} 관람자에게 보낼 글 보관 실패: {e}"),
        }
    }

    /// [`fan_share`](Self::fan_share) 의 몸 — `conn` 은 부르는 쪽 트랜잭션. 돌려주는 값: 깨울 관람자.
    fn fan_share_in(conn: &Connection, conversation_id: i64) -> io::Result<Vec<i64>> {
        let Some(conv) = chat::by_id(conn, conversation_id).map_err(db_err)? else {
            return Ok(Vec::new());
        };
        let mut ms = chat::messages(conn, conversation_id).map_err(db_err)?;
        ms.sort_by_key(|m| m.id);
        let top = ms.last().map_or(0, |m| m.id);
        let mut peers = Vec::new();
        for w in share::watchers(conn, conversation_id).map_err(db_err)? {
            if w.sent_upto >= top {
                continue;
            }
            for m in ms
                .iter()
                .filter(|m| m.id > w.sent_upto && m.author != Author::Context)
            {
                Self::stage_share(conn, &conv, w.peer_id, &Self::share_post(&conv, m))?;
            }
            share::set_sent(conn, conversation_id, w.peer_id, top).map_err(db_err)?;
            peers.push(w.peer_id);
        }
        Ok(peers)
    }

    /// 이미 관람자에게 간 글(커서 안)을 고쳤거나(`SharePost` 덮어쓰기) 지웠다(`ShareDelete`) — 그 관람자 줄에. 돌려주는 값: 깨울 관람자.
    /// 커서 뒤 글은 [`fan_share`](Self::fan_share) 가 지금 모습으로 보낸다.
    pub(super) fn share_change(
        conn: &Connection,
        conv: &Conversation,
        m: &Message,
        deleted: bool,
    ) -> io::Result<Vec<i64>> {
        let mut peers = Vec::new();
        for w in share::watchers(conn, conv.id).map_err(db_err)? {
            if w.sent_upto < m.id || m.author == Author::Context {
                continue;
            }
            let req = if deleted {
                Request::ShareDelete {
                    conv: conv.uuid.clone(),
                    msg: m.uuid.clone(),
                }
            } else {
                Self::share_post(conv, m)
            };
            Self::stage_share(conn, conv, w.peer_id, &req)?;
            peers.push(w.peer_id);
        }
        Ok(peers)
    }

    /// 공유 프레임을 그 상대 줄에(`msg_id` 없음 — 글을 나르는 행이 아니라 편집 판정 · 보관 표시에 끼지 않는다).
    pub(super) fn stage_share(
        conn: &Connection,
        conv: &Conversation,
        peer_id: i64,
        req: &Request,
    ) -> io::Result<()> {
        let frame = serde_json::to_string(req).map_err(io::Error::other)?;
        outbox::add(
            conn,
            peer_id,
            Some(conv.id),
            None,
            SHARE,
            &frame,
            None,
            "pending",
        )
        .map(drop)
        .map_err(db_err)
    }

    /// 글 하나의 `SharePost` — `owner` = 이 PC(대화를 가진 쪽) 글.
    fn share_post(conv: &Conversation, m: &Message) -> Request {
        let mut msg = ShareMsg {
            uuid: m.uuid.clone(),
            author: m.author.as_str().into(),
            text: m.text.clone(),
            lc: Some(m.lc),
            at: m.created_at,
            owner: m.mine,
            via: m.via.clone(),
            agent: m.agent.clone(),
            ok: (m.state == State::Error).then_some(false),
            edited: m.edited_at.is_some(),
        };
        // 제어 문자는 JSON 에서 6배가 된다 — 프레임 상한을 넘으면 본문을 줄인다(넘는 프레임은 보내지 못해 그 관람자 줄을 막다가 한도에 이르면 관람이 끝난다).
        let size = serde_json::to_vec(&msg).map_or(usize::MAX, |b| b.len());
        if size > MAX_FRAME - 256 {
            msg.text = fit_to(msg.text, MAX_TEXT / 4);
        }
        Request::SharePost {
            conv: conv.uuid.clone(),
            msg,
        }
    }

    /// 공유 프레임이 거절됐다 — 관람자가 이 기능을 모르거나(`unsupported`) 이 PC 를 페어링된 상대로 보지 않으면(`not_paired`) 그 관람자를
    /// 뺀다(관람자 목록에서 빠진다). 원 상대 알림 · 관람자의 나가기 거절은 로그만.
    pub(super) fn share_refused(&self, item: &outbox::Item, resp: &Response) {
        let code = match resp {
            Response::Error { code } => an_net::proto::clean_code(code),
            _ => "unknown",
        };
        log::info!(
            "상대 {} 가 공유 프레임({})을 받지 않았다: {code}",
            item.peer_id,
            item.kind
        );
        if item.kind != SHARE || !matches!(code, "not_paired" | "unsupported") {
            return;
        }
        let conv = item
            .conv_id
            .and_then(|id| chat::by_id(&self.db(), id).ok().flatten());
        let Some(conv) = conv.filter(|c| c.dir != Dir::Shared) else {
            return;
        };
        match self.unshare(&conv, item.peer_id, false) {
            Ok(true) => log::warn!(
                "대화 {} 관람자 {} 를 뺐다 — 상대가 받지 않음({code})",
                conv.uuid,
                item.peer_id
            ),
            Ok(false) => {}
            Err(e) => log::warn!("대화 {} 관람자 빼기 실패: {e}", conv.uuid),
        }
    }

    /// 공유 머리(C — 관람자 PC) — 대화를 가진 상대의 사본을 만들거나 다시 받는다(글을 비운다). 같은 uuid 가 내 대화 · 다른 상대의 것이면 `exists`.
    pub(super) fn inbound_share(
        &self,
        peer: &Peer,
        conv: String,
        title: String,
        agent: String,
        (owner_dir, other): (String, String),
    ) -> io::Result<Response> {
        if !matches!(owner_dir.as_str(), "out" | "in") {
            return Ok(Response::error("bad_request"));
        }
        // 제목은 이 PC 제목 규칙에 맞을 때만(아니면 첫 글 앞부분).
        let title = an_core::run_opts::clean_title(&title).ok().flatten();
        let (agent, other) = (
            an_net::proto::clean_name(&agent),
            an_net::proto::clean_name(&other),
        );
        let db = self.db();
        let tx = an_core::db::write_tx(&db).map_err(db_err)?;
        let copy = share::open_copy(
            &tx,
            &conv,
            peer.id,
            &agent,
            title.as_deref(),
            (&other, &owner_dir),
        )
        .map_err(db_err)?;
        if copy.is_none() {
            return Ok(Response::error("exists"));
        }
        tx.commit().map_err(db_err)?;
        Ok(Response::Ok)
    }

    /// 그 상대가 보낸 살아 있는 사본(C — 관람자 PC, 빠지지 않은 것).
    fn live_copy(conn: &Connection, peer: &Peer, conv: &str) -> io::Result<Option<Conversation>> {
        let Some(c) = chat::find(conn, conv, peer.id)
            .map_err(db_err)?
            .filter(|c| c.dir == Dir::Shared)
        else {
            return Ok(None);
        };
        let ended = share::copy(conn, c.id)
            .map_err(db_err)?
            .is_some_and(|x| x.ended);
        Ok((!ended).then_some(c))
    }

    /// 공유 글(C — 관람자 PC) — 그 상대 사본에만, 실행 · 알림 없이. 같은 uuid 가 다른 대화의 글이면 `exists`.
    pub(super) fn inbound_share_post(&self, peer: &Peer, conv: String, msg: ShareMsg) -> io::Result<Response> {
        let author = match msg.author.as_str() {
            "human" => Author::Human,
            "agent" => Author::Agent,
            _ => return Ok(Response::error("bad_request")),
        };
        if msg.text.trim().is_empty() || msg.text.len() > MAX_TEXT {
            return Ok(Response::error("bad_request"));
        }
        let agent = msg.agent.as_deref().map(an_net::proto::clean_name);
        let db = self.db();
        let Some(c) = Self::live_copy(&db, peer, &conv)? else {
            return Ok(Response::error("unknown"));
        };
        let post = share::Post {
            uuid: &msg.uuid,
            author,
            text: &msg.text,
            lc: msg.lc,
            at: msg.at,
            owner: msg.owner,
            via: msg.via.as_deref(),
            agent: agent.as_deref(),
            ok: msg.ok.unwrap_or(true),
            edited: msg.edited,
        };
        Ok(if share::upsert(&db, c.id, &post).map_err(db_err)? {
            Response::Ok
        } else {
            Response::error("exists")
        })
    }

    /// 공유 글 지움(C — 관람자 PC) — 그 상대 사본의 글만.
    pub(super) fn inbound_share_delete(&self, peer: &Peer, conv: String, msg: String) -> io::Result<Response> {
        let db = self.db();
        let Some(c) = Self::live_copy(&db, peer, &conv)? else {
            return Ok(Response::error("unknown"));
        };
        let tx = an_core::db::write_tx(&db).map_err(db_err)?;
        share::delete(&tx, c.id, &msg).map_err(db_err)?;
        tx.commit().map_err(db_err)?;
        Ok(Response::Ok)
    }

    /// 공유 끝 — 관람자 PC(C): 대화를 가진 PC 가 뺐다(사본은 읽기 전용으로 남는다). 대화를 가진 PC(A): 관람자가 나갔다(관람자 행을 지우고 원 상대에게 관람자 수).
    pub(super) fn inbound_share_end(&self, peer: &Peer, conv: String) -> io::Result<Response> {
        let own = {
            let db = self.db();
            if let Some(c) = chat::find(&db, &conv, peer.id)
                .map_err(db_err)?
                .filter(|c| c.dir == Dir::Shared)
            {
                share::end(&db, c.id).map_err(db_err)?;
                return Ok(Response::Ok);
            }
            share::own_by_uuid(&db, &conv).map_err(db_err)?
        };
        if let Some(c) = own {
            self.unshare(&c, peer.id, false)?;
        }
        Ok(Response::Ok)
    }

    /// 원 상대(B)가 받은 관람자 수 — 그 상대 소유 대화(사본 아님)에만. 바뀌었을 때만 알린다(재전송은 알림 없음).
    pub(super) fn inbound_share_notice(&self, peer: &Peer, conv: String, n: u32) -> io::Result<Response> {
        if usize::try_from(n).map_or(true, |n| n > share::MAX_WATCHERS) {
            return Ok(Response::error("bad_request"));
        }
        let (c, changed) = {
            let db = self.db();
            let Some(c) = chat::find_own(&db, &conv, peer.id).map_err(db_err)? else {
                return Ok(Response::error("unknown"));
            };
            let changed = share::set_watched(&db, c.id, i64::from(n)).map_err(db_err)?;
            (c, changed)
        };
        if changed {
            self.notify(
                c.id,
                peer,
                c.agent.clone(),
                MessageKind::Share,
                format!("관람자 {n}"),
            );
        }
        Ok(Response::Ok)
    }
}
