mod common;
use an_agent::runner::Session;
use an_core::agents;
use an_core::chat::{self, Author, Dir, State};
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
            .any(|e| matches!(e, PeerEvent::Message { .. }))
    })
    .await;
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
