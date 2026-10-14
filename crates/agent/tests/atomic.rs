//! 글과 보내기 행은 함께 남거나 함께 없다(#1c 적대적 평가 라운드 1) — 글만 기록되고 보낼 행이 없는 채로 꺼지는 틈이 없다.
//! 보내기 행 적기를 DB 트리거로 실패시켜, 그때 글(사람 요청 · 답 · 자동 왕복 넘김)도 남지 않는지 본다. 포기는 보내는 중인 행을 덮지 않는다.
mod common;
use an_core::agents;
use an_core::chat::Author;
use std::time::Duration;

fn block_outbox(s: &common::Side, kind: &str) {
    common::db(s)
        .execute_batch(&format!(
            "CREATE TRIGGER block_{kind} BEFORE INSERT ON outbox WHEN NEW.kind = '{kind}'
             BEGIN SELECT RAISE(ABORT, 'outbox 막음'); END;"
        ))
        .unwrap();
}

fn count(s: &common::Side, sql: &str) -> i64 {
    common::db(s).query_row(sql, [], |r| r.get(0)).unwrap()
}

#[tokio::test(flavor = "multi_thread")]
async fn request_not_recorded_without_outbox_row() {
    let a = common::side("atq-a", Duration::ZERO).await;
    let b = common::side("atq-b", Duration::ZERO).await;
    let (b_id, _) = common::pair(&a, &b).await;
    agents::add(&common::db(&b), "review", ".", "").unwrap();
    block_outbox(&a, "message");
    assert!(a.chat.send(b_id, "review", None, "질문").await.is_err());
    assert_eq!(count(&a, "SELECT COUNT(*) FROM messages"), 0, "보낼 행 없이 글만 남지 않는다");
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert!(b.runs.lock().unwrap().is_empty());
}

#[tokio::test(flavor = "multi_thread")]
async fn reply_not_recorded_without_outbox_row() {
    let a = common::side("atr-a", Duration::ZERO).await;
    let b = common::side("atr-b", Duration::ZERO).await;
    let (b_id, _) = common::pair(&a, &b).await;
    agents::add(&common::db(&b), "review", ".", "").unwrap();
    block_outbox(&b, "reply");
    a.chat.send(b_id, "review", None, "질문").await.unwrap();
    common::eventually(|| b.runs.lock().unwrap().len() == 1).await;
    tokio::time::sleep(Duration::from_millis(500)).await;
    let conv: i64 = count(&b, "SELECT id FROM conversations");
    let agent_msgs = an_core::chat::messages(&common::db(&b), conv)
        .unwrap()
        .into_iter()
        .filter(|m| m.author == Author::Agent)
        .count();
    assert_eq!(agent_msgs, 0, "보낼 행 없이 답만 남지 않는다");
}

/// 자동 왕복 넘김 — `sent` 표시와 보내기 행도 함께(적대적 평가 라운드 2). 행을 못 적으면 `sent` 로 남지 않고 자동 왕복을 멈춘다.
#[tokio::test(flavor = "multi_thread")]
async fn relay_step_not_marked_sent_without_outbox_row() {
    let a = common::side("ats-a", Duration::ZERO).await;
    let b = common::side("ats-b", Duration::ZERO).await;
    let (b_id, _) = common::pair(&a, &b).await;
    agents::add(&common::db(&a), "mine", ".", "").unwrap();
    agents::add(&common::db(&b), "review", ".", "").unwrap();
    block_outbox(&a, "relay_step");
    let (conv, _) = a
        .chat
        .start_relay(b_id, "review", "mine", 3, "시작", None)
        .await
        .unwrap();
    common::eventually(|| a.runs.lock().unwrap().len() == 1).await;
    // 고정 대기(0.5초) 대신 멈출 때까지 — 부하 걸린 PC 에서 넘김이 늦게 와도 판정이 앞서지 않게.
    let relay_on = || -> bool {
        common::db(&a)
            .query_row("SELECT relay_on FROM conversations WHERE id = ?1", [conv.id], |r| r.get::<_, Option<bool>>(0))
            .unwrap()
            .unwrap_or(false)
    };
    common::eventually(|| !relay_on()).await;
    let stuck = an_core::chat::messages(&common::db(&a), conv.id)
        .unwrap()
        .into_iter()
        .filter(|m| m.author == Author::Agent && m.state == an_core::chat::State::Sent)
        .count();
    assert_eq!(stuck, 0, "보낼 행 없이 sent 로 남지 않는다");
    assert_eq!(b.runs.lock().unwrap().len(), 1);
}

/// 포기는 기다리는 행만 — 지금 보내는 중(`inflight`)인 다른 글은 그 결과가 정한다(적대적 평가 라운드 2).
#[tokio::test(flavor = "multi_thread")]
async fn give_up_leaves_inflight_rows() {
    let a = common::side("atg-a", Duration::ZERO).await;
    let c = common::db(&a);
    let waiting = an_core::outbox::add(&c, 99, Some(1), Some(1), "message", "{}", None, "pending").unwrap();
    an_core::outbox::add(&c, 99, Some(2), Some(2), "message", "{}", None, "inflight").unwrap();
    let ids: Vec<i64> = an_core::outbox::queued_for(&c, 99)
        .unwrap()
        .into_iter()
        .map(|it| it.id)
        .collect();
    assert_eq!(ids, [waiting]);
}
