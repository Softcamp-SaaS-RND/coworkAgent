//! 대화 왕복 조율(S1 · D17 · D19) — 보내기(사람 → 원격 에이전트), 받기(대기열 → `claude -p` → 답 되돌리기), 답 받기.
//! 원격에서 온 것은 신뢰하지 않는 입력: 페어링 확인은 `Node`, 에이전트 이름 · 본문 크기 · 대화 소유는 여기서 본다.
use crate::runner::{RunInput, RunOutput, Session};
use an_core::agents;
use an_core::chat::{self, Author, Conversation, Dir, Message, State};
use an_core::db::Connection;
use an_core::peers::Peer;
use an_net::node::{MessageKind, Node, PeerEvent};
use an_net::proto::{MAX_TEXT, Request, Response};
use std::collections::HashMap;
use std::io;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

/// 에이전트 실행 — 운영은 `runner::run_claude`, 테스트는 대역. `spawn_blocking` 안에서 불린다.
pub type Run = Arc<dyn Fn(RunInput) -> io::Result<RunOutput> + Send + Sync>;

/// 자동 왕복 상한의 최댓값(왕복, D24).
pub const MAX_RELAY_CAP: i64 = 50;

/// 자동 왕복 상한 기본값(왕복, D24) — 설정(`RELAY_CAP_KEY`)이 없을 때.
pub const DEFAULT_RELAY_CAP: i64 = 10;

/// 설정 키 — 새 자동 왕복 대화의 상한 기본값(1 ~ `MAX_RELAY_CAP`).
pub const RELAY_CAP_KEY: &str = "relay_cap_default";

/// 자동 왕복 끝 알림 사유 — 상한.
const RELAY_END_CAP: &str = "상한 도달";

/// 새 자동 왕복 대화의 상한 기본값 — 설정이 없거나 범위 밖이면 `DEFAULT_RELAY_CAP`.
pub fn default_relay_cap(conn: &Connection) -> i64 {
    an_core::settings::get(conn, RELAY_CAP_KEY)
        .ok()
        .flatten()
        .and_then(|v| v.parse().ok())
        .filter(|v| (1..=MAX_RELAY_CAP).contains(v))
        .unwrap_or(DEFAULT_RELAY_CAP)
}

/// 알림 미리 보기 길이(글자).
const PREVIEW_CHARS: usize = 80;

/// 앱이 실행 도중 끝나 결과가 없는 메시지에 돌려줄 답.
const INTERRUPTED: &str = "실행 도중 상대 앱이 종료돼 결과가 없다 — 다시 보내 주세요";

pub struct Chat {
    db: Mutex<Connection>,
    node: Arc<Node>,
    run: Run,
    /// 대화별 직렬 작업 깨우기(D19 — 같은 세션 동시 재개 금지). 대기열 자체는 DB(`queued` 메시지, id 순).
    // ponytail: 대화마다 작업 하나를 계속 둔다 — 대화가 아주 많아지면 유휴 작업 정리가 필요하다.
    queues: Mutex<HashMap<i64, mpsc::UnboundedSender<()>>>,
    /// 대화별 보내기 잠금 — 저장 순서 = 상대 도착 순서(동시 보내기가 상대 세션에서 뒤바뀌지 않게).
    // ponytail: 대화마다 잠금 하나를 계속 둔다 — `queues` 와 같은 상한.
    outbound: Mutex<HashMap<i64, Arc<tokio::sync::Mutex<()>>>>,
}

fn db_err(e: an_core::db::Error) -> io::Error {
    io::Error::other(e)
}

/// UUID v4 문자열 — `claude --session-id` 는 유효한 UUID 만 받는다.
pub fn new_uuid() -> String {
    let mut b = [0u8; 16];
    getrandom::fill(&mut b).expect("OS 난수원을 쓸 수 없다");
    b[6] = (b[6] & 0x0f) | 0x40;
    b[8] = (b[8] & 0x3f) | 0x80;
    let h: String = b.iter().map(|x| format!("{x:02x}")).collect();
    format!(
        "{}-{}-{}-{}-{}",
        &h[..8],
        &h[8..12],
        &h[12..16],
        &h[16..20],
        &h[20..]
    )
}

fn preview(text: &str) -> String {
    text.chars().take(PREVIEW_CHARS).collect()
}

/// `MAX_TEXT` 바이트 안으로(글자 경계에서) 자른다.
fn fit(mut text: String) -> String {
    if text.len() <= MAX_TEXT {
        return text;
    }
    let mut cut = MAX_TEXT - "\n…(잘림)".len();
    while !text.is_char_boundary(cut) {
        cut -= 1;
    }
    text.truncate(cut);
    text.push_str("\n…(잘림)");
    text
}

fn refused(resp: Response) -> io::Error {
    match resp {
        Response::Error { code } => io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!("상대가 거절: {code}"),
        ),
        other => io::Error::new(
            io::ErrorKind::InvalidData,
            format!("예상하지 못한 응답: {other:?}"),
        ),
    }
}

fn addr(peer: &Peer) -> io::Result<SocketAddr> {
    peer.addr.parse().map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("주소 형식 오류: {}", peer.addr),
        )
    })
}

impl Chat {
    /// `conn` 은 피어 노드와 같은 DB 파일의 다른 연결.
    pub fn new(conn: Connection, node: Arc<Node>, run: Run) -> Arc<Self> {
        Arc::new(Self {
            db: Mutex::new(conn),
            node,
            run,
            queues: Mutex::new(HashMap::new()),
            outbound: Mutex::new(HashMap::new()),
        })
    }

    /// 노드의 대화 요청 처리기로 붙는다.
    pub fn attach(self: &Arc<Self>) {
        let me = self.clone();
        self.node.on_request(move |peer, req| me.handle(peer, req));
    }

    /// 시작 시 — 지난 실행에서 남은 `queued` 는 실행하고, `running`(실행 도중 종료)은 오류 답으로 끝낸다. tokio 런타임 안에서 부른다.
    pub fn recover(self: &Arc<Self>) -> io::Result<usize> {
        let ids = chat::pending_conversations(&self.db()).map_err(db_err)?;
        for &id in &ids {
            self.wake(id);
        }
        Ok(ids.len())
    }

    /// 대화 DB — await 를 넘겨 쥐지 않는다.
    pub fn db(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.db.lock().expect("대화 DB 잠금이 오염됐다")
    }

    fn paired_peer(&self, peer_id: i64) -> io::Result<Peer> {
        let peer = self
            .node
            .peers()?
            .into_iter()
            .find(|p| p.id == peer_id)
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::NotFound, format!("피어 {peer_id} 없음"))
            })?;
        if !peer.paired() {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "페어링되지 않은 상대",
            ));
        }
        Ok(peer)
    }

    /// 상대 PC 의 에이전트 이름.
    pub async fn remote_agents(&self, peer_id: i64) -> io::Result<Vec<String>> {
        let peer = self.paired_peer(peer_id)?;
        match self
            .node
            .send(addr(&peer)?, Some(peer.spki.clone()), &Request::Agents)
            .await?
        {
            (Response::Agents { names }, _) => Ok(names),
            (other, _) => Err(refused(other)),
        }
    }

    /// 사람 → 원격 에이전트. `conversation` 이 있으면 그 대화에 이어 쓴다. 상대가 받아 대기열에 넣으면 끝(답은 나중에 `reply`).
    pub async fn send(
        &self,
        peer_id: i64,
        agent: &str,
        conversation: Option<i64>,
        text: &str,
    ) -> io::Result<(Conversation, Message)> {
        if text.trim().is_empty() || text.len() > MAX_TEXT {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("본문은 1 ~ {MAX_TEXT} 바이트"),
            ));
        }
        let peer = self.paired_peer(peer_id)?;
        let conv = {
            let db = self.db();
            match conversation {
                Some(id) => chat::by_id(&db, id)
                    .map_err(db_err)?
                    .filter(|c| c.dir == Dir::Out && c.peer_id == peer_id)
                    .ok_or_else(|| {
                        io::Error::new(io::ErrorKind::NotFound, format!("대화 {id} 없음"))
                    })?,
                None => chat::open_conversation(&db, &new_uuid(), peer_id, agent, Dir::Out)
                    .map_err(db_err)?,
            }
        };
        // 사람이 쓰면 자동 왕복은 멈춘다(D24 — 사람 입력 대기).
        if conversation.is_some() {
            let _ = chat::relay_on(&self.db(), conv.id, false, false);
        }
        self.send_human(conv, &peer, text, None).await
    }

    /// 두 에이전트 자동 왕복 대화를 연다(D24) — `my_agent` 는 이 PC 프로필, 상한 1 ~ 50 왕복.
    pub async fn start_relay(
        &self,
        peer_id: i64,
        agent: &str,
        my_agent: &str,
        cap: i64,
        text: &str,
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
        let peer = self.paired_peer(peer_id)?;
        let conv = {
            let db = self.db();
            if agents::by_name(&db, my_agent).map_err(db_err)?.is_none() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("내 에이전트 {my_agent} 가 없다"),
                ));
            }
            let c = chat::open_conversation(&db, &new_uuid(), peer_id, agent, Dir::Out)
                .map_err(db_err)?;
            chat::set_relay(&db, c.id, my_agent, cap).map_err(db_err)?;
            chat::by_id(&db, c.id).map_err(db_err)?.unwrap_or(c)
        };
        self.send_human(conv, &peer, text, Some(my_agent.to_string()))
            .await
    }

    /// 사람 글을 상대 에이전트에게 실행 요청으로. `relay` = 자동 왕복을 여는 첫 요청(내 에이전트 이름).
    async fn send_human(
        &self,
        conv: Conversation,
        peer: &Peer,
        text: &str,
        relay: Option<String>,
    ) -> io::Result<(Conversation, Message)> {
        // 저장부터 상대 응답까지 대화 단위로 한 번에 하나.
        let lock = self
            .outbound
            .lock()
            .expect("보내기 잠금이 오염됐다")
            .entry(conv.id)
            .or_default()
            .clone();
        let _turn = lock.lock().await;
        let msg = chat::add_message(
            &self.db(),
            conv.id,
            &new_uuid(),
            Author::Human,
            text,
            State::Sent,
        )
        .map_err(db_err)?;
        let req = Request::Message {
            conv: conv.uuid.clone(),
            msg: msg.uuid.clone(),
            agent: conv.agent.clone(),
            text: text.to_string(),
            lc: Some(msg.lc),
            by_agent: None,
            relay,
        };
        match self
            .node
            .send(addr(peer)?, Some(peer.spki.clone()), &req)
            .await
        {
            Ok((Response::Ok, _)) => Ok((conv, msg)),
            // 명시적 거절만 실패로 확정한다.
            Ok((other, _)) => {
                let _ = chat::set_state(&self.db(), msg.id, State::Error);
                Err(refused(other))
            }
            // 전송 오류는 상대가 받았는지 모른다 — `sent` 로 두어 늦게 온 답을 받고, 같은 일을 새로 보내 중복 실행하지 않게 알린다.
            Err(e) => Err(io::Error::new(
                e.kind(),
                format!(
                    "전달 확인 안 됨({e}) — 상대가 받았을 수 있다. 답을 기다린 뒤 다시 보내세요"
                ),
            )),
        }
    }

    /// 대화에 사람 글(D18 · D23) — `to_agent` 면 대화 에이전트에게 실행 요청, 아니면 실행 없이 양쪽 이력에만.
    /// 보낸 대화(`out`)의 실행 요청은 [`send`](Self::send), 그 밖은 내 이력에 남기고 상대에 `post` 로 미러한다.
    /// 세 번째 값은 경고 — 내 에이전트 실행은 받아들였지만 상대 이력 미러가 실패한 경우(실패로 돌려주면 다시 보내 두 번 실행된다).
    pub async fn post(
        self: &Arc<Self>,
        conversation_id: i64,
        text: &str,
        to_agent: bool,
    ) -> io::Result<(Conversation, Message, Option<String>)> {
        if text.trim().is_empty() || text.len() > MAX_TEXT {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("본문은 1 ~ {MAX_TEXT} 바이트"),
            ));
        }
        let conv = chat::by_id(&self.db(), conversation_id)
            .map_err(db_err)?
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("대화 {conversation_id} 없음"),
                )
            })?;
        if conv.dir == Dir::Out && to_agent {
            let (c, m) = self
                .send(conv.peer_id, &conv.agent, Some(conv.id), text)
                .await?;
            return Ok((c, m, None));
        }
        let peer = self.paired_peer(conv.peer_id)?;
        // 사람이 쓰면 자동 왕복은 멈춘다(D24) — 연 쪽은 끄고, 받은 쪽은 에이전트발 요청을 거절하고 연 쪽에 알린다.
        if conv.dir == Dir::Out {
            let _ = chat::relay_on(&self.db(), conv.id, false, false);
        } else {
            // 거절은 바로, 연 쪽 알림은 기다리지 않는다(상대가 꺼져 있어도 글쓰기가 늦어지지 않게).
            let _ = chat::set_blocked(&self.db(), conv.id, true);
            let (me, c, p) = (self.clone(), conv.clone(), peer.clone());
            tokio::spawn(async move { me.block_relay(&c, &p).await });
        }
        let lock = self
            .outbound
            .lock()
            .expect("보내기 잠금이 오염됐다")
            .entry(conv.id)
            .or_default()
            .clone();
        let _turn = lock.lock().await;
        // 실행 요청도 먼저 `sent` 로 — 상대 이력에 사람 글이 먼저 닿은 뒤에 실행해야 답 미러가 붙는다.
        let msg = chat::add_human(
            &self.db(),
            conv.id,
            &new_uuid(),
            text,
            State::Sent,
            true,
            to_agent,
        )
        .map_err(db_err)?;
        let req = Request::Post {
            conv: conv.uuid.clone(),
            msg: msg.uuid.clone(),
            author: "human".into(),
            text: text.to_string(),
            exec: to_agent,
            reply_to: None,
            ok: None,
            turns: None,
            duration_ms: None,
            lc: Some(msg.lc),
        };
        let sent = match addr(&peer) {
            Ok(a) => self.node.send(a, Some(peer.spki.clone()), &req).await,
            Err(e) => Err(e),
        };
        if to_agent {
            // 내 에이전트는 미러 결과와 무관하게 실행한다.
            let _ = chat::set_state(&self.db(), msg.id, State::Queued);
            self.wake(conv.id);
        }
        let msg = chat::message_by_uuid(&self.db(), &msg.uuid)
            .map_err(db_err)?
            .unwrap_or(msg);
        let mirror_failed = match sent {
            Ok((Response::Ok, _)) => {
                if !to_agent {
                    let _ = chat::set_state(&self.db(), msg.id, State::Done);
                }
                return Ok((conv, msg, None));
            }
            Ok((other, _)) => {
                if !to_agent {
                    let _ = chat::set_state(&self.db(), msg.id, State::Error);
                    return Err(refused(other));
                }
                refused(other).to_string()
            }
            Err(e) => {
                if !to_agent {
                    return Err(io::Error::new(
                        e.kind(),
                        format!("상대 이력에 남았는지 확인 안 됨({e}) — 내 이력에는 남았다"),
                    ));
                }
                e.to_string()
            }
        };
        log::warn!("대화 {} 미러 실패: {mirror_failed}", conv.uuid);
        Ok((
            conv,
            msg,
            Some(format!(
                "내 에이전트는 실행한다 — 상대 이력에는 남기지 못했다({mirror_failed}). 다시 보내지 마세요"
            )),
        ))
    }

    /// 페어링된 상대의 요청 — 막히지 않는다(실행은 대기열로).
    fn handle(self: &Arc<Self>, peer: &Peer, req: Request) -> Response {
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
            } => self.inbound_message(peer, conv, msg, agent, text, (lc, by_agent, relay)),
            Request::Relay { conv, on } => self.inbound_relay(peer, conv, on),
            reply @ Request::Reply { .. } => self.inbound_reply(peer, reply),
            post @ Request::Post { .. } => self.inbound_post(peer, post),
            _ => Ok(Response::error("bad_request")),
        };
        result.unwrap_or_else(|e| {
            log::warn!("대화 요청 처리 실패: {e}");
            Response::error("bad_request")
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
        (lc, by_agent, relay): (Option<i64>, Option<String>, Option<String>),
    ) -> io::Result<Response> {
        if text.trim().is_empty() || text.len() > MAX_TEXT || conv.is_empty() || msg.is_empty() {
            return Ok(Response::error("bad_request"));
        }
        // 자동 왕복을 여는 사람 요청은 "시작" 하나로, 그 뒤 에이전트발 요청은 중간 글(알림 묶기).
        let kind = match (relay.is_some(), by_agent.is_some()) {
            (_, true) => MessageKind::RelayStep,
            (true, false) => MessageKind::RelayStart,
            (false, false) => MessageKind::Request,
        };
        let conversation_id = {
            let db = self.db();
            if agents::by_name(&db, &agent).map_err(db_err)?.is_none() {
                return Ok(Response::error("unknown_agent"));
            }
            let Ok(c) = chat::open_conversation(&db, &conv, peer.id, &agent, Dir::In) else {
                return Ok(Response::error("bad_request")); // 다른 상대의 대화 uuid
            };
            if c.dir != Dir::In || c.agent != agent {
                return Ok(Response::error("bad_request"));
            }
            // 자동 왕복 대화로 기록(받은 쪽) — 첫 실행 실패의 끝 알림 · 상대 에이전트 이름 표시에 쓴다.
            if let Some(name) = relay.as_deref().or(by_agent.as_deref()) {
                chat::mark_relayed(&db, c.id, &an_net::proto::clean_name(name)).map_err(db_err)?;
            }
            if by_agent.is_some() && c.relay_blocked {
                return Ok(Response::error("stopped"));
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
                // 저장이 곧 대기열 등록 — 실행 순서는 id 순(저장 순)이다.
                let m = if by_agent.is_some() {
                    chat::add_agent_request(&db, c.id, &msg, &text, State::Queued)
                        .map_err(db_err)?
                } else {
                    chat::add_message(&db, c.id, &msg, Author::Human, &text, State::Queued)
                        .map_err(db_err)?
                };
                chat::observe(&db, m.id, lc).map_err(db_err)?;
            }
            c.id
        };
        self.wake(conversation_id);
        self.notify(peer, agent, kind, preview(&text));
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
        } = post
        else {
            return Ok(Response::error("bad_request"));
        };
        if text.len() > MAX_TEXT || msg.is_empty() {
            return Ok(Response::error("bad_request"));
        }
        let mut kind = MessageKind::Request;
        let agent = {
            let mut db = self.db();
            let Some(c) = chat::find(&db, &conv, peer.id).map_err(db_err)? else {
                return Ok(Response::error("unknown"));
            };
            if chat::message_by_uuid(&db, &msg).map_err(db_err)?.is_some() {
                return Ok(Response::Ok); // 같은 글 재전송
            }
            match author.as_str() {
                "human" => {
                    if text.trim().is_empty() || (exec && c.dir != Dir::Out) {
                        return Ok(Response::error("bad_request"));
                    }
                    let m = chat::add_human(
                        &db,
                        c.id,
                        &msg,
                        &text,
                        if exec { State::Sent } else { State::Done },
                        false,
                        exec,
                    )
                    .map_err(db_err)?;
                    chat::observe(&db, m.id, lc).map_err(db_err)?;
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
                    let tx = db.transaction().map_err(db_err)?;
                    let m = chat::add_agent_request(&tx, c.id, &msg, &text, State::Done)
                        .map_err(db_err)?;
                    chat::set_flags(&tx, m.id, false, false, State::Done).map_err(db_err)?;
                    chat::observe(&tx, m.id, lc).map_err(db_err)?;
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
                    let tx = db.transaction().map_err(db_err)?;
                    let r = chat::add_reply(
                        &tx,
                        c.id,
                        &msg,
                        &text,
                        state,
                        asked.id,
                        turns,
                        duration_ms,
                    )
                    .map_err(db_err)?;
                    chat::observe(&tx, r.id, lc).map_err(db_err)?;
                    chat::set_state(&tx, asked.id, state).map_err(db_err)?;
                    tx.commit().map_err(db_err)?;
                }
                _ => return Ok(Response::error("bad_request")),
            }
            c.agent
        };
        let shown = if kind == MessageKind::RelayEnd {
            RELAY_END_CAP.to_string()
        } else {
            preview(&text)
        };
        self.notify(peer, agent, kind, shown);
        Ok(Response::Ok)
    }

    /// 알림 이벤트(S9) — 상대 이름 · PC 와 종류(거르기는 알림 쪽이 한다).
    fn notify(&self, peer: &Peer, agent: String, kind: MessageKind, preview: String) {
        self.node.emit(PeerEvent::Message {
            name: peer.name.clone(),
            host: peer.host.clone(),
            agent,
            preview,
            kind,
        });
    }

    /// 자동 왕복이 저절로 멈췄다 — 사유와 함께 알린다(사람이 멈춘 것은 부르지 않는다).
    fn relay_end(&self, conv: &Conversation, reason: &str) {
        if let Ok(peer) = self.paired_peer(conv.peer_id) {
            self.notify(
                &peer,
                conv.agent.clone(),
                MessageKind::RelayEnd,
                reason.to_string(),
            );
        }
    }

    /// 자동 왕복 알림 — 그 상대 소유 대화만. 연 쪽은 `on = false` 만(받은 쪽 멈춤), 받은 쪽은 거절 켜기 · 끄기.
    fn inbound_relay(&self, peer: &Peer, conv: String, on: bool) -> io::Result<Response> {
        let db = self.db();
        let Some(c) = chat::find(&db, &conv, peer.id).map_err(db_err)? else {
            return Ok(Response::error("unknown"));
        };
        let event = match (c.dir, on) {
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
            self.notify(peer, c.agent, kind, text.to_string());
        }
        Ok(Response::Ok)
    }

    /// 받은 쪽 멈춤 — 에이전트발 요청을 거절하고 연 쪽에 알린다(알림 실패는 거절로 충분).
    async fn block_relay(&self, conv: &Conversation, peer: &Peer) {
        let _ = chat::set_blocked(&self.db(), conv.id, true);
        let req = Request::Relay {
            conv: conv.uuid.clone(),
            on: false,
        };
        if let Ok(a) = addr(peer) {
            let _ = self.node.send(a, Some(peer.spki.clone()), &req).await;
        }
    }

    /// 자동 왕복 멈춤 · 재개(D24). 연 쪽: 끄기 / 켜기(카운트 0, 받은 쪽 거절 해제, 아직 넘기지 않은 상대 답이 있으면 바로 넘김).
    /// 받은 쪽: 끄기만(거절 + 알림).
    pub async fn relay(
        self: &Arc<Self>,
        conversation_id: i64,
        on: bool,
    ) -> io::Result<Conversation> {
        let conv = chat::by_id(&self.db(), conversation_id)
            .map_err(db_err)?
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "없는 대화"))?;
        let peer = self.paired_peer(conv.peer_id)?;
        match (conv.dir, on) {
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
                    let mut db = self.db();
                    let tx = db.transaction().map_err(db_err)?;
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
        } = reply
        else {
            return Ok(Response::error("bad_request"));
        };
        if text.len() > MAX_TEXT || msg.is_empty() {
            return Ok(Response::error("bad_request"));
        }
        let agent = {
            let mut db = self.db();
            let Some(c) = chat::find(&db, &conv, peer.id)
                .map_err(db_err)?
                .filter(|c| c.dir == Dir::Out)
            else {
                return Ok(Response::error("unknown"));
            };
            if chat::message_by_uuid(&db, &msg).map_err(db_err)?.is_some() {
                return Ok(Response::Ok); // 같은 답 재전송
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
            let tx = db.transaction().map_err(db_err)?;
            let reply =
                chat::add_reply(&tx, c.id, &msg, &text, state, asked.id, turns, duration_ms)
                    .map_err(db_err)?;
            chat::observe(&tx, reply.id, lc).map_err(db_err)?;
            chat::set_state(&tx, asked.id, state).map_err(db_err)?;
            // 자동 왕복 중이고 성공한 답이며 상한 전이면 이 답을 내 에이전트 입력으로(연 쪽 구동, D24). 답 기록과 한 트랜잭션 —
            // 사이에 끊겨도 "켜짐인데 대기열 없음" 이 남지 않는다.
            match c.relay.as_ref().filter(|r| r.on) {
                Some(r) if ok && r.count < r.cap => {
                    chat::set_flags(&tx, reply.id, false, true, State::Queued).map_err(db_err)?;
                    relay_next = Some(c.id);
                }
                // 상한 · 상대 실행 실패 — 멈추고 사람 입력을 기다린다.
                Some(_) => {
                    chat::relay_on(&tx, c.id, false, false).map_err(db_err)?;
                    ended = Some(if ok {
                        RELAY_END_CAP
                    } else {
                        "상대 에이전트 실행 실패"
                    });
                }
                None => {}
            }
            tx.commit().map_err(db_err)?;
            c.agent
        };
        if let Some(id) = relay_next {
            self.wake(id);
        }
        self.notify(peer, agent.clone(), kind, preview(&text));
        if let Some(reason) = ended {
            self.notify(peer, agent, MessageKind::RelayEnd, reason.to_string());
        }
        Ok(Response::Ok)
    }

    /// 대화 작업을 깨운다 — 없으면 만든다(대화 안에서는 DB 순서대로 하나씩).
    fn wake(self: &Arc<Self>, conversation_id: i64) {
        let mut queues = self.queues.lock().expect("대기열 잠금이 오염됐다");
        let tx = queues.entry(conversation_id).or_insert_with(|| {
            let (tx, mut rx) = mpsc::unbounded_channel::<()>();
            let me = self.clone();
            tokio::spawn(async move {
                // 한 번 꺼낸 메시지는 다시 꺼내지 않는다 — 상태 저장이 실패해도 같은 메시지를 되풀이하지 않게. 커서(가장 큰 id)가
                // 아니라 꺼낸 것의 집합 — 낮은 id 가 늦게 `queued` 가 돼도 빠지지 않는다.
                let mut taken = std::collections::HashSet::new();
                while rx.recv().await.is_some() {
                    loop {
                        let next = chat::pending(&me.db(), conversation_id)
                            .map(|ms| ms.into_iter().find(|m| !taken.contains(&m.id)));
                        match next {
                            Ok(Some(m)) => {
                                taken.insert(m.id);
                                me.process(conversation_id, m).await;
                            }
                            Ok(None) => break,
                            Err(e) => {
                                log::error!("대화 {conversation_id} 대기열 읽기 실패: {e}");
                                break;
                            }
                        }
                    }
                }
            });
            tx
        });
        if tx.send(()).is_err() {
            log::error!("대화 {conversation_id} 작업이 멈췄다");
        }
    }

    /// 사람 메시지 하나 — 실행 → 기록(한 트랜잭션) → 답 보내기. 기록이 실패하면 성공 답을 보내지 않는다.
    async fn process(&self, conversation_id: i64, asked: Message) {
        let loaded = {
            let db = self.db();
            chat::by_id(&db, conversation_id).and_then(|c| {
                let agent = match &c {
                    Some(c) => agents::by_name(&db, &c.agent)?,
                    None => None,
                };
                Ok(c.map(|c| (c, agent)))
            })
        };
        let (conv, agent) = match loaded {
            Ok(Some(found)) => found,
            Ok(None) => return log::error!("대화 {conversation_id} 를 찾지 못했다"),
            Err(e) => return log::error!("대화 {conversation_id} 읽기 실패: {e}"),
        };
        // 연 쪽에서 실행 요청이 된 상대 에이전트 답 = 자동 왕복 — 실행 에이전트는 내 에이전트.
        let relaying = conv.dir == Dir::Out && asked.author == Author::Agent;
        // 그사이 멈췄으면(사람이 씀 · 멈춤) 넘기지 않은 상대 답으로 되돌린다 — 재개 때 다시 넘긴다.
        if relaying && !conv.relay.as_ref().is_some_and(|r| r.on) {
            let _ = chat::set_flags(&self.db(), asked.id, false, false, State::Done);
            return;
        }
        let agent = if relaying {
            let _ = chat::relay_step(&self.db(), conv.id);
            conv.relay
                .as_ref()
                .and_then(|r| agents::by_name(&self.db(), &r.agent).ok().flatten())
        } else {
            agent
        };
        let result = if asked.state == State::Running {
            Err(INTERRUPTED.to_string())
        } else if let Some(agent) = agent {
            self.execute(&conv, agent, &asked).await
        } else {
            Err(format!("에이전트 {} 가 삭제됐다", conv.agent))
        };
        let (text, ok, run) = match result {
            Ok(out) => (out.text.clone(), !out.is_error, Some(out)),
            Err(e) => (format!("실행 실패: {e}"), false, None),
        };
        let mut text = fit(text);
        let mut ok = ok;
        let reply_uuid = new_uuid();
        let state = if ok { State::Done } else { State::Error };
        let recorded = self.record(&conv, &asked, run.as_ref(), &reply_uuid, &text, state);
        if let Err(e) = &recorded {
            log::error!("대화 {} 기록 실패: {e}", conv.uuid);
            let _ = chat::set_state(&self.db(), asked.id, State::Error);
            (text, ok) = (format!("실행 실패: 받는 PC 기록 오류 — {e}"), false);
        }
        let (turns, duration_ms) = (
            run.as_ref().and_then(|o| o.turns),
            run.as_ref().and_then(|o| o.duration_ms),
        );
        let lc = recorded
            .as_ref()
            .ok()
            .and_then(|id| {
                chat::messages(&self.db(), conv.id)
                    .ok()?
                    .into_iter()
                    .find(|m| m.id == *id)
            })
            .map(|m| m.lc);
        if relaying {
            // 내 에이전트 답 — 이 PC 글로.
            if let Ok(id) = recorded {
                let _ = chat::set_flags(&self.db(), id, true, false, state);
            }
            return self
                .relay_forward(&conv, recorded.ok(), ok, &text, lc, Some(&asked.uuid))
                .await;
        }
        // 받은 쪽: 자동 왕복 요청(에이전트발 · 자동 왕복을 연 첫 사람 요청)을 내 에이전트가 실패하면 자동 왕복이 끝난다.
        let relay_request = conv.dir == Dir::In
            && (asked.author == Author::Agent
                || (conv.relay.is_some()
                    && !asked.mine
                    && matches!(
                        chat::last_exec_before(&self.db(), conv.id, &asked),
                        Ok(None)
                    )));
        if !ok && relay_request {
            self.relay_end(&conv, "내 에이전트 실행 실패");
        }
        // 상대의 실행 요청이면 답(`reply`), 이 PC 사람의 실행 요청이면 상대 이력에 미러(`post`).
        let req = if asked.mine {
            Request::Post {
                conv: conv.uuid.clone(),
                msg: reply_uuid,
                author: "agent".into(),
                text,
                exec: false,
                reply_to: Some(asked.uuid.clone()),
                ok: Some(ok),
                turns,
                duration_ms,
                lc,
            }
        } else {
            Request::Reply {
                conv: conv.uuid.clone(),
                msg: reply_uuid,
                reply_to: asked.uuid.clone(),
                text,
                ok,
                turns,
                duration_ms,
                lc,
            }
        };
        let delivered = match self.paired_peer(conv.peer_id) {
            Ok(peer) => match addr(&peer) {
                Ok(a) => matches!(
                    self.node.send(a, Some(peer.spki.clone()), &req).await,
                    Ok((Response::Ok, _))
                ),
                Err(_) => false,
            },
            Err(_) => false,
        };
        if !delivered {
            log::warn!("대화 {} 답을 전달하지 못했다", conv.uuid);
            if let Ok(id) = recorded {
                let _ = chat::set_state(&self.db(), id, State::Undelivered);
            }
        }
    }

    /// 자동 왕복 — 내 에이전트 답을 상대 에이전트 실행 요청으로 보낸다. 실패 · 멈춤 · 거절이면 자동 왕복을 끈다.
    async fn relay_forward(
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
            };
            let sent = match self.paired_peer(conv.peer_id) {
                Ok(peer) => match addr(&peer) {
                    Ok(a) => self.node.send(a, Some(peer.spki.clone()), &req).await,
                    Err(e) => Err(e),
                },
                Err(e) => Err(e),
            };
            if !matches!(sent, Ok((Response::Ok, _))) {
                log::warn!("대화 {} 자동 왕복 마지막 답 기록 실패", conv.uuid);
                let _ = chat::set_state(&self.db(), msg.id, State::Undelivered);
            }
            return;
        }
        // 보내기 전에 `sent` 실행 요청으로 — 저장 못 하면 보내지 않는다(늦게 온 답을 받을 근거가 없다).
        if let Err(e) = chat::set_flags(&self.db(), msg.id, true, true, State::Sent) {
            log::error!("대화 {} 자동 왕복 상태 저장 실패 — 멈춤: {e}", conv.uuid);
            let _ = chat::relay_on(&self.db(), conv.id, false, false);
            self.relay_end(conv, "상태 저장 실패");
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
        };
        let sent = match self.paired_peer(conv.peer_id) {
            Ok(peer) => match addr(&peer) {
                Ok(a) => self.node.send(a, Some(peer.spki.clone()), &req).await,
                Err(e) => Err(e),
            },
            Err(e) => Err(e),
        };
        if !matches!(sent, Ok((Response::Ok, _))) {
            log::warn!("대화 {} 자동 왕복 전달 실패 — 멈춤", conv.uuid);
            let _ = chat::relay_on(&self.db(), conv.id, false, false);
            // 명시적 거절만 실패로 확정한다. 전송 오류는 상대가 받았는지 모른다 — `sent` 로 두어 늦게 온 답을 받는다.
            if sent.is_ok() {
                let _ = chat::set_state(&self.db(), msg.id, State::Error);
                self.relay_end(conv, "상대가 거절");
            } else {
                self.relay_end(conv, "전달 확인 안 됨");
            }
        }
    }

    /// `running` 표시 후 실행. 표시를 못 하면 실행하지 않는다(재시작 뒤 중복 실행 판별이 안 되므로).
    async fn execute(
        &self,
        conv: &Conversation,
        agent: agents::Agent,
        asked: &Message,
    ) -> Result<RunOutput, String> {
        chat::set_state(&self.db(), asked.id, State::Running).map_err(|e| e.to_string())?;
        let session = conv
            .session
            .clone()
            .map(Session::Resume)
            .unwrap_or_else(|| Session::New(new_uuid()));
        let input = RunInput {
            agent,
            session,
            text: self.with_notes(conv, asked).map_err(|e| e.to_string())?,
        };
        let run = self.run.clone();
        match tokio::task::spawn_blocking(move || run(input)).await {
            Ok(Ok(out)) => Ok(out),
            Ok(Err(e)) => Err(e.to_string()),
            Err(e) => Err(format!("실행 작업 실패: {e}")),
        }
    }

    /// 실행 입력 — 지난 실행 요청 뒤 그 사이 사람 글을 발화자와 함께 앞에 붙인다(없으면 요청 본문 그대로).
    fn with_notes(
        &self,
        conv: &Conversation,
        asked: &Message,
    ) -> Result<String, an_core::db::Error> {
        let notes = {
            let db = self.db();
            let after = chat::last_exec_before(&db, conv.id, asked)?;
            chat::notes_between(&db, conv.id, after.as_ref(), asked)?
        };
        // 자동 왕복의 에이전트 글 = 상대 에이전트의 요청(D24) — 사람 요청과 구분해 표시한다.
        let from_agent = asked.author == Author::Agent;
        if notes.is_empty() && !from_agent {
            return Ok(asked.text.clone());
        }
        let peer_name = self
            .node
            .peers()
            .ok()
            .and_then(|ps| ps.into_iter().find(|p| p.id == conv.peer_id))
            .map(|p| format!("{} · {}", p.name, p.host))
            .unwrap_or_else(|| "상대".into());
        let who = |m: &Message| {
            if m.author == Author::Agent {
                format!("상대 에이전트({peer_name})")
            } else if m.mine {
                "이 PC 사용자".to_string()
            } else {
                peer_name.clone()
            }
        };
        let mut text = String::new();
        if !notes.is_empty() {
            text.push_str("[대화 메모 — 실행 요청 아님]\n");
            for n in &notes {
                text.push_str(&format!("- {}: {}\n", who(n), n.text));
            }
            text.push('\n');
        }
        text.push_str(&format!("[요청 — {}]\n{}", who(asked), asked.text));
        Ok(text)
    }

    /// 세션 · 사람 메시지 상태 · 에이전트 답(원 요청 · 실행 정보)을 한 트랜잭션으로. 돌려주는 값: 답 메시지 id.
    fn record(
        &self,
        conv: &Conversation,
        asked: &Message,
        run: Option<&RunOutput>,
        reply_uuid: &str,
        text: &str,
        state: State,
    ) -> Result<i64, an_core::db::Error> {
        let mut db = self.db();
        let tx = db.transaction()?;
        if let Some(run) = run {
            chat::set_session(&tx, conv.id, &run.session)?;
        }
        chat::set_state(&tx, asked.id, state)?;
        let (turns, duration_ms) = run.map_or((None, None), |o| (o.turns, o.duration_ms));
        let reply = chat::add_reply(
            &tx,
            conv.id,
            reply_uuid,
            text,
            state,
            asked.id,
            turns,
            duration_ms,
        )?;
        tx.commit()?;
        Ok(reply.id)
    }
}
