//! 에이전트 실행 — 깨우기 · 실행 · 결과 기록.

use super::*;

/// 실행 입력의 구분자 — 원격 글 안에서는 지운다.
const FRAMES: [&str; 6] = [
    "</원격 본문>",
    "<원격 본문>",
    "</대화 메모>",
    "<대화 메모>",
    "</대화 요약>",
    "<대화 요약>",
];

/// 원격 글에서 구분자 글자를 지운다(지운 자리에 새 구분자가 생기지 않을 때까지).
fn unframe(text: &str) -> String {
    let mut t = text.to_string();
    while let Some(f) = FRAMES.iter().find(|f| t.contains(**f)) {
        t = t.replace(f, "");
    }
    t
}

/// `MAX_TEXT` 바이트 안으로(글자 경계에서) 자른다.
fn fit(text: String) -> String {
    fit_to(text, MAX_TEXT)
}

/// 끝난 실행 — 결과 · 실행한 에이전트 · 정한 실행 옵션.
struct Ran {
    out: RunOutput,
    agent: agents::Agent,
    model: String,
    effort: String,
}

/// 실행 실패 — 내 쪽 행 · 로그에 남길 상세와 상대에게 보내는 정해진 문구(stderr · 경로 · DB 오류를 싣지 않는다).
struct Failure {
    detail: String,
    public: &'static str,
}

impl Failure {
    /// 실행기 오류 — 종류로 문구를 고른다.
    /// `codex` = Codex 에이전트의 실행 — 버전 · 찾지 못함 문구의 도구 이름.
    fn run(e: &std::io::Error, codex: bool) -> Self {
        let public = match (e.kind(), codex) {
            (std::io::ErrorKind::TimedOut, _) => "실행 실패: 받는 PC 의 시간 제한(30분)을 넘겼다",
            (std::io::ErrorKind::Unsupported, false) => "실행 실패: 받는 PC 의 Claude Code 버전이 낮다",
            (std::io::ErrorKind::Unsupported, true) => "실행 실패: 받는 PC 의 Codex CLI 버전이 낮다",
            (std::io::ErrorKind::NotFound, false) => "실행 실패: 받는 PC 에서 Claude Code 를 찾지 못했다",
            (std::io::ErrorKind::NotFound, true) => "실행 실패: 받는 PC 에서 Codex CLI 를 찾지 못했다",
            _ => RUN_FAILED,
        };
        Failure {
            detail: e.to_string(),
            public,
        }
    }

    /// 받는 PC 의 기록 · 조회 오류.
    fn record(e: impl std::fmt::Display) -> Self {
        Failure {
            detail: format!("받는 PC 기록 오류 — {e}"),
            public: RECORD_FAILED,
        }
    }
}

const RUN_FAILED: &str = "실행 실패: 받는 PC 에서 실행하지 못했다 — 원인은 받는 PC 화면에 있다";
const RECORD_FAILED: &str = "실행 실패: 받는 PC 기록 오류";
const RESULT_ERROR: &str =
    "실행 실패: 받는 PC 의 에이전트가 오류로 끝났다 — 원인은 받는 PC 화면에 있다";
const EMPTY_ANSWER: &str = "실행 실패: 받는 PC 의 에이전트가 빈 답을 냈다";
const FOLDER_BLOCKED: &str =
    "실행 실패: 받는 PC 의 에이전트 폴더를 쓸 수 없다 — 받는 PC 에서 폴더를 바꿔야 한다";

impl Chat {
    /// 대화 작업을 깨운다 — 없으면 만든다(대화 안에서는 DB 순서대로 하나씩).
    pub(super) fn wake(self: &Arc<Self>, conversation_id: i64) {
        let mut queues = an_core::sync::lock(&self.queues);
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
    async fn process(self: &Arc<Self>, conversation_id: i64, asked: Message) {
        if asked.author == Author::Context {
            return self.run_context(conversation_id, asked).await;
        }
        // 실행 문 — 먼저 줄을 서고, 차례가 온 뒤의 대화 · 에이전트 · 멈춤 상태로 판단한다(기다리는 동안 바뀐 것을 따른다).
        let ticket = self.admit(conversation_id, &asked).await;
        // 실행 시작 = 대기 글 · 열린 대화일 때만 `running` 으로 한 번에(줄을 서는 동안 나가기 · 상대 나감 · 삭제가 취소했으면 실행하지 않는다).
        // 앱이 실행 도중 끝난 글(`running`)은 아래에서 실패 답으로 끝낸다.
        if asked.state != State::Running {
            let started = chat::start_run(&self.db(), asked.id);
            match started {
                Ok(true) => {}
                Ok(false) => return,
                Err(e) => return log::error!("대화 {conversation_id} 실행 시작 기록 실패: {e}"),
            }
        }
        let loaded = {
            let db = self.db();
            chat::by_id(&db, conversation_id).and_then(|c| {
                // 실행 에이전트를 대화(세션)와 같은 DB 시점에서 — 따로 다시 읽으면 그 사이 폴더 · 상위 읽기가 바뀌어 비운 세션을 새 세대로
                // 이어 쓴다.
                let agent = match &c {
                    // 연 쪽에서 실행 요청이 된 상대 에이전트 답 = 자동 왕복 — 내 왕복 에이전트.
                    Some(c) if c.dir == Dir::Out && asked.author == Author::Agent => match &c.relay {
                        Some(r) => agents::by_name(&db, &r.agent)?,
                        None => None,
                    },
                    // 멘션이면 부른 에이전트.
                    Some(c) => agents::by_name(&db, asked.agent.as_deref().unwrap_or(&c.agent))?,
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
        if conv.dir == Dir::In && !asked.mine {
            self.send_progress(&conv, &asked, "running", 0);
        }
        // 연 쪽에서 실행 요청이 된 상대 에이전트 답 = 자동 왕복 — 실행 에이전트는 내 에이전트.
        let relaying = conv.dir == Dir::Out && asked.author == Author::Agent;
        // 그사이 멈췄으면(사람이 씀 · 멈춤) 넘기지 않은 상대 답으로 되돌린다 — 재개 때 다시 넘긴다.
        if relaying && !conv.relay.as_ref().is_some_and(|r| r.on) {
            let _ = chat::set_flags(&self.db(), asked.id, false, false, State::Done);
            return;
        }
        if relaying {
            let _ = chat::relay_step(&self.db(), conv.id);
        }
        // 실패는 (내 쪽 글, 상대에게 보낼 글) 둘 — 중단 · 에이전트 삭제는 상대도 알아도 되는 사실이라 같은 글.
        let result = if asked.state == State::Running {
            let t = format!("실행 실패: {INTERRUPTED}");
            Err((t.clone(), t))
        } else if let Some(agent) = agent {
            self.execute(&conv, agent, &asked)
                .await
                .map_err(|f| (format!("실행 실패: {}", f.detail), f.public.to_string()))
        } else {
            let t = format!(
                "실행 실패: 에이전트 {} 가 삭제됐다",
                asked.agent.as_deref().unwrap_or(&conv.agent)
            );
            Err((t.clone(), t))
        };
        drop(ticket);
        let (text, public, ok, run) = match result {
            // 빈 답은 실패로 — 받는 쪽이 받지 않는다.
            Ok(ran) if ran.out.text.trim().is_empty() => (
                EMPTY_ANSWER.to_string(),
                Some(EMPTY_ANSWER.to_string()),
                false,
                Some(ran),
            ),
            // claude 가 오류로 끝난 결과(`is_error`)는 원문을 내 쪽에만 — 인증 · 사용량 · 경로가 실릴 수 있다.
            Ok(ran) if ran.out.is_error => (
                ran.out.text.clone(),
                Some(RESULT_ERROR.to_string()),
                false,
                Some(ran),
            ),
            Ok(ran) => (ran.out.text.clone(), None, true, Some(ran)),
            Err((mine, theirs)) => (mine, Some(theirs), false, None),
        };
        let mut text = fit(text);
        let mut public = public;
        let mut ok = ok;
        let reply_uuid = new_uuid();
        let state = if ok { State::Done } else { State::Error };
        let (turns, duration_ms) = (
            run.as_ref().and_then(|r| r.out.turns),
            run.as_ref().and_then(|r| r.out.duration_ms),
        );
        let (model, effort) = (
            run.as_ref().map(|r| r.model.clone()).filter(|m| !m.is_empty()),
            run.as_ref().map(|r| r.effort.clone()).filter(|e| !e.is_empty()),
        );
        let ctx = run.as_ref().and_then(|r| r.out.ctx_tokens);
        // 상대의 실행 요청이면 답(`reply`), 이 PC 사람의 실행 요청이면 상대 이력에 미러(`post`).
        let reply_req = |sent: String, lc: Option<i64>, ok: bool| {
            if asked.mine {
                Request::Post {
                    conv: conv.uuid.clone(),
                    msg: reply_uuid.clone(),
                    author: "agent".into(),
                    text: sent,
                    exec: false,
                    reply_to: Some(asked.uuid.clone()),
                    ok: Some(ok),
                    turns,
                    duration_ms,
                    lc,
                    via: None,
                    model: model.clone(),
                    effort: effort.clone(),
                    request: false,
                    ctx,
                }
            } else {
                Request::Reply {
                    conv: conv.uuid.clone(),
                    msg: reply_uuid.clone(),
                    reply_to: asked.uuid.clone(),
                    text: sent,
                    ok,
                    turns,
                    duration_ms,
                    lc,
                    model: model.clone(),
                    effort: effort.clone(),
                    ctx,
                }
            }
        };
        let kind = if asked.mine { REPLY_POST } else { REPLY };
        // 답과 보내기 행을 한 트랜잭션으로(기록 뒤 보내기 전에 꺼져도 줄이 보낸다). 자동 왕복 넘김은 `relay_forward` 가 보낸다.
        let outgoing = public.clone().unwrap_or_else(|| text.clone());
        let staging =
            (!relaying).then_some((kind, |lc: i64| reply_req(outgoing.clone(), Some(lc), ok)));
        let recorded = self.record(
            &conv,
            &asked,
            run.as_ref(),
            &reply_uuid,
            (&text, state),
            staging
                .as_ref()
                .map(|(k, f)| (*k, f as &dyn Fn(i64) -> Request)),
        );
        let (recorded, staged, closed) = match recorded {
            Ok(Some((id, staged, closed))) => (Ok(id), staged, closed),
            Ok(None) => return log::info!("대화 {} 를 지웠다 — 답을 버린다", conv.uuid),
            Err(e) => (Err(e), None, false),
        };
        // 자동 압축 — 이 PC 에서 돈 세션이 문턱을 넘었다. 구분선 글은 대화 줄 맨 앞에 서서 다음 글보다 먼저 처리된다.
        if recorded.is_ok()
            && ok
            && !closed
            && ctx.is_some_and(|n| n >= an_core::settings::AUTO_COMPACT_AT)
        {
            let on = an_core::settings::auto_compact(&self.db()).unwrap_or_else(|e| {
                log::warn!("자동 압축 설정 읽기 실패 — 켜짐으로 본다: {e}");
                true
            });
            if on && let Err(e) = self.request_context(&conv, "auto_compact") {
                log::warn!("대화 {} 자동 압축 요청 실패: {e}", conv.uuid);
            }
        }
        if let Err(e) = &recorded {
            log::error!("대화 {} 기록 실패: {e}", conv.uuid);
            let _ = chat::set_state(&self.db(), asked.id, State::Error);
            (text, ok) = (format!("실행 실패: 받는 PC 기록 오류 — {e}"), false);
            public = Some(RECORD_FAILED.to_string());
        }
        // 상대에게 가는 글 — 실패면 정해진 문구.
        let sent = public.unwrap_or_else(|| text.clone());
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
            if closed {
                return;
            }
            return self
                .relay_forward(&conv, recorded.ok(), ok, &sent, lc, Some(&asked.uuid))
                .await;
        }
        // 나간 대화 — 답은 이력에만(상대에게 보내지 않는다).
        if closed {
            return;
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
        // 닿지 않으면 보관 — 줄이 다시 보낸다. 상대가 거절했을 때만 지금 "답 미전달". 기록이 실패했으면(행 없음) 정해진 문구로 지금 적고 보낸다.
        let req = reply_req(sent, lc, ok);
        let shipped = match staged {
            Some(id) => self.dispatch(&conv, id, &req).await,
            None => {
                self.ship(&conv, recorded.as_ref().ok().copied(), kind, &req)
                    .await
            }
        };
        let delivered = matches!(shipped, Ok(Shipped::Answered(Response::Ok) | Shipped::Kept));
        if !delivered {
            log::warn!("대화 {} 답을 전달하지 못했다", conv.uuid);
            if let Ok(id) = recorded {
                let _ = chat::set_state(&self.db(), id, State::Undelivered);
            }
        }
    }

    /// 집중(이 PC 설정) 중인가 — 못 읽으면 받는다: 실행이 같은 설정 오류로 실패 답을 낸다(기본값으로 감추지 않는다).
    pub(super) fn focused(&self, conn: &Connection) -> bool {
        an_core::settings::focus(conn).unwrap_or_else(|e| {
            log::warn!("집중 설정 읽기 실패 — 받는다: {e}");
            false
        })
    }

    /// 이미 받은 글의 재전송인가 — 같은 uuid 가 있고 자동 왕복 재개의 승격(상한에서 기록만 받은 상대 에이전트 글 → 실행 요청)이 아니다.
    pub(super) fn resent(&self, peer_id: i64, conv: &str, msg: &str, by_agent: bool) -> io::Result<bool> {
        let db = self.db();
        let Some(old) = chat::message_by_uuid(&db, msg).map_err(db_err)? else {
            return Ok(false);
        };
        let same = chat::find_own(&db, conv, peer_id)
            .map_err(db_err)?
            .is_some_and(|c| c.id == old.conversation_id);
        Ok(!(by_agent && same && old.author == Author::Agent && !old.mine && !old.exec))
    }

    /// 상대가 거절한 내 글 — 실패 + 거절 코드. 집중(`focus`)이면 그 상대를 접속 · 집중으로 남긴다(거절도 접속 증거 — 다음 Ping 을 기다리지
    /// 않게).
    pub(super) fn refused_by_peer(&self, msg_id: i64, peer_id: i64, resp: &Response) {
        let db = self.db();
        let _ = chat::set_state(&db, msg_id, State::Error);
        if let Response::Error { code } = resp {
            let _ = chat::set_refused(&db, msg_id, Some(an_net::proto::clean_code(code)));
            if code == FOCUS {
                let _ = an_core::peers::set_presence(&db, peer_id, true);
            }
        }
    }

    /// `running` 표시(`process` 가 먼저 한다) 뒤 실행. 표시를 못 하면 실행하지 않는다(재시작 뒤 중복 실행 판별이 안 되므로).
    async fn execute(
        &self,
        conv: &Conversation,
        agent: agents::Agent,
        asked: &Message,
    ) -> Result<Ran, Failure> {
        // 등록 · 수정 때 거른 규칙을 실행 직전에 다시 — 규칙이 생기기 전에 등록된 폴더 · 등록 뒤 링크가 바뀐 폴더.
        if let Some(why) = an_core::paths::agent_folder_problem(std::path::Path::new(&agent.folder))
        {
            return Err(Failure {
                detail: format!("에이전트 {} 폴더 — {why}", agent.name),
                public: FOLDER_BLOCKED,
            });
        }
        // `running` 표시는 `process` 가 했다(`chat::start_run` — 조건부).
        // 세션 — 대화 에이전트는 대화의 것, 멘션한 에이전트는 대화 × 에이전트.
        let saved = match asked.agent.as_deref() {
            Some(name) => {
                chat::agent_session(&self.db(), conv.id, name).map_err(Failure::record)?
            }
            None => conv.session.clone(),
        };
        let session = saved
            .map(Session::Resume)
            .unwrap_or_else(|| Session::New(new_uuid()));
        let resumed = matches!(session, Session::Resume(_));
        // 그 상대의 덮어쓰기 → 전역 설정 → 기본값. Codex 에이전트는 모델 · effort 인자 없이 그 PC Codex 기본값 — 답 기록 · 상대 meta 에 싣지 않는다.
        let codex = agent.engine == an_core::agents::CODEX;
        let (model, effort) = if codex {
            (String::new(), String::new())
        } else {
            let db = self.db();
            let peer = an_core::peers::by_id(&db, conv.peer_id).map_err(Failure::record)?;
            an_core::run_opts::resolve(&db, peer.as_ref()).map_err(Failure::record)?
        };
        // 상대가 보낸 실행 요청이면 단계 요약을 `step_every` 에 한 번 이하로 보낸 쪽에 알린다 — 실행기는 바뀔 때마다 마지막 값을 두고,
        // 이 작업이 그 사이 바뀐 것만 보낸다.
        let latest: Arc<Mutex<Option<StepDetail>>> = Arc::default();
        let remote = conv.dir == Dir::In && !asked.mine;
        let steps = if remote {
            let l = latest.clone();
            StepSink::new(move |d| *an_core::sync::lock(&l) = Some(d.clone()))
        } else {
            StepSink::default()
        };
        let ticker = self.me.upgrade().filter(|_| remote).map(|me| {
            let (conv, asked, every) = (
                conv.clone(),
                asked.clone(),
                *an_core::sync::lock(&self.step_every),
            );
            let latest = Arc::downgrade(&latest);
            tokio::spawn(async move {
                loop {
                    tokio::time::sleep(every).await;
                    let Some(latest) = latest.upgrade() else {
                        return;
                    };
                    let step = an_core::sync::lock(&latest).take();
                    if let Some(d) = step {
                        me.send_step(&conv, &asked, "running", 0, Some(d));
                    }
                }
            })
        });
        let mut text = self.with_notes(conv, asked).map_err(Failure::record)?;
        // 압축 뒤 첫 실행 — 새 세션 앞에 요약을 한 번 붙인다(세션을 남기면 `record` 가 지운다).
        if !resumed {
            let seed = chat::ctx_seed(&self.db(), conv.id, asked.agent.as_deref().unwrap_or(""))
                .map_err(Failure::record)?;
            if let Some(seed) = seed {
                text = format!(
                    "[이전 대화 요약 — 컨텍스트 압축, 실행 요청 아님]\n<대화 요약>\n{}\n</대화 요약>\n\n{text}",
                    unframe(&seed)
                );
            }
        }
        let input = RunInput {
            agent: agent.clone(),
            session,
            text,
            model: model.clone(),
            effort: effort.clone(),
            steps,
        };
        self.note_cmd(conv.id, &input);
        let run = self.run.clone();
        let first = input.clone();
        let ran = match tokio::task::spawn_blocking(move || run(first)).await {
            // 이어 쓸 세션이 사라졌다 — 새 세션으로 한 번 더. 답과 새 세션은 아래 기록에서 그대로 남아 다음 요청은 새 세션을 잇는다.
            Ok(Err(e)) if resumed && crate::runner::session_missing(&e) => {
                log::warn!("대화 {} — {e} · 새 세션으로 다시 실행", conv.id);
                let run = self.run.clone();
                let again = RunInput {
                    session: Session::New(new_uuid()),
                    ..input
                };
                self.note_cmd(conv.id, &again);
                tokio::task::spawn_blocking(move || run(again)).await
            }
            other => other,
        };
        if let Some(t) = ticker {
            t.abort();
        }
        match ran {
            Ok(Ok(out)) => Ok(Ran {
                out,
                agent,
                model,
                effort,
            }),
            Ok(Err(e)) => Err(Failure::run(&e, codex)),
            Err(e) => Err(Failure {
                detail: format!("실행 작업 실패: {e}"),
                public: RUN_FAILED,
            }),
        }
    }

    /// 대화의 마지막 실행 명령 — 띄우기 직전에 남긴다(실패한 실행도). 못 남겨도 실행은 그대로(경고만).
    fn note_cmd(&self, conversation_id: i64, input: &RunInput) {
        let line = crate::runner::launch_line(input);
        if let Err(e) = chat::set_cmd(&self.db(), conversation_id, &line, chat::unix_now()) {
            log::warn!("대화 {conversation_id} 실행 명령 기록 실패: {e}");
        }
    }

    /// 실행 입력 — 지난 실행 요청 뒤 그 사이 사람 글(메모)을 발화자와 함께 앞에, 요청 본문은 늘 머리말 뒤 구분자 안에.
    /// 메모는 최근 [`MAX_NOTES`] 개 · 본문 합 [`MAX_NOTES_BYTES`] 까지(오래된 것부터 버린다). 본문 · 메모 안의 구분자 글자는
    /// 지운다 — 원격 글이 틀을 일찍 닫고 이 PC 사용자의 지시처럼 꾸미지 못하게.
    fn with_notes(
        &self,
        conv: &Conversation,
        asked: &Message,
    ) -> Result<String, an_core::db::Error> {
        let mut notes = {
            let db = self.db();
            let after = chat::last_exec_before(&db, conv.id, asked)?;
            chat::notes_between(&db, conv.id, after.as_ref(), asked)?
        };
        let mut bytes = 0;
        let keep = notes
            .iter()
            .rev()
            .take_while(|n| {
                bytes += n.text.len();
                bytes <= MAX_NOTES_BYTES
            })
            .take(MAX_NOTES)
            .count();
        let dropped = notes.len() - keep;
        notes.drain(..dropped);
        let peer_name = self
            .node
            .peers()
            .ok()
            .and_then(|ps| ps.into_iter().find(|p| p.id == conv.peer_id))
            .map(|p| unframe(&format!("{} · {}", p.name, p.host)))
            .unwrap_or_default();
        // 자동 왕복의 에이전트 글 = 상대 에이전트의 요청 — 사람 요청과 구분해 표시한다. 상대 이름은 상대가 정한다 — 늘 `상대(…)` 안에
        // 넣어 `이 PC 사용자` 를 흉내 내지 못하게 하고 구분자는 지운다.
        let who = |m: &Message| {
            if m.author == Author::Agent {
                format!("상대 에이전트({peer_name})")
            } else if m.mine {
                "이 PC 사용자".to_string()
            } else {
                format!("상대({peer_name})")
            }
        };
        let mut text = String::new();
        if !notes.is_empty() {
            text.push_str("[대화 메모 — 실행 요청 아님]\n<대화 메모>\n");
            if dropped > 0 {
                text.push_str(&format!("(앞의 메모 {dropped}개 생략)\n"));
            }
            for n in &notes {
                // 메모 안 줄바꿈 뒤는 들여 쓴다 — 본문에 `- 이 PC 사용자: …` 줄을 꾸며 넣어도 발화자 줄과 구별된다.
                let body = unframe(&n.text).replace('\n', "\n  ");
                text.push_str(&format!("- {}: {body}\n", who(n)));
            }
            text.push_str("</대화 메모>\n\n");
        }
        text.push_str(&format!(
            "[요청 — {}]\n<원격 본문>\n{}\n</원격 본문>",
            who(asked),
            unframe(&asked.text)
        ));
        Ok(text)
    }

    /// 세션 · 사람 메시지 상태 · 에이전트 답(원 요청 · 실행 정보)을 한 트랜잭션으로. `run` = 끝난 실행(실패면 없음).
    /// 돌려주는 값: (답 메시지 id, 답을 보낼 행 id, 나간 대화인가) — 실행하는 동안 대화를 지웠으면 `None`.
    fn record(
        &self,
        conv: &Conversation,
        asked: &Message,
        run: Option<&Ran>,
        reply_uuid: &str,
        (text, state): (&str, State),
        staging: Option<(&'static str, &dyn Fn(i64) -> Request)>,
    ) -> Result<Option<(i64, Option<i64>, bool)>, an_core::db::Error> {
        let db = self.db();
        let tx = an_core::db::write_tx(&db)?;
        // 실행하는 동안 지웠다 — 답을 남기지 않는다. 나간 대화면 이력에만(보낼 행 없음).
        let Some(now) = chat::by_id(&tx, conv.id)? else {
            return Ok(None);
        };
        let closed = now.closed();
        let staging = staging.filter(|_| !closed);
        if let Some(ran) = run {
            // 실행 도중 폴더가 바뀌었거나 지웠다 다시 등록했으면(세대가 다르다 — 세션을 지웠다) 옛 세션을 되살리지 않는다.
            if agents::by_name(&tx, &ran.agent.name)?.is_some_and(|now| now.uid == ran.agent.uid) {
                match asked.agent.as_deref() {
                    Some(name) => chat::set_agent_session(&tx, conv.id, name, &ran.out.session)?,
                    None => chat::set_session(&tx, conv.id, &ran.out.session)?,
                }
                chat::drop_ctx_seed(&tx, conv.id, asked.agent.as_deref().unwrap_or(""))?;
            }
        }
        chat::set_state(&tx, asked.id, state)?;
        let reply = chat::add_reply(
            &tx,
            conv.id,
            reply_uuid,
            text,
            state,
            asked.id,
            run.and_then(|r| r.out.turns),
            run.and_then(|r| r.out.duration_ms),
            run.map(|r| r.model.as_str()).filter(|m| !m.is_empty()),
            run.map(|r| r.effort.as_str()).filter(|e| !e.is_empty()),
        )?;
        if asked.agent.is_some() {
            chat::set_agent(&tx, reply.id, asked.agent.as_deref())?;
        }
        chat::set_ctx_tokens(&tx, reply.id, run.and_then(|r| r.out.ctx_tokens))?;
        // 답을 보낼 행 — 답과 같은 트랜잭션.
        let staged = match staging {
            Some((kind, frame)) => {
                let frame = serde_json::to_string(&frame(reply.lc))
                    .map_err(|e| an_core::db::Error::ToSqlConversionFailure(Box::new(e)))?;
                Some(outbox::add(
                    &tx,
                    conv.peer_id,
                    Some(conv.id),
                    Some(reply.id),
                    kind,
                    &frame,
                    None,
                    "inflight",
                )?)
            }
            None => None,
        };
        tx.commit()?;
        Ok(Some((reply.id, staged, closed)))
    }
}
