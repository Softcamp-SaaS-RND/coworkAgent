//! 보낼 함 — 단계 · 전달 · 재시도 · 상태 묻기.

use super::*;

impl Chat {
    /// 보내기 설정 — 시험용(재시도 첫 간격 · 최대 간격 · 포기 한도, 초).
    pub fn set_retry(&self, base: i64, max: i64, give_up: i64) {
        *an_core::sync::lock(&self.retry) = Retry { base, max, give_up };
    }

    /// 보내기 행을 적는다(`inflight` — 곧 [`Chat::dispatch`] 가 보낸다. 그 전에 꺼지면 다시 켤 때 줄이 보낸다). `conn` 은 글을 적는 트랜잭션일
    /// 수 있다 — 글과 행이 함께 남거나 함께 없다.
    pub(super) fn stage(
        conn: &Connection,
        conv: &Conversation,
        msg_id: Option<i64>,
        kind: &'static str,
        req: &Request,
    ) -> io::Result<i64> {
        let frame = serde_json::to_string(req).map_err(io::Error::other)?;
        outbox::add(
            conn,
            conv.peer_id,
            Some(conv.id),
            msg_id,
            kind,
            &frame,
            None,
            "inflight",
        )
        .map_err(db_err)
    }

    /// 상대에게 대화 프레임을 보낸다 — outbox 에 먼저 적고, 그 상대 줄에 앞선 행이 없으면 바로 한 번. 응답이면 행을 지우고 응답을, 전송
    /// 오류 · 앞선 행이면 줄에 남기고 `Kept`(상대 줄이 ack 까지 다시 보내고 [`settle`](Self::settle) 이 뒷일을 한다).
    pub(super) async fn ship(
        &self,
        conv: &Conversation,
        msg_id: Option<i64>,
        kind: &'static str,
        req: &Request,
    ) -> io::Result<Shipped> {
        let id = Self::stage(&self.db(), conv, msg_id, kind, req)?;
        self.dispatch(conv, id, req).await
    }

    /// [`stage`](Self::stage) 로 이미 적은 행을 보낸다 — 글과 행을 한 DB 트랜잭션으로 적은 곳(사람 요청 · 답)이 커밋 뒤 부른다.
    pub(super) async fn dispatch(&self, conv: &Conversation, id: i64, req: &Request) -> io::Result<Shipped> {
        // 방금 남긴 글을 관람자에게(글을 남기는 길목).
        self.fan_share(conv.id);
        let peer_id = conv.peer_id;
        let behind = {
            let db = self.db();
            // 같은 대화 순서만 지킨다. 진행 · 멈춤 알림은 앞질러도 된다 — 답보다 늦게 닿은 진행은 받는 쪽이 버린다(`unknown`).
            // 공유 프레임은 다른 상대(관람자)의 줄이다 — 꺼진 관람자의 행이 원 상대로 가는 보내기를 밀어내지 않게.
            // 닿지 않고 있는 상대면 바로 보내지 않는다 — 화면이 연결 시간 초과(최대 10초)를 기다리지 않게, 상대가 돌아오면(Ping · 요청) 줄이 보낸다.
            outbox::has_before(&db, conv.id, id, &[PROGRESS, RELAY, SHARE, SHARE_NOTICE])
                .map_err(db_err)?
                || outbox::failing(&db, peer_id).map_err(db_err)?
        };
        let me = self.me.upgrade().filter(|_| !behind);
        let Some(me) = me else {
            outbox::set_state(&self.db(), id, "pending").map_err(db_err)?;
            self.kick(peer_id);
            return Ok(Shipped::Kept);
        };
        // 보내기 · 뒷정리는 따로 띄운 작업에서 — 부른 쪽(HTTP 처리기 등)이 기다리다 취소돼도 행이 `inflight` 로 남지 않는다.
        let req = req.clone();
        let sent = tokio::spawn(async move {
            let got = me.send_to(peer_id, &req).await;
            match &got {
                Ok(resp) => {
                    me.acked(id, resp);
                }
                Err(e) => me.backoff(id, e),
            }
            // 그새 뒤에 선 행은 줄이 이어 보낸다.
            me.kick(peer_id);
            got
        })
        .await;
        Ok(match sent {
            Ok(Ok(resp)) => Shipped::Answered(resp),
            _ => Shipped::Kept,
        })
    }

    /// 줄에만 넣는다(바로 보내지 않음) — `key` 가 같은 기다리는 행이 있으면 그 행을 바꾼다(진행 — 마지막 값만).
    pub(super) fn enqueue(
        &self,
        conv: &Conversation,
        msg_id: Option<i64>,
        kind: &'static str,
        req: &Request,
        key: Option<&str>,
    ) {
        let added = serde_json::to_string(req)
            .map_err(io::Error::other)
            .and_then(|frame| {
                let db = self.db();
                outbox::add(
                    &db,
                    conv.peer_id,
                    Some(conv.id),
                    msg_id,
                    kind,
                    &frame,
                    key,
                    "pending",
                )
                .map_err(db_err)
            });
        match added {
            Ok(_) => self.kick(conv.peer_id),
            Err(e) => log::warn!("보낼 글 보관 실패({kind}): {e}"),
        }
        self.fan_share(conv.id);
    }

    /// 페어링된 상대에 요청 하나(키 고정) — 응답 내용은 부르는 쪽이 본다.
    async fn send_to(&self, peer_id: i64, req: &Request) -> io::Result<Response> {
        let peer = self.paired_peer(peer_id)?;
        match self.node.send(addr(&peer)?, Some(peer.spki), req).await?.0 {
            // 상대가 잠시 처리하지 못했다(DB 잠금 등) — 거절이 아니라 다시 보낼 일.
            Response::Error { code } if code == RETRY => Err(io::Error::other(
                "상대가 잠시 처리하지 못했다 — 다시 보낸다",
            )),
            resp => Ok(resp),
        }
    }

    /// 전송 오류 — 간격을 늘려 줄로 되돌리거나, 기다림 합이 한도를 넘으면 실패 보관 + [`give_up`](Self::give_up).
    fn backoff(&self, id: i64, e: &io::Error) {
        let r = *an_core::sync::lock(&self.retry);
        let item = match outbox::get(&self.db(), id) {
            Ok(Some(it)) => it,
            Ok(None) => return,
            Err(err) => return log::warn!("보낼 글 {id} 읽기 실패: {err}"),
        };
        let delay = r
            .base
            .saturating_mul(1i64 << item.tries.clamp(0, 20))
            .min(r.max);
        // 직전 간격까지만 센다(앞당긴 재시도 · 앱을 꺼 둔 시간은 한도에 넣지 않는다).
        let prev = r
            .base
            .saturating_mul(1i64 << (item.tries - 1).clamp(0, 20))
            .min(r.max);
        let why = e.to_string();
        let waited = match outbox::retry(&self.db(), id, delay, prev, &why) {
            Ok(w) => w,
            Err(err) => return log::warn!("보낼 글 {id} 재시도 기록 실패: {err}"),
        };
        if waited < r.give_up {
            return;
        }
        // 그 상대에 한도만큼 닿지 않았다 — 줄에 기다리는 글도 함께 포기한다(뒤 글이 하나씩 24시간을 다시 기다리지 않게). 지금 보내는 중
        // (`inflight`)인 글은 그 결과가 정한다 — 닿은 글을 "보내지 못함" 으로 덮지 않게.
        let queued = outbox::queued_for(&self.db(), item.peer_id).unwrap_or_default();
        log::warn!(
            "상대 {} 에 {waited}초 동안 닿지 않아 보내기 포기 — {}건: {why}",
            item.peer_id,
            queued.len()
        );
        let mut unshared = Vec::new();
        for it in queued {
            // 컨텍스트 처리 결과는 포기하지 않는다 — 요청자 구분선이 `sent` 에 영영 묶이지 않게, 돌아오면 줄이 보낸다.
            if it.kind == CTX_DONE {
                continue;
            }
            // 나가기 · 기록 지우기도 — 돌아오면 줄이 보낸다(상대 대화가 읽기 전용이 · 비워지게).
            if it.kind == LEAVE || it.kind == CLEAR {
                continue;
            }
            // 공유 프레임 — 관람자에 한도만큼 닿지 않으면 그 관람을 끝낸다(행을 지우고 관람자에서 뺀다 — 꺼진 관람자 몫이 끝없이 쌓이지 않게).
            // 원 상대 알림은 버린다. 사본은 관람자 PC 에 남고, 빼기 표시 한 행이 돌아오면 간다(다시 넣으면 처음부터).
            if matches!(it.kind.as_str(), SHARE | SHARE_NOTICE) {
                // 빼기 표시(`ShareEnd`)는 남긴다 — 돌아온 관람자 사본이 "공유 해제됨" 이 되게(대화마다 한 행).
                if matches!(
                    serde_json::from_str::<Request>(&it.frame),
                    Ok(Request::ShareEnd { .. })
                ) {
                    continue;
                }
                if let Err(err) = outbox::remove(&self.db(), it.id) {
                    log::warn!("보낼 공유 글 {} 지우기 실패: {err}", it.id);
                }
                if it.kind == SHARE
                    && let Some(conv) = it.conv_id
                    && !unshared.contains(&conv)
                {
                    unshared.push(conv);
                }
                continue;
            }
            // 편집 · 삭제 반영은 실패 보관과 '상대 반영 실패' 기록을 한 트랜잭션으로 — 하나만 남지 않게(실패하면 행은 줄에 남아 다시 포기한다).
            if matches!(it.kind.as_str(), outbox::EDIT | outbox::DELETE) {
                let failed = (|| {
                    let db = self.db();
                    let tx = an_core::db::write_tx(&db).map_err(db_err)?;
                    outbox::fail(&tx, it.id, &why).map_err(db_err)?;
                    Self::note_sync_failed(&tx, &it)?;
                    tx.commit().map_err(db_err)
                })();
                if let Err(err) = failed {
                    log::warn!("보낼 글 {} 실패 보관 기록 실패: {err}", it.id);
                }
                continue;
            }
            if let Err(err) = outbox::fail(&self.db(), it.id, &why) {
                log::warn!("보낼 글 {} 실패 보관 기록 실패: {err}", it.id);
            }
            self.give_up(&it);
        }
        for conv in unshared {
            let found = chat::by_id(&self.db(), conv);
            match found {
                Ok(Some(c)) => {
                    if let Err(err) = self.unshare(&c, item.peer_id, true) {
                        log::warn!("대화 {conv} 관람자 {} 빼기 실패: {err}", item.peer_id);
                    }
                }
                Ok(None) => {}
                Err(err) => log::warn!("대화 {conv} 읽기 실패: {err}"),
            }
        }
    }

    /// 상대 줄을 깨운다(없으면 띄운다 — 상대마다 tokio 작업 하나).
    pub(super) fn kick(&self, peer_id: i64) {
        let Some(me) = self.me.upgrade() else {
            return;
        };
        let mut lanes = an_core::sync::lock(&self.lanes);
        // 줄 작업이 끝났으면(panic) 다시 띄운다 — 죽은 작업을 깨우며 그 상대가 멈추지 않게.
        let wake = match lanes.get(&peer_id) {
            Some((wake, h)) if !h.is_finished() => wake.clone(),
            _ => {
                let wake = Arc::new(tokio::sync::Notify::new());
                let handle = tokio::spawn(me.lane(peer_id, wake.clone()));
                lanes.insert(peer_id, (wake.clone(), handle));
                wake
            }
        };
        wake.notify_one();
    }

    /// 상대 줄 — 맨 앞 행부터 하나씩(앞 행이 못 가면 뒤도 기다린다 — 대화 순서 보존). 바로 보내는 중(`inflight`)이면 끝나고 깨울 때까지.
    async fn lane(self: Arc<Self>, peer_id: i64, wake: Arc<tokio::sync::Notify>) {
        loop {
            let head = outbox::head(&self.db(), peer_id);
            let item = match head {
                Ok(Some(it)) if it.state == "pending" => it,
                Ok(_) => {
                    wake.notified().await;
                    continue;
                }
                Err(e) => {
                    log::warn!("상대 {peer_id} 보낼 글 읽기 실패: {e}");
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    continue;
                }
            };
            if item.due_in > 0 {
                let wait = Duration::from_secs(item.due_in.unsigned_abs());
                tokio::select! {
                    _ = tokio::time::sleep(wait) => {}
                    _ = wake.notified() => {}
                }
                continue;
            }
            self.deliver(item).await;
        }
    }

    async fn deliver(&self, item: outbox::Item) {
        let req: Request = match serde_json::from_str(&item.frame) {
            Ok(r) => r,
            Err(e) => {
                log::warn!("보낼 글 {} 프레임 형식 오류 — 보관: {e}", item.id);
                let _ = outbox::fail(&self.db(), item.id, "프레임 형식 오류");
                return;
            }
        };
        // 결과를 먼저 받는다 — `if let` 조건의 DB 잠금은 블록 끝까지 살아 있다(await 를 넘기면 안 된다).
        let marked = outbox::set_state(&self.db(), item.id, "inflight");
        if let Err(e) = marked {
            log::warn!("보낼 글 {} 상태 기록 실패: {e}", item.id);
            tokio::time::sleep(Duration::from_secs(5)).await;
            return;
        }
        // 컨텍스트 프레임 — 이 종류를 모르는 상대(0.4.0)는 응답 없이 연결을 끊어 24시간 다시 보내게 된다. 먼저 Ping 으로 묻는다.
        if matches!(item.kind.as_str(), CTX | CTX_DONE) {
            match self.node.knows_context(item.peer_id).await {
                Ok(true) => {}
                Ok(false) => {
                    let resp = Response::error(CTX_UNSUPPORTED);
                    if self.acked(item.id, &resp) {
                        self.settle(&item, &resp);
                    }
                    return;
                }
                Err(e) => return self.backoff(item.id, &e),
            }
        }
        // 공유 프레임도 같다 — 모르는 상대면 관람자를 빼거나(`settle`) 원 상대 알림을 버린다.
        // ponytail: 공유 글마다 Ping 한 번 더 — 이력이 길어 느리면 상대별로 아는지 기억한다.
        if matches!(item.kind.as_str(), SHARE | SHARE_NOTICE) {
            let known = match self.node.knows_share(item.peer_id).await {
                Ok(k) => k,
                // Ping 을 거절했다 = 상대가 이 PC 를 페어링된 상대로 보지 않는다(`not_paired`).
                Err(e) if e.kind() == io::ErrorKind::PermissionDenied => {
                    let resp = Response::error("not_paired");
                    if self.acked(item.id, &resp) {
                        self.settle(&item, &resp);
                    }
                    return;
                }
                Err(e) => return self.backoff(item.id, &e),
            };
            if !known {
                let resp = Response::error("unsupported");
                if self.acked(item.id, &resp) {
                    self.settle(&item, &resp);
                }
                return;
            }
        }
        // 나가기도 — 모르는 상대(이전 버전)에는 보내지 않고 행을 버린다(그쪽은 보낼 때 `left` 를 받는다). Ping 거절 = 페어링 아님 — 버린다.
        if item.kind == LEAVE {
            match self.node.knows_leave(item.peer_id).await {
                Ok(true) => {}
                Ok(false) => {
                    log::info!("상대 {} 는 나가기를 모른다 — 보내지 않는다", item.peer_id);
                    let _ = self.acked(item.id, &Response::Ok);
                    return;
                }
                Err(e) if e.kind() == io::ErrorKind::PermissionDenied => {
                    let _ = self.acked(item.id, &Response::error("not_paired"));
                    return;
                }
                Err(e) => return self.backoff(item.id, &e),
            }
        }
        // 기록 지우기 — 그 사이 이전 버전이 된 상대에는 보내지 않는다(상대 쪽은 그대로 — 로그).
        if item.kind == CLEAR {
            match self.node.knows_clear(item.peer_id).await {
                Ok(true) => {}
                Ok(false) => {
                    // 내 활동에 "반영하지 못함 — 기록 삭제" 로 남긴다(`acked` 의 반영 실패 기록).
                    log::warn!("상대 {} 는 기록 지우기를 모른다 — 보내지 않는다(상대 쪽 기록은 그대로)", item.peer_id);
                    let _ = self.acked(item.id, &Response::error("unsupported"));
                    return;
                }
                Err(e) if e.kind() == io::ErrorKind::PermissionDenied => {
                    let _ = self.acked(item.id, &Response::error("not_paired"));
                    return;
                }
                Err(e) => return self.backoff(item.id, &e),
            }
        }
        // 빠진 관람자 — 빼기 · 나가기 · 포기 뒤 남은(보내던 중 · 다시 보낼) 공유 글은 보내지 않고 지운다. 빼기 표시(`ShareEnd`)는 보낸다.
        // 확인은 능력 확인(Ping) 뒤, 보내기 바로 앞에서. ponytail: 확인 ~ 보내기 사이(ms)에 빠지면 그 한 글은 간다 — 막으려면
        // 관람자별 잠금으로 빼기와 줄 세우거나 프레임에 공유 세대를 싣는다.
        if item.kind == SHARE && !matches!(req, Request::ShareEnd { .. }) && !self.watching(&item) {
            let _ = self.acked(item.id, &Response::Ok);
            return;
        }
        match self.send_to(item.peer_id, &req).await {
            Ok(resp) => {
                if self.acked(item.id, &resp) {
                    self.settle(&item, &resp);
                }
            }
            Err(e) => self.backoff(item.id, &e),
        }
    }

    /// 응답을 받은 행을 지운다 — 편집 · 삭제 반영 · 기록 지우기가 거절됐으면 그 대화의 '상대 반영 실패' 기록과 행 지우기를 한 트랜잭션으로(그 사이에
    /// 꺼져도 실패가 사라지지 않게). 그 트랜잭션이 실패하면(어느 종류의 행이든) 행을 줄로 되돌려 다시 보낸다 — 거짓, 뒷일(`settle`)은 다시 받은
    /// 응답이 한다(프레임은 uuid 로 멱등이라 다시 보내도 상대에서 한 번만 적용된다).
    fn acked(&self, id: i64, resp: &Response) -> bool {
        let noted = (|| {
            let db = self.db();
            let tx = an_core::db::write_tx(&db).map_err(db_err)?;
            let item = outbox::get(&tx, id).map_err(db_err)?;
            let sync = item
                .as_ref()
                .is_some_and(|it| matches!(it.kind.as_str(), outbox::EDIT | outbox::DELETE | CLEAR));
            if sync && !matches!(resp, Response::Ok) {
                Self::note_sync_failed(&tx, item.as_ref().expect("위에서 확인"))?;
            }
            outbox::remove(&tx, id).map_err(db_err)?;
            tx.commit().map_err(db_err)?;
            Ok::<_, io::Error>(sync)
        })();
        match noted {
            Ok(_) => true,
            Err(e) => {
                log::warn!("보낸 글 {id} 정산 실패 — 다시 보낸다: {e}");
                self.backoff(id, &e);
                false
            }
        }
    }

    fn conv_of(&self, msg_id: i64) -> Option<Conversation> {
        let db = self.db();
        let m = chat::message_by_id(&db, msg_id).ok().flatten()?;
        chat::by_id(&db, m.conversation_id).ok().flatten()
    }

    /// ack 뒤 처리 — 줄이 나중에 보낸 글의 응답을 그 자리의 처리와 같게(거절이면 글 실패 · 자동 왕복 끔, 미러 `Ok` 면 완료).
    fn settle(&self, item: &outbox::Item, resp: &Response) {
        let ok = matches!(resp, Response::Ok);
        // 편집 · 삭제 반영 — 거절 기록은 응답 정산(`acked`)이 행 지우기와 함께 했다.
        if matches!(item.kind.as_str(), outbox::EDIT | outbox::DELETE) {
            return;
        }
        // 공유 프레임 — 실어 나르는 글이 없다.
        if matches!(item.kind.as_str(), SHARE | SHARE_NOTICE) {
            if !ok {
                self.share_refused(item, resp);
            }
            return;
        }
        let Some(msg_id) = item.msg_id else {
            if !ok {
                log::info!(
                    "상대 {} 가 {} 를 받지 않았다: {resp:?}",
                    item.peer_id,
                    item.kind
                );
            }
            return;
        };
        match (item.kind.as_str(), ok) {
            // 상대가 멈춘 뒤 닿은 넘김 — 멈춤 알림이 먼저 온 것과 같게(내 답은 실패가 아니다).
            (RELAY_STEP, false) if matches!(resp, Response::Error { code } if code == STOPPED) => {
                if let Some(conv) = self.conv_of(msg_id) {
                    self.relay_stopped(&conv, msg_id);
                }
            }
            (MESSAGE | RELAY_OPEN | RELAY_STEP, false) => {
                self.refused_by_peer(msg_id, item.peer_id, resp);
                if item.kind == MESSAGE {
                    return;
                }
                if let Some(conv) = self.conv_of(msg_id) {
                    let _ = chat::relay_on(&self.db(), conv.id, false, false);
                    if item.kind == RELAY_STEP {
                        let focus = matches!(resp, Response::Error { code } if code == FOCUS);
                        self.relay_end(
                            &conv,
                            if focus {
                                "상대가 집중 중"
                            } else {
                                "상대가 거절"
                            },
                        );
                    }
                }
            }
            (NOTE, true) => {
                let _ = chat::set_state(&self.db(), msg_id, State::Done);
            }
            (NOTE, false) => {
                let _ = chat::set_state(&self.db(), msg_id, State::Error);
            }
            (REPLY | REPLY_POST | RELAY_LAST, false) => {
                let _ = chat::set_state(&self.db(), msg_id, State::Undelivered);
            }
            (REQ_STATE, false) => self.req_state_refused(msg_id, resp),
            (CTX, false) => {
                let code = match resp {
                    Response::Error { code } => an_net::proto::clean_code(code).to_string(),
                    _ => "unknown".into(),
                };
                self.context_refused(msg_id, &code);
            }
            (_, false) => log::info!(
                "상대 {} 가 {} 를 받지 않았다: {resp:?}",
                item.peer_id,
                item.kind
            ),
            _ => {}
        }
    }

    /// 다시 보내기 포기(한도) — 내 글은 "보내지 못함"(실패 + `unreachable`), 답은 "답 미전달", 자동 왕복은 멈춘다.
    fn give_up(&self, item: &outbox::Item) {
        let Some(msg_id) = item.msg_id else {
            return;
        };
        match item.kind.as_str() {
            MESSAGE | RELAY_OPEN | RELAY_STEP | NOTE => {
                {
                    let db = self.db();
                    let _ = chat::set_state(&db, msg_id, State::Error);
                    let _ = chat::set_refused(&db, msg_id, Some(UNREACHABLE));
                }
                let relay = matches!(item.kind.as_str(), RELAY_OPEN | RELAY_STEP);
                if let Some(conv) = self.conv_of(msg_id).filter(|_| relay) {
                    let _ = chat::relay_on(&self.db(), conv.id, false, false);
                    if item.kind == RELAY_STEP {
                        self.relay_end(&conv, "상대에 닿지 않음");
                    }
                }
            }
            REPLY | REPLY_POST | RELAY_LAST => {
                let _ = chat::set_state(&self.db(), msg_id, State::Undelivered);
            }
            CTX => self.context_refused(msg_id, UNREACHABLE),
            // 받은 요청의 상태를 요청자에 알리지 못했다 — 그 글에 표시, 받은 사람이 "다시 알리기".
            REQ_STATE => {
                let _ = chat::set_refused(&self.db(), msg_id, Some(UNREACHABLE));
            }
            _ => {}
        }
    }

    /// 상대에서 무엇이든 받았다 — 그 상대 줄에 남은 글을 지금 보낸다(돌아온 증거).
    pub(super) fn peer_back(&self, peer_id: i64) {
        let waiting = outbox::head(&self.db(), peer_id);
        match waiting {
            Ok(Some(_)) => {
                let hurried = outbox::hurry(&self.db(), peer_id);
                if let Err(e) = hurried {
                    log::warn!("상대 {peer_id} 보낼 글 앞당기기 실패: {e}");
                }
                self.kick(peer_id);
            }
            Ok(None) => {}
            Err(e) => log::warn!("상대 {peer_id} 보낼 글 읽기 실패: {e}"),
        }
    }

    /// 내 글의 보관 상태(웹UI — 보내는 중 · 보내지 못함). 상대에 닿은 글은 `None`.
    pub fn outbox_of(&self, m: &Message) -> Option<outbox::Item> {
        if !m.mine {
            return None;
        }
        outbox::for_message(&self.db(), m.id).ok().flatten()
    }

    /// 상태 질의 — 보낸 쪽이 자기 실행 요청의 지금 상태를 상대에 묻는다(바로 — 보관하지 않음). `queued` · `running` 이면 진행을 갱신한다.
    pub async fn ask_status(&self, m: &Message) -> io::Result<Response> {
        let conv = chat::by_id(&self.db(), m.conversation_id)
            .map_err(db_err)?
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "없는 대화"))?;
        let req = Request::Status {
            conv: conv.uuid.clone(),
            msg: m.uuid.clone(),
        };
        let resp = self.send_to(conv.peer_id, &req).await?;
        self.peer_back(conv.peer_id);
        if let Response::Status { state, ahead } = &resp
            && matches!(state.as_str(), "queued" | "running")
            && m.state == State::Sent
        {
            let mut p = an_core::sync::lock(&self.progress);
            let detail = p.get(&m.id).and_then(|x| x.detail.clone());
            p.insert(
                m.id,
                Progress {
                    state: state.clone(),
                    ahead: if state == "running" { 0 } else { *ahead },
                    detail: if state == "running" { detail } else { None },
                },
            );
        }
        Ok(resp)
    }

    /// 상대가 묻는 자기 실행 요청의 상태 — 그 상대 대화의 상대 실행 요청 글만. 끝났으면 `done`(그 상대에 남은 답이 있으면 지금 보낸다).
    pub(super) fn inbound_status(&self, peer: &Peer, conv: String, msg: String) -> io::Result<Response> {
        let found = {
            let db = self.db();
            match chat::find_own(&db, &conv, peer.id).map_err(db_err)? {
                Some(c) => chat::message_by_uuid(&db, &msg)
                    .map_err(db_err)?
                    .filter(|m| m.conversation_id == c.id && !m.mine && m.exec),
                None => None,
            }
        };
        let Some(m) = found else {
            return Ok(Response::Status {
                state: "unknown".into(),
                ahead: 0,
            });
        };
        let (state, ahead) = match m.state {
            State::Queued => {
                let (_, _, waiting) = self.run_queue();
                let ahead = waiting.iter().position(|&id| id == m.id).unwrap_or(0);
                ("queued", u32::try_from(ahead).unwrap_or(u32::MAX))
            }
            State::Running => ("running", 0),
            _ => {
                self.peer_back(peer.id);
                ("done", 0)
            }
        };
        Ok(Response::Status {
            state: state.into(),
            ahead,
        })
    }
}
