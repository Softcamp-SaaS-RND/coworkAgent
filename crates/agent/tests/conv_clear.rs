//! 기록 삭제(#1x) — 내 글만 삭제(이 PC 만 비움 · 대화 남음) · 모든 사용자 글 삭제(`Clear` — 상대 PC · 관람자 사본도 비움), 받는 쪽은 그 대화의
//! 참여 상대가 보낸 것만 · 나간 대화는 거절, 이전 버전 상대는 아무것도 지우지 않고 거절, 실행 중이면 막고(이 PC) · 끝난 뒤 비운다(받는 쪽).
mod common;
use an_core::chat::{self, Author, Conversation, Dir, State};
use an_core::{agents, groups, share};
use an_net::identity::Identity;
use an_net::proto::{Request, Response};
use std::time::Duration;

struct Three {
    a: common::Side,
    b: common::Side,
    c: common::Side,
    /// A 쪽 B · C 의 id.
    b_id: i64,
    c_id: i64,
}

/// A · B · C — A 는 B · C 와 페어링, B 에 에이전트 `review`(실행 지연 `delay`).
async fn three(tag: &str, delay: Duration) -> Three {
    let a = common::side(&format!("{tag}-a"), Duration::ZERO).await;
    let b = common::side(&format!("{tag}-b"), delay).await;
    let c = common::side(&format!("{tag}-c"), Duration::ZERO).await;
    let (b_id, _) = common::pair(&a, &b).await;
    let (c_id, _) = common::pair(&a, &c).await;
    agents::add(&common::db(&b), "review", ".", "").unwrap();
    a.chat.set_retry(1, 1, 3600);
    Three { a, b, c, b_id, c_id }
}

fn texts(s: &common::Side, conv: i64) -> Vec<String> {
    chat::messages(&common::db(s), conv)
        .unwrap()
        .into_iter()
        .map(|m| m.text)
        .collect()
}

fn find(s: &common::Side, uuid: &str, dir_shared: bool) -> Option<Conversation> {
    chat::list(&common::db(s))
        .unwrap()
        .into_iter()
        .find(|c| c.uuid == uuid && (c.dir == Dir::Shared) == dir_shared)
}

fn count(s: &common::Side, sql: &str, id: i64) -> i64 {
    common::db(s).query_row(sql, [id], |r| r.get(0)).unwrap()
}

/// A 가 B 의 review 에 묻고 답을 받은 대화.
async fn asked(t: &Three, q: &str) -> Conversation {
    let (c, _) = t.a.chat.send(t.b_id, "review", None, q).await.unwrap();
    common::eventually(|| {
        chat::messages(&common::db(&t.a), c.id)
            .unwrap()
            .iter()
            .any(|m| m.author == Author::Agent && m.state == State::Done)
    })
    .await;
    c
}

#[tokio::test(flavor = "multi_thread")]
async fn clear_mine_keeps_room() {
    let t = three("cl-mine", Duration::ZERO).await;
    let c = asked(&t, "질문").await;
    let db = common::db(&t.a);
    let g = groups::add(&db, "묶음").unwrap();
    groups::join(&db, g.id, c.id).unwrap();
    chat::set_starred(&db, c.id, true).unwrap();
    chat::set_title(&db, c.id, Some("제목")).unwrap();
    drop(db);
    t.a.chat.clear_conversation(c.id, false).await.unwrap();
    let after = chat::by_id(&common::db(&t.a), c.id).unwrap().expect("대화는 남는다");
    assert!(after.session.is_none(), "세션 기록 비움");
    assert!(texts(&t.a, c.id).is_empty());
    for sql in [
        "SELECT count(*) FROM agent_sessions WHERE conversation_id = ?1",
        "SELECT count(*) FROM relay_events WHERE conversation_id = ?1",
        "SELECT count(*) FROM messages WHERE conversation_id = ?1",
    ] {
        assert_eq!(count(&t.a, sql, c.id), 0, "{sql}");
    }
    assert_eq!(count(&t.a, "SELECT count(*) FROM group_convs WHERE conversation_id = ?1", c.id), 1, "그룹 그대로");
    assert_eq!(chat::marks(&common::db(&t.a), c.id).unwrap(), Some((Some("제목".into()), true)), "제목 · 즐겨찾기 그대로");
    // 상대 PC 사본은 그대로.
    let at_b = find(&t.b, &c.uuid, false).unwrap();
    assert_eq!(texts(&t.b, at_b.id).len(), 2);
    // 다시 쓸 수 있다.
    t.a.chat.post(c.id, "또 질문", true).await.unwrap();
    common::eventually(|| texts(&t.a, c.id).iter().any(|x| x == "답: 또 질문")).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn clear_all_both_sides() {
    let t = three("cl-all", Duration::ZERO).await;
    let c = asked(&t, "질문").await;
    t.a.chat.clear_conversation(c.id, true).await.unwrap();
    assert!(texts(&t.a, c.id).is_empty());
    let at_b = find(&t.b, &c.uuid, false).unwrap();
    common::eventually(|| texts(&t.b, at_b.id).is_empty()).await;
    assert!(chat::by_id(&common::db(&t.b), at_b.id).unwrap().is_some(), "상대 대화도 남는다");
    common::eventually(|| count(&t.a, "SELECT count(*) FROM outbox WHERE peer_id = ?1 AND kind = 'clear'", t.b_id) == 0).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn clear_all_observer_copy() {
    let t = three("cl-obs", Duration::ZERO).await;
    let c = asked(&t, "공유할 질문").await;
    t.a.chat.share_add(c.id, t.c_id).await.unwrap();
    common::eventually(|| find(&t.c, &c.uuid, true).is_some_and(|x| texts(&t.c, x.id).len() == 2)).await;
    t.a.chat.clear_conversation(c.id, true).await.unwrap();
    common::eventually(|| find(&t.c, &c.uuid, true).is_some_and(|x| texts(&t.c, x.id).is_empty())).await;
    assert_eq!(share::watchers(&common::db(&t.a), c.id).unwrap().len(), 1, "관람은 그대로");
    // 관람자는 기록 삭제가 없다(읽기 전용).
    let copy = find(&t.c, &c.uuid, true).unwrap();
    let e = t.c.chat.clear_conversation(copy.id, false).await.unwrap_err();
    assert_eq!(e.kind(), std::io::ErrorKind::InvalidInput, "{e}");
}

#[tokio::test(flavor = "multi_thread")]
async fn clear_inbound_wrong_peer() {
    // C 가 A–B 대화 uuid 로 B 에 Clear 를 흉내 — C 는 그 대화의 참여 상대가 아니다(B 와 페어링해 보낸다).
    let t = three("cl-wrong", Duration::ZERO).await;
    let c = asked(&t, "질문").await;
    let (_, c_at_b) = common::pair(&t.b, &t.c).await;
    let _ = c_at_b;
    let at_b = find(&t.b, &c.uuid, false).unwrap();
    let peer_b = an_core::peers::list(&common::db(&t.c))
        .unwrap()
        .into_iter()
        .find(|p| p.addr == t.b.addr.to_string())
        .unwrap();
    let resp = t
        .c
        .node
        .send(t.b.addr, Some(peer_b.spki.clone()), &Request::Clear { conv: c.uuid.clone(), id: an_agent::chat::new_uuid() })
        .await
        .unwrap()
        .0;
    assert_eq!(resp, Response::error("unknown"));
    assert_eq!(texts(&t.b, at_b.id).len(), 2, "다른 상대 것은 지우지 않는다");
    // uuid 꼴이 아니면 invalid.
    let resp = t
        .c
        .node
        .send(t.b.addr, Some(peer_b.spki), &Request::Clear { conv: "x".into(), id: an_agent::chat::new_uuid() })
        .await
        .unwrap()
        .0;
    assert_eq!(resp, Response::error("invalid"));
}

#[tokio::test(flavor = "multi_thread")]
async fn clear_inbound_left() {
    // B 가 그 대화를 나갔다 — A 의 Clear 는 거절(`left`), 닫힌 대화에는 모든 사용자 글 삭제가 없다.
    let t = three("cl-left", Duration::ZERO).await;
    let c = asked(&t, "질문").await;
    let at_b = find(&t.b, &c.uuid, false).unwrap();
    t.b.chat.leave_conversation(at_b.id).unwrap();
    common::eventually(|| chat::by_id(&common::db(&t.a), c.id).unwrap().unwrap().peer_left_at.is_some()).await;
    let e = t.a.chat.clear_conversation(c.id, true).await.unwrap_err();
    assert_eq!(e.kind(), std::io::ErrorKind::InvalidInput, "{e}");
    assert_eq!(texts(&t.a, c.id).len(), 2, "거절이면 아무것도 지우지 않는다");
    // 프레임을 직접 보내도 받는 쪽이 거절한다.
    let peer_b = an_core::peers::by_id(&common::db(&t.a), t.b_id).unwrap().unwrap();
    let resp = t
        .a
        .node
        .send(t.b.addr, Some(peer_b.spki.clone()), &Request::Clear { conv: c.uuid.clone(), id: an_agent::chat::new_uuid() })
        .await
        .unwrap()
        .0;
    assert_eq!(resp, Response::error(chat::LEFT));
    assert_eq!(texts(&t.b, at_b.id).len(), 2);
    // 내 글만 삭제는 닫힌 대화에도 된다.
    t.a.chat.clear_conversation(c.id, false).await.unwrap();
    assert!(texts(&t.a, c.id).is_empty());
}

#[tokio::test(flavor = "multi_thread")]
async fn clear_all_old_peer() {
    let t = three("cl-old", Duration::ZERO).await;
    let c = asked(&t, "질문").await;
    // B 를 이전 버전(기록 지우기를 모름)으로 — Ping 에 `clear` 없는 Pong.
    let old = common::old_peer(
        Identity::load_or_create(&t.b.paths.identity_file()).unwrap(),
        serde_json::json!({"type": "pong", "focus": false, "ctx": true, "share": true, "leave": true}),
    )
    .await;
    let real = {
        let conn = common::db(&t.a);
        let was: String = conn
            .query_row("SELECT addr FROM peers WHERE id = ?1", [t.b_id], |r| r.get(0))
            .unwrap();
        conn.execute("UPDATE peers SET addr = ?1 WHERE id = ?2", (old.to_string(), t.b_id))
            .unwrap();
        was
    };
    let e = t.a.chat.clear_conversation(c.id, true).await.unwrap_err();
    assert_eq!(e.kind(), std::io::ErrorKind::Unsupported, "{e}");
    assert_eq!(e.to_string(), an_agent::chat::CLEAR_UNKNOWN);
    assert_eq!(texts(&t.a, c.id).len(), 2, "아무것도 지우지 않는다");
    assert_eq!(count(&t.a, "SELECT count(*) FROM outbox WHERE peer_id = ?1 AND kind = 'clear'", t.b_id), 0);
    common::db(&t.a)
        .execute("UPDATE peers SET addr = ?1 WHERE id = ?2", (real, t.b_id))
        .unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn clear_busy_running() {
    let t = three("cl-busy", Duration::from_millis(2500)).await;
    let c = asked(&t, "먼저").await;
    // B 가 실행 중 — A 는 실행이 없어 비운다, B 는 `retry` → 실행이 끝난 뒤 비운다.
    t.a.chat.post(c.id, "느린 질문", true).await.unwrap();
    let at_b = find(&t.b, &c.uuid, false).unwrap();
    common::eventually(|| {
        chat::messages(&common::db(&t.b), at_b.id)
            .unwrap()
            .iter()
            .any(|m| m.state == State::Running)
    })
    .await;
    // B 쪽에서 직접 — 도는 실행이 있으면 막는다(이 PC).
    let e = t.b.chat.clear_conversation(at_b.id, false).await.unwrap_err();
    assert_eq!(e.kind(), std::io::ErrorKind::ResourceBusy, "{e}");
    t.a.chat.clear_conversation(c.id, true).await.unwrap();
    // `Clear` 가 닿을 시간(재시도 1초 간격) — 실행이 도는 동안(2.5초)은 비우지 않는다.
    tokio::time::sleep(Duration::from_millis(900)).await;
    assert!(
        chat::messages(&common::db(&t.b), at_b.id)
            .unwrap()
            .iter()
            .any(|m| m.state == State::Running),
        "아직 실행 중"
    );
    assert!(texts(&t.b, at_b.id).len() >= 3, "실행 중에는 비우지 않는다");
    common::eventually(|| texts(&t.b, at_b.id).is_empty()).await;
    common::eventually(|| count(&t.a, "SELECT count(*) FROM outbox WHERE peer_id = ?1 AND kind = 'clear'", t.b_id) == 0).await;
    // 지운 질문의 늦은 답은 A 에 되살아나지 않는다.
    tokio::time::sleep(Duration::from_millis(500)).await;
    assert_eq!(texts(&t.a, c.id), Vec::<String>::new());
}


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

/// 적대적 평가 R1 — 같은 `Clear`(같은 id)가 다시 와도(응답 유실 · 재전송) 두 번 지우지 않는다: 그 사이 쓴 글은 남는다.
#[tokio::test(flavor = "multi_thread")]
async fn clear_resend_idempotent() {
    let t = three("cl-idem", Duration::ZERO).await;
    let c = asked(&t, "질문").await;
    let at_b = find(&t.b, &c.uuid, false).unwrap();
    let peer_b = an_core::peers::by_id(&common::db(&t.a), t.b_id).unwrap().unwrap();
    let clear = Request::Clear { conv: c.uuid.clone(), id: an_agent::chat::new_uuid() };
    let resp = t.a.node.send(t.b.addr, Some(peer_b.spki.clone()), &clear).await.unwrap().0;
    assert_eq!(resp, Response::Ok);
    assert!(texts(&t.b, at_b.id).is_empty());
    // 그 사이 새 글 — 같은 지우기가 다시 와도 남는다.
    t.a.chat.post(c.id, "새 질문", true).await.unwrap();
    common::eventually(|| texts(&t.b, at_b.id).iter().any(|x| x == "답: 새 질문")).await;
    let resp = t.a.node.send(t.b.addr, Some(peer_b.spki.clone()), &clear).await.unwrap().0;
    assert_eq!(resp, Response::Ok);
    assert!(texts(&t.b, at_b.id).iter().any(|x| x == "새 질문"), "재전송은 다시 지우지 않는다");
}

/// 적대적 평가 R1 — 지운 뒤 쓴 글이 `Clear` 보다 먼저 닿지 않는다(같은 대화 줄 — 상대가 꺼져 있어도): 상대에는 새 글만 남는다.
#[tokio::test(flavor = "multi_thread")]
async fn clear_before_later_posts() {
    let t = three("cl-order", Duration::ZERO).await;
    let c = asked(&t, "옛 질문").await;
    let at_b = find(&t.b, &c.uuid, false).unwrap();
    let real = readdr(&t.a, t.b_id, &closed_port());
    t.a.chat.clear_conversation(c.id, true).await.unwrap();
    // 지우기 행은 그 대화 줄의 앞 행이다 — 뒤에 쓴 글은 바로 보내지 않고 그 뒤에 선다(`dispatch` 의 `has_before`).
    assert!(an_core::outbox::has_before(&common::db(&t.a), c.id, i64::MAX, &[]).unwrap(), "지우기가 그 대화 줄을 막는다");
    // 상대가 꺼진 사이 새 글 — 지우기 행 뒤에 선다.
    let _ = t.a.chat.post(c.id, "새 질문", true).await;
    readdr(&t.a, t.b_id, &real);
    common::eventually(|| texts(&t.b, at_b.id).iter().any(|x| x == "답: 새 질문")).await;
    let left = texts(&t.b, at_b.id);
    assert!(!left.iter().any(|x| x == "옛 질문" || x == "답: 옛 질문"), "{left:?}");
    assert!(left.iter().any(|x| x == "새 질문"), "새 글은 지우기 뒤에 닿아 남는다: {left:?}");
}

/// 적대적 평가 R1 — 기록 삭제로 지운 글이 같은 uuid 로 늦게 다시 와도(응답 유실 뒤 재전송) 되살아나거나 다시 실행되지 않는다.
#[tokio::test(flavor = "multi_thread")]
async fn cleared_message_resend_not_rerun() {
    let t = three("cl-rerun", Duration::ZERO).await;
    let c = asked(&t, "질문").await;
    let asked_msg = chat::messages(&common::db(&t.a), c.id).unwrap().into_iter().find(|m| m.text == "질문").unwrap();
    let at_b = find(&t.b, &c.uuid, false).unwrap();
    let runs = t.b.runs.lock().unwrap().len();
    t.b.chat.clear_conversation(at_b.id, false).await.unwrap();
    let peer_b = an_core::peers::by_id(&common::db(&t.a), t.b_id).unwrap().unwrap();
    let again = Request::Message {
        conv: c.uuid.clone(),
        msg: asked_msg.uuid.clone(),
        agent: "review".into(),
        text: "질문".into(),
        lc: None,
        by_agent: None,
        relay: None,
        via: None,
        request: false,
        mention: None,
    };
    let resp = t.a.node.send(t.b.addr, Some(peer_b.spki.clone()), &again).await.unwrap().0;
    assert_eq!(resp, Response::Ok);
    tokio::time::sleep(Duration::from_millis(500)).await;
    assert!(texts(&t.b, at_b.id).is_empty(), "되살아나지 않는다");
    assert_eq!(t.b.runs.lock().unwrap().len(), runs, "다시 실행하지 않는다");
}

/// 적대적 평가 R1 — 꺼져 있던 상대가 이전 버전으로 돌아오면 지우기를 보내지 않고 내 활동에 "반영하지 못함 — 기록 삭제" 를 남긴다.
#[tokio::test(flavor = "multi_thread")]
async fn clear_late_old_peer_noted() {
    let t = three("cl-late", Duration::ZERO).await;
    let c = asked(&t, "질문").await;
    readdr(&t.a, t.b_id, &closed_port());
    t.a.chat.clear_conversation(c.id, true).await.unwrap();
    assert!(texts(&t.a, c.id).is_empty(), "꺼진 상대 — 이 PC 는 비운다");
    let old = common::old_peer(
        Identity::load_or_create(&t.b.paths.identity_file()).unwrap(),
        serde_json::json!({"type": "pong", "focus": false, "ctx": true, "share": true, "leave": true}),
    )
    .await;
    readdr(&t.a, t.b_id, &old.to_string());
    common::eventually(|| count(&t.a, "SELECT count(*) FROM outbox WHERE peer_id = ?1 AND kind = 'clear'", t.b_id) == 0).await;
    assert_eq!(
        count(&t.a, "SELECT count(*) FROM relay_events WHERE conversation_id = ?1 AND kind = 'sync_failed' AND detail = '기록 삭제'", c.id),
        1,
        "내 활동에 남는다"
    );
}

/// 적대적 평가 R1 — 모든 사용자 글 삭제 뒤 곧바로 나가도(상대가 꺼져 있던 사이) 기다리던 지우기는 남아 상대 쪽도 비운다.
#[tokio::test(flavor = "multi_thread")]
async fn clear_then_leave_still_clears_peer() {
    let t = three("cl-leave", Duration::ZERO).await;
    let c = asked(&t, "질문").await;
    let at_b = find(&t.b, &c.uuid, false).unwrap();
    let real = readdr(&t.a, t.b_id, &closed_port());
    t.a.chat.clear_conversation(c.id, true).await.unwrap();
    t.a.chat.delete_conversation(c.id).unwrap();
    assert_eq!(count(&t.a, "SELECT count(*) FROM outbox WHERE peer_id = ?1 AND kind = 'clear' AND conv_id IS NULL", t.b_id), 1, "떼어 남김");
    readdr(&t.a, t.b_id, &real);
    common::eventually(|| texts(&t.b, at_b.id).is_empty()).await;
    common::eventually(|| chat::by_id(&common::db(&t.b), at_b.id).unwrap().is_some_and(|x| x.peer_left_at.is_some())).await;
}

/// 적대적 평가 R2 — 그 대화에서 상대에게 보내는 중(`inflight`)인 글이 있으면 모든 사용자 글 삭제를 막는다(옛 글이 지우기보다 늦게 닿아 되살아나지 않게).
#[tokio::test(flavor = "multi_thread")]
async fn clear_all_waits_inflight() {
    let t = three("cl-fly", Duration::ZERO).await;
    let c = asked(&t, "질문").await;
    // 보내는 중인 옛 글 흉내 — 그 대화 · 그 상대 줄의 `inflight` 행.
    an_core::outbox::add(&common::db(&t.a), t.b_id, Some(c.id), None, "message", "{}", None, "inflight").unwrap();
    let e = t.a.chat.clear_conversation(c.id, true).await.unwrap_err();
    assert_eq!(e.kind(), std::io::ErrorKind::ResourceBusy, "{e}");
    assert_eq!(texts(&t.a, c.id).len(), 2, "막으면 아무것도 지우지 않는다");
    // 내 글만 삭제는 이 PC 만이라 된다.
    t.a.chat.clear_conversation(c.id, false).await.unwrap();
    assert!(texts(&t.a, c.id).is_empty());
}
