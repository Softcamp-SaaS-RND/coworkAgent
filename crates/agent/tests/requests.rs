//! 요청 목록(#1d) — 두 PC 통합: 요청은 실행 없이 받는 PC 목록에 '대기', 받은 쪽만 수용 · 반려 · 완료(메모), 상태는 요청자에게(꺼져 있으면
//! 나중에), 집중 중에도 받음, 위조 · 규칙 위반 거부.
mod common;
use an_agent::chat::Compose;
use an_core::chat::{self, Author, ReqState, State};
use an_core::agents;
use an_net::node::{MessageKind, PeerEvent};
use an_net::proto::{Request, Response};
use std::io::ErrorKind;
use std::time::{Duration, Instant};

fn ask(request: bool) -> Compose<'static> {
    Compose {
        request,
        ..Default::default()
    }
}

async fn pair_review(tag: &str) -> (common::Side, common::Side, i64, i64) {
    let a = common::side(&format!("{tag}-a"), Duration::ZERO).await;
    let b = common::side(&format!("{tag}-b"), Duration::ZERO).await;
    let (b_id, a_id) = common::pair(&a, &b).await;
    agents::add(&common::db(&b), "review", ".", "").unwrap();
    (a, b, b_id, a_id)
}

fn msg(s: &common::Side, uuid: &str) -> chat::Message {
    chat::message_by_uuid(&common::db(s), uuid).unwrap().unwrap()
}

fn has(s: &common::Side, uuid: &str) -> bool {
    chat::message_by_uuid(&common::db(s), uuid).unwrap().is_some()
}

fn runs(s: &common::Side) -> usize {
    s.runs.lock().unwrap().len()
}

/// 조건이 참이 될 때까지(최대 `secs` 초).
async fn within(secs: u64, mut f: impl FnMut() -> bool) {
    let until = Instant::now() + Duration::from_secs(secs);
    while !f() {
        assert!(Instant::now() < until, "{secs}초 안에 조건이 맞지 않았다");
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

fn invalid(e: std::io::Error) -> bool {
    e.kind() == ErrorKind::InvalidInput
}

#[tokio::test(flavor = "multi_thread")]
async fn new_conversation_request_not_run() {
    let (a, b, b_id, _) = pair_review("new").await;
    let (_, sent) = a
        .chat
        .send_with(b_id, "review", None, "로그 형식 바꿔 주세요", ask(true))
        .await
        .unwrap();
    // 보낸 쪽 — 답을 기다리지 않는 글(닿으면 끝).
    assert!(sent.request && !sent.exec, "{sent:?}");
    assert_eq!(sent.state, State::Done);
    assert_eq!(sent.req_state, Some(ReqState::Wait));
    // 받은 쪽 — 실행 · 대기열 없이 '대기'.
    common::eventually(|| has(&b, &sent.uuid)).await;
    let got = msg(&b, &sent.uuid);
    assert!(got.request && !got.exec && !got.mine && got.author == Author::Human, "{got:?}");
    assert_eq!((got.state, got.req_state), (State::Done, Some(ReqState::Wait)));
    assert!(chat::pending(&common::db(&b), got.conversation_id).unwrap().is_empty());
    tokio::time::sleep(Duration::from_millis(200)).await;
    assert_eq!(runs(&b), 0, "요청은 실행하지 않는다");
    assert!(
        b.events
            .lock()
            .unwrap()
            .iter()
            .any(|e| matches!(e, PeerEvent::Message { kind: MessageKind::Ask, .. })),
        "받은 요청 알림"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn continued_and_inbound_conversation_requests_not_run() {
    let (a, b, b_id, _) = pair_review("cont").await;
    let (conv, _) = a
        .chat
        .send_with(b_id, "review", None, "질의 하나", ask(false))
        .await
        .unwrap();
    common::eventually(|| runs(&b) == 1).await;
    // 이어 쓰기 요청 — 입력창 대상이 에이전트여도 실행 없음.
    let (_, r, _) = a.chat.post_with(conv.id, "이어 쓴 요청", true, ask(true)).await.unwrap();
    common::eventually(|| has(&b, &r.uuid)).await;
    assert_eq!(msg(&b, &r.uuid).req_state, Some(ReqState::Wait));
    // 받은 대화(B)의 요청 — B 의 에이전트를 실행하지 않고 A 에 '대기' 로.
    let bc = chat::list(&common::db(&b)).unwrap().remove(0);
    let (_, bm, warning) = b.chat.post_with(bc.id, "B 의 요청", true, ask(true)).await.unwrap();
    assert_eq!(warning, None);
    assert!(!bm.exec, "{bm:?}");
    common::eventually(|| has(&a, &bm.uuid)).await;
    let am = msg(&a, &bm.uuid);
    assert!(am.request && !am.mine && !am.exec, "{am:?}");
    assert_eq!(am.req_state, Some(ReqState::Wait));
    tokio::time::sleep(Duration::from_millis(200)).await;
    assert_eq!((runs(&a), runs(&b)), (0, 1), "질의만 실행");
}

#[tokio::test(flavor = "multi_thread")]
async fn focus_refuses_query_but_takes_request() {
    let (a, b, b_id, _) = pair_review("focus").await;
    an_core::settings::set(&common::db(&b), an_core::settings::FOCUS_KEY, "1").unwrap();
    let (_, r) = a
        .chat
        .send_with(b_id, "review", None, "집중 중 요청", ask(true))
        .await
        .unwrap();
    common::eventually(|| has(&b, &r.uuid)).await;
    assert_eq!(msg(&b, &r.uuid).req_state, Some(ReqState::Wait));
    let err = a
        .chat
        .send_with(b_id, "review", None, "집중 중 질의", ask(false))
        .await
        .unwrap_err();
    assert!(err.to_string().contains("집중"), "{err}");
    assert_eq!(runs(&b), 0);
}

#[tokio::test(flavor = "multi_thread")]
async fn receiver_moves_state_with_note() {
    let (a, b, b_id, _) = pair_review("move").await;
    let (conv, q) = a
        .chat
        .send_with(b_id, "review", None, "질의", ask(false))
        .await
        .unwrap();
    let (_, r, _) = a.chat.post_with(conv.id, "요청", true, ask(true)).await.unwrap();
    common::eventually(|| has(&b, &r.uuid) && has(&b, &q.uuid)).await;
    let id = msg(&b, &r.uuid).id;
    // 같은 상태로는 바꾸지 않는다(대기 → 대기).
    assert!(invalid(b.chat.set_req_state(id, ReqState::Wait, "", None).await.unwrap_err()));
    let (m, warning) = b
        .chat
        .set_req_state(id, ReqState::Accepted, " 이번 주 안에 ", Some(true))
        .await
        .unwrap();
    assert_eq!(warning, None);
    assert_eq!(
        (m.req_state, m.req_note.as_deref(), m.saved_at.is_some()),
        (Some(ReqState::Accepted), Some("이번 주 안에"), true)
    );
    common::eventually(|| msg(&a, &r.uuid).req_state == Some(ReqState::Accepted)).await;
    assert_eq!(msg(&a, &r.uuid).req_note.as_deref(), Some("이번 주 안에"));
    b.chat.set_req_state(id, ReqState::Done, "반영함", None).await.unwrap();
    common::eventually(|| msg(&a, &r.uuid).req_state == Some(ReqState::Done)).await;
    // 끝난 요청도 되돌린다(완료 → 대기 — 사용자 2026-10-19) — 요청자에게 그대로 간다.
    b.chat.set_req_state(id, ReqState::Wait, "다시 봄", None).await.unwrap();
    common::eventually(|| msg(&a, &r.uuid).req_state == Some(ReqState::Wait)).await;
    assert_eq!(msg(&a, &r.uuid).req_note.as_deref(), Some("다시 봄"));
    // 쓴 쪽 · 질의 · 답은 못 바꾼다.
    assert!(invalid(
        a.chat
            .set_req_state(msg(&a, &r.uuid).id, ReqState::Accepted, "", None)
            .await
            .unwrap_err()
    ));
    assert!(invalid(
        b.chat
            .set_req_state(msg(&b, &q.uuid).id, ReqState::Accepted, "", None)
            .await
            .unwrap_err()
    ));
    // 새 요청으로 메모 규칙 — 거부되면 아무것도 바뀌지 않는다.
    let (_, r2, _) = a.chat.post_with(conv.id, "요청 둘", true, ask(true)).await.unwrap();
    common::eventually(|| has(&b, &r2.uuid)).await;
    let id2 = msg(&b, &r2.uuid).id;
    for bad in ["탭\t가", &"가".repeat(201)] {
        assert!(invalid(b.chat.set_req_state(id2, ReqState::Rejected, bad, None).await.unwrap_err()));
    }
    assert_eq!(msg(&b, &r2.uuid).req_state, Some(ReqState::Wait));
    b.chat
        .set_req_state(id2, ReqState::Rejected, &"가".repeat(200), None)
        .await
        .unwrap();
    common::eventually(|| msg(&a, &r2.uuid).req_state == Some(ReqState::Rejected)).await;
}

/// 의견 여러 줄(#1z) — 받은 쪽 줄바꿈이 요청자에게 그대로(보낼 때 U+2028, 받을 때 되돌림).
#[tokio::test(flavor = "multi_thread")]
async fn note_lines_reach_requester() {
    let (a, b, b_id, _) = pair_review("lines").await;
    let (_, r) = a
        .chat
        .send_with(b_id, "review", None, "요청", ask(true))
        .await
        .unwrap();
    common::eventually(|| has(&b, &r.uuid)).await;
    let id = msg(&b, &r.uuid).id;
    let (m, warning) = b
        .chat
        .set_req_state(id, ReqState::Accepted, "오늘\r\n금요일까지", None)
        .await
        .unwrap();
    assert_eq!((m.req_note.as_deref(), warning), (Some("오늘\n금요일까지"), None));
    common::eventually(|| msg(&a, &r.uuid).req_state == Some(ReqState::Accepted)).await;
    assert_eq!(msg(&a, &r.uuid).req_note.as_deref(), Some("오늘\n금요일까지"));
}

#[tokio::test(flavor = "multi_thread")]
async fn state_reaches_requester_after_it_returns() {
    let (a, b, b_id, a_id) = pair_review("off").await;
    b.chat.set_retry(1, 1, 600);
    let (_, r) = a
        .chat
        .send_with(b_id, "review", None, "요청", ask(true))
        .await
        .unwrap();
    common::eventually(|| has(&b, &r.uuid)).await;
    // 요청자(A)가 꺼졌다 — 닫힌 포트.
    let conn = common::db(&b);
    let was: String = conn
        .query_row("SELECT addr FROM peers WHERE id = ?1", [a_id], |r| r.get(0))
        .unwrap();
    let closed = std::net::TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap();
    conn.execute("UPDATE peers SET addr = ?1 WHERE id = ?2", (closed.to_string(), a_id))
        .unwrap();
    let (m, warning) = b
        .chat
        .set_req_state(msg(&b, &r.uuid).id, ReqState::Accepted, "확인", None)
        .await
        .unwrap();
    assert_eq!((m.req_state, warning), (Some(ReqState::Accepted), None), "보관 — 경고 없음");
    tokio::time::sleep(Duration::from_millis(1200)).await;
    assert_eq!(msg(&a, &r.uuid).req_state, Some(ReqState::Wait));
    conn.execute("UPDATE peers SET addr = ?1 WHERE id = ?2", (&was, a_id)).unwrap();
    within(10, || msg(&a, &r.uuid).req_state == Some(ReqState::Accepted)).await;
    assert_eq!(msg(&a, &r.uuid).req_note.as_deref(), Some("확인"));
    assert!(
        a.events
            .lock()
            .unwrap()
            .iter()
            .any(|e| matches!(e, PeerEvent::Message { kind: MessageKind::AskState, .. })),
        "요청 상태 알림"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn forged_state_and_rule_violations_refused() {
    let (a, b, b_id, _) = pair_review("forge").await;
    let (conv, q) = a
        .chat
        .send_with(b_id, "review", None, "질의", ask(false))
        .await
        .unwrap();
    let (other, oreq) = a
        .chat
        .send_with(b_id, "review", None, "다른 대화 요청", ask(true))
        .await
        .unwrap();
    common::eventually(|| {
        chat::messages(&common::db(&a), conv.id)
            .unwrap()
            .iter()
            .any(|m| m.author == Author::Agent)
    })
    .await;
    let answer = chat::messages(&common::db(&a), conv.id)
        .unwrap()
        .into_iter()
        .find(|m| m.author == Author::Agent)
        .unwrap()
        .uuid;
    let pa = b.node.peers().unwrap().remove(0);
    let forge = |conv: &str, msg: &str, state: &str| Request::ReqState {
        conv: conv.into(),
        msg: msg.into(),
        state: state.into(),
        note: None,
    };
    // 내 질의 · 답 · 다른 대화의 요청은 `unknown`, 모르는 상태는 `bad_request` — 쓴 쪽 기록은 그대로.
    for (frame, want) in [
        (forge(&conv.uuid, &q.uuid, "accepted"), "unknown"),
        (forge(&conv.uuid, &answer, "accepted"), "unknown"),
        (forge(&conv.uuid, &oreq.uuid, "accepted"), "unknown"),
        (forge(&other.uuid, &oreq.uuid, "maybe"), "bad_request"),
    ] {
        let (resp, _) = b.node.send(a.addr, Some(pa.spki.clone()), &frame).await.unwrap();
        assert_eq!(resp, Response::error(want), "{frame:?}");
    }
    assert_eq!(msg(&a, &oreq.uuid).req_state, Some(ReqState::Wait));
    // 요청은 멘션 · CLI 와 함께 쓰지 않는다 — 글을 남기지 않는다.
    let before = chat::messages(&common::db(&a), conv.id).unwrap().len();
    let mention = Compose {
        request: true,
        mention: Some("guide"),
        ..Default::default()
    };
    assert!(invalid(a.chat.post_with(conv.id, "멘션 요청", true, mention).await.unwrap_err()));
    let cli = Compose {
        request: true,
        via: Some("cli"),
        ..Default::default()
    };
    assert!(invalid(a.chat.post_with(conv.id, "CLI 요청", true, cli).await.unwrap_err()));
    assert!(invalid(a.chat.send_with(b_id, "review", None, "CLI 새 요청", cli).await.unwrap_err()));
    assert_eq!(chat::messages(&common::db(&a), conv.id).unwrap().len(), before);
    // 자동 왕복을 여는 요청도 없다 — 대화를 만들지 않는다.
    agents::add(&common::db(&a), "mine", ".", "").unwrap();
    let convs = chat::list(&common::db(&a)).unwrap().len();
    assert!(invalid(
        a.chat
            .start_relay_with(b_id, "review", "mine", 3, "요청으로 자동 왕복", ask(true))
            .await
            .unwrap_err()
    ));
    assert_eq!(chat::list(&common::db(&a)).unwrap().len(), convs);
}

#[tokio::test(flavor = "multi_thread")]
async fn state_give_up_marked_and_sent_again() {
    let (a, b, b_id, a_id) = pair_review("again").await;
    let (_, r) = a
        .chat
        .send_with(b_id, "review", None, "요청", ask(true))
        .await
        .unwrap();
    common::eventually(|| has(&b, &r.uuid)).await;
    let id = msg(&b, &r.uuid).id;
    // 요청자(A)가 오래 꺼져 있다 — 시험용 포기 2초.
    b.chat.set_retry(1, 1, 2);
    let conn = common::db(&b);
    let was: String = conn
        .query_row("SELECT addr FROM peers WHERE id = ?1", [a_id], |r| r.get(0))
        .unwrap();
    let closed = std::net::TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap();
    conn.execute("UPDATE peers SET addr = ?1 WHERE id = ?2", (closed.to_string(), a_id))
        .unwrap();
    b.chat.set_req_state(id, ReqState::Accepted, "확인", None).await.unwrap();
    within(15, || msg(&b, &r.uuid).refused.as_deref() == Some(an_agent::chat::UNREACHABLE)).await;
    assert_eq!(msg(&b, &r.uuid).req_state, Some(ReqState::Accepted), "이 PC 값은 남는다");
    assert_eq!(msg(&a, &r.uuid).req_state, Some(ReqState::Wait));
    // 다시 알리기 — 같은 상태 · 메모를 다시 보낸다(그 밖 옮김 규칙은 그대로).
    conn.execute("UPDATE peers SET addr = ?1 WHERE id = ?2", (&was, a_id)).unwrap();
    b.chat.set_retry(1, 1, 600);
    let (m, _) = b.chat.set_req_state(id, ReqState::Accepted, "확인", None).await.unwrap();
    assert_eq!(m.refused, None, "다시 알리면 표시를 지운다");
    within(10, || msg(&a, &r.uuid).req_state == Some(ReqState::Accepted)).await;
    assert!(invalid(b.chat.set_req_state(id, ReqState::Accepted, "", None).await.unwrap_err()), "알린 뒤 같은 상태는 다시 안 된다");
    // 스키마 14 가 옛 완료 알림을 바꾼 프레임 꼴도 그대로 읽힌다.
    let legacy: Request =
        serde_json::from_str(r#"{"type":"req_state","conv":"c","msg":"m","state":"done","note":null}"#).unwrap();
    assert!(matches!(legacy, Request::ReqState { ref state, note: None, .. } if state == "done"));
    // 같은 상태 · 메모를 다시 받으면(응답 유실 재전송) 시각 · 알림을 다시 만들지 않는다(R2).
    let asks = |s: &common::Side| {
        s.events
            .lock()
            .unwrap()
            .iter()
            .filter(|e| matches!(e, PeerEvent::Message { kind: MessageKind::AskState, .. }))
            .count()
    };
    let (before_at, before_n) = (msg(&a, &r.uuid).req_at, asks(&a));
    let pa = b.node.peers().unwrap().remove(0);
    let conv_uuid = chat::by_id(&common::db(&a), msg(&a, &r.uuid).conversation_id).unwrap().unwrap().uuid;
    let same = Request::ReqState {
        conv: conv_uuid.clone(),
        msg: r.uuid.clone(),
        state: "accepted".into(),
        note: Some("확인".into()),
    };
    tokio::time::sleep(Duration::from_millis(1100)).await; // 시각이 바뀌면 보이게
    let (resp, _) = b.node.send(a.addr, Some(pa.spki.clone()), &same).await.unwrap();
    assert_eq!(resp, Response::Ok);
    assert_eq!((msg(&a, &r.uuid).req_at, asks(&a)), (before_at, before_n), "같은 상태 재전송은 그대로");
    // 요청자가 거절하면(그 글이 없다 — `unknown`) 그 글에 거절 표시 · 경고, 새로 고친 뒤에도 같은 상태로 다시 알린다(R2).
    common::db(&a)
        .execute("UPDATE messages SET uuid = 'x' || uuid WHERE uuid = ?1", [&r.uuid])
        .unwrap();
    let (m, warning) = b.chat.set_req_state(id, ReqState::Done, "반영", None).await.unwrap();
    assert!(warning.is_some(), "거절은 경고");
    assert_eq!((m.req_state, m.refused.as_deref()), (Some(ReqState::Done), Some("unknown")));
    common::db(&a)
        .execute("UPDATE messages SET uuid = substr(uuid, 2) WHERE uuid = ?1", [format!("x{}", r.uuid)])
        .unwrap();
    let (m, warning) = b.chat.set_req_state(id, ReqState::Done, "반영", None).await.unwrap();
    assert_eq!((m.refused, warning), (None, None), "다시 알리기 — 표시를 지운다");
    within(10, || msg(&a, &r.uuid).req_state == Some(ReqState::Done)).await;
}

/// 내 요청 상태 바뀜(#1q) — 받은 쪽이 수용 · 완료하면 요청자 내 활동에 그 요청이 맨 위 · 안 본 것(안 본 수 +1). 같은 상태 재전송은 기록하지 않고,
/// 본 뒤에는 0, 다시 바뀌면 다시 1.
#[tokio::test(flavor = "multi_thread")]
async fn activity_my_request_state_unseen() {
    let (a, b, b_id, _) = pair_review("act").await;
    let (conv, r) = a
        .chat
        .send_with(b_id, "review", None, "QA 서버 올려 주세요", ask(true))
        .await
        .unwrap();
    let (_, r2, _) = a.chat.post_with(conv.id, "둘째 요청", true, ask(true)).await.unwrap();
    common::eventually(|| has(&b, &r.uuid) && has(&b, &r2.uuid)).await;
    let mine = |s: &common::Side| {
        chat::activity(&common::db(s), None, chat::ActivityFilter::MyRequest, None, 50)
            .unwrap()
            .into_iter()
            .map(|x| (x.message.unwrap().uuid, x.unseen))
            .collect::<Vec<_>>()
    };
    let unseen = |s: &common::Side| chat::activity_unseen(&common::db(s), None).unwrap();
    let seen = |s: &common::Side| {
        let db = common::db(s);
        chat::mark_activity_seen(&db, None, chat::activity_upto(&db).unwrap()).unwrap();
    };
    // 막 보낸 요청은 안 본 것이 아니다.
    assert_eq!(unseen(&a), 0);
    assert!(mine(&a).iter().all(|(_, u)| !u));
    tokio::time::sleep(Duration::from_millis(1100)).await; // 상태 바뀐 시각이 보낸 시각보다 뒤가 되게
    b.chat
        .set_req_state(msg(&b, &r.uuid).id, ReqState::Accepted, "오늘", None)
        .await
        .unwrap();
    common::eventually(|| unseen(&a) == 1).await;
    assert_eq!(mine(&a)[0], (r.uuid.clone(), true), "상태가 바뀐 요청이 맨 위 · 안 본 것");
    assert_eq!(mine(&a)[1], (r2.uuid.clone(), false));
    // 같은 상태 · 메모 재전송 — 기록 없음.
    let pa = b.node.peers().unwrap().remove(0);
    let again = Request::ReqState {
        conv: conv.uuid.clone(),
        msg: r.uuid.clone(),
        state: "accepted".into(),
        note: Some("오늘".into()),
    };
    seen(&a);
    let (resp, _) = b.node.send(a.addr, Some(pa.spki.clone()), &again).await.unwrap();
    assert_eq!(resp, Response::Ok);
    assert_eq!(unseen(&a), 0, "재전송은 다시 알리지 않는다");
    b.chat
        .set_req_state(msg(&b, &r.uuid).id, ReqState::Done, "", None)
        .await
        .unwrap();
    common::eventually(|| unseen(&a) == 1).await;
    seen(&a);
    assert_eq!(unseen(&a), 0);
    assert!(mine(&a).iter().all(|(_, u)| !u));
}

/// 상태 저장과 내 활동 기록은 한 트랜잭션(#1q 적대적 평가 R1) — 기록이 실패하면 상태도 바뀌지 않고(`retry`), 보낸 쪽이 다시 보내면 둘 다 남는다.
#[tokio::test(flavor = "multi_thread")]
async fn activity_my_request_atomic_with_state() {
    let (a, b, b_id, _) = pair_review("act-tx").await;
    b.chat.set_retry(1, 1, 600);
    let (_, r) = a
        .chat
        .send_with(b_id, "review", None, "원자성 요청", ask(true))
        .await
        .unwrap();
    common::eventually(|| has(&b, &r.uuid)).await;
    // A 의 `req_state` 기록만 실패하게.
    common::db(&a)
        .execute_batch(
            "CREATE TRIGGER fail_req_state BEFORE INSERT ON relay_events WHEN NEW.kind = 'req_state'
             BEGIN SELECT RAISE(ABORT, '주입 실패'); END;",
        )
        .unwrap();
    b.chat
        .set_req_state(msg(&b, &r.uuid).id, ReqState::Accepted, "확인", None)
        .await
        .unwrap();
    tokio::time::sleep(Duration::from_millis(1500)).await;
    assert_eq!(msg(&a, &r.uuid).req_state, Some(ReqState::Wait), "기록이 실패하면 상태도 그대로");
    common::db(&a).execute_batch("DROP TRIGGER fail_req_state;").unwrap();
    within(10, || msg(&a, &r.uuid).req_state == Some(ReqState::Accepted)).await;
    assert_eq!(chat::activity_unseen(&common::db(&a), None).unwrap(), 1, "다시 보낸 상태가 안 본 것으로");
}
