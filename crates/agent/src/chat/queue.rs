//! 실행 큐 — 동시 실행 상한 · 차례 · 진행 알림.

use super::*;

impl Chat {
    /// 실행 문 — (상한, 실행 중 메시지 id, 기다리는 메시지 id). 기다림 = 문의 줄(닿은 순서) 뒤에, 같은 대화의 앞 실행을 기다려 아직 문에
    /// 닿지 않은 실행 요청(DB `queued`, id 순).
    pub fn run_queue(&self) -> (usize, Vec<i64>, Vec<i64>) {
        let (cap, running, mut waiting) = self.gate.snapshot();
        let db = self.db();
        let blocked = chat::pending_conversations(&db).and_then(|cs| {
            let mut ids = Vec::new();
            for c in cs {
                ids.extend(
                    chat::pending(&db, c)?
                        .into_iter()
                        .filter(|m| m.state == State::Queued)
                        .map(|m| m.id),
                );
            }
            Ok(ids)
        });
        match blocked {
            Ok(mut ids) => {
                ids.retain(|id| !running.contains(id) && !waiting.contains(id));
                ids.sort_unstable();
                waiting.extend(ids);
            }
            Err(e) => log::warn!("실행 목록 — 대화 대기열 읽기 실패: {e}"),
        }
        (cap, running, waiting)
    }

    /// 같은 대화의 앞 실행 때문에 기다리는 상대 실행 요청 — 닿자마자 `queued` 를 알린다(대화 작업이 문에 데려가기 전에는 문이 모른다).
    /// 앞 사람 수 = 같은 대화의 앞 요청 + 지금 문 줄.
    pub(super) fn tell_waiting(&self, conversation_id: i64, uuid: &str) {
        let found = {
            let db = self.db();
            chat::by_id(&db, conversation_id)
                .and_then(|c| Ok((c, chat::pending(&db, conversation_id)?)))
        };
        let (Some(conv), pending) = (match found {
            Ok(x) => x,
            Err(e) => return log::info!("대화 {conversation_id} 대기 알림 — 읽기 실패: {e}"),
        }) else {
            return;
        };
        let Some(i) = pending.iter().position(|m| m.uuid == uuid) else {
            return;
        };
        if i == 0 || conv.dir != Dir::In || pending[i].mine {
            return;
        }
        let ahead = i + self.gate.snapshot().2.len();
        self.send_progress(&conv, &pending[i], "queued", ahead);
    }

    /// 동시 실행 상한을 바꾸고 이 PC 에 저장한다(1 ~ `MAX_RUNS`).
    pub fn set_max_runs(&self, n: usize) -> io::Result<()> {
        if !(1..=MAX_RUNS).contains(&n) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("동시 실행 상한은 1 ~ {MAX_RUNS}"),
            ));
        }
        an_core::settings::set(&self.db(), MAX_RUNS_KEY, &n.to_string()).map_err(db_err)?;
        self.gate.set_cap(n);
        Ok(())
    }

    /// 상대가 알려 준 차례 — 아직 답을 기다리는(`sent`) 글만. 답이 온 글의 항목은 여기서 지운다.
    pub fn progress_of(&self, m: &Message) -> Option<Progress> {
        let mut p = an_core::sync::lock(&self.progress);
        if m.state != State::Sent {
            p.remove(&m.id);
            return None;
        }
        p.get(&m.id).cloned()
    }

    /// 실행 문을 지난다(PC 전체 동시 실행 상한) — 상대가 보낸 실행 요청이면 차례를 알린다(기다리면 `queued` → `running`, 바로면 `running`).
    pub(super) async fn admit(&self, conversation_id: i64, asked: &Message) -> crate::gate::Ticket {
        let conv = chat::by_id(&self.db(), conversation_id).ok().flatten();
        let remote = conv.filter(|c| c.dir == Dir::In && !asked.mine);
        let (mut ticket, ahead) = self.gate.enter(asked.id);
        if let Some(k) = ahead {
            if let Some(c) = &remote {
                self.send_progress(c, asked, "queued", k);
            }
            ticket.ready().await;
        }
        // 실행 시작 알림(`running`)은 `process` 가 시작을 기록한 뒤(`chat::start_run` — 줄을 서는 동안 지운 대화에 행을 남기지 않게).
        ticket
    }

    /// 상대 실행 요청의 차례를 보낸 쪽에 알린다 — 그 상대 보내기 줄에(같은 글의 기다리는 진행은 마지막 값으로 바꾼다). 답과 같은 줄이라
    /// 순서가 지켜지고, 닿지 않으면 다시 보낸다.
    pub(super) fn send_progress(&self, conv: &Conversation, asked: &Message, state: &str, ahead: usize) {
        self.send_step(conv, asked, state, ahead, None);
    }

    pub(super) fn send_step(
        &self,
        conv: &Conversation,
        asked: &Message,
        state: &str,
        ahead: usize,
        detail: Option<StepDetail>,
    ) {
        let req = Request::Progress {
            conv: conv.uuid.clone(),
            msg: asked.uuid.clone(),
            state: state.into(),
            ahead: u32::try_from(ahead).unwrap_or(u32::MAX),
            detail,
        };
        let key = format!("progress:{}", asked.uuid);
        self.enqueue(conv, None, PROGRESS, &req, Some(&key));
    }

    /// 단계 요약 간격 — 시험용.
    pub fn set_step_every(&self, every: std::time::Duration) {
        *an_core::sync::lock(&self.step_every) = every;
    }
}
