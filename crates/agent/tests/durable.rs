//! 보내는 쪽 보관(#1c · D7) — 두 PC 통합: 꺼진 상대 · 꺼진 묻는 쪽 · 재시작 · 한 번만 실행 · 상대 격리 · 포기 · 상태 질의.
mod common;
use an_agent::chat::{Chat, UNREACHABLE};
use an_core::chat::{self, Author, State};
use an_core::{agents, db, outbox};
use std::time::{Duration, Instant};

/// 상대 주소를 닫힌 포트로 — 꺼진 상대. 돌려주는 값: 원래 주소.
fn unplug(s: &common::Side, peer: i64) -> String {
    let closed = std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap();
    let conn = common::db(s);
    let was: String = conn
        .query_row("SELECT addr FROM peers WHERE id = ?1", [peer], |r| r.get(0))
        .unwrap();
    conn.execute(
        "UPDATE peers SET addr = ?1 WHERE id = ?2",
        (closed.to_string(), peer),
    )
    .unwrap();
    was
}

fn plug(s: &common::Side, peer: i64, addr: &str) {
    common::db(s)
        .execute("UPDATE peers SET addr = ?1 WHERE id = ?2", (addr, peer))
        .unwrap();
}

/// 대화 `conv` 의 성공 답 수(그 쪽 DB).
fn answers(s: &common::Side, conv: i64) -> usize {
    chat::messages(&common::db(s), conv)
        .unwrap()
        .iter()
        .filter(|m| m.author == Author::Agent && m.state == State::Done)
        .count()
}

/// 조건이 참이 될 때까지(최대 `secs` 초).
async fn within(secs: u64, mut f: impl FnMut() -> bool) {
    let until = Instant::now() + Duration::from_secs(secs);
    while !f() {
        assert!(Instant::now() < until, "{secs}초 안에 조건이 맞지 않았다");
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

async fn pair_review(tag: &str, delay: Duration) -> (common::Side, common::Side, i64, i64) {
    let a = common::side(&format!("{tag}-a"), Duration::ZERO).await;
    let b = common::side(&format!("{tag}-b"), delay).await;
    let (b_id, a_id) = common::pair(&a, &b).await;
    agents::add(&common::db(&b), "review", ".", "").unwrap();
    (a, b, b_id, a_id)
}

#[tokio::test(flavor = "multi_thread")]
async fn request_to_offline_peer_runs_when_it_returns() {
    let (a, b, b_id, _) = pair_review("off", Duration::ZERO).await;
    a.chat.set_retry(1, 1, 600);
    let was = unplug(&a, b_id);
    let (conv, sent) = a.chat.send(b_id, "review", None, "꺼진 동안").await.unwrap();
    assert_eq!(sent.state, State::Sent);
    assert!(a.chat.outbox_of(&sent).is_some(), "보관");
    tokio::time::sleep(Duration::from_millis(1500)).await;
    assert!(b.runs.lock().unwrap().is_empty());
    plug(&a, b_id, &was);
    within(10, || answers(&a, conv.id) == 1).await;
    assert!(a.chat.outbox_of(&sent).is_none(), "닿으면 보관에서 빠진다");
    assert_eq!(b.runs.lock().unwrap().len(), 1);
}

#[tokio::test(flavor = "multi_thread")]
async fn reply_reaches_asker_after_it_returns_and_status_asks_again() {
    let (a, b, b_id, a_id) = pair_review("rep", Duration::from_millis(500)).await;
    // 답하는 쪽은 스스로 다시 보내지 않게(1시간) — 묻는 쪽의 상태 질의가 깨운다.
    b.chat.set_retry(3600, 3600, 86400);
    let (conv, sent) = a.chat.send(b_id, "review", None, "느린 질문").await.unwrap();
    common::eventually(|| b.runs.lock().unwrap().len() == 1).await;
    let was = unplug(&b, a_id);
    // 답은 b 에 보관 — "답 미전달" 이 아니다.
    let b_conv = chat::list(&common::db(&b)).unwrap()[0].id;
    within(5, || {
        chat::messages(&common::db(&b), b_conv)
            .unwrap()
            .iter()
            .any(|m| m.author == Author::Agent && b.chat.outbox_of(m).is_some())
    })
    .await;
    let reply = chat::messages(&common::db(&b), b_conv).unwrap().pop().unwrap();
    assert_eq!(reply.state, State::Done);
    assert_eq!(answers(&a, conv.id), 0);
    plug(&b, a_id, &was);
    // 묻는 쪽이 상태를 물으면 끝났다고 답하고 남은 답을 지금 보낸다.
    let asked = chat::message_by_id(&common::db(&a), sent.id).unwrap().unwrap();
    let status = a.chat.ask_status(&asked).await.unwrap();
    assert_eq!(
        status,
        an_net::proto::Response::Status {
            state: "done".into(),
            ahead: 0
        }
    );
    within(5, || answers(&a, conv.id) == 1).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn restart_resumes_kept_frames() {
    let (a, b, b_id, _) = pair_review("rst", Duration::ZERO).await;
    // 지금 실행은 다시 보내지 않는다(1시간) — 다시 켠 실행(새 Chat · 같은 DB)이 이어 보낸다.
    a.chat.set_retry(3600, 3600, 86400);
    let was = unplug(&a, b_id);
    let (conv, _) = a.chat.send(b_id, "review", None, "재시작 전").await.unwrap();
    plug(&a, b_id, &was);
    let restarted = Chat::new(
        db::open(&a.paths.db_file()).unwrap(),
        a.node.clone(),
        common::fake_run(a.runs.clone(), Duration::ZERO),
    );
    restarted.set_retry(1, 1, 600);
    restarted.recover().unwrap();
    within(10, || answers(&a, conv.id) == 1).await;
    assert_eq!(b.runs.lock().unwrap().len(), 1);
}

#[tokio::test(flavor = "multi_thread")]
async fn same_frame_twice_runs_once() {
    let (a, b, b_id, _) = pair_review("once", Duration::ZERO).await;
    let (conv, sent) = a.chat.send(b_id, "review", None, "한 번만").await.unwrap();
    common::eventually(|| answers(&a, conv.id) == 1).await;
    // 같은 프레임을 다시 보관하고(재시도 · 재시작에서 생기는 중복) 다른 실행이 보낸다.
    let frame = an_net::proto::Request::Message {
        conv: conv.uuid.clone(),
        msg: sent.uuid.clone(),
        agent: "review".into(),
        text: "한 번만".into(),
        lc: Some(sent.lc),
        by_agent: None,
        relay: None,
        via: None,
        request: false,
        mention: None,
    };
    outbox::add(
        &common::db(&a),
        b_id,
        Some(conv.id),
        Some(sent.id),
        "message",
        &serde_json::to_string(&frame).unwrap(),
        None,
        "pending",
    )
    .unwrap();
    let again = Chat::new(
        db::open(&a.paths.db_file()).unwrap(),
        a.node.clone(),
        common::fake_run(a.runs.clone(), Duration::ZERO),
    );
    again.recover().unwrap();
    within(5, || outbox::head(&common::db(&a), b_id).unwrap().is_none()).await;
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert_eq!(b.runs.lock().unwrap().len(), 1, "같은 uuid 는 한 번만 실행");
    assert_eq!(answers(&a, conv.id), 1);
}

#[tokio::test(flavor = "multi_thread")]
async fn one_dead_peer_does_not_delay_another() {
    let a = common::side("iso-a", Duration::ZERO).await;
    let b = common::side("iso-b", Duration::ZERO).await;
    let c = common::side("iso-c", Duration::ZERO).await;
    let (b_id, _) = common::pair(&a, &b).await;
    let (c_id, _) = common::pair(&a, &c).await;
    agents::add(&common::db(&b), "review", ".", "").unwrap();
    agents::add(&common::db(&c), "review", ".", "").unwrap();
    a.chat.set_retry(1, 1, 600);
    unplug(&a, b_id);
    for i in 0..3 {
        a.chat.send(b_id, "review", None, &format!("b{i}")).await.unwrap();
    }
    let started = Instant::now();
    let (conv, _) = a.chat.send(c_id, "review", None, "c").await.unwrap();
    common::eventually(|| answers(&a, conv.id) == 1).await;
    assert!(started.elapsed() < Duration::from_secs(2), "{:?}", started.elapsed());
    assert!(outbox::head(&common::db(&a), b_id).unwrap().is_some(), "b 는 아직 보관");
}

#[tokio::test(flavor = "multi_thread")]
async fn gives_up_after_limit_and_marks_unreachable() {
    let (a, _b, b_id, _) = pair_review("give", Duration::ZERO).await;
    a.chat.set_retry(1, 1, 2);
    unplug(&a, b_id);
    let (conv, sent) = a.chat.send(b_id, "review", None, "닿지 않음").await.unwrap();
    within(10, || {
        chat::message_by_id(&common::db(&a), sent.id)
            .unwrap()
            .is_some_and(|m| m.state == State::Error)
    })
    .await;
    let m = chat::message_by_id(&common::db(&a), sent.id).unwrap().unwrap();
    assert_eq!(m.refused.as_deref(), Some(UNREACHABLE));
    assert_eq!(a.chat.outbox_of(&m).unwrap().state, "failed");
    assert!(outbox::head(&common::db(&a), b_id).unwrap().is_none(), "줄에서 빠진다");
    assert_eq!(answers(&a, conv.id), 0);
}
