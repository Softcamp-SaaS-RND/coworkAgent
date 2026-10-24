//! 요청 상태 — 바꾸기 · 상대에 알리기 · 받기.

use super::*;

impl Chat {
    /// 받은 요청의 상태 · 메모를 바꾼다 — 받은 쪽 사람만, 지금 상태 말고 네 상태 어디로든(`ReqState::can_become`). 저장(`saved`)도 같은 트랜잭션.
    /// 상태와 보내기 행(`ReqState`)은 한 트랜잭션 — 쓴 쪽이 꺼져 있으면 줄이 나중에 보낸다. 바뀐 글과 경고(상대가 거절했을 때만).
    pub async fn set_req_state(
        &self,
        id: i64,
        state: ReqState,
        note: &str,
        saved: Option<bool>,
    ) -> io::Result<(Message, Option<String>)> {
        let note = chat::clean_req_note(note)
            .map_err(|m| io::Error::new(io::ErrorKind::InvalidInput, m))?;
        let conv = {
            let db = self.db();
            let m = chat::message_by_id(&db, id)
                .map_err(db_err)?
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "없는 메시지"))?;
            if m.author != Author::Human || m.mine || !m.request {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "상태는 받은 요청만 바꾼다",
                ));
            }
            chat::by_id(&db, m.conversation_id)
                .map_err(db_err)?
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "없는 대화"))?
        };
        Self::writable(&conv)?;
        // 확인 · 쓰기는 대화 차례 안에서 — 두 창이 함께 바꿔도 옮김 규칙이 지켜지고 상대에 같은 순서로 간다.
        let lock = self.turn(conv.id);
        let _turn = lock.lock().await;
        let (m, req, staged) = {
            let db = self.db();
            let tx = an_core::db::write_tx(&db).map_err(db_err)?;
            let now = chat::message_by_id(&tx, id)
                .map_err(db_err)?
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "없는 메시지"))?;
            // 다시 알리기 — 요청자에 알리지 못한 상태(오래 닿지 않아 포기 · 요청자가 거절)는 같은 상태로 다시 보낼 수 있다.
            let again = now.req_state == Some(state) && now.refused.is_some();
            if !again && !now.req_state.unwrap_or(ReqState::Wait).can_become(state) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "이미 그 상태다",
                ));
            }
            chat::set_req_state(&tx, id, state, note.as_deref()).map_err(db_err)?;
            chat::set_refused(&tx, id, None).map_err(db_err)?;
            if let Some(on) = saved {
                chat::set_saved(&tx, id, on).map_err(db_err)?;
            }
            let req = Request::ReqState {
                conv: conv.uuid.clone(),
                msg: now.uuid.clone(),
                state: state.as_str().into(),
                note: note.as_deref().map(chat::note_to_wire),
            };
            // 보내기 행은 그 요청 글에 잇는다 — 포기하면 그 글에 "요청자에게 알리지 못함"(`give_up`).
            let staged = Self::stage(&tx, &conv, Some(id), REQ_STATE, &req)?;
            tx.commit().map_err(db_err)?;
            let m = chat::message_by_id(&db, id).map_err(db_err)?.unwrap_or(now);
            (m, req, staged)
        };
        let warning = match self.dispatch(&conv, staged, &req).await {
            Ok(Shipped::Answered(Response::Ok) | Shipped::Kept) => None,
            Ok(Shipped::Answered(other)) => {
                // 거절은 그 글에 남긴다 — 새로 고친 뒤에도 "다시 알리기" 로 복구한다.
                self.req_state_refused(id, &other);
                Some(refused(other).to_string())
            }
            Err(e) => Some(e.to_string()),
        }
        .map(|e| format!("이 PC 에는 바꿨다 — 상대에 알리지 못했다({e})"));
        let m = chat::message_by_id(&self.db(), id)
            .map_err(db_err)?
            .unwrap_or(m);
        Ok((m, warning))
    }

    /// 요청자가 상태 알림을 거절했다 — 그 받은 요청 글에 거절 코드(화면 "다시 알리기").
    pub(super) fn req_state_refused(&self, msg_id: i64, resp: &Response) {
        let code = match resp {
            Response::Error { code } => an_net::proto::clean_code(code),
            _ => "unknown",
        };
        let _ = chat::set_refused(&self.db(), msg_id, Some(code));
    }

    /// 상대가 내 요청에 단 상태 — 그 상대 소유 대화의 내 요청 사람 글에만(받은 쪽이 정한다 — 옮김 검사 없이 받은 대로, 줄이 순서를
    /// 지킨다).
    pub(super) fn inbound_req_state(
        &self,
        peer: &Peer,
        conv: String,
        msg: String,
        state: String,
        note: Option<String>,
    ) -> io::Result<Response> {
        let (Some(state), Ok(note)) = (
            ReqState::parse(&state),
            chat::clean_req_note(&chat::note_from_wire(note.as_deref().unwrap_or(""))),
        ) else {
            return Ok(Response::error("bad_request"));
        };
        let (cid, agent, text) = {
            let db = self.db();
            let Some(c) = chat::find_open(&db, &conv, peer.id).map_err(db_err)? else {
                return Ok(Response::error("unknown"));
            };
            let mine = chat::message_by_uuid(&db, &msg)
                .map_err(db_err)?
                .filter(|m| {
                    m.conversation_id == c.id && m.author == Author::Human && m.mine && m.request
                });
            let Some(m) = mine else {
                return Ok(Response::error("unknown"));
            };
            // 같은 상태 · 메모의 재전송(응답 유실 · 다시 알리기)은 시각 · 알림을 다시 만들지 않는다.
            if m.req_state == Some(state) && m.req_note == note {
                return Ok(Response::Ok);
            }
            // 상태와 내 활동 기록(안 본 것)을 한 트랜잭션으로 — 기록만 빠지면 재전송이 "같은 상태" 로 일찍 돌아가 영영 남지 않는다.
            let tx = an_core::db::write_tx(&db).map_err(db_err)?;
            chat::set_req_state(&tx, m.id, state, note.as_deref()).map_err(db_err)?;
            chat::note_req_state(&tx, m.id).map_err(db_err)?;
            tx.commit().map_err(db_err)?;
            (c.id, c.agent, note)
        };
        let label = match state {
            ReqState::Wait => "요청 대기",
            ReqState::Accepted => "요청 수용",
            ReqState::Rejected => "요청 반려",
            ReqState::Done => "요청 완료",
        };
        let shown = text.map_or(label.to_string(), |n| format!("{label} — {n}"));
        self.notify(cid, peer, agent, MessageKind::AskState, preview(&shown));
        Ok(Response::Ok)
    }
}
