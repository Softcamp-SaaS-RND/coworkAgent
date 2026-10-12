mod common;
use an_agent::chat::Compose;
use an_agent::runner::Session;
use an_core::chat::{self, Author, Dir, State};
use an_core::{agents, run_opts, settings};
use an_net::node::{Me, Node, PeerEvent};
use an_net::proto::{Request, Response};
use std::time::{Duration, Instant};

fn agent_messages(conn: &an_core::db::Connection, conv: i64) -> Vec<chat::Message> {
    chat::messages(conn, conv)
        .unwrap()
        .into_iter()
        .filter(|m| m.author == Author::Agent)
        .collect()
}

#[tokio::test(flavor = "multi_thread")]
async fn human_to_remote_agent_roundtrip() {
    let (a, b) = (
        common::side("rt-a", Duration::ZERO).await,
        common::side("rt-b", Duration::ZERO).await,
    );
    let (b_id, _) = common::pair(&a, &b).await;
    agents::add(&common::db(&b), "review", ".", None, "").unwrap();
    // A 가 B 를 부르는 표시 이름 — A 의 알림에 쓰인다(B 는 모른다).
    an_core::peers::set_tile(&common::db(&a), b_id, Some("검토팀"), None).unwrap();

    let (conv, sent) = a.chat.send(b_id, "review", None, "안녕").await.unwrap();
    assert_eq!(conv.dir, Dir::Out);
    assert_eq!((sent.author, sent.state), (Author::Human, State::Sent));
    // 답이 돌아와 A 이력에 남는다.
    let a_db = common::db(&a);
    common::eventually(|| agent_messages(&a_db, conv.id).len() == 1).await;
    let reply = agent_messages(&a_db, conv.id).remove(0);
    assert_eq!(
        (reply.text.as_str(), reply.state),
        ("답: 안녕", State::Done)
    );
    // 답이 오면 보낸 메시지도 끝 상태.
    assert_eq!(
        chat::messages(&a_db, conv.id).unwrap()[0].state,
        State::Done
    );
    // B 이력에도 같은 대화(같은 uuid)로 사람 · 에이전트 메시지가 남는다.
    let b_db = common::db(&b);
    let b_conv = chat::list(&b_db).unwrap().remove(0);
    assert_eq!(
        (b_conv.uuid.as_str(), b_conv.dir),
        (conv.uuid.as_str(), Dir::In)
    );
    let ms = chat::messages(&b_db, b_conv.id).unwrap();
    assert_eq!(ms.len(), 2);
    assert_eq!(
        (ms[0].author, ms[0].text.as_str(), ms[0].state),
        (Author::Human, "안녕", State::Done)
    );
    assert_eq!(
        (ms[1].author, ms[1].text.as_str(), ms[1].state),
        (Author::Agent, "답: 안녕", State::Done)
    );
    // 실행기는 프로필로 한 번, 새 세션으로.
    {
        let runs = b.runs.lock().unwrap();
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].agent.name, "review");
        assert!(matches!(runs[0].session, Session::New(_)));
    }
    // 알림 이벤트 — B 는 받은 요청, A 는 받은 답.
    assert!(
        b.events
            .lock()
            .unwrap()
            .iter()
            .any(|e| matches!(e, PeerEvent::Message { agent, .. } if agent == "review"))
    );
    common::eventually(|| {
        a.events
            .lock()
            .unwrap()
            .iter()
            .any(|e| matches!(e, PeerEvent::Message { name, .. } if name == "검토팀"))
    })
    .await;
    assert!(
        b.events
            .lock()
            .unwrap()
            .iter()
            .all(|e| !matches!(e, PeerEvent::Message { name, .. } if name == "검토팀")),
        "표시 이름은 이 PC 에만"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn followup_resumes_session() {
    let (a, b) = (
        common::side("fu-a", Duration::ZERO).await,
        common::side("fu-b", Duration::ZERO).await,
    );
    let (b_id, _) = common::pair(&a, &b).await;
    agents::add(&common::db(&b), "dev", ".", None, "").unwrap();
    let (conv, _) = a.chat.send(b_id, "dev", None, "첫 질문").await.unwrap();
    let a_db = common::db(&a);
    common::eventually(|| agent_messages(&a_db, conv.id).len() == 1).await;
    // 같은 대화에 이어 쓴다.
    let (again, _) = a
        .chat
        .send(b_id, "dev", Some(conv.id), "이어서")
        .await
        .unwrap();
    assert_eq!(again.id, conv.id);
    common::eventually(|| agent_messages(&a_db, conv.id).len() == 2).await;
    let runs = b.runs.lock().unwrap();
    let first = match &runs[0].session {
        Session::New(id) => id.clone(),
        other => panic!("첫 실행은 새 세션: {other:?}"),
    };
    assert_eq!(
        runs[1].session,
        Session::Resume(first.clone()),
        "저장된 세션으로 재개"
    );
    let b_db = common::db(&b);
    assert_eq!(
        chat::list(&b_db).unwrap()[0].session.as_deref(),
        Some(first.as_str())
    );
}

/// 실행 도중 폴더를 바꾸면(세션을 지운다) 끝난 옛 실행이 세션을 되살리지 않는다 — 다음 요청은 새 세션. 지우고 같은 이름 · 같은 폴더로
/// 다시 등록해도 같다(에이전트 세대가 다르다).
#[tokio::test(flavor = "multi_thread")]
async fn folder_change_during_run_drops_stale_session() {
    let (a, b) = (
        common::side("fc-a", Duration::ZERO).await,
        common::side("fc-b", Duration::from_millis(800)).await,
    );
    let (b_id, _) = common::pair(&a, &b).await;
    let b_db = common::db(&b);
    let dev = agents::add(&b_db, "dev", ".", None, "").unwrap();
    let (conv, _) = a.chat.send(b_id, "dev", None, "첫 질문").await.unwrap();
    common::eventually(|| b.runs.lock().unwrap().len() == 1).await;
    // 옛 폴더의 실행이 아직 끝나지 않았다.
    let elsewhere = std::env::temp_dir().display().to_string();
    agents::update(&b_db, dev.id, &elsewhere, &dev.tools, &dev.prompt, None).unwrap();
    let a_db = common::db(&a);
    common::eventually(|| agent_messages(&a_db, conv.id).len() == 1).await;
    assert_eq!(
        chat::list(&b_db).unwrap()[0].session,
        None,
        "옛 세션을 남기지 않는다"
    );
    a.chat
        .send(b_id, "dev", Some(conv.id), "이어서")
        .await
        .unwrap();
    common::eventually(|| agent_messages(&a_db, conv.id).len() == 2).await;
    {
        let runs = b.runs.lock().unwrap();
        assert!(
            matches!(runs[1].session, Session::New(_)),
            "{:?}",
            runs[1].session
        );
    }
    a.chat
        .send(b_id, "dev", Some(conv.id), "셋째")
        .await
        .unwrap();
    common::eventually(|| b.runs.lock().unwrap().len() == 3).await;
    // 실행 중에 지우고 같은 이름 · 같은 폴더로 다시 등록.
    let now = agents::by_name(&b_db, "dev").unwrap().unwrap();
    assert!(agents::remove(&b_db, now.id).unwrap());
    agents::add(&b_db, "dev", &elsewhere, None, "").unwrap();
    common::eventually(|| agent_messages(&a_db, conv.id).len() == 3).await;
    assert_eq!(
        chat::list(&b_db).unwrap()[0].session,
        None,
        "지운 에이전트의 세션을 되살리지 않는다"
    );
    a.chat
        .send(b_id, "dev", Some(conv.id), "넷째")
        .await
        .unwrap();
    common::eventually(|| agent_messages(&a_db, conv.id).len() == 4).await;
    let runs = b.runs.lock().unwrap();
    assert!(
        matches!(runs[3].session, Session::New(_)),
        "{:?}",
        runs[3].session
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn unpaired_or_unknown_agent_refused() {
    let (a, b) = (
        common::side("rf-a", Duration::ZERO).await,
        common::side("rf-b", Duration::ZERO).await,
    );
    let (b_id, _) = common::pair(&a, &b).await;
    agents::add(&common::db(&b), "review", ".", None, "").unwrap();
    // 페어링된 상대: 목록 조회는 된다, 없는 에이전트는 거부.
    assert_eq!(
        a.chat.remote_agents(b_id).await.unwrap(),
        vec!["review".to_string()]
    );
    let err = a.chat.send(b_id, "nope", None, "x").await.unwrap_err();
    assert!(err.to_string().contains("unknown_agent"), "{err}");
    // 너무 긴 본문은 보내기 전에(상대 거절이 아니라 여기서) 거부.
    let err = a
        .chat
        .send(b_id, "review", None, &"가".repeat(6000))
        .await
        .unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput, "{err}");
    // 페어링하지 않은 키: 목록 · 메시지 · 답 전부 not_paired, 실행 없음.
    let stranger = Node::new(
        an_net::identity::Identity::from_pkcs8(
            ring::signature::Ed25519KeyPair::generate_pkcs8(&ring::rand::SystemRandom::new())
                .unwrap()
                .as_ref()
                .to_vec(),
        )
        .unwrap(),
        an_core::db::Connection::open_in_memory().unwrap(),
        Me {
            name: "x".into(),
            host: "x".into(),
            port: 1,
        },
    );
    for req in [
        Request::Agents,
        Request::Message {
            conv: "c".into(),
            msg: "m".into(),
            agent: "review".into(),
            text: "rm -rf".into(),
            lc: None,
            by_agent: Some("mine".into()),
            relay: None,
            via: None,
            request: false,
            mention: None,
        },
        Request::Relay {
            conv: "c".into(),
            on: true,
        },
        Request::Reply {
            conv: "c".into(),
            msg: "m2".into(),
            reply_to: "m".into(),
            text: "x".into(),
            ok: true,
            turns: None,
            duration_ms: None,
            lc: None,
            model: None,
            effort: None,
        },
        Request::Done {
            conv: "c".into(),
            msg: "m".into(),
            done: true,
        },
    ] {
        let (resp, _) = stranger.send(b.addr, None, &req).await.unwrap();
        assert_eq!(resp, Response::error("not_paired"), "{req:?}");
    }
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert!(b.runs.lock().unwrap().is_empty());
}

#[tokio::test(flavor = "multi_thread")]
async fn inbound_does_not_wait_for_run() {
    let (a, b) = (
        common::side("nw-a", Duration::ZERO).await,
        common::side("nw-b", Duration::from_millis(700)).await,
    );
    let (b_id, _) = common::pair(&a, &b).await;
    agents::add(&common::db(&b), "slow", ".", None, "").unwrap();
    // 실행이 0.7초 걸려도 보내기는 바로 끝난다(받는 쪽이 대기열에 넣고 바로 응답).
    let started = Instant::now();
    let (conv, _) = a.chat.send(b_id, "slow", None, "하나").await.unwrap();
    a.chat
        .send(b_id, "slow", Some(conv.id), "둘")
        .await
        .unwrap();
    assert!(
        started.elapsed() < Duration::from_millis(500),
        "{:?}",
        started.elapsed()
    );
    // 같은 대화는 직렬 — 두 답이 보낸 순서대로, 둘째는 첫째 세션을 재개.
    let a_db = common::db(&a);
    common::eventually(|| agent_messages(&a_db, conv.id).len() == 2).await;
    let texts: Vec<_> = agent_messages(&a_db, conv.id)
        .into_iter()
        .map(|m| m.text)
        .collect();
    assert_eq!(texts, ["답: 하나", "답: 둘"]);
    assert!(matches!(
        b.runs.lock().unwrap()[1].session,
        Session::Resume(_)
    ));
}

#[tokio::test(flavor = "multi_thread")]
async fn forged_or_duplicate_reply_refused() {
    let (a, b) = (
        common::side("fg-a", Duration::ZERO).await,
        common::side("fg-b", Duration::ZERO).await,
    );
    let (b_id, _) = common::pair(&a, &b).await;
    agents::add(&common::db(&b), "review", ".", None, "").unwrap();
    let (conv, sent) = a.chat.send(b_id, "review", None, "안녕").await.unwrap();
    let a_db = common::db(&a);
    common::eventually(|| agent_messages(&a_db, conv.id).len() == 1).await;
    // 페어링된 상대라도: 이미 답이 온 메시지 · 없는 메시지에 대한 답은 받지 않는다.
    for (reply_to, msg) in [(sent.uuid.as_str(), "r2"), ("bogus", "r3")] {
        let req = Request::Reply {
            conv: conv.uuid.clone(),
            msg: msg.into(),
            reply_to: reply_to.into(),
            text: "위조".into(),
            ok: true,
            turns: None,
            duration_ms: None,
            lc: None,
            model: None,
            effort: None,
        };
        let (resp, _) = b.node.send(a.addr, None, &req).await.unwrap();
        assert_eq!(resp, Response::error("unknown"), "{reply_to}");
    }
    assert_eq!(agent_messages(&a_db, conv.id).len(), 1);
}

#[tokio::test(flavor = "multi_thread")]
async fn restart_recovers_queue() {
    let (a, b) = (
        common::side("rc-a", Duration::ZERO).await,
        common::side("rc-b", Duration::ZERO).await,
    );
    let (b_id, a_id) = common::pair(&a, &b).await;
    agents::add(&common::db(&b), "review", ".", None, "").unwrap();
    // 지난 실행에서 남은 상태: m1 은 실행 도중 종료(running), m2 는 대기(queued).
    let (a_db, b_db) = (common::db(&a), common::db(&b));
    let ca = chat::open_conversation(&a_db, "conv-r", b_id, "review", Dir::Out).unwrap();
    let cb = chat::open_conversation(&b_db, "conv-r", a_id, "review", Dir::In).unwrap();
    for (uuid, text, state) in [("m1", "하나", State::Running), ("m2", "둘", State::Queued)] {
        chat::add_message(&a_db, ca.id, uuid, Author::Human, text, State::Sent).unwrap();
        chat::add_message(&b_db, cb.id, uuid, Author::Human, text, state).unwrap();
    }
    assert_eq!(b.chat.recover().unwrap(), 1);
    common::eventually(|| agent_messages(&a_db, ca.id).len() == 2).await;
    let replies = agent_messages(&a_db, ca.id);
    // 실행 도중 끝난 것은 다시 돌리지 않고 오류 답, 대기는 실행.
    assert_eq!(replies[0].state, State::Error);
    assert!(replies[0].text.contains("다시 보내"), "{}", replies[0].text);
    assert_eq!(
        (replies[1].text.as_str(), replies[1].state),
        ("답: 둘", State::Done)
    );
    let runs = b.runs.lock().unwrap();
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].text, "둘");
}

#[tokio::test(flavor = "multi_thread")]
async fn storage_failure_sends_error_reply() {
    let (a, b) = (
        common::side("sf-a", Duration::ZERO).await,
        common::side("sf-b", Duration::ZERO).await,
    );
    let (b_id, _) = common::pair(&a, &b).await;
    let b_db = common::db(&b);
    agents::add(&b_db, "review", ".", None, "").unwrap();
    // B 의 답 저장이 실패한다(디스크 가득 참 흉내).
    b_db.execute_batch(
        "CREATE TRIGGER fail_reply BEFORE INSERT ON messages WHEN NEW.author = 'agent'
         BEGIN SELECT RAISE(ABORT, 'disk full'); END;",
    )
    .unwrap();
    let (conv, _) = a.chat.send(b_id, "review", None, "안녕").await.unwrap();
    let a_db = common::db(&a);
    common::eventually(|| agent_messages(&a_db, conv.id).len() == 1).await;
    let reply = agent_messages(&a_db, conv.id).remove(0);
    // 성공 답이 아니라 오류 답 — B 는 세션 · 상태를 함께 되돌린다.
    assert_eq!(reply.state, State::Error);
    assert!(reply.text.contains("기록 오류"), "{}", reply.text);
    let b_conv = chat::list(&b_db).unwrap().remove(0);
    assert_eq!(b_conv.session, None);
    assert_eq!(
        chat::messages(&b_db, b_conv.id).unwrap()[0].state,
        State::Error
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn unconfirmed_delivery_keeps_waiting_for_reply() {
    let (a, b) = (
        common::side("uc-a", Duration::ZERO).await,
        common::side("uc-b", Duration::ZERO).await,
    );
    let (b_id, _) = common::pair(&a, &b).await;
    agents::add(&common::db(&b), "review", ".", None, "").unwrap();
    let a_db = common::db(&a);
    // 대화를 하나 연 뒤, 상대 주소를 닫힌 포트로 — 응답(ACK)을 못 받는 전송 오류.
    let (conv, _) = a.chat.send(b_id, "review", None, "첫").await.unwrap();
    common::eventually(|| agent_messages(&a_db, conv.id).len() == 1).await;
    let closed = std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap();
    a_db.execute(
        "UPDATE peers SET addr = ?1 WHERE id = ?2",
        (closed.to_string(), b_id),
    )
    .unwrap();
    let err = a
        .chat
        .send(b_id, "review", Some(conv.id), "둘")
        .await
        .unwrap_err();
    assert!(err.to_string().contains("전달 확인 안 됨"), "{err}");
    // 실패로 확정하지 않는다 — 상대가 받아 실행했다면 늦게 온 답을 받는다.
    let pending = chat::messages(&a_db, conv.id).unwrap().pop().unwrap();
    assert_eq!((pending.text.as_str(), pending.state), ("둘", State::Sent));
    let req = Request::Reply {
        conv: conv.uuid.clone(),
        msg: "late".into(),
        reply_to: pending.uuid.clone(),
        text: "늦은 답".into(),
        ok: true,
        turns: None,
        duration_ms: None,
        lc: None,
        model: None,
        effort: None,
    };
    let (resp, _) = b.node.send(a.addr, None, &req).await.unwrap();
    assert_eq!(resp, Response::Ok);
    assert_eq!(
        chat::messages(&a_db, conv.id).unwrap()[2].state,
        State::Done
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn concurrent_sends_keep_order() {
    let (a, b) = (
        common::side("co-a", Duration::ZERO).await,
        common::side("co-b", Duration::ZERO).await,
    );
    let (b_id, _) = common::pair(&a, &b).await;
    agents::add(&common::db(&b), "review", ".", None, "").unwrap();
    let (conv, _) = a.chat.send(b_id, "review", None, "시작").await.unwrap();
    // 다음 연결 하나를 0.3초 늦추는 중계를 사이에 둔다 — 먼저 보낸 것이 늦게 도착하는 네트워크.
    let relay = common::delaying_relay(b.addr, Duration::from_millis(300)).await;
    common::db(&a)
        .execute(
            "UPDATE peers SET addr = ?1 WHERE id = ?2",
            (relay.to_string(), b_id),
        )
        .unwrap();
    let first = {
        let chat = a.chat.clone();
        tokio::spawn(async move { chat.send(b_id, "review", Some(conv.id), "1").await })
    };
    tokio::time::sleep(Duration::from_millis(50)).await;
    let second = {
        let chat = a.chat.clone();
        tokio::spawn(async move { chat.send(b_id, "review", Some(conv.id), "2").await })
    };
    first.await.unwrap().unwrap();
    second.await.unwrap().unwrap();
    let human = |conn: &an_core::db::Connection, id: i64| -> Vec<String> {
        chat::messages(conn, id)
            .unwrap()
            .into_iter()
            .filter(|m| m.author == Author::Human)
            .map(|m| m.text)
            .collect()
    };
    let mine = human(&common::db(&a), conv.id);
    let b_db = common::db(&b);
    let theirs = human(&b_db, chat::list(&b_db).unwrap()[0].id);
    assert_eq!(mine, theirs);
}

#[tokio::test(flavor = "multi_thread")]
async fn reply_carries_run_meta() {
    let (a, b) = (
        common::side("rm-a", Duration::ZERO).await,
        common::side("rm-b", Duration::ZERO).await,
    );
    let (b_id, _) = common::pair(&a, &b).await;
    agents::add(&common::db(&b), "review", ".", None, "").unwrap();
    let (conv, sent) = a.chat.send(b_id, "review", None, "안녕").await.unwrap();
    let a_db = common::db(&a);
    common::eventually(|| agent_messages(&a_db, conv.id).len() == 1).await;
    // 양쪽 모두 답에 원 요청 · 실행 정보(실행기 대역 2턴 · 1.5초)가 남는다.
    let mine = agent_messages(&a_db, conv.id).remove(0);
    assert_eq!(
        (mine.reply_to, mine.turns, mine.duration_ms),
        (Some(sent.id), Some(2), Some(1500))
    );
    let b_db = common::db(&b);
    let theirs = chat::messages(&b_db, chat::list(&b_db).unwrap()[0].id).unwrap();
    assert_eq!(
        (theirs[1].reply_to, theirs[1].turns, theirs[1].duration_ms),
        (Some(theirs[0].id), Some(2), Some(1500))
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn reply_without_meta_accepted() {
    let (a, b) = (
        common::side("om-a", Duration::ZERO).await,
        common::side("om-b", Duration::from_millis(400)).await,
    );
    let (b_id, _) = common::pair(&a, &b).await;
    agents::add(&common::db(&b), "review", ".", None, "").unwrap();
    let (conv, sent) = a.chat.send(b_id, "review", None, "안녕").await.unwrap();
    // 이전 버전 상대의 답 — 실행 정보 필드가 없는 프레임.
    let old = serde_json::json!({
        "type": "reply", "conv": conv.uuid, "msg": "old-1", "reply_to": sent.uuid, "text": "옛 답", "ok": true
    });
    let req: Request = serde_json::from_value(old).unwrap();
    let (resp, _) = b.node.send(a.addr, None, &req).await.unwrap();
    assert_eq!(resp, Response::Ok);
    let a_db = common::db(&a);
    let got = agent_messages(&a_db, conv.id).remove(0);
    assert_eq!(
        (got.text.as_str(), got.turns, got.duration_ms, got.reply_to),
        ("옛 답", None, None, Some(sent.id))
    );
}

/// 실행한 model · effort 가 답에 남는다 — 실행한 쪽 · 보낸 쪽 둘 다(받은 요청의 답 `Reply` · 받은 쪽 사람 요청의 답 미러 `Post`).
#[tokio::test(flavor = "multi_thread")]
async fn reply_records_model_effort_both_sides() {
    let (a, b) = (
        common::side("mx-a", Duration::ZERO).await,
        common::side("mx-b", Duration::ZERO).await,
    );
    let (b_id, _) = common::pair(&a, &b).await;
    let b_db = common::db(&b);
    agents::add(&b_db, "dev", ".", None, "").unwrap();
    settings::set(&b_db, run_opts::MODEL_KEY, "opus").unwrap();
    settings::set(&b_db, run_opts::EFFORT_KEY, "low").unwrap();
    let (conv, _) = a.chat.send(b_id, "dev", None, "질문").await.unwrap();
    let a_db = common::db(&a);
    common::eventually(|| agent_messages(&a_db, conv.id).len() == 1).await;
    let meta = |m: &chat::Message| (m.model.clone(), m.effort.clone());
    let want = (Some("opus".to_string()), Some("low".to_string()));
    assert_eq!(
        meta(&agent_messages(&a_db, conv.id)[0]),
        want,
        "보낸 쪽(Reply)"
    );
    let bc = chat::list(&b_db).unwrap().remove(0);
    assert_eq!(meta(&agent_messages(&b_db, bc.id)[0]), want, "실행한 쪽");
    // 받은 쪽 사람이 쓴 실행 요청의 답 — 미러(Post)로 보낸 쪽에도.
    b.chat.post(bc.id, "B 쪽 질문", true).await.unwrap();
    common::eventually(|| agent_messages(&a_db, conv.id).len() == 2).await;
    assert_eq!(
        meta(&agent_messages(&a_db, conv.id)[1]),
        want,
        "보낸 쪽(Post)"
    );
    // 받은 model · effort 는 글자 규칙을 지난 것만 남긴다 — 틀리면 버린다(신뢰하지 않는 입력).
    chat::add_human(
        &a_db,
        conv.id,
        "raw-ask",
        "원시 질문",
        State::Sent,
        true,
        true,
    )
    .unwrap();
    let req = Request::Reply {
        conv: conv.uuid.clone(),
        msg: "raw-reply".into(),
        reply_to: "raw-ask".into(),
        text: "원시 답".into(),
        ok: true,
        turns: None,
        duration_ms: None,
        lc: None,
        model: Some("bad model!".into()),
        effort: Some("turbo".into()),
    };
    let (resp, _) = b.node.send(a.addr, None, &req).await.unwrap();
    assert_eq!(resp, Response::Ok);
    let raw = chat::message_by_uuid(&a_db, "raw-reply").unwrap().unwrap();
    assert_eq!(meta(&raw), (None, None), "틀린 값은 버린다");
}

/// 집중(받는 쪽 설정 `focus`) — 새 실행 요청은 `focus` 로 거부: 받은 쪽은 실행 · 기록 · 알림 없음, 보낸 쪽 글은 실패 + 거절 코드 + 그 상대를
/// 접속 · 집중으로. 집중을 끄면 다시 받고, 집중 중에도 내 요청의 답은 받는다.
#[tokio::test(flavor = "multi_thread")]
async fn focus_refuses_new_request_unrecorded() {
    let (a, b) = (
        common::side("fr-a", Duration::ZERO).await,
        common::side("fr-b", Duration::from_millis(400)).await,
    );
    let (b_id, _) = common::pair(&a, &b).await;
    agents::add(&common::db(&b), "review", ".", None, "").unwrap();
    settings::set(&common::db(&b), settings::FOCUS_KEY, "1").unwrap();
    let err = a
        .chat
        .send(b_id, "review", None, "집중 중에 묻는다")
        .await
        .unwrap_err();
    assert!(err.to_string().contains("상대가 집중 중"), "{err}");
    let conv = chat::list(&common::db(&a)).unwrap().remove(0);
    let m = chat::messages(&common::db(&a), conv.id).unwrap().remove(0);
    assert_eq!(
        (m.state, m.refused.as_deref()),
        (State::Error, Some("focus"))
    );
    let p = an_core::peers::by_id(&common::db(&a), b_id)
        .unwrap()
        .unwrap();
    assert!(p.focus && p.seen_at.is_some(), "거절도 접속 증거 {p:?}");
    assert!(
        chat::list(&common::db(&b)).unwrap().is_empty(),
        "받은 쪽 기록 없음"
    );
    assert!(b.runs.lock().unwrap().is_empty());
    assert!(
        !b.events
            .lock()
            .unwrap()
            .iter()
            .any(|e| matches!(e, PeerEvent::Message { .. })),
        "받은 쪽 알림 없음"
    );
    // 집중을 끄면 다시 받는다 — 같은 대화 이어 쓰기. 그 답이 오기 전에 A 가 집중을 켜도 답은 받는다.
    settings::set(&common::db(&b), settings::FOCUS_KEY, "0").unwrap();
    a.chat
        .send(b_id, "review", Some(conv.id), "다시 묻는다")
        .await
        .unwrap();
    settings::set(&common::db(&a), settings::FOCUS_KEY, "1").unwrap();
    common::eventually(|| agent_messages(&common::db(&a), conv.id).len() == 1).await;
    assert_eq!(b.runs.lock().unwrap().len(), 1);
    // 이미 받은 글의 재전송(같은 uuid)은 새 요청이 아니다 — B 가 집중이어도 Ok, 다시 실행하지 않는다.
    settings::set(&common::db(&b), settings::FOCUS_KEY, "1").unwrap();
    let again = chat::messages(&common::db(&a), conv.id)
        .unwrap()
        .into_iter()
        .find(|m| m.text == "다시 묻는다")
        .unwrap();
    let peer = a.node.peers().unwrap().remove(0);
    let resend = Request::Message {
        conv: conv.uuid.clone(),
        msg: again.uuid.clone(),
        agent: "review".into(),
        text: again.text.clone(),
        lc: None,
        by_agent: None,
        relay: None,
        via: None,
        request: false,
        mention: None,
    };
    let (resp, _) = a
        .node
        .send(b.addr, Some(peer.spki.clone()), &resend)
        .await
        .unwrap();
    assert_eq!(resp, Response::Ok);
    assert_eq!(b.runs.lock().unwrap().len(), 1, "다시 실행하지 않는다");
}

/// 질의 · 요청(메시지 단위, 기본 질의)이 양쪽 이력에 남고, 완료 표시는 받은 쪽만 바꿔 쓴 쪽에 상태로 간다(취소도). 쓴 쪽 · 질의 · 답에는 표시하지
/// 못한다. 받은 대화에서 B 가 쓴 요청(미러 `post`)도 같다 — 그 글의 받은 쪽은 A. 알리지 못하면 경고 — 이 PC 표시는 남는다.
#[tokio::test(flavor = "multi_thread")]
async fn ask_kind_and_done_sync_both_sides() {
    let (a, b) = (
        common::side("ak-a", Duration::ZERO).await,
        common::side("ak-b", Duration::ZERO).await,
    );
    let (b_id, a_id) = common::pair(&a, &b).await;
    agents::add(&common::db(&b), "review", ".", None, "").unwrap();
    let ask = |request| Compose {
        request,
        ..Default::default()
    };
    let (conv, q) = a
        .chat
        .send_with(b_id, "review", None, "질의 하나", ask(false))
        .await
        .unwrap();
    let (_, r) = a
        .chat
        .send_with(b_id, "review", Some(conv.id), "요청 하나", ask(true))
        .await
        .unwrap();
    assert!(!q.request && r.request);
    common::eventually(|| agent_messages(&common::db(&a), conv.id).len() == 2).await;
    let a_msg = |uuid: &str| {
        chat::message_by_uuid(&common::db(&a), uuid)
            .unwrap()
            .unwrap()
    };
    let b_msg = |uuid: &str| {
        chat::message_by_uuid(&common::db(&b), uuid)
            .unwrap()
            .unwrap()
    };
    assert_eq!(
        (b_msg(&q.uuid).request, b_msg(&r.uuid).request),
        (false, true),
        "받은 쪽에도 종류"
    );
    // 받은 쪽(B)만 완료 표시 — 쓴 쪽(A)에 상태로 간다.
    let (m, warning) = b
        .chat
        .mark_done(b_msg(&r.uuid).id, true, None)
        .await
        .unwrap();
    assert!(m.done_at.is_some() && warning.is_none(), "{warning:?}");
    assert!(a_msg(&r.uuid).done_at.is_some(), "쓴 쪽은 받은 대로");
    // 쓴 쪽 · 질의 · 답은 못 바꾼다.
    let invalid = |e: std::io::Error| e.kind() == std::io::ErrorKind::InvalidInput;
    assert!(invalid(
        a.chat
            .mark_done(a_msg(&r.uuid).id, false, None)
            .await
            .unwrap_err()
    ));
    assert!(invalid(
        b.chat
            .mark_done(b_msg(&q.uuid).id, true, None)
            .await
            .unwrap_err()
    ));
    let bc = chat::list(&common::db(&b)).unwrap().remove(0);
    let reply = agent_messages(&common::db(&b), bc.id).remove(0);
    assert!(invalid(
        b.chat.mark_done(reply.id, true, None).await.unwrap_err()
    ));
    // 취소도 같은 길.
    b.chat
        .mark_done(b_msg(&r.uuid).id, false, None)
        .await
        .unwrap();
    assert_eq!(a_msg(&r.uuid).done_at, None);
    // 페어링된 상대의 조작 프레임 — 내 질의 · 답 · 다른 대화의 요청이면 `unknown`, 쓴 쪽 기록은 그대로.
    let (other, oreq) = a
        .chat
        .send_with(b_id, "review", None, "다른 대화 요청", ask(true))
        .await
        .unwrap();
    common::eventually(|| agent_messages(&common::db(&a), other.id).len() == 1).await;
    let pa = b.node.peers().unwrap().remove(0);
    let answer = agent_messages(&common::db(&a), conv.id).remove(0).uuid;
    for msg in [&q.uuid, &answer, &oreq.uuid] {
        let forged = Request::Done {
            conv: conv.uuid.clone(),
            msg: msg.clone(),
            done: true,
        };
        let (resp, _) = b
            .node
            .send(a.addr, Some(pa.spki.clone()), &forged)
            .await
            .unwrap();
        assert_eq!(resp, Response::error("unknown"), "{msg}");
        assert_eq!(a_msg(msg).done_at, None, "{msg}");
    }
    // 받은 대화에서 B 가 쓴 요청(미러) — 받은 쪽 = A.
    let (_, bm, _) = b
        .chat
        .post_with(bc.id, "B 의 요청", true, ask(true))
        .await
        .unwrap();
    common::eventually(|| {
        chat::message_by_uuid(&common::db(&a), &bm.uuid)
            .unwrap()
            .is_some()
    })
    .await;
    let am = a_msg(&bm.uuid);
    assert!(am.request && !am.mine, "{am:?}");
    a.chat.mark_done(am.id, true, None).await.unwrap();
    assert!(b_msg(&bm.uuid).done_at.is_some());
    // 쓴 쪽에 닿지 못하면 경고 — 이 PC 표시는 남고 쓴 쪽은 그대로.
    common::db(&b)
        .execute(
            "UPDATE peers SET addr = '127.0.0.1:1' WHERE id = ?1",
            [a_id],
        )
        .unwrap();
    let (m, warning) = b
        .chat
        .mark_done(b_msg(&r.uuid).id, true, None)
        .await
        .unwrap();
    assert!(m.done_at.is_some(), "{m:?}");
    assert!(
        warning
            .as_deref()
            .is_some_and(|w| w.contains("상대에 알리지 못했다")),
        "{warning:?}"
    );
    assert_eq!(a_msg(&r.uuid).done_at, None);
}

/// 멘션(보낸 대화 · 자동 왕복 아님) — 받은 쪽은 부른 에이전트를 그 에이전트 세션으로 실행하고(대화 에이전트 세션은 그대로), 양쪽 요청 · 답에
/// 부른 이름이 남는다. 같은 에이전트를 다시 부르면 그 세션을 잇는다. 대화 에이전트 이름은 멘션이 아니다. 없는 에이전트는 상대가 거절하고,
/// 받은 대화에서는 멘션하지 않는다.
#[tokio::test(flavor = "multi_thread")]
async fn mention_runs_named_agent_own_session() {
    let (a, b) = (
        common::side("mn-a", Duration::ZERO).await,
        common::side("mn-b", Duration::ZERO).await,
    );
    let (b_id, _) = common::pair(&a, &b).await;
    agents::add(&common::db(&b), "review", ".", None, "").unwrap();
    agents::add(&common::db(&b), "guide", ".", None, "").unwrap();
    let mention = |name| Compose {
        mention: Some(name),
        ..Default::default()
    };
    let (conv, _) = a.chat.send(b_id, "review", None, "처음").await.unwrap();
    common::eventually(|| agent_messages(&common::db(&a), conv.id).len() == 1).await;
    let (_, asked) = a
        .chat
        .send_with(
            b_id,
            "review",
            Some(conv.id),
            "@guide 봐 주세요",
            mention("guide"),
        )
        .await
        .unwrap();
    assert_eq!(asked.agent.as_deref(), Some("guide"));
    common::eventually(|| agent_messages(&common::db(&a), conv.id).len() == 2).await;
    a.chat
        .send_with(
            b_id,
            "review",
            Some(conv.id),
            "@guide 하나 더",
            mention("guide"),
        )
        .await
        .unwrap();
    common::eventually(|| agent_messages(&common::db(&a), conv.id).len() == 3).await;
    let (_, plain) = a
        .chat
        .send_with(
            b_id,
            "review",
            Some(conv.id),
            "@review 다시",
            mention("review"),
        )
        .await
        .unwrap();
    assert_eq!(plain.agent, None, "대화 에이전트는 멘션이 아니다");
    common::eventually(|| agent_messages(&common::db(&a), conv.id).len() == 4).await;
    let runs = b.runs.lock().unwrap().clone();
    let names: Vec<_> = runs.iter().map(|r| r.agent.name.as_str()).collect();
    assert_eq!(names, ["review", "guide", "guide", "review"]);
    let id = |s: &Session| match s {
        Session::New(id) | Session::Resume(id) => id.clone(),
    };
    assert!(
        matches!(runs[1].session, Session::New(_)) && id(&runs[1].session) != id(&runs[0].session),
        "멘션한 에이전트는 자기 세션"
    );
    assert!(
        matches!(runs[2].session, Session::Resume(_))
            && id(&runs[2].session) == id(&runs[1].session),
        "같은 에이전트를 다시 부르면 그 세션"
    );
    assert_eq!(
        id(&runs[3].session),
        id(&runs[0].session),
        "대화 에이전트 세션은 그대로"
    );
    for side in [&a, &b] {
        let db = common::db(side);
        let c = chat::list(&db).unwrap().remove(0);
        let got: Vec<_> = chat::messages(&db, c.id)
            .unwrap()
            .into_iter()
            .map(|m| m.agent)
            .collect();
        let g = Some("guide".to_string());
        assert_eq!(
            got,
            [None, None, g.clone(), g.clone(), g.clone(), g, None, None],
            "요청 · 답 모두"
        );
    }
    let bc = chat::list(&common::db(&b)).unwrap().remove(0);
    assert_eq!(
        chat::agent_session(&common::db(&b), bc.id, "guide").unwrap(),
        Some(id(&runs[1].session))
    );
    assert_eq!(bc.session, Some(id(&runs[0].session)));
    // 없는 에이전트 — 상대가 거절.
    let e = a
        .chat
        .send_with(b_id, "review", Some(conv.id), "@nobody", mention("nobody"))
        .await
        .unwrap_err();
    assert_eq!(e.kind(), std::io::ErrorKind::PermissionDenied, "{e}");
    // 받은 대화에서는 멘션하지 않는다.
    let e = b
        .chat
        .post_with(bc.id, "@guide", true, mention("guide"))
        .await
        .unwrap_err();
    assert_eq!(e.kind(), std::io::ErrorKind::InvalidInput, "{e}");
    // 첫 글이 받은 쪽에 닿지 않은 대화(집중 거절)의 멘션 — 받은 쪽은 대화를 대화 에이전트로 열고 그 글만 멘션으로 남긴다.
    settings::set(&common::db(&b), settings::FOCUS_KEY, "1").unwrap();
    a.chat
        .send(b_id, "review", None, "닿지 않는 첫 글")
        .await
        .unwrap_err();
    settings::set(&common::db(&b), settings::FOCUS_KEY, "0").unwrap();
    let late = chat::list(&common::db(&a))
        .unwrap()
        .into_iter()
        .find(|c| c.id != conv.id)
        .unwrap();
    let (_, asked) = a
        .chat
        .send_with(
            b_id,
            "review",
            Some(late.id),
            "@guide 처음 닿는 글",
            mention("guide"),
        )
        .await
        .unwrap();
    common::eventually(|| agent_messages(&common::db(&a), late.id).len() == 1).await;
    let db = common::db(&b);
    let bl = chat::find(&db, &late.uuid, chat::list(&db).unwrap()[0].peer_id)
        .unwrap()
        .unwrap();
    assert_eq!(bl.agent, "review", "대화 에이전트로 연다");
    let got = chat::message_by_uuid(&db, &asked.uuid).unwrap().unwrap();
    assert_eq!(got.agent.as_deref(), Some("guide"));
}
