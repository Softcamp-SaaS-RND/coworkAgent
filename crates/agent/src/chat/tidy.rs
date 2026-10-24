//! 글 고치기 · 지우기 — 양쪽 PC 동기화.

use super::*;

impl Chat {
    /// 편집 · 삭제할 수 있는 글인가 — 내 사람 글 ∧ 끝남(`done` · `error` · `undelivered`) ∧ 그 글을 실어 나르는 보관 행이 기다리지 않음.
    /// `conn` 은 부르는 쪽 연결(트랜잭션일 수 있다).
    pub fn tidy_ok(conn: &Connection, m: &Message) -> io::Result<bool> {
        if m.author != Author::Human
            || !m.mine
            || !matches!(m.state, State::Done | State::Error | State::Undelivered)
        {
            return Ok(false);
        }
        let carrying = outbox::for_message(conn, m.id).map_err(db_err)?;
        Ok(carrying.is_none_or(|o| o.state == "failed"))
    }

    /// 고를 글들 — 모두 있고 한 대화이며 편집 · 삭제할 수 있어야 한다(아니면 `InvalidInput` · `NotFound`). 그 대화.
    fn tidy_targets(conn: &Connection, ids: &[i64]) -> io::Result<(Conversation, Vec<Message>)> {
        let mut out = Vec::with_capacity(ids.len());
        for &id in ids {
            let m = chat::message_by_id(conn, id)
                .map_err(db_err)?
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "없는 메시지"))?;
            if out
                .first()
                .is_some_and(|f: &Message| f.conversation_id != m.conversation_id)
            {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "한 대화의 글만 고른다",
                ));
            }
            if !Self::tidy_ok(conn, &m)? {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "편집 · 삭제는 끝난 내 사람 글만",
                ));
            }
            out.push(m);
        }
        let first = out
            .first()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "고른 글이 없다"))?;
        let conv = chat::by_id(conn, first.conversation_id)
            .map_err(db_err)?
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "없는 대화"))?;
        Self::writable(&conv)?;
        Ok((conv, out))
    }

    /// 내 사람 글 본문 편집(이력 정리, 실행 · 알림 없음). `mirror` 면 같은 트랜잭션에 상대 반영 행(`Edit`)을 적고 커밋 뒤 보낸다.
    /// 바뀐 글과 경고(상대 창에 반영하지 못했을 때).
    pub async fn edit_message(
        &self,
        id: i64,
        text: &str,
        mirror: bool,
    ) -> io::Result<(Message, Option<String>)> {
        if text.trim().is_empty() || text.len() > MAX_TEXT {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("본문은 1 ~ {MAX_TEXT} 바이트"),
            ));
        }
        let conv_id = chat::message_by_id(&self.db(), id)
            .map_err(db_err)?
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "없는 메시지"))?
            .conversation_id;
        // 확인 · 쓰기 · 반영은 대화 차례 안에서 — 상대에 쓴 순서대로 간다.
        let lock = self.turn(conv_id);
        let _turn = lock.lock().await;
        let (conv, m, staged, viewers) = {
            let db = self.db();
            let tx = an_core::db::write_tx(&db).map_err(db_err)?;
            let (conv, mut ms) = Self::tidy_targets(&tx, &[id])?;
            let m = ms.remove(0);
            chat::edit_text(&tx, id, text).map_err(db_err)?;
            // 이미 관람자에게 간 글이면 고친 글을 함께.
            let edited = chat::message_by_id(&tx, id)
                .map_err(db_err)?
                .unwrap_or_else(|| m.clone());
            let viewers = Self::share_change(&tx, &conv, &edited, false)?;
            let staged = if mirror {
                let req = Request::Edit {
                    conv: conv.uuid.clone(),
                    msg: m.uuid.clone(),
                    text: text.to_string(),
                };
                vec![(Self::stage(&tx, &conv, Some(id), outbox::EDIT, &req)?, req)]
            } else {
                Vec::new()
            };
            tx.commit().map_err(db_err)?;
            let m = chat::message_by_id(&db, id).map_err(db_err)?.unwrap_or(m);
            (conv, m, staged, viewers)
        };
        for p in viewers {
            self.kick(p);
        }
        let warning = self.mirror_tidy(&conv, staged).await;
        Ok((m, warning))
    }

    /// 내 사람 글 여러 개를 흔적 없이 지운다(이력 정리). 한 대화의 글만, 하나라도 지울 수 없으면 아무것도 지우지 않는다. `mirror` 면
    /// 글마다 상대 반영 행(`Delete`)을 같은 트랜잭션에 적고 커밋 뒤 보낸다. 지운 수와 경고.
    pub async fn delete_messages(
        &self,
        ids: &[i64],
        mirror: bool,
    ) -> io::Result<(usize, Option<String>)> {
        let mut ids = ids.to_vec();
        ids.sort_unstable();
        ids.dedup();
        let first = *ids
            .first()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "고른 글이 없다"))?;
        let conv_id = chat::message_by_id(&self.db(), first)
            .map_err(db_err)?
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "없는 메시지"))?
            .conversation_id;
        let lock = self.turn(conv_id);
        let _turn = lock.lock().await;
        let (conv, staged, viewers) = {
            let db = self.db();
            let tx = an_core::db::write_tx(&db).map_err(db_err)?;
            let (conv, ms) = Self::tidy_targets(&tx, &ids)?;
            let mut staged = Vec::new();
            let mut viewers = Vec::new();
            for m in &ms {
                viewers.extend(Self::share_change(&tx, &conv, m, true)?);
                if mirror {
                    let req = Request::Delete {
                        conv: conv.uuid.clone(),
                        msg: m.uuid.clone(),
                    };
                    staged.push((
                        Self::stage(&tx, &conv, Some(m.id), outbox::DELETE, &req)?,
                        req,
                    ));
                }
                chat::delete_message(&tx, m.id).map_err(db_err)?;
            }
            tx.commit().map_err(db_err)?;
            (conv, staged, viewers)
        };
        for p in viewers {
            self.kick(p);
        }
        let warning = self.mirror_tidy(&conv, staged).await;
        Ok((ids.len(), warning))
    }

    /// 적어 둔 편집 · 삭제 반영 행을 차례로 보낸다 — 상대가 거절하면 내 활동 '상대 반영 실패' + 경고. 닿지 않으면 줄이 다시 보낸다(경고 없음).
    async fn mirror_tidy(
        &self,
        conv: &Conversation,
        staged: Vec<(i64, Request)>,
    ) -> Option<String> {
        let mut warning = None;
        for (id, req) in staged {
            let got = match self.dispatch(conv, id, &req).await {
                Ok(Shipped::Answered(Response::Ok) | Shipped::Kept) => None,
                // '상대 반영 실패' 기록은 응답 정산(`acked`)이 행 지우기와 함께 했다.
                Ok(Shipped::Answered(other)) => Some(refused(other).to_string()),
                Err(e) => Some(e.to_string()),
            };
            warning = warning.or(got);
        }
        warning.map(|e| format!("이 PC 에는 반영했다 — 상대 창에 반영하지 못했다({e})"))
    }

    /// 상대 반영 실패 — 편집 · 삭제 반영 · 기록 지우기 행의 대화에 내 활동 기록(`sync_failed`, 사유 "편집 — {미리보기}" · "삭제" · "기록 삭제"). `conn` 은 행을 정리하는
    /// 트랜잭션 — 기록과 행 정리가 함께 남거나 함께 없다.
    pub(super) fn note_sync_failed(conn: &Connection, item: &outbox::Item) -> io::Result<()> {
        // 나가며 떼어 낸 기록 지우기는 대화가 없다 — 남길 자리가 없다.
        let Some(conv) = item.conv_id else {
            return if item.kind == CLEAR {
                Ok(())
            } else {
                Err(io::Error::new(io::ErrorKind::InvalidData, "반영 행에 대화가 없다"))
            };
        };
        // 편집 실패는 그 글에 잇는다 — 나중에 글을 지우면 본문 미리보기도 함께 사라진다(완전 삭제). 삭제 실패는 본문이 없고
        // 글 id 도 이미 없다.
        let (what, message) = match serde_json::from_str::<Request>(&item.frame) {
            Ok(Request::Edit { text, .. }) => (format!("편집 — {}", preview(&text)), item.msg_id),
            Ok(Request::Clear { .. }) => ("기록 삭제".to_string(), None),
            _ => ("삭제".to_string(), None),
        };
        chat::add_relay_event(conn, conv, chat::RelayMark::SyncFailed(message), &what)
            .map_err(db_err)
    }

    /// 상대가 쓴 사람 글(그 상대 소유 대화의 `!mine` 사람 글). 아니면 `None`.
    fn theirs(
        conn: &Connection,
        peer: &Peer,
        conv: &str,
        msg: &str,
    ) -> io::Result<Option<Message>> {
        let Some(c) = chat::find_open(conn, conv, peer.id).map_err(db_err)? else {
            return Ok(None);
        };
        Ok(chat::message_by_uuid(conn, msg)
            .map_err(db_err)?
            .filter(|m| m.conversation_id == c.id && m.author == Author::Human && !m.mine))
    }

    /// 상대가 자기 사람 글을 편집했다 — 그 글에만, 실행 · 알림 없이. 실행 대기 · 실행 중이면 `busy`. 같은 본문 재전송은 `Ok`.
    pub(super) fn inbound_edit(
        &self,
        peer: &Peer,
        conv: String,
        msg: String,
        text: String,
    ) -> io::Result<Response> {
        if text.trim().is_empty() || text.len() > MAX_TEXT {
            return Ok(Response::error("bad_request"));
        }
        let viewers = {
            let db = self.db();
            let Some(m) = Self::theirs(&db, peer, &conv, &msg)? else {
                return Ok(Response::error("unknown"));
            };
            if matches!(m.state, State::Queued | State::Running) {
                return Ok(Response::error("busy"));
            }
            if m.text == text && m.edited_at.is_some() {
                return Ok(Response::Ok);
            }
            // 고친 글과 관람자 행은 한 트랜잭션.
            let tx = an_core::db::write_tx(&db).map_err(db_err)?;
            chat::edit_text(&tx, m.id, &text).map_err(db_err)?;
            let edited = chat::message_by_id(&tx, m.id).map_err(db_err)?.unwrap_or(m);
            let viewers = match chat::by_id(&tx, edited.conversation_id).map_err(db_err)? {
                Some(c) => Self::share_change(&tx, &c, &edited, false)?,
                None => Vec::new(),
            };
            tx.commit().map_err(db_err)?;
            viewers
        };
        for p in viewers {
            self.kick(p);
        }
        Ok(Response::Ok)
    }

    /// 상대가 자기 사람 글을 지웠다 — 그 글만 흔적 없이, 실행 · 알림 없이. 이미 없으면 `Ok`(재전송), 실행 대기 · 실행 중이면 `busy`.
    pub(super) fn inbound_delete(&self, peer: &Peer, conv: String, msg: String) -> io::Result<Response> {
        let viewers = {
            let db = self.db();
            let Some(c) = chat::find_open(&db, &conv, peer.id).map_err(db_err)? else {
                return Ok(Response::error("unknown"));
            };
            if chat::message_by_uuid(&db, &msg).map_err(db_err)?.is_none() {
                return Ok(Response::Ok);
            }
            let Some(m) = Self::theirs(&db, peer, &conv, &msg)? else {
                return Ok(Response::error("unknown"));
            };
            if matches!(m.state, State::Queued | State::Running) {
                return Ok(Response::error("busy"));
            }
            let tx = an_core::db::write_tx(&db).map_err(db_err)?;
            let viewers = Self::share_change(&tx, &c, &m, true)?;
            chat::delete_message(&tx, m.id).map_err(db_err)?;
            tx.commit().map_err(db_err)?;
            viewers
        };
        for p in viewers {
            self.kick(p);
        }
        Ok(Response::Ok)
    }
}
