//! CLI 트랜잭션 — 키 · 이어 받기 · 감시.

use super::*;

impl Chat {
    /// CLI 트랜잭션 감시 설정 — 시험용(상태를 묻는 간격 · 멈춤까지, 초).
    pub fn set_watch(&self, every: i64, stall_after: i64) {
        *an_core::sync::lock(&self.watch) = Watch { every, stall_after };
    }

    pub fn tx_by_key(&self, key: &str) -> io::Result<Option<cli_tx::Tx>> {
        cli_tx::by_key(&self.db(), key).map_err(db_err)
    }

    pub fn tx_of_message(&self, message_id: i64) -> io::Result<Option<cli_tx::Tx>> {
        cli_tx::by_message(&self.db(), message_id).map_err(db_err)
    }

    /// 키 있는 CLI 보내기 잠금 — 같은 키 확인부터 보내기까지 잡는다.
    pub async fn keyed(&self) -> tokio::sync::MutexGuard<'_, ()> {
        self.keyed.lock().await
    }

    /// 이어 받기(`wait`) — 멈춤을 풀고 다시 감시한다.
    pub fn tx_resume(&self, id: i64) -> io::Result<()> {
        cli_tx::resume(&self.db(), id).map_err(db_err)
    }

    /// 트랜잭션의 지금 상태 — 끝나지 않았고 물을 때면 상대에 상태를 묻는 작업을 띄운다(돌아오기를 기다리지 않음 — 다음 조회가 결과를 본다).
    pub fn tx_view(&self, id: i64) -> io::Result<Option<TxView>> {
        let found = {
            let db = self.db();
            let Some(tx) = cli_tx::by_id(&db, id).map_err(db_err)? else {
                return Ok(None);
            };
            let Some(message) = chat::message_by_id(&db, tx.message_id).map_err(db_err)? else {
                return Ok(None);
            };
            let reply = chat::messages(&db, message.conversation_id)
                .map_err(db_err)?
                .into_iter()
                .find(|r| r.author == Author::Agent && r.reply_to == Some(message.id));
            (tx, message, reply)
        };
        let (tx, message, reply) = found;
        let outbox = self.outbox_of(&message);
        let progress = self.progress_of(&message);
        let state = match (&reply, &outbox) {
            (Some(r), _) if r.state == State::Done => "answered",
            (Some(r), _) if r.state == State::Error => "failed",
            _ if message.state == State::Error => "failed",
            _ if tx.stalled_at.is_some() => "stalled",
            (_, Some(o)) if o.state == "failed" => "failed",
            (_, Some(_)) => "sending",
            _ => match progress.as_ref().map(|p| p.state.as_str()) {
                Some("queued") => "queued",
                Some("running") => "running",
                _ => "sent",
            },
        };
        if matches!(state, "sending" | "sent" | "queued" | "running") {
            self.watch_tx(&tx, &message, state == "sending");
        }
        Ok(Some(TxView {
            tx,
            state,
            message,
            reply,
            progress,
            outbox,
        }))
    }

    /// 상대에 닿는지 본다 — 보관 중이면 그 상대 줄이 닿지 않고 있는지, 아니면 상태 질의. 닿으면 닿지 않음을 지우고, 아니면 시작 시각부터
    /// 멈춤까지 센다.
    fn watch_tx(&self, tx: &cli_tx::Tx, message: &Message, sending: bool) {
        let w = *an_core::sync::lock(&self.watch);
        let due = cli_tx::due(&self.db(), tx.id, w.every);
        if !matches!(due, Ok(true)) || !an_core::sync::lock(&self.watching).insert(tx.id) {
            return;
        }
        let Some(me) = self.me.upgrade() else {
            return;
        };
        let _ = cli_tx::asked(&self.db(), tx.id);
        let (id, message) = (tx.id, message.clone());
        tokio::spawn(async move {
            let reached = if sending {
                let peer = chat::by_id(&me.db(), message.conversation_id)
                    .ok()
                    .flatten()
                    .map(|c| c.peer_id);
                peer.is_some_and(|p| matches!(outbox::failing(&me.db(), p), Ok(false)))
            } else {
                me.ask_status(&message).await.is_ok()
            };
            let marked = if reached {
                cli_tx::reached(&me.db(), id).map(|_| false)
            } else {
                cli_tx::unreachable(&me.db(), id, w.stall_after)
            };
            match marked {
                Ok(true) => log::info!(
                    "CLI 트랜잭션 {id} — 상대에 {}초 닿지 않아 멈춤",
                    w.stall_after
                ),
                Ok(false) => {}
                Err(e) => log::warn!("CLI 트랜잭션 {id} 감시 기록 실패: {e}"),
            }
            an_core::sync::lock(&me.watching).remove(&id);
        });
    }
}
