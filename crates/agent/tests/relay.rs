mod common;
use an_core::agents;
use an_core::chat::{self, Author, State};
use an_net::proto::{Request, Response};
use std::time::Duration;

fn uuids(conn: &an_core::db::Connection, conv: i64) -> Vec<String> {
    chat::messages(conn, conv)
        .unwrap()
        .into_iter()
        .map(|m| m.uuid)
        .collect()
}

fn relay_of(s: &common::Side, conv: i64) -> chat::Relay {
    chat::by_id(&common::db(s), conv)
        .unwrap()
        .unwrap()
        .relay
        .unwrap()
}

/// 받은 쪽 대화(하나뿐).
fn b_conv(b: &common::Side) -> chat::Conversation {
    chat::list(&common::db(b)).unwrap().remove(0)
}

/// 두 쪽 — A 에 내 에이전트 `mine`, B 에 상대 에이전트 `review`.
async fn two(tag: &str, a_delay: Duration) -> (common::Side, common::Side, i64) {
    let (a, b) = (
        common::side(&format!("{tag}-a"), a_delay).await,
        common::side(&format!("{tag}-b"), Duration::ZERO).await,
    );
    let (b_id, _) = common::pair(&a, &b).await;
    agents::add(&common::db(&a), "mine", ".", None, "").unwrap();
    agents::add(&common::db(&b), "review", ".", None, "").unwrap();
    (a, b, b_id)
}

/// 더 늘지 않을 때까지(0.3초 동안 그대로) 기다린다.
async fn settled(s: &common::Side, conv: i64) -> usize {
    let mut n = usize::MAX;
    for _ in 0..30 {
        tokio::time::sleep(Duration::from_millis(300)).await;
        let now = chat::messages(&common::db(s), conv).unwrap().len();
        if now == n {
            return n;
        }
        n = now;
    }
    panic!("9초 동안 멈추지 않았다({n}건)");
}

#[tokio::test(flavor = "multi_thread")]
async fn relay_runs_until_cap() {
    let (a, b, b_id) = two("rc", Duration::ZERO).await;
    assert!(
        a.chat
            .start_relay(b_id, "review", "mine", 51, "x")
            .await
            .is_err()
    );
    assert!(
        a.chat
            .start_relay(b_id, "review", "없음", 2, "x")
            .await
            .is_err()
    );
    let (conv, _) = a
        .chat
        .start_relay(b_id, "review", "mine", 2, "시작")
        .await
        .unwrap();
    // 사람 → B 답 → A 답(1) → B 답 → A 답(2, 상한 — B 에 기록만) — 양쪽 2번씩.
    assert_eq!(settled(&a, conv.id).await, 5);
    assert_eq!(
        (a.runs.lock().unwrap().len(), b.runs.lock().unwrap().len()),
        (2, 2)
    );
    let r = relay_of(&a, conv.id);
    assert_eq!((r.count, r.on), (2, false));
    // 양쪽 같은 순서.
    let bc = b_conv(&b);
    assert_eq!(settled(&b, bc.id).await, 5);
    let last = chat::messages(&common::db(&b), bc.id)
        .unwrap()
        .pop()
        .unwrap();
    assert_eq!(
        (last.author, last.mine, last.exec, last.state),
        (Author::Agent, false, false, State::Done),
        "상한의 마지막 답은 실행 요청이 아니다"
    );
    assert_eq!(
        uuids(&common::db(&a), conv.id),
        uuids(&common::db(&b), bc.id)
    );
    // 누구의 에이전트인지: A 이력의 3번째 = 내 에이전트, B 이력의 3번째 = 상대 에이전트의 실행 요청.
    let am = chat::messages(&common::db(&a), conv.id).unwrap();
    assert_eq!((am[2].author, am[2].mine), (Author::Agent, true));
    assert_eq!((am[1].author, am[1].mine), (Author::Agent, false));
    let bm = chat::messages(&common::db(&b), bc.id).unwrap();
    assert_eq!(
        (bm[2].author, bm[2].mine, bm[2].exec),
        (Author::Agent, false, true)
    );
    // 실행 입력에 "상대 에이전트" 표시(양쪽).
    assert!(
        a.runs.lock().unwrap()[0]
            .text
            .starts_with("[요청 — 상대 에이전트(")
    );
    assert!(
        b.runs.lock().unwrap()[1]
            .text
            .starts_with("[요청 — 상대 에이전트(")
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn human_write_stops_relay() {
    // A 에이전트가 도는 동안 A 사람이 쓴다.
    let (a, b, b_id) = two("hw", Duration::from_millis(400)).await;
    let (conv, _) = a
        .chat
        .start_relay(b_id, "review", "mine", 10, "시작")
        .await
        .unwrap();
    common::eventually(|| a.runs.lock().unwrap().len() == 1).await;
    a.chat.post(conv.id, "잠깐", false).await.unwrap();
    // 내 에이전트 답은 남지만 넘기지 않는다.
    assert_eq!(settled(&a, conv.id).await, 4);
    assert_eq!(b.runs.lock().unwrap().len(), 1);
    assert!(!relay_of(&a, conv.id).on);
    let last = chat::messages(&common::db(&a), conv.id)
        .unwrap()
        .pop()
        .unwrap();
    assert_eq!(
        (last.author, last.mine, last.state),
        (Author::Agent, true, State::Done)
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn receiver_stop_refuses_agent_requests() {
    // A 에이전트가 도는 동안 B 사람이 쓴다.
    let (a, b, b_id) = two("rs", Duration::from_millis(400)).await;
    let (conv, _) = a
        .chat
        .start_relay(b_id, "review", "mine", 10, "시작")
        .await
        .unwrap();
    common::eventually(|| a.runs.lock().unwrap().len() == 1).await;
    let bc = b_conv(&b);
    b.chat.post(bc.id, "B 가 멈춘다", false).await.unwrap();
    common::eventually(|| !relay_of(&a, conv.id).on).await;
    settled(&a, conv.id).await;
    assert_eq!(
        b.runs.lock().unwrap().len(),
        1,
        "B 에이전트는 다시 돌지 않는다"
    );
    assert!(b_conv(&b).relay_blocked);
    // 연 쪽은 알림으로 먼저 멈춰 넘기지 않았다(거절당한 오류가 아니다).
    let last = chat::messages(&common::db(&a), conv.id)
        .unwrap()
        .pop()
        .unwrap();
    assert_eq!(
        (last.author, last.mine, last.state),
        (Author::Agent, true, State::Done)
    );
    // 에이전트발 실행 요청은 거절, 사람 요청은 그대로.
    let peer = a.node.peers().unwrap().remove(0);
    let ask = |msg: &str, by_agent: Option<String>| Request::Message {
        conv: conv.uuid.clone(),
        msg: msg.into(),
        agent: "review".into(),
        text: "더".into(),
        lc: None,
        by_agent,
        relay: None,
    };
    let (resp, _) = a
        .node
        .send(
            b.addr,
            Some(peer.spki.clone()),
            &ask("x1", Some("mine".into())),
        )
        .await
        .unwrap();
    assert_eq!(resp, Response::error("stopped"));
    let (resp, _) = a
        .node
        .send(b.addr, Some(peer.spki.clone()), &ask("x2", None))
        .await
        .unwrap();
    assert_eq!(resp, Response::Ok);
}

#[tokio::test(flavor = "multi_thread")]
async fn resume_restarts_count() {
    let (a, b, b_id) = two("rr", Duration::ZERO).await;
    let (conv, _) = a
        .chat
        .start_relay(b_id, "review", "mine", 1, "시작")
        .await
        .unwrap();
    // 상한 1: 사람 → B 답 → A 답(기록만) — 양쪽 1번씩.
    assert_eq!(settled(&a, conv.id).await, 3);
    assert_eq!(relay_of(&a, conv.id).count, 1);
    assert_eq!(
        (a.runs.lock().unwrap().len(), b.runs.lock().unwrap().len()),
        (1, 1)
    );
    // 받은 쪽 멈춤 — 재개는 연 쪽만.
    let bc = b_conv(&b);
    b.chat.relay(bc.id, false).await.unwrap();
    assert!(b_conv(&b).relay_blocked);
    assert!(b.chat.relay(bc.id, true).await.is_err());
    // 연 쪽 재개: 카운트 0 → 마지막 상대 답을 넘겨 다시 상한까지, 받은 쪽 거절도 풀린다.
    let r = a.chat.relay(conv.id, true).await.unwrap().relay.unwrap();
    assert_eq!((r.count, r.on), (0, true));
    assert!(!b_conv(&b).relay_blocked);
    // 재개: 기록만 했던 A 답을 같은 글로 넘김 → B 답 → A 답(상한, 기록만).
    assert_eq!(settled(&a, conv.id).await, 5);
    assert_eq!(
        (a.runs.lock().unwrap().len(), b.runs.lock().unwrap().len()),
        (2, 2)
    );
    let bc = b_conv(&b);
    assert_eq!(
        uuids(&common::db(&a), conv.id),
        uuids(&common::db(&b), bc.id)
    );
    let r = relay_of(&a, conv.id);
    assert_eq!((r.count, r.on), (1, false));
}

#[tokio::test(flavor = "multi_thread")]
async fn remote_failure_stops_relay() {
    let (a, b, b_id) = two("rf", Duration::ZERO).await;
    let (conv, _) = a
        .chat
        .start_relay(b_id, "review", "mine", 10, "실패하라")
        .await
        .unwrap();
    // 상대 실행 실패 답은 내 에이전트 입력이 되지 않는다.
    assert_eq!(settled(&a, conv.id).await, 2);
    assert_eq!(
        (a.runs.lock().unwrap().len(), b.runs.lock().unwrap().len()),
        (0, 1)
    );
    assert!(!relay_of(&a, conv.id).on);
    let last = chat::messages(&common::db(&a), conv.id)
        .unwrap()
        .pop()
        .unwrap();
    assert_eq!((last.state, last.exec), (State::Error, false));
}

#[tokio::test(flavor = "multi_thread")]
async fn late_reply_after_transport_error_accepted() {
    // A 에이전트가 도는 동안 B 주소가 끊긴다 — 넘기기는 전송 오류(상대가 받았는지 모름).
    let (a, b, b_id) = two("lr", Duration::from_millis(400)).await;
    let (conv, _) = a
        .chat
        .start_relay(b_id, "review", "mine", 10, "시작")
        .await
        .unwrap();
    common::eventually(|| a.runs.lock().unwrap().len() == 1).await;
    common::db(&a)
        .execute(
            "UPDATE peers SET addr = '127.0.0.1:1' WHERE id = ?1",
            [b_id],
        )
        .unwrap();
    common::eventually(|| !relay_of(&a, conv.id).on).await;
    let forwarded = chat::messages(&common::db(&a), conv.id)
        .unwrap()
        .pop()
        .unwrap();
    assert_eq!(
        (forwarded.mine, forwarded.exec, forwarded.state),
        (true, true, State::Sent)
    );
    // 실제로는 B 가 받아 실행했고 답이 늦게 온다 — 받아야 한다.
    let pa = b.node.peers().unwrap().remove(0);
    let late = Request::Reply {
        conv: conv.uuid.clone(),
        msg: "late".into(),
        reply_to: forwarded.uuid.clone(),
        text: "늦은 답".into(),
        ok: true,
        turns: None,
        duration_ms: None,
        lc: None,
    };
    let (resp, _) = b
        .node
        .send(a.addr, Some(pa.spki.clone()), &late)
        .await
        .unwrap();
    assert_eq!(resp, Response::Ok);
    let ms = chat::messages(&common::db(&a), conv.id).unwrap();
    assert_eq!(ms.last().unwrap().text, "늦은 답");
    assert_eq!(ms[ms.len() - 2].state, State::Done);
}

#[tokio::test(flavor = "multi_thread")]
async fn stop_during_run_then_resume() {
    // A 에이전트가 도는 동안 A 가 멈춤 → 그 답은 넘기지 않음 → 재개하면 그 답을 넘긴다.
    let (a, b, b_id) = two("sr", Duration::from_millis(400)).await;
    let (conv, _) = a
        .chat
        .start_relay(b_id, "review", "mine", 10, "시작")
        .await
        .unwrap();
    common::eventually(|| a.runs.lock().unwrap().len() == 1).await;
    a.chat.relay(conv.id, false).await.unwrap();
    assert_eq!(settled(&a, conv.id).await, 3);
    assert_eq!(b.runs.lock().unwrap().len(), 1);
    a.chat.relay(conv.id, true).await.unwrap();
    common::eventually(|| b.runs.lock().unwrap().len() == 2).await;
    a.chat.relay(conv.id, false).await.unwrap();
    settled(&a, conv.id).await;
    let bc = b_conv(&b);
    settled(&b, bc.id).await;
    assert_eq!(
        uuids(&common::db(&a), conv.id),
        uuids(&common::db(&b), bc.id)
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn relay_events_start_step_end() {
    use an_net::node::{MessageKind, PeerEvent};
    let kinds = |s: &common::Side| -> Vec<MessageKind> {
        s.events
            .lock()
            .unwrap()
            .iter()
            .filter_map(|e| match e {
                PeerEvent::Message { kind, .. } => Some(*kind),
                _ => None,
            })
            .collect()
    };
    let (a, b, b_id) = two("ev", Duration::ZERO).await;
    let (conv, _) = a
        .chat
        .start_relay(b_id, "review", "mine", 3, "시작")
        .await
        .unwrap();
    assert_eq!(settled(&a, conv.id).await, 7);
    settled(&b, b_conv(&b).id).await;
    // 양쪽 모두 시작 1 · 중간 · 끝 1 — 기본(시작 · 끝만)이면 알림은 시작 · 끝 두 건.
    assert_eq!(
        kinds(&b),
        [
            MessageKind::RelayStart,
            MessageKind::RelayStep,
            MessageKind::RelayStep,
            MessageKind::RelayEnd
        ]
    );
    assert_eq!(
        kinds(&a),
        [
            MessageKind::RelayStart,
            MessageKind::RelayStep,
            MessageKind::RelayStep,
            MessageKind::RelayEnd
        ]
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn relay_events_first_run_fails() {
    use an_net::node::{MessageKind, PeerEvent};
    let kinds = |s: &common::Side| -> Vec<MessageKind> {
        s.events
            .lock()
            .unwrap()
            .iter()
            .filter_map(|e| match e {
                PeerEvent::Message { kind, .. } => Some(*kind),
                _ => None,
            })
            .collect()
    };
    let (a, b, b_id) = two("ef", Duration::ZERO).await;
    let (conv, _) = a
        .chat
        .start_relay(b_id, "review", "mine", 3, "실패하라")
        .await
        .unwrap();
    assert_eq!(settled(&a, conv.id).await, 2);
    // 첫 실행이 실패해도 양쪽 모두 시작 · 끝.
    assert_eq!(kinds(&a), [MessageKind::RelayStart, MessageKind::RelayEnd]);
    assert_eq!(kinds(&b), [MessageKind::RelayStart, MessageKind::RelayEnd]);
    // 받은 쪽은 자동 왕복 대화와 상대 에이전트 이름을 안다.
    assert_eq!(b_conv(&b).relay.unwrap().agent, "mine");
}
