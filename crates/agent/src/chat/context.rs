//! 컨텍스트 — 압축 · 비우기.

use super::*;

const CTX_FAILED: &str = "compact_failed";
/// 이 PC 도 처리한 구분선 글에 남기는 상대 쪽 실패 코드(부분 처리).
fn peer_code(code: &str) -> String {
    format!("peer_{code}")
}

/// 압축 요약 상한(바이트) — 통신 본문 한도 안.
const MAX_SUMMARY: usize = 8 * 1024;
/// 압축 요약 하나 — `key` = `ctx_seeds.agent`(빈 글자 = 대화 세션 주인), `agent` · `uid` = 요약을 만든 에이전트와 그 세대.
struct Seed {
    key: String,
    agent: String,
    uid: String,
    text: String,
}

/// 구분선에 보일 요약 — 세션 하나면 그 글, 여럿이면 에이전트별 머리.
fn summary_text(conv: &Conversation, seeds: &[Seed]) -> String {
    let text = match seeds {
        [one] => one.text.clone(),
        many => many
            .iter()
            .map(|s| {
                format!(
                    "## {}\n{}",
                    if s.key.is_empty() {
                        &conv.agent
                    } else {
                        &s.key
                    },
                    s.text
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n"),
    };
    fit_to(text, MAX_SUMMARY)
}

impl Chat {
    /// 컨텍스트 압축 · 비우기(화면) — `op` = `compact` · `clear`. 이 PC 에 그 대화 세션이 있으면(받은 대화 · 자동 왕복) 대화 줄에서 처리하고,
    /// 상대 PC 에 있을 수 있으면(보낸 대화 · 자동 왕복) 상대에 요청한다(꺼져 있으면 보관했다 보낸다). 진행 중인 압축 · 비우기가 있으면 거절.
    pub fn context(self: &Arc<Self>, conversation_id: i64, op: &str) -> io::Result<Message> {
        if !matches!(op, "compact" | "clear") {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "op 는 compact · clear",
            ));
        }
        let conv = chat::by_id(&self.db(), conversation_id)
            .map_err(db_err)?
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "없는 대화"))?;
        self.request_context(&conv, op)
    }

    /// 구분선 글 + (이 PC 처리면) 대화 줄 깨우기 + (상대 쪽이면) 요청 줄 — 한 트랜잭션. 자동 압축도 여기로.
    pub(super) fn request_context(self: &Arc<Self>, conv: &Conversation, op: &str) -> io::Result<Message> {
        Self::writable(conv)?;
        let local = conv.dir == Dir::In || conv.relay.is_some();
        let remote = conv.dir == Dir::Out || conv.relay.is_some();
        let marker = {
            let db = self.db();
            let tx = an_core::db::write_tx(&db).map_err(db_err)?;
            // 수동은 실행 대기 · 실행 중 글이 있으면 거절 — 앞 요청이 기대한 맥락을 먼저 비우지 않게. 자동 압축은 방금 끝난 실행 뒤라 줄 앞으로.
            let manual = op != "auto_compact";
            let busy = chat::messages(&tx, conv.id)
                .map_err(db_err)?
                .into_iter()
                .any(|m| {
                    let ctx = m.author == Author::Context
                        && matches!(m.state, State::Queued | State::Running | State::Sent);
                    let run = manual && m.exec && matches!(m.state, State::Queued | State::Running);
                    ctx || run
                });
            if busy {
                return Err(io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    "압축 · 비우기 · 실행이 진행 중 — 끝난 뒤에",
                ));
            }
            let state = if local { State::Queued } else { State::Sent };
            let marker = chat::add_context(&tx, conv.id, &new_uuid(), op, state, true, local)
                .map_err(db_err)?;
            if remote {
                let req = Request::Context {
                    conv: conv.uuid.clone(),
                    msg: marker.uuid.clone(),
                    op: op.into(),
                    text: String::new(),
                };
                let frame = serde_json::to_string(&req).map_err(io::Error::other)?;
                outbox::add(
                    &tx,
                    conv.peer_id,
                    Some(conv.id),
                    Some(marker.id),
                    CTX,
                    &frame,
                    None,
                    "pending",
                )
                .map_err(db_err)?;
            }
            tx.commit().map_err(db_err)?;
            marker
        };
        if local {
            self.wake(conv.id);
        }
        if remote {
            self.kick(conv.peer_id);
        }
        Ok(marker)
    }

    /// 상대가 보낸 컨텍스트 프레임 — 요청(`compact` · `auto_compact` · `clear`)은 내 세션을 대화 줄에서 처리하고 다시 넘기지 않는다. 결과
    /// (`done` · `failed`)는 내가 상대에 요청한 구분선 글에 적는다.
    pub(super) fn inbound_context(
        self: &Arc<Self>,
        peer: &Peer,
        conv: String,
        msg: String,
        op: String,
        text: String,
    ) -> io::Result<Response> {
        if !an_net::proto::CTX_OPS.contains(&op.as_str()) || text.len() > MAX_TEXT {
            return Ok(Response::error("bad_request"));
        }
        let db = self.db();
        let Some(c) = chat::find_open(&db, &conv, peer.id).map_err(db_err)? else {
            return Ok(Response::error("unknown"));
        };
        let found = chat::message_by_uuid(&db, &msg).map_err(db_err)?;
        match op.as_str() {
            "done" | "failed" => {
                let Some(m) =
                    found.filter(|m| m.conversation_id == c.id && m.author == Author::Context)
                else {
                    return Ok(Response::error("unknown"));
                };
                // 이 PC 도 처리한 글(자동 왕복) — 상대 실패는 숨기지 않고 `peer_{코드}` 로(내 처리 결과 · 상태는 그대로).
                if m.exec {
                    if op == "failed" {
                        chat::set_refused(
                            &db,
                            m.id,
                            Some(&peer_code(an_net::proto::clean_code(&text))),
                        )
                        .map_err(db_err)?;
                    }
                    return Ok(Response::Ok);
                }
                if m.state != State::Sent {
                    return Ok(Response::Ok);
                }
                if op == "done" {
                    chat::finish_context(&db, m.id, State::Done, &text).map_err(db_err)?;
                } else {
                    let tx = an_core::db::write_tx(&db).map_err(db_err)?;
                    chat::finish_context(&tx, m.id, State::Error, "").map_err(db_err)?;
                    chat::set_refused(&tx, m.id, Some(an_net::proto::clean_code(&text)))
                        .map_err(db_err)?;
                    tx.commit().map_err(db_err)?;
                }
                Ok(Response::Ok)
            }
            _ => {
                if found.is_some() {
                    return Ok(Response::Ok); // 같은 요청 재전송
                }
                // 내게 그 대화 세션이 없는 대화(내가 보낸 대화 · 자동 왕복 아님) — 처리할 것이 없다.
                if c.dir == Dir::Out && c.relay.is_none() {
                    return Ok(Response::error("bad_request"));
                }
                chat::add_context(&db, c.id, &msg, &op, State::Queued, false, true)
                    .map_err(db_err)?;
                drop(db);
                self.wake(c.id);
                Ok(Response::Ok)
            }
        }
    }

    /// 상대에 보낸 컨텍스트 요청이 거절 · 포기됐다 — 이 PC 가 처리하지 않는 글만 실패로(자동 왕복이면 내 처리 결과가 그대로).
    pub(super) fn context_refused(&self, msg_id: i64, code: &str) {
        let db = self.db();
        match chat::message_by_id(&db, msg_id) {
            Ok(Some(m)) if !m.exec && m.state == State::Sent => {
                let _ = chat::finish_context(&db, m.id, State::Error, "");
                let _ = chat::set_refused(&db, m.id, Some(code));
            }
            Ok(Some(m)) if m.exec => {
                let _ = chat::set_refused(&db, m.id, Some(&peer_code(code)));
            }
            Ok(_) => log::info!("컨텍스트 요청 {msg_id} 상대 거절({code}) — 이미 끝난 글"),
            Err(e) => log::warn!("컨텍스트 요청 {msg_id} 읽기 실패: {e}"),
        }
    }

    /// 대화 줄의 구분선 글 처리 — 비우기는 세션만, 압축은 세션마다 요약 실행 뒤 세션을 비우고 요약을 다음 첫 실행용으로 남긴다. 세션 비우기 ·
    /// 요약 남기기 · 구분선 끝 · (상대가 요청한 글이면) 결과 회신 줄은 한 트랜잭션 — 사이에 꺼져도 요청자가 결과를 영영 못 받는 일이 없다.
    pub(super) async fn run_context(&self, conversation_id: i64, marker: Message) {
        let conv = match chat::by_id(&self.db(), conversation_id) {
            Ok(Some(c)) => c,
            Ok(None) => return log::error!("대화 {conversation_id} 를 찾지 못했다"),
            Err(e) => return log::error!("대화 {conversation_id} 읽기 실패: {e}"),
        };
        let op = marker.ctx_op.clone().unwrap_or_default();
        let result: Result<Vec<Seed>, &'static str> = if marker.state == State::Running {
            Err(CTX_FAILED) // 앱이 도중에 끝났다 — 세션은 그대로(비우기 · 요약 기록은 끝 트랜잭션에서만)
        } else {
            // 대기 구분선 · 열린 대화일 때만(나가기 · 삭제가 취소했으면 처리하지 않는다).
            if !chat::start_run(&self.db(), marker.id).unwrap_or(false) {
                return;
            }
            if op == "clear" {
                Ok(Vec::new())
            } else {
                self.compact(&conv, &marker).await
            }
        };
        let summary = match &result {
            Ok(seeds) => summary_text(&conv, seeds),
            Err(_) => String::new(),
        };
        let reply = (!marker.mine).then(|| Request::Context {
            conv: conv.uuid.clone(),
            msg: marker.uuid.clone(),
            op: if result.is_ok() { "done" } else { "failed" }.into(),
            text: match result {
                Ok(_) => summary.clone(),
                Err(code) => code.to_string(),
            },
        });
        let saved = (|| -> io::Result<()> {
            let db = self.db();
            let tx = an_core::db::write_tx(&db).map_err(db_err)?;
            match &result {
                Ok(seeds) => {
                    chat::clear_sessions(&tx, conv.id, false).map_err(db_err)?;
                    for s in seeds {
                        // 요약 실행 뒤 그 에이전트 폴더가 바뀌었거나 지웠다 다시 등록됐으면(세대가 다르다) 옛 요약을 남기지 않는다.
                        let same = agents::by_name(&tx, &s.agent)
                            .map_err(db_err)?
                            .is_some_and(|a| a.uid == s.uid);
                        if same {
                            chat::set_ctx_seed(&tx, conv.id, &s.key, &s.text).map_err(db_err)?;
                        }
                    }
                    chat::finish_context(&tx, marker.id, State::Done, &summary).map_err(db_err)?;
                }
                Err(code) => {
                    chat::finish_context(&tx, marker.id, State::Error, "").map_err(db_err)?;
                    chat::set_refused(&tx, marker.id, Some(code)).map_err(db_err)?;
                }
            }
            if let Some(req) = &reply {
                let frame = serde_json::to_string(req).map_err(io::Error::other)?;
                outbox::add(
                    &tx,
                    conv.peer_id,
                    Some(conv.id),
                    Some(marker.id),
                    CTX_DONE,
                    &frame,
                    None,
                    "pending",
                )
                .map_err(db_err)?;
            }
            tx.commit().map_err(db_err)
        })();
        if let Err(e) = saved {
            log::error!("대화 {} 컨텍스트 기록 실패: {e}", conv.uuid);
            let _ = chat::set_state(&self.db(), marker.id, State::Error);
            return;
        }
        if reply.is_some() {
            self.kick(conv.peer_id);
        }
    }

    /// 압축 — 대화 세션마다 그 세션을 이어 요약 실행(실행 문을 지난다). 하나라도 실패하면 아무것도 바꾸지 않는다(세션 그대로). 사라진 세션은 건너뛴다.
    /// 기록(세션 비우기 · 요약)은 부른 쪽 끝 트랜잭션이 한다.
    async fn compact(
        &self,
        conv: &Conversation,
        marker: &Message,
    ) -> Result<Vec<Seed>, &'static str> {
        let sessions = chat::sessions(&self.db(), conv).map_err(|e| {
            log::warn!("대화 {} 세션 읽기 실패: {e}", conv.uuid);
            CTX_FAILED
        })?;
        if sessions.is_empty() {
            return Ok(Vec::new());
        }
        let (mut ticket, ahead) = self.gate.enter(marker.id);
        if ahead.is_some() {
            ticket.ready().await;
        }
        let mut seeds = Vec::new();
        for (name, session) in sessions {
            // 대화 에이전트 세션 — 받은 대화는 대화 에이전트, 자동 왕복을 연 쪽은 내 왕복 에이전트.
            let who = name
                .clone()
                .unwrap_or_else(|| match (&conv.dir, &conv.relay) {
                    (Dir::Out, Some(r)) => r.agent.clone(),
                    _ => conv.agent.clone(),
                });
            // 세션과 세대(`uid`)를 같은 DB 시점에서 — 실행 자리 · 앞 세션 요약을 기다리는 동안 폴더 · 상위 읽기가 바뀌어 그 세션이 비었으면
            // 메모리에 든 옛 세션을 새 세대로 이어 요약하지 않는다.
            let (agent, opts, live) = {
                let db = self.db();
                let agent = agents::by_name(&db, &who).ok().flatten();
                let peer = an_core::peers::by_id(&db, conv.peer_id).ok().flatten();
                let live = chat::by_id(&db, conv.id)
                    .ok()
                    .flatten()
                    .and_then(|now| chat::sessions(&db, &now).ok())
                    .is_some_and(|now| now.iter().any(|(n, s)| *n == name && *s == session));
                (agent, an_core::run_opts::resolve(&db, peer.as_ref()).ok(), live)
            };
            if !live {
                log::info!("대화 {} 압축 — 기다리는 동안 비운 세션 건너뜀({who})", conv.uuid);
                continue;
            }
            let (Some(agent), Some((model, effort))) = (agent, opts) else {
                log::warn!(
                    "대화 {} 압축 — 에이전트 {who} 없음 · 실행 옵션 읽기 실패",
                    conv.uuid
                );
                return Err(CTX_FAILED);
            };
            if an_core::paths::agent_folder_problem(std::path::Path::new(&agent.folder)).is_some() {
                return Err(CTX_FAILED);
            }
            let uid = agent.uid.clone();
            let input = RunInput {
                // 요약은 셸 없이 — 세션에 남은 원격 글의 지시가 요약 실행 때 명령으로 이어지지 않게.
                agent: agents::Agent { shell: false, ..agent },
                session: Session::Resume(session),
                text: SUMMARY_PROMPT.to_string(),
                model,
                effort,
                steps: StepSink::default(),
            };
            let run = self.run.clone();
            match tokio::task::spawn_blocking(move || run(input)).await {
                Ok(Ok(out)) if !out.is_error && !out.text.trim().is_empty() => seeds.push(Seed {
                    key: name.unwrap_or_default(),
                    agent: who,
                    uid,
                    text: fit_to(out.text, MAX_SUMMARY),
                }),
                Ok(Err(e)) if crate::runner::session_missing(&e) => {
                    log::info!("대화 {} 압축 — 사라진 세션 건너뜀: {e}", conv.uuid);
                }
                other => {
                    log::warn!("대화 {} 압축 요약 실패: {other:?}", conv.uuid);
                    return Err(CTX_FAILED);
                }
            }
        }
        Ok(seeds)
    }
}
