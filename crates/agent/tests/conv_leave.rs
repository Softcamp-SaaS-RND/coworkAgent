//! 대화 나가기 · 삭제(#1w — D40) — 두 PC(A · B, 공유는 C 까지): 나간 쪽 · 상대 쪽 읽기 전용, 나간 대화로 온 요청 거절(`left`), 이전 버전 상대,
//! 실행 중 막음, 공유 끝, 삭제 정리 · 되살아나지 않음 · 번호 재사용 없음.
mod common;
use an_core::chat::{self, Author, Conversation, Dir, State};
use an_core::{agents, groups, share};
use an_net::identity::Identity;
use an_net::proto::{Request, Response};
use std::time::Duration;

struct Two {
    a: common::Side,
    b: common::Side,
    /// A 쪽 B 의 id · B 쪽 A 의 id.
    b_id: i64,
    a_at_b: i64,
}

/// A · B 페어링, B 에 에이전트 `review`(B 실행 지연 `delay`).
async fn two(tag: &str, delay: Duration) -> Two {
    let a = common::side(&format!("{tag}-a"), Duration::ZERO).await;
    let b = common::side(&format!("{tag}-b"), delay).await;
    let (b_id, a_at_b) = common::pair(&a, &b).await;
    agents::add(&common::db(&b), "review", ".", "").unwrap();
    Two { a, b, b_id, a_at_b }
}

fn conv(s: &common::Side, id: i64) -> Option<Conversation> {
    chat::by_id(&common::db(s), id).unwrap()
}

/// 그 uuid 의 대화(공유 사본 제외).
fn by_uuid(s: &common::Side, uuid: &str) -> Option<Conversation> {
    chat::list(&common::db(s))
        .unwrap()
        .into_iter()
        .find(|c| c.uuid == uuid && c.dir != Dir::Shared)
}

fn copy_of(s: &common::Side, uuid: &str) -> Option<Conversation> {
    chat::list(&common::db(s))
        .unwrap()
        .into_iter()
        .find(|c| c.uuid == uuid && c.dir == Dir::Shared)
}

fn answered(s: &common::Side, conv: i64) -> bool {
    chat::messages(&common::db(s), conv)
        .unwrap()
        .iter()
        .any(|m| m.author == Author::Agent && m.state == State::Done)
}

/// A 가 B 의 review 에 묻고 답을 받은 대화.
async fn asked(t: &Two, q: &str) -> Conversation {
    let (c, _) = t.a.chat.send(t.b_id, "review", None, q).await.unwrap();
    common::eventually(|| answered(&t.a, c.id)).await;
    c
}

fn count(s: &common::Side, sql: &str, id: i64) -> i64 {
    common::db(s).query_row(sql, [id], |r| r.get(0)).unwrap()
}

/// 상대 주소를 바꾼다 — 돌려주는 값: 원래 주소.
fn readdr(s: &common::Side, peer: i64, to: &str) -> String {
    let conn = common::db(s);
    let was: String = conn
        .query_row("SELECT addr FROM peers WHERE id = ?1", [peer], |r| r.get(0))
        .unwrap();
    conn.execute("UPDATE peers SET addr = ?1 WHERE id = ?2", (to, peer))
        .unwrap();
    was
}

fn closed_port() -> String {
    std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .to_string()
}

#[tokio::test(flavor = "multi_thread")]
async fn leave_marks_both_read_only() {
    let t = two("lv-both", Duration::ZERO).await;
    let c = asked(&t, "질문").await;
    let before = chat::messages(&common::db(&t.a), c.id).unwrap().len();
    t.a.chat.leave_conversation(c.id).unwrap();
    let mine = conv(&t.a, c.id).unwrap();
    assert!(mine.left_at.is_some() && mine.peer_left_at.is_none());
    assert_eq!(
        chat::messages(&common::db(&t.a), c.id).unwrap().len(),
        before,
        "이력 그대로"
    );
    common::eventually(|| by_uuid(&t.b, &c.uuid).is_some_and(|x| x.peer_left_at.is_some())).await;
    let theirs = by_uuid(&t.b, &c.uuid).unwrap();
    assert_eq!(theirs.left_at, None);
    // 양쪽 읽기 전용.
    let e = t.a.chat.post(c.id, "더", true).await.unwrap_err();
    assert_eq!(e.kind(), std::io::ErrorKind::InvalidInput, "{e}");
    let e = t.b.chat.post(theirs.id, "더", false).await.unwrap_err();
    assert_eq!(e.kind(), std::io::ErrorKind::InvalidInput, "{e}");
    // 두 번 나가도 그대로.
    t.a.chat.leave_conversation(c.id).unwrap();
    assert_eq!(conv(&t.a, c.id).unwrap().left_at, mine.left_at);
}

#[tokio::test(flavor = "multi_thread")]
async fn leave_refuses_inbound() {
    // B 가 A 의 에이전트에 묻는 대화 — A 가 나가고 B 는 아직 모른다(나가기가 닿지 않음).
    let t = two("lv-refuse", Duration::ZERO).await;
    agents::add(&common::db(&t.a), "mine", ".", "").unwrap();
    let (c, _) = t.b.chat.send(t.a_at_b, "mine", None, "질문").await.unwrap();
    common::eventually(|| answered(&t.b, c.id)).await;
    let at_a = by_uuid(&t.a, &c.uuid).unwrap();
    let real = readdr(&t.a, t.b_id, &closed_port());
    t.a.chat.leave_conversation(at_a.id).unwrap();
    let runs = t.a.runs.lock().unwrap().len();
    let _ = t.b.chat.send(t.a_at_b, "mine", Some(c.id), "또 질문").await;
    common::eventually(|| {
        chat::messages(&common::db(&t.b), c.id)
            .unwrap()
            .iter()
            .any(|m| m.text == "또 질문" && m.state == State::Error && m.refused.as_deref() == Some("left"))
    })
    .await;
    assert_eq!(t.a.runs.lock().unwrap().len(), runs, "실행하지 않는다");
    assert!(
        !chat::messages(&common::db(&t.a), at_a.id)
            .unwrap()
            .iter()
            .any(|m| m.text == "또 질문"),
        "기록하지 않는다"
    );
    // 나가기가 닿으면 B 쪽도 나간 대화 — B 의 Ping 이 A 의 줄을 깨운다(돌아온 증거).
    readdr(&t.a, t.b_id, &real);
    let _ = t.b.node.ping(t.a_at_b).await;
    common::eventually(|| conv(&t.b, c.id).is_some_and(|x| x.peer_left_at.is_some())).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn leave_old_peer_gets_refusal() {
    let t = two("lv-old", Duration::ZERO).await;
    agents::add(&common::db(&t.a), "mine", ".", "").unwrap();
    let (c, _) = t.b.chat.send(t.a_at_b, "mine", None, "질문").await.unwrap();
    common::eventually(|| answered(&t.b, c.id)).await;
    let at_a = by_uuid(&t.a, &c.uuid).unwrap();
    // B 를 이전 버전(나가기를 모름)으로 — Ping 에 `leave` 없는 Pong.
    let old = common::old_peer(
        Identity::load_or_create(&t.b.paths.identity_file()).unwrap(),
        serde_json::json!({"type": "pong", "focus": false, "ctx": true, "share": true}),
    )
    .await;
    let real = readdr(&t.a, t.b_id, &old.to_string());
    t.a.chat.leave_conversation(at_a.id).unwrap();
    common::eventually(|| {
        count(&t.a, "SELECT count(*) FROM outbox WHERE peer_id = ?1 AND kind = 'leave'", t.b_id) == 0
    })
    .await;
    readdr(&t.a, t.b_id, &real);
    assert_eq!(conv(&t.b, c.id).unwrap().peer_left_at, None, "나가기를 보내지 않았다");
    // 그쪽은 보낼 때 거절을 받는다.
    let _ = t.b.chat.send(t.a_at_b, "mine", Some(c.id), "또").await;
    common::eventually(|| {
        chat::messages(&common::db(&t.b), c.id)
            .unwrap()
            .iter()
            .any(|m| m.text == "또" && m.refused.as_deref() == Some("left"))
    })
    .await;
}

#[tokio::test(flavor = "multi_thread")]
async fn leave_blocked_while_running() {
    let t = two("lv-run", Duration::from_millis(1500)).await;
    let (c, asked_msg) = t.a.chat.send(t.b_id, "review", None, "느린 질문").await.unwrap();
    chat::set_relay(&common::db(&t.a), c.id, "x", 5).unwrap();
    common::eventually(|| {
        by_uuid(&t.b, &c.uuid).is_some_and(|x| {
            chat::messages(&common::db(&t.b), x.id)
                .unwrap()
                .iter()
                .any(|m| m.state == State::Running)
        })
    })
    .await;
    let at_b = by_uuid(&t.b, &c.uuid).unwrap();
    // 도는 실행이 있으면 막는다(나가기 · 삭제).
    let e = t.b.chat.leave_conversation(at_b.id).unwrap_err();
    assert_eq!(e.kind(), std::io::ErrorKind::ResourceBusy, "{e}");
    let e = t.b.chat.delete_conversation(at_b.id).unwrap_err();
    assert_eq!(e.kind(), std::io::ErrorKind::ResourceBusy, "{e}");
    assert!(conv(&t.b, at_b.id).unwrap().left_at.is_none());
    // A 는 도는 실행이 없다 — 나간다: 자동 왕복 끔 · 답을 기다리던 내 글은 닫힌다.
    t.a.chat.leave_conversation(c.id).unwrap();
    let mine = conv(&t.a, c.id).unwrap();
    assert!(!mine.relay.unwrap().on, "자동 왕복 꺼짐");
    let m = chat::message_by_id(&common::db(&t.a), asked_msg.id).unwrap().unwrap();
    assert_eq!((m.state, m.refused.as_deref()), (State::Error, Some("left")));
    common::eventually(|| conv(&t.b, at_b.id).unwrap().peer_left_at.is_some()).await;
    // B 의 실행이 끝나도 답은 B 이력에만(상대에게 보내지 않는다).
    common::eventually(|| answered(&t.b, at_b.id)).await;
    common::eventually(|| {
        count(&t.b, "SELECT count(*) FROM outbox WHERE conv_id = ?1", at_b.id) == 0
    })
    .await;
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert!(!answered(&t.a, c.id), "나간 A 에 답이 가지 않았다");
    assert!(answered(&t.b, at_b.id), "보내지 않아 '답 미전달' 도 아니다");
    // 실행 대기 글만 남으면 나갈 수 있다 — 대기 글은 취소.
    let q = chat::add_message(
        &common::db(&t.b),
        at_b.id,
        &an_agent::chat::new_uuid(),
        Author::Human,
        "대기",
        State::Queued,
    )
    .unwrap();
    t.b.chat.leave_conversation(at_b.id).unwrap();
    let q = chat::message_by_id(&common::db(&t.b), q.id).unwrap().unwrap();
    assert_eq!((q.state, q.refused.as_deref()), (State::Error, Some("left")));
    assert!(conv(&t.b, at_b.id).unwrap().left_at.is_some());
}

struct Three {
    t: Two,
    c: common::Side,
    c_id: i64,
}

async fn three(tag: &str) -> Three {
    let t = two(tag, Duration::ZERO).await;
    let c = common::side(&format!("{tag}-c"), Duration::ZERO).await;
    let (c_id, _) = common::pair(&t.a, &c).await;
    Three { t, c, c_id }
}

/// A 가 B 에 묻고 C 에 공유한 대화 — C 사본이 이력을 받을 때까지.
async fn shared(s: &Three) -> Conversation {
    let conv = asked(&s.t, "공유할 질문").await;
    s.t.a.chat.share_add(conv.id, s.c_id).await.unwrap();
    common::eventually(|| {
        copy_of(&s.c, &conv.uuid)
            .is_some_and(|x| chat::messages(&common::db(&s.c), x.id).unwrap().len() == 2)
    })
    .await;
    conv
}

fn copy_ended(s: &common::Side, uuid: &str) -> bool {
    copy_of(s, uuid).is_some_and(|x| {
        share::copy(&common::db(s), x.id)
            .unwrap()
            .is_some_and(|c| c.ended)
    })
}

#[tokio::test(flavor = "multi_thread")]
async fn leave_ends_shares() {
    let s = three("lv-share").await;
    let conv = shared(&s).await;
    s.t.a.chat.leave_conversation(conv.id).unwrap();
    assert!(share::watchers(&common::db(&s.t.a), conv.id).unwrap().is_empty());
    common::eventually(|| copy_ended(&s.c, &conv.uuid)).await;
    assert_eq!(
        chat::messages(&common::db(&s.c), copy_of(&s.c, &conv.uuid).unwrap().id)
            .unwrap()
            .len(),
        2,
        "사본은 읽기 전용으로 남는다"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn leave_shared_copy() {
    let s = three("lv-copy").await;
    let conv = shared(&s).await;
    let copy = copy_of(&s.c, &conv.uuid).unwrap();
    s.c.chat.leave_conversation(copy.id).unwrap();
    assert!(copy_ended(&s.c, &conv.uuid), "사본 나가기 = 공유 나가기");
    common::eventually(|| share::watchers(&common::db(&s.t.a), conv.id).unwrap().is_empty()).await;
    assert!(conv_state_open(&s.t.a, conv.id), "원 대화는 그대로");
}

fn conv_state_open(s: &common::Side, id: i64) -> bool {
    conv(s, id).is_some_and(|c| !c.closed())
}

#[tokio::test(flavor = "multi_thread")]
async fn delete_cleans_local_keeps_peer() {
    let t = two("dl-clean", Duration::ZERO).await;
    let c = asked(&t, "지울 질문").await;
    let ids: Vec<i64> = chat::messages(&common::db(&t.a), c.id)
        .unwrap()
        .iter()
        .map(|m| m.id)
        .collect();
    {
        // 딸린 항목 — 그룹 · ★ · 저장 · 에이전트 세션 · 요약 · 기록 · CLI 트랜잭션 · 실패 보관.
        let db = common::db(&t.a);
        let g = groups::add(&db, "묶음").unwrap();
        groups::change(&db, g.id, &[c.id], &[]).unwrap();
        chat::set_starred(&db, c.id, true).unwrap();
        chat::set_saved(&db, ids[0], true).unwrap();
        chat::set_session(&db, c.id, "s1").unwrap();
        chat::set_agent_session(&db, c.id, "other", "s2").unwrap();
        chat::set_ctx_seed(&db, c.id, "", "요약").unwrap();
        chat::add_relay_event(&db, c.id, chat::RelayMark::Start, "시작").unwrap();
        db.execute(
            "INSERT INTO cli_tx (key, message_id, created_at) VALUES ('k1', ?1, 0)",
            [ids[0]],
        )
        .unwrap();
        db.execute(
            "INSERT INTO outbox (peer_id, conv_id, msg_id, kind, frame, state, created_at) VALUES (?1, ?2, ?3, 'note', '{}', 'failed', 0)",
            (t.b_id, c.id, ids[0]),
        )
        .unwrap();
    }
    t.a.chat.delete_conversation(c.id).unwrap();
    assert!(conv(&t.a, c.id).is_none());
    for sql in [
        "SELECT count(*) FROM messages WHERE conversation_id = ?1",
        "SELECT count(*) FROM agent_sessions WHERE conversation_id = ?1",
        "SELECT count(*) FROM ctx_seeds WHERE conversation_id = ?1",
        "SELECT count(*) FROM group_convs WHERE conversation_id = ?1",
        "SELECT count(*) FROM shares WHERE conversation_id = ?1",
        "SELECT count(*) FROM relay_events WHERE conversation_id = ?1",
        "SELECT count(*) FROM outbox WHERE conv_id = ?1",
    ] {
        assert_eq!(count(&t.a, sql, c.id), 0, "{sql}");
    }
    for &m in &ids {
        assert_eq!(count(&t.a, "SELECT count(*) FROM cli_tx WHERE message_id = ?1", m), 0);
        assert_eq!(count(&t.a, "SELECT count(*) FROM outbox WHERE msg_id = ?1", m), 0);
    }
    assert_eq!(
        count(&t.a, "SELECT count(*) FROM left_convs WHERE peer_id = ?1", t.b_id),
        1,
        "묘비"
    );
    // 상대 사본은 그대로 · 나감을 받는다.
    common::eventually(|| by_uuid(&t.b, &c.uuid).is_some_and(|x| x.peer_left_at.is_some())).await;
    let theirs = by_uuid(&t.b, &c.uuid).unwrap();
    assert_eq!(chat::messages(&common::db(&t.b), theirs.id).unwrap().len(), ids.len());
}

#[tokio::test(flavor = "multi_thread")]
async fn delete_not_revived() {
    let t = two("dl-revive", Duration::ZERO).await;
    // B 가 A 의 에이전트에 묻는 대화 — A 쪽은 받은 대화(없으면 새로 만드는 쪽).
    agents::add(&common::db(&t.a), "mine", ".", "").unwrap();
    let (c, _) = t.b.chat.send(t.a_at_b, "mine", None, "질문").await.unwrap();
    common::eventually(|| answered(&t.b, c.id)).await;
    let at_a = by_uuid(&t.a, &c.uuid).unwrap();
    t.a.chat.delete_conversation(at_a.id).unwrap();
    let pin = Some(t.a.node.identity().spki().to_vec());
    for request in [false, true] {
        let req = Request::Message {
            conv: c.uuid.clone(),
            msg: an_agent::chat::new_uuid(),
            agent: "mine".into(),
            text: "되살아나라".into(),
            lc: None,
            by_agent: None,
            relay: None,
            via: None,
            request,
            mention: None,
        };
        let (resp, _) = t.b.node.send(t.a.addr, pin.clone(), &req).await.unwrap();
        assert_eq!(resp, Response::error("left"), "요청 {request}");
    }
    assert!(by_uuid(&t.a, &c.uuid).is_none(), "대화가 생기지 않았다");
}

#[tokio::test(flavor = "multi_thread")]
async fn delete_id_not_reused() {
    let t = two("dl-id", Duration::ZERO).await;
    let c1 = asked(&t, "하나").await;
    let c2 = asked(&t, "둘").await;
    assert!(c2.id > c1.id);
    t.a.chat.delete_conversation(c2.id).unwrap();
    let c3 = asked(&t, "셋").await;
    assert!(c3.id > c2.id, "지운 번호 {} 를 다시 쓰지 않는다 — 새 {}", c2.id, c3.id);
}

#[tokio::test(flavor = "multi_thread")]
async fn delete_shared_copy() {
    let s = three("dl-copy").await;
    let conv = shared(&s).await;
    let copy = copy_of(&s.c, &conv.uuid).unwrap();
    s.c.chat.delete_conversation(copy.id).unwrap();
    assert!(copy_of(&s.c, &conv.uuid).is_none(), "사본 없음");
    assert_eq!(count(&s.c, "SELECT count(*) FROM messages WHERE conversation_id = ?1", copy.id), 0);
    // 공유 나가기는 대화를 가진 PC 에 닿는다(떼어 남긴 끝 행).
    common::eventually(|| share::watchers(&common::db(&s.t.a), conv.id).unwrap().is_empty()).await;
    // 사본은 묘비를 남기지 않는다 — 다시 초대하면 새 사본.
    s.t.a.chat.share_add(conv.id, s.c_id).await.unwrap();
    common::eventually(|| copy_of(&s.c, &conv.uuid).is_some_and(|x| !copy_ended(&s.c, &x.uuid))).await;
}

/// 적대적 평가 R1 — `Leave` 를 보낸 상대가 같은 대화로 글을 보내도 받지 않는다(기록 · 실행 없음 — 원격 입력은 신뢰하지 않는다).
#[tokio::test(flavor = "multi_thread")]
async fn leave_peer_left_refuses_raw() {
    let t = two("lv-raw", Duration::ZERO).await;
    let c = asked(&t, "질문").await;
    t.a.chat.leave_conversation(c.id).unwrap();
    common::eventually(|| by_uuid(&t.b, &c.uuid).is_some_and(|x| x.peer_left_at.is_some())).await;
    let at_b = by_uuid(&t.b, &c.uuid).unwrap();
    let (msgs, runs) = (
        chat::messages(&common::db(&t.b), at_b.id).unwrap().len(),
        t.b.runs.lock().unwrap().len(),
    );
    let pin = Some(t.b.node.identity().spki().to_vec());
    let req = Request::Message {
        conv: c.uuid.clone(),
        msg: an_agent::chat::new_uuid(),
        agent: "review".into(),
        text: "나간 뒤에 보냄".into(),
        lc: None,
        by_agent: None,
        relay: None,
        via: None,
        request: false,
        mention: None,
    };
    let (resp, _) = t.a.node.send(t.b.addr, pin, &req).await.unwrap();
    assert_eq!(resp, Response::error("left"));
    tokio::time::sleep(Duration::from_millis(200)).await;
    assert_eq!(chat::messages(&common::db(&t.b), at_b.id).unwrap().len(), msgs, "기록 없음");
    assert_eq!(t.b.runs.lock().unwrap().len(), runs, "실행 없음");
}

/// 적대적 평가 R1 — 확인 뒤 끼어든 나가기 · 삭제: 실행 시작은 대기 글 · 열린 대화일 때만(`start_run`), 묘비 대화는 다시 열리지 않는다(`open_conversation`).
#[tokio::test(flavor = "multi_thread")]
async fn delete_race_guards() {
    let t = two("dl-race", Duration::ZERO).await;
    agents::add(&common::db(&t.a), "mine", ".", "").unwrap();
    let (c, _) = t.b.chat.send(t.a_at_b, "mine", None, "질문").await.unwrap();
    common::eventually(|| answered(&t.b, c.id)).await;
    let at_a = by_uuid(&t.a, &c.uuid).unwrap();
    let db = common::db(&t.a);
    let queued = |text: &str| {
        chat::add_message(&db, at_a.id, &an_agent::chat::new_uuid(), Author::Human, text, State::Queued)
            .unwrap()
            .id
    };
    // 열린 대화의 대기 글은 시작된다 · 이미 시작했으면 다시 아님.
    let q1 = queued("하나");
    assert!(chat::start_run(&db, q1).unwrap());
    assert!(!chat::start_run(&db, q1).unwrap(), "대기 글만");
    db.execute("UPDATE messages SET state = 'done' WHERE id = ?1", [q1]).unwrap();
    // 줄을 서는 동안 나갔다 — 대기 글이 취소돼 시작되지 않는다(취소를 되돌려도 나간 대화라 시작 안 됨).
    let q2 = queued("둘");
    t.a.chat.leave_conversation(at_a.id).unwrap();
    // 이력을 바꾸는 받기 처리기는 잠금 안에서 닫힌 대화를 걸러 낸다(R2 — 편집 · 삭제 · 답 · 요청 상태 …).
    assert!(chat::find_own(&db, &c.uuid, t.b_id).unwrap().is_some());
    assert!(chat::find_open(&db, &c.uuid, t.b_id).unwrap().is_none());
    assert!(!chat::start_run(&db, q2).unwrap());
    db.execute("UPDATE messages SET state = 'queued' WHERE id = ?1", [q2]).unwrap();
    assert!(!chat::start_run(&db, q2).unwrap(), "나간 대화");
    // 지운 뒤 — 글도 대화도 없고, 같은 uuid 로 다시 열지 못한다(묘비).
    t.a.chat.delete_conversation(at_a.id).unwrap();
    assert!(!chat::start_run(&db, q2).unwrap());
    assert!(chat::open_conversation(&db, &c.uuid, t.b_id, "mine", Dir::In).is_err());
    assert!(by_uuid(&t.a, &c.uuid).is_none());
}

/// 적대적 평가 R2 — 실행 문에서 기다리던 대화를 지우면, 자리가 나도 실행 · 진행 알림 행이 생기지 않는다(지운 대화 번호를 가리키는 보낼 글 0).
#[tokio::test(flavor = "multi_thread")]
async fn delete_while_waiting_leaves_no_rows() {
    let t = two("dl-wait", Duration::from_millis(1500)).await;
    t.b.chat.set_max_runs(1).unwrap();
    let (c1, _) = t.a.chat.send(t.b_id, "review", None, "먼저").await.unwrap();
    common::eventually(|| {
        by_uuid(&t.b, &c1.uuid).is_some_and(|x| {
            chat::messages(&common::db(&t.b), x.id).unwrap().iter().any(|m| m.state == State::Running)
        })
    })
    .await;
    let (c2, _) = t.a.chat.send(t.b_id, "review", None, "기다림").await.unwrap();
    common::eventually(|| by_uuid(&t.b, &c2.uuid).is_some()).await;
    let at_b2 = by_uuid(&t.b, &c2.uuid).unwrap();
    let at_b1 = by_uuid(&t.b, &c1.uuid).unwrap();
    // B 의 보낼 줄을 A 에 닿지 않게 — 생긴 행이 보내져 사라지지 않고 남아 보이게.
    readdr(&t.b, t.a_at_b, &closed_port());
    let runs = t.b.runs.lock().unwrap().len();
    t.b.chat.delete_conversation(at_b2.id).unwrap();
    // 앞 실행이 끝나 자리가 난 뒤.
    common::eventually(|| answered(&t.b, at_b1.id)).await;
    tokio::time::sleep(Duration::from_millis(500)).await;
    assert_eq!(t.b.runs.lock().unwrap().len(), runs, "지운 대화의 글은 실행하지 않는다");
    assert_eq!(count(&t.b, "SELECT count(*) FROM outbox WHERE conv_id = ?1", at_b2.id), 0);
    assert!(by_uuid(&t.b, &c2.uuid).is_none());
}
