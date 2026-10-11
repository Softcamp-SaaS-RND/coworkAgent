mod common;
use an_core::agents;
use an_core::chat::{self, Author, Dir, State};
use an_net::node::{Me, Node};
use an_net::proto::{Request, Response};
use std::time::Duration;

/// (작성자, 내 것, 실행 요청, 본문) — 이력 비교용.
fn history(conn: &an_core::db::Connection, conv: i64) -> Vec<(Author, bool, bool, String)> {
    chat::messages(conn, conv)
        .unwrap()
        .into_iter()
        .map(|m| (m.author, m.mine, m.exec, m.text))
        .collect()
}

fn b_conv(b: &common::Side) -> (an_core::db::Connection, i64) {
    let db = common::db(b);
    let id = chat::list(&db).unwrap()[0].id;
    (db, id)
}

#[tokio::test(flavor = "multi_thread")]
async fn human_note_both_ways() {
    let (a, b) = (
        common::side("hn-a", Duration::ZERO).await,
        common::side("hn-b", Duration::ZERO).await,
    );
    let (b_id, _) = common::pair(&a, &b).await;
    agents::add(&common::db(&b), "review", ".", None, "").unwrap();
    let (conv, _) = a.chat.send(b_id, "review", None, "질문").await.unwrap();
    let a_db = common::db(&a);
    common::eventually(|| chat::messages(&a_db, conv.id).unwrap().len() == 2).await;
    // 보낸 쪽 사람 → 사람(실행 없음).
    let (_, note, _) = a.chat.post(conv.id, "A 메모", false).await.unwrap();
    assert_eq!((note.mine, note.exec), (true, false));
    // 받은 쪽 사람 → 사람.
    let (b_db, bc) = b_conv(&b);
    b.chat.post(bc, "B 메모", false).await.unwrap();
    let a_hist = history(&a_db, conv.id);
    assert_eq!(a_hist[2], (Author::Human, true, false, "A 메모".into()));
    assert_eq!(a_hist[3], (Author::Human, false, false, "B 메모".into()));
    let b_hist = history(&b_db, bc);
    assert_eq!(b_hist[2], (Author::Human, false, false, "A 메모".into()));
    assert_eq!(b_hist[3], (Author::Human, true, false, "B 메모".into()));
    // 사람 글은 실행되지 않는다.
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert_eq!(b.runs.lock().unwrap().len(), 1);
    assert_eq!(
        chat::messages(&a_db, conv.id).unwrap()[2].state,
        State::Done
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn receiver_asks_own_agent() {
    let (a, b) = (
        common::side("ro-a", Duration::ZERO).await,
        common::side("ro-b", Duration::ZERO).await,
    );
    let (b_id, _) = common::pair(&a, &b).await;
    agents::add(&common::db(&b), "review", ".", None, "").unwrap();
    let (conv, _) = a.chat.send(b_id, "review", None, "첫").await.unwrap();
    let a_db = common::db(&a);
    common::eventually(|| chat::messages(&a_db, conv.id).unwrap().len() == 2).await;
    // 받은 쪽 사람이 자기 에이전트에게.
    let (b_db, bc) = b_conv(&b);
    b.chat.post(bc, "B 가 묻는다", true).await.unwrap();
    common::eventually(|| chat::messages(&a_db, conv.id).unwrap().len() == 4).await;
    assert_eq!(b.runs.lock().unwrap().len(), 2, "B 에이전트가 실행");
    // 양쪽 같은 순서 · 같은 발화자.
    let a_hist = history(&a_db, conv.id);
    assert_eq!(
        a_hist[2],
        (Author::Human, false, true, "B 가 묻는다".into())
    );
    assert_eq!(a_hist[3].0, Author::Agent);
    assert_eq!(a_hist[3].3, "답: B 가 묻는다");
    common::eventually(|| chat::messages(&b_db, bc).unwrap().len() == 4).await;
    let b_hist = history(&b_db, bc);
    assert_eq!(b_hist[2], (Author::Human, true, true, "B 가 묻는다".into()));
    assert_eq!((b_hist[3].0, b_hist[3].1), (Author::Agent, true));
    // A 쪽 답은 그 사람 글의 답이고, 글은 끝 상태.
    let ms = chat::messages(&a_db, conv.id).unwrap();
    assert_eq!(ms[3].reply_to, Some(ms[2].id));
    assert_eq!((ms[2].state, ms[3].state), (State::Done, State::Done));
}

#[tokio::test(flavor = "multi_thread")]
async fn notes_prepended_to_run() {
    let (a, b) = (
        common::side("np-a", Duration::ZERO).await,
        common::side("np-b", Duration::ZERO).await,
    );
    let (b_id, _) = common::pair(&a, &b).await;
    agents::add(&common::db(&b), "review", ".", None, "").unwrap();
    let (conv, _) = a.chat.send(b_id, "review", None, "첫 요청").await.unwrap();
    let a_db = common::db(&a);
    common::eventually(|| chat::messages(&a_db, conv.id).unwrap().len() == 2).await;
    a.chat.post(conv.id, "A 의 메모", false).await.unwrap();
    let (_, bc) = b_conv(&b);
    b.chat.post(bc, "B 의 메모", false).await.unwrap();
    a.chat.post(conv.id, "둘째 요청", true).await.unwrap();
    common::eventually(|| b.runs.lock().unwrap().len() == 2).await;
    let runs = b.runs.lock().unwrap();
    // 첫 실행은 본문 그대로, 둘째는 그 사이 사람 글이 발화자와 함께 앞에.
    assert_eq!(runs[0].text, "첫 요청");
    let input = &runs[1].text;
    assert!(input.starts_with("[대화 메모 — 실행 요청 아님]"), "{input}");
    assert!(
        input.contains("- user-np-a · PC-np-a: A 의 메모"),
        "{input}"
    );
    assert!(input.contains("- 이 PC 사용자: B 의 메모"), "{input}");
    assert!(
        input.ends_with("[요청 — user-np-a · PC-np-a]\n둘째 요청"),
        "{input}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn post_refused_when_foreign() {
    let (a, b) = (
        common::side("pf-a", Duration::ZERO).await,
        common::side("pf-b", Duration::ZERO).await,
    );
    let (b_id, _) = common::pair(&a, &b).await;
    agents::add(&common::db(&b), "review", ".", None, "").unwrap();
    let (conv, sent) = a.chat.send(b_id, "review", None, "질문").await.unwrap();
    let a_db = common::db(&a);
    common::eventually(|| chat::messages(&a_db, conv.id).unwrap().len() == 2).await;
    let post =
        |conv: &str, msg: &str, author: &str, exec: bool, reply_to: Option<String>| Request::Post {
            conv: conv.into(),
            msg: msg.into(),
            author: author.into(),
            text: "x".into(),
            exec,
            reply_to,
            ok: Some(true),
            turns: None,
            duration_ms: None,
            lc: None,
        };
    // 없는 대화 · 이상한 작성자.
    let (r, _) = a
        .node
        .send(b.addr, None, &post("nope", "p1", "human", false, None))
        .await
        .unwrap();
    assert_eq!(r, Response::error("unknown"));
    let (r, _) = a
        .node
        .send(b.addr, None, &post(&conv.uuid, "p2", "robot", false, None))
        .await
        .unwrap();
    assert_eq!(r, Response::error("bad_request"));
    // B(에이전트 쪽)는 A 가 보낸 에이전트 답 · 실행 요청 글을 받지 않는다 — A 는 이 대화의 에이전트가 없다.
    let (r, _) = a
        .node
        .send(
            b.addr,
            None,
            &post(&conv.uuid, "p3", "agent", false, Some(sent.uuid.clone())),
        )
        .await
        .unwrap();
    assert_eq!(r, Response::error("bad_request"));
    let (r, _) = a
        .node
        .send(b.addr, None, &post(&conv.uuid, "p4", "human", true, None))
        .await
        .unwrap();
    assert_eq!(r, Response::error("bad_request"));
    // A 쪽: B 가 A 자신의 글에 에이전트 답을 덧붙이려 하면 거부(상대가 쓴 실행 요청에만).
    let (r, _) = b
        .node
        .send(
            a.addr,
            None,
            &post(&conv.uuid, "p5", "agent", false, Some(sent.uuid.clone())),
        )
        .await
        .unwrap();
    assert_eq!(r, Response::error("unknown"));
    // 같은 글 재전송은 한 번만.
    for _ in 0..2 {
        let (r, _) = a
            .node
            .send(b.addr, None, &post(&conv.uuid, "p6", "human", false, None))
            .await
            .unwrap();
        assert_eq!(r, Response::Ok);
    }
    let (b_db, bc) = b_conv(&b);
    assert_eq!(chat::messages(&b_db, bc).unwrap().len(), 3);
    // 페어링하지 않은 키는 not_paired.
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
    let (r, _) = stranger
        .send(b.addr, None, &post(&conv.uuid, "p7", "human", false, None))
        .await
        .unwrap();
    assert_eq!(r, Response::error("not_paired"));
    assert_eq!(chat::list(&b_db).unwrap()[0].dir, Dir::In);
}

#[tokio::test(flavor = "multi_thread")]
async fn concurrent_notes_same_order_both_sides() {
    let (a, b) = (
        common::side("cc-a", Duration::ZERO).await,
        common::side("cc-b", Duration::ZERO).await,
    );
    let (b_id, _) = common::pair(&a, &b).await;
    agents::add(&common::db(&b), "review", ".", None, "").unwrap();
    let (conv, _) = a.chat.send(b_id, "review", None, "시작").await.unwrap();
    let a_db = common::db(&a);
    common::eventually(|| chat::messages(&a_db, conv.id).unwrap().len() == 2).await;
    let (b_db, bc) = b_conv(&b);
    // 양쪽이 동시에 여러 번 — 각자 먼저 저장하고 상대에 미러한다.
    for round in 0..5 {
        let (ta, tb) = (format!("A{round}"), format!("B{round}"));
        let (ra, rb) = tokio::join!(
            a.chat.post(conv.id, &ta, false),
            b.chat.post(bc, &tb, false)
        );
        ra.unwrap();
        rb.unwrap();
    }
    let order = |conn: &an_core::db::Connection, id| -> Vec<String> {
        chat::messages(conn, id)
            .unwrap()
            .into_iter()
            .map(|m| m.uuid)
            .collect()
    };
    common::eventually(|| chat::messages(&a_db, conv.id).unwrap().len() == 12).await;
    common::eventually(|| chat::messages(&b_db, bc).unwrap().len() == 12).await;
    assert_eq!(
        order(&a_db, conv.id),
        order(&b_db, bc),
        "양쪽 이력 순서가 같다"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn own_run_not_failed_by_mirror() {
    let (a, b) = (
        common::side("mf-a", Duration::ZERO).await,
        common::side("mf-b", Duration::ZERO).await,
    );
    let (b_id, a_id) = common::pair(&a, &b).await;
    agents::add(&common::db(&b), "review", ".", None, "").unwrap();
    let (conv, _) = a.chat.send(b_id, "review", None, "시작").await.unwrap();
    let a_db = common::db(&a);
    common::eventually(|| chat::messages(&a_db, conv.id).unwrap().len() == 2).await;
    // A 가 닿지 않는다 — B 쪽에서 A 주소를 닫힌 포트로.
    let (b_db, bc) = b_conv(&b);
    let closed = std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap();
    b_db.execute(
        "UPDATE peers SET addr = ?1 WHERE id = ?2",
        (closed.to_string(), a_id),
    )
    .unwrap();
    // 내 에이전트 실행은 받아들였으니 성공 + 경고(실패로 돌려주면 다시 보내 두 번 실행된다).
    let (_, m, warning) = b.chat.post(bc, "B 질문", true).await.unwrap();
    assert!(warning.unwrap().contains("다시 보내지"), "경고");
    common::eventually(|| b.runs.lock().unwrap().len() == 2).await;
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert_eq!(b.runs.lock().unwrap().len(), 2, "한 번만 실행");
    assert!(chat::message_by_uuid(&b_db, &m.uuid).unwrap().unwrap().exec);
}

#[tokio::test(flavor = "multi_thread")]
async fn late_queued_request_not_skipped() {
    let (a, b) = (
        common::side("lq-a", Duration::ZERO).await,
        common::side("lq-b", Duration::ZERO).await,
    );
    let (b_id, _) = common::pair(&a, &b).await;
    agents::add(&common::db(&b), "review", ".", None, "").unwrap();
    let (conv, _) = a.chat.send(b_id, "review", None, "첫").await.unwrap();
    let a_db = common::db(&a);
    common::eventually(|| b.runs.lock().unwrap().len() == 1).await;
    common::eventually(|| chat::messages(&a_db, conv.id).unwrap().len() == 2).await;
    let (b_db, bc) = b_conv(&b);
    // B 사람의 실행 요청이 미러 중(`sent`, 낮은 id)일 때 A 의 요청(높은 id)이 먼저 실행된다.
    let low = chat::add_human(&b_db, bc, "low", "늦게 대기열에", State::Sent, true, true).unwrap();
    a.chat
        .send(b_id, "review", Some(conv.id), "둘")
        .await
        .unwrap();
    common::eventually(|| b.runs.lock().unwrap().len() == 2).await;
    // 이제 낮은 id 가 `queued` — 작업을 깨우면 실행돼야 한다.
    chat::set_state(&b_db, low.id, State::Queued).unwrap();
    b.chat.recover().unwrap();
    common::eventually(|| b.runs.lock().unwrap().len() == 3).await;
    assert_eq!(b.runs.lock().unwrap()[2].text, "늦게 대기열에");
}
