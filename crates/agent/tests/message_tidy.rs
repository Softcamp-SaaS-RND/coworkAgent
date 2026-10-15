//! 이력 정리(#1f) — 두 PC 통합: 끝난 내 사람 글만 편집 · 삭제, 내 창에서만 · 상대 창에도(꺼져 있으면 나중에), 실행 · 알림 없음, 상대 반영
//! 실패는 내 활동, 위조 거부.
mod common;
use an_agent::chat::{Chat, Compose};
use an_core::agents;
use an_core::chat::{self, ActivityFilter, ActivityKind, AskFilter, Author, Message, ReqState};
use an_net::proto::{Request, Response};
use std::io::ErrorKind;
use std::time::{Duration, Instant};

fn ask(request: bool) -> Compose<'static> {
    Compose {
        request,
        ..Default::default()
    }
}

async fn pair_review(tag: &str, delay: Duration) -> (common::Side, common::Side, i64, i64) {
    let a = common::side(&format!("tidy-{tag}-a"), Duration::ZERO).await;
    let b = common::side(&format!("tidy-{tag}-b"), delay).await;
    let (b_id, a_id) = common::pair(&a, &b).await;
    agents::add(&common::db(&b), "review", ".", "").unwrap();
    (a, b, b_id, a_id)
}

fn msg(s: &common::Side, uuid: &str) -> Option<Message> {
    chat::message_by_uuid(&common::db(s), uuid).unwrap()
}

fn tidy(s: &common::Side, m: &Message) -> bool {
    let now = msg(s, &m.uuid).unwrap();
    Chat::tidy_ok(&common::db(s), &now).unwrap()
}

fn answer_of(s: &common::Side, conv: i64) -> Option<Message> {
    chat::messages(&common::db(s), conv)
        .unwrap()
        .into_iter()
        .find(|m| m.author == Author::Agent)
}

fn invalid(e: std::io::Error) -> bool {
    e.kind() == ErrorKind::InvalidInput
}

/// 조건이 참이 될 때까지(최대 `secs` 초).
async fn within(secs: u64, mut f: impl FnMut() -> bool) {
    let until = Instant::now() + Duration::from_secs(secs);
    while !f() {
        assert!(Instant::now() < until, "{secs}초 안에 조건이 맞지 않았다");
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

/// 상대 주소를 닫힌 포트로(꺼진 PC) — 원래 주소를 돌려준다.
fn unplug(s: &common::Side, peer: i64) -> String {
    let conn = common::db(s);
    let was: String = conn
        .query_row("SELECT addr FROM peers WHERE id = ?1", [peer], |r| r.get(0))
        .unwrap();
    let closed = std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap();
    conn.execute(
        "UPDATE peers SET addr = ?1 WHERE id = ?2",
        (closed.to_string(), peer),
    )
    .unwrap();
    was
}

fn replug(s: &common::Side, peer: i64, addr: &str) {
    common::db(s)
        .execute("UPDATE peers SET addr = ?1 WHERE id = ?2", (addr, peer))
        .unwrap();
}

fn sync_failed(s: &common::Side) -> Vec<String> {
    chat::activity(&common::db(s), None, ActivityFilter::Failed, None, 50)
        .unwrap()
        .into_iter()
        .filter(|a| a.kind == ActivityKind::SyncFailed)
        .filter_map(|a| a.detail)
        .collect()
}

#[tokio::test(flavor = "multi_thread")]
async fn only_finished_own_human_messages() {
    let (a, b, b_id, a_id) = pair_review("only", Duration::from_secs(3)).await;
    // 실행 중 — 답이 오기 전 내 실행 요청은 아직 끝나지 않았다.
    let (conv, q) = a
        .chat
        .send_with(b_id, "review", None, "질의", ask(false))
        .await
        .unwrap();
    assert!(!tidy(&a, &q), "답을 기다리는 글");
    assert!(invalid(
        a.chat.edit_message(q.id, "고침", false).await.unwrap_err()
    ));
    assert!(invalid(
        a.chat.delete_messages(&[q.id], false).await.unwrap_err()
    ));
    common::eventually(|| answer_of(&a, conv.id).is_some()).await;
    assert!(tidy(&a, &q), "답이 온 글");
    // 답 · 상대가 쓴 글은 아니다.
    let answer = answer_of(&a, conv.id).unwrap();
    assert!(!tidy(&a, &answer));
    assert!(invalid(
        a.chat
            .edit_message(answer.id, "고침", false)
            .await
            .unwrap_err()
    ));
    let bc = chat::list(&common::db(&b)).unwrap().remove(0);
    let (_, theirs, _) = b
        .chat
        .post_with(bc.id, "B 의 요청", true, ask(true))
        .await
        .unwrap();
    common::eventually(|| msg(&a, &theirs.uuid).is_some()).await;
    let theirs_on_a = msg(&a, &theirs.uuid).unwrap();
    assert!(!tidy(&a, &theirs_on_a), "상대 글");
    assert!(invalid(
        a.chat
            .delete_messages(&[theirs_on_a.id], false)
            .await
            .unwrap_err()
    ));
    // 보내는 중 — 상대가 꺼져 있어 보관된 글.
    let was = unplug(&a, b_id);
    let (_, held, _) = a
        .chat
        .post_with(conv.id, "보관될 요청", true, ask(true))
        .await
        .unwrap();
    assert!(!tidy(&a, &held), "보내는 중");
    // 하나라도 안 되면 아무것도 지우지 않는다.
    assert!(invalid(
        a.chat
            .delete_messages(&[q.id, held.id], false)
            .await
            .unwrap_err()
    ));
    assert!(msg(&a, &q.uuid).is_some());
    replug(&a, b_id, &was);
    let _ = a_id;
}

#[tokio::test(flavor = "multi_thread")]
async fn delete_mine_only_here() {
    let (a, b, b_id, _) = pair_review("here", Duration::ZERO).await;
    let (conv, q) = a
        .chat
        .send_with(b_id, "review", None, "지울 질의 xyzzy", ask(false))
        .await
        .unwrap();
    common::eventually(|| answer_of(&a, conv.id).is_some()).await;
    let (_, r, _) = a
        .chat
        .post_with(conv.id, "지울 요청 xyzzy", true, ask(true))
        .await
        .unwrap();
    common::eventually(|| msg(&b, &r.uuid).is_some()).await;
    chat::set_saved(&common::db(&a), q.id, true).unwrap();
    let (n, warning) = a.chat.delete_messages(&[q.id, r.id], false).await.unwrap();
    assert_eq!((n, warning), (2, None));
    let db = common::db(&a);
    assert!(msg(&a, &q.uuid).is_none() && msg(&a, &r.uuid).is_none());
    assert!(
        chat::saved(&db, chat::Scope::All, None, 50)
            .unwrap()
            .is_empty(),
        "저장된 항목"
    );
    let found = chat::search(&db, "xyzzy", 50).unwrap().messages;
    assert!(found.iter().all(|m| m.id != q.id && m.id != r.id), "검색");
    let mine = chat::messages_across(
        &db,
        Some(true),
        chat::Scope::All,
        None,
        Some(AskFilter::Request(None)),
        None,
        50,
    )
    .unwrap();
    assert!(mine.is_empty(), "보낸 요청 목록");
    assert!(answer_of(&a, conv.id).is_some(), "답은 남는다");
    // 상대 창은 그대로.
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert!(msg(&b, &q.uuid).is_some() && msg(&b, &r.uuid).is_some());
}

#[tokio::test(flavor = "multi_thread")]
async fn delete_mirrored_leaves_peer_request_list() {
    let (a, b, b_id, _) = pair_review("mirror", Duration::ZERO).await;
    let (conv, q) = a
        .chat
        .send_with(b_id, "review", None, "질의", ask(false))
        .await
        .unwrap();
    common::eventually(|| answer_of(&a, conv.id).is_some()).await;
    let (_, r, _) = a
        .chat
        .post_with(conv.id, "요청", true, ask(true))
        .await
        .unwrap();
    common::eventually(|| msg(&b, &r.uuid).is_some()).await;
    let theirs = |s: &common::Side| {
        chat::messages_across(
            &common::db(s),
            Some(false),
            chat::Scope::All,
            None,
            Some(AskFilter::Request(None)),
            None,
            50,
        )
        .unwrap()
    };
    assert_eq!(theirs(&b).len(), 1, "B 의 받은 요청 목록");
    let (n, warning) = a.chat.delete_messages(&[q.id, r.id], true).await.unwrap();
    assert_eq!((n, warning), (2, None));
    common::eventually(|| msg(&b, &q.uuid).is_none() && msg(&b, &r.uuid).is_none()).await;
    assert!(theirs(&b).is_empty(), "상대 요청 목록에서도 빠진다");
    let bc = chat::list(&common::db(&b)).unwrap().remove(0);
    assert!(answer_of(&b, bc.id).is_some(), "상대 쪽 답도 남는다");
}

#[tokio::test(flavor = "multi_thread")]
async fn edit_mirrored_keeps_order_and_state() {
    let (a, b, b_id, _) = pair_review("edit", Duration::ZERO).await;
    let (conv, r) = a
        .chat
        .send_with(b_id, "review", None, "로그 형식 바꿔 주세요", ask(true))
        .await
        .unwrap();
    common::eventually(|| msg(&b, &r.uuid).is_some()).await;
    b.chat
        .set_req_state(
            msg(&b, &r.uuid).unwrap().id,
            ReqState::Accepted,
            "확인",
            None,
        )
        .await
        .unwrap();
    common::eventually(|| msg(&a, &r.uuid).unwrap().req_state == Some(ReqState::Accepted)).await;
    let before = (msg(&a, &r.uuid).unwrap(), msg(&b, &r.uuid).unwrap());
    // 이 PC 에만 — 상대는 그대로.
    let (m, warning) = a
        .chat
        .edit_message(r.id, "로그 형식 바꿔 주세요(JSON)", false)
        .await
        .unwrap();
    assert_eq!(
        (m.text.as_str(), warning),
        ("로그 형식 바꿔 주세요(JSON)", None)
    );
    assert!(m.edited_at.is_some());
    // 상대 창에도.
    let (m, warning) = a
        .chat
        .edit_message(r.id, "로그 형식을 JSON 으로", true)
        .await
        .unwrap();
    assert_eq!(warning, None);
    common::eventually(|| msg(&b, &r.uuid).unwrap().text == "로그 형식을 JSON 으로").await;
    let on_b = msg(&b, &r.uuid).unwrap();
    assert!(on_b.edited_at.is_some());
    for (now, was) in [(&m, &before.0), (&on_b, &before.1)] {
        assert_eq!(
            (now.lc, &now.uuid, now.req_state, &now.req_note),
            (was.lc, &was.uuid, was.req_state, &was.req_note)
        );
    }
    assert_eq!(
        chat::messages(&common::db(&a), conv.id).unwrap().len(),
        1,
        "새 글 없음"
    );
    // 공백만 · 너무 긴 본문은 거절.
    assert!(invalid(
        a.chat.edit_message(r.id, "  ", true).await.unwrap_err()
    ));
}

#[tokio::test(flavor = "multi_thread")]
async fn tidy_runs_and_notifies_nothing() {
    let (a, b, b_id, _) = pair_review("quiet", Duration::ZERO).await;
    let (conv, q) = a
        .chat
        .send_with(b_id, "review", None, "질의", ask(false))
        .await
        .unwrap();
    common::eventually(|| answer_of(&a, conv.id).is_some()).await;
    let (_, q2, _) = a
        .chat
        .post_with(conv.id, "둘째 질의", true, ask(false))
        .await
        .unwrap();
    common::eventually(|| chat::messages(&common::db(&a), conv.id).unwrap().len() == 4).await;
    let counts = |s: &common::Side| (s.runs.lock().unwrap().len(), s.events.lock().unwrap().len());
    let before = (counts(&a), counts(&b));
    a.chat.edit_message(q.id, "고친 질의", true).await.unwrap();
    a.chat.delete_messages(&[q2.id], true).await.unwrap();
    common::eventually(|| {
        msg(&b, &q2.uuid).is_none() && msg(&b, &q.uuid).unwrap().text == "고친 질의"
    })
    .await;
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert_eq!((counts(&a), counts(&b)), before, "실행 · 알림 없음");
    assert_eq!(chat::messages(&common::db(&a), conv.id).unwrap().len(), 3);
}

#[tokio::test(flavor = "multi_thread")]
async fn offline_peer_gets_it_later() {
    let (a, b, b_id, _) = pair_review("later", Duration::ZERO).await;
    a.chat.set_retry(1, 1, 600);
    let (conv, q) = a
        .chat
        .send_with(b_id, "review", None, "질의", ask(false))
        .await
        .unwrap();
    common::eventually(|| answer_of(&a, conv.id).is_some()).await;
    let (_, r, _) = a
        .chat
        .post_with(conv.id, "요청", true, ask(true))
        .await
        .unwrap();
    common::eventually(|| msg(&b, &r.uuid).is_some()).await;
    let was = unplug(&a, b_id);
    let (_, warning) = a
        .chat
        .edit_message(q.id, "꺼진 동안 고침", true)
        .await
        .unwrap();
    assert_eq!(warning, None, "보관 — 경고 없음");
    assert!(
        tidy(&a, &q),
        "반영 대기는 글을 '보내는 중' 으로 만들지 않는다"
    );
    let (_, warning) = a.chat.delete_messages(&[r.id], true).await.unwrap();
    assert_eq!(warning, None);
    tokio::time::sleep(Duration::from_millis(1200)).await;
    assert_eq!(msg(&b, &q.uuid).unwrap().text, "질의");
    assert!(msg(&b, &r.uuid).is_some());
    replug(&a, b_id, &was);
    within(10, || {
        msg(&b, &q.uuid).unwrap().text == "꺼진 동안 고침" && msg(&b, &r.uuid).is_none()
    })
    .await;
    assert!(sync_failed(&a).is_empty());
}

#[tokio::test(flavor = "multi_thread")]
async fn give_up_and_refusal_noted_in_activity() {
    let (a, b, b_id, _) = pair_review("fail", Duration::ZERO).await;
    let (conv, q) = a
        .chat
        .send_with(b_id, "review", None, "질의", ask(false))
        .await
        .unwrap();
    common::eventually(|| answer_of(&a, conv.id).is_some()).await;
    // 거절 — 상대 쪽 글이 이미 없다(상대가 지운 경우와 같다) → 편집은 `unknown`.
    common::db(&b)
        .execute("DELETE FROM messages WHERE uuid = ?1", [&q.uuid])
        .unwrap();
    let (m, warning) = a.chat.edit_message(q.id, "고친 질의", true).await.unwrap();
    assert_eq!(m.text, "고친 질의", "이 PC 에는 반영");
    assert!(warning.is_some_and(|w| w.contains("상대 창에 반영하지 못했다")));
    assert_eq!(sync_failed(&a), vec!["편집 — 고친 질의".to_string()]);
    // 포기 — 상대가 오래 꺼져 있다(시험용 2초).
    let (_, r, _) = a
        .chat
        .post_with(conv.id, "요청", true, ask(true))
        .await
        .unwrap();
    common::eventually(|| msg(&b, &r.uuid).is_some()).await;
    a.chat.set_retry(1, 1, 2);
    let _was = unplug(&a, b_id);
    a.chat.delete_messages(&[r.id], true).await.unwrap();
    within(15, || sync_failed(&a).contains(&"삭제".to_string())).await;
    assert!(msg(&b, &r.uuid).is_some(), "상대에는 남는다");
    let unseen = chat::activity_unseen(&common::db(&a), None).unwrap();
    assert!(unseen >= 2, "안 본 수에 든다: {unseen}");
}

#[tokio::test(flavor = "multi_thread")]
async fn forged_edit_delete_refused() {
    let (a, b, b_id, _) = pair_review("forge", Duration::ZERO).await;
    let c = common::side("tidy-forge-c", Duration::ZERO).await;
    let (a_on_c, _) = common::pair(&c, &a).await;
    agents::add(&common::db(&a), "mine", ".", "").unwrap();
    // A 의 질의(A 글) · 그 답(B 에이전트) · B 가 쓴 글 · C 가 A 에 쓴 글.
    let (conv, q) = a
        .chat
        .send_with(b_id, "review", None, "A 의 질의", ask(false))
        .await
        .unwrap();
    common::eventually(|| answer_of(&a, conv.id).is_some()).await;
    let answer = answer_of(&a, conv.id).unwrap();
    let bc = chat::list(&common::db(&b)).unwrap().remove(0);
    let (_, bm, _) = b
        .chat
        .post_with(bc.id, "B 의 글", true, ask(true))
        .await
        .unwrap();
    common::eventually(|| msg(&a, &bm.uuid).is_some()).await;
    let (cconv, cm) = c
        .chat
        .send_with(a_on_c, "mine", None, "C 의 글", ask(true))
        .await
        .unwrap();
    common::eventually(|| msg(&a, &cm.uuid).is_some()).await;
    let pa = b.node.peers().unwrap().remove(0);
    let send = |req: Request| {
        let (node, addr, spki) = (b.node.clone(), a.addr, pa.spki.clone());
        async move { node.send(addr, Some(spki), &req).await.unwrap().0 }
    };
    let edit = |conv: &str, msg: &str, text: &str| Request::Edit {
        conv: conv.into(),
        msg: msg.into(),
        text: text.into(),
    };
    let del = |conv: &str, msg: &str| Request::Delete {
        conv: conv.into(),
        msg: msg.into(),
    };
    let nowhere = "00000000-0000-4000-8000-000000000000";
    for (req, want) in [
        (
            edit(&conv.uuid, &q.uuid, "위조"),
            Response::error("unknown"),
        ),
        (del(&conv.uuid, &q.uuid), Response::error("unknown")),
        (
            edit(&conv.uuid, &answer.uuid, "위조"),
            Response::error("unknown"),
        ),
        (del(&conv.uuid, &answer.uuid), Response::error("unknown")),
        (
            edit(&cconv.uuid, &cm.uuid, "위조"),
            Response::error("unknown"),
        ),
        (del(&cconv.uuid, &cm.uuid), Response::error("unknown")),
        (edit(nowhere, &bm.uuid, "위조"), Response::error("unknown")),
        (del(nowhere, &bm.uuid), Response::error("unknown")),
        (
            edit(&conv.uuid, &bm.uuid, "  "),
            Response::error("bad_request"),
        ),
    ] {
        assert_eq!(send(req.clone()).await, want, "{req:?}");
    }
    assert_eq!(msg(&a, &q.uuid).unwrap().text, "A 의 질의");
    assert!(msg(&a, &answer.uuid).is_some() && msg(&a, &cm.uuid).is_some());
    assert_eq!(msg(&a, &cm.uuid).unwrap().text, "C 의 글");
    // 실행 대기 · 실행 중인 상대 글은 `busy`.
    common::db(&a)
        .execute(
            "UPDATE messages SET state = 'running' WHERE uuid = ?1",
            [&bm.uuid],
        )
        .unwrap();
    assert_eq!(
        send(edit(&conv.uuid, &bm.uuid, "고침")).await,
        Response::error("busy")
    );
    assert_eq!(
        send(del(&conv.uuid, &bm.uuid)).await,
        Response::error("busy")
    );
    common::db(&a)
        .execute(
            "UPDATE messages SET state = 'done' WHERE uuid = ?1",
            [&bm.uuid],
        )
        .unwrap();
    // 그 상대가 쓴 글이면 적용 — 같은 재전송 · 이미 없는 글 삭제는 `Ok`.
    assert_eq!(
        send(edit(&conv.uuid, &bm.uuid, "B 가 고침")).await,
        Response::Ok
    );
    assert_eq!(
        send(edit(&conv.uuid, &bm.uuid, "B 가 고침")).await,
        Response::Ok
    );
    assert_eq!(msg(&a, &bm.uuid).unwrap().text, "B 가 고침");
    assert_eq!(send(del(&conv.uuid, &bm.uuid)).await, Response::Ok);
    assert!(msg(&a, &bm.uuid).is_none());
    assert_eq!(send(del(&conv.uuid, &bm.uuid)).await, Response::Ok);
}

/// 거절 정산이 원자적이다(적대적 평가 라운드 1) — '상대 반영 실패' 기록이 실패하면 반영 행을 지우지 않고 다시 보내, 기록이 될 때 함께 지운다.
#[tokio::test(flavor = "multi_thread")]
async fn refusal_note_and_row_removal_are_atomic() {
    let (a, b, b_id, _) = pair_review("atomic", Duration::ZERO).await;
    a.chat.set_retry(1, 1, 600);
    let (conv, q) = a
        .chat
        .send_with(b_id, "review", None, "질의", ask(false))
        .await
        .unwrap();
    common::eventually(|| answer_of(&a, conv.id).is_some()).await;
    // 상대 쪽 글이 없다 → 편집은 거절된다. 이 PC 의 기록 쓰기를 막는다(쓰기 실패 주입).
    common::db(&b)
        .execute("DELETE FROM messages WHERE uuid = ?1", [&q.uuid])
        .unwrap();
    common::db(&a)
        .execute_batch(
            "CREATE TRIGGER no_events BEFORE INSERT ON relay_events BEGIN SELECT RAISE(ABORT, '주입'); END;",
        )
        .unwrap();
    let rows = |s: &common::Side| -> i64 {
        common::db(s)
            .query_row(
                "SELECT count(*) FROM outbox WHERE kind = 'msg_edit'",
                [],
                |r| r.get(0),
            )
            .unwrap()
    };
    let (_, warning) = a.chat.edit_message(q.id, "고친 질의", true).await.unwrap();
    assert!(warning.is_some(), "바로 보낸 결과는 거절");
    assert!(sync_failed(&a).is_empty());
    assert_eq!(rows(&a), 1, "기록 못 했으면 행이 남는다");
    // 쓰기가 돌아오면 줄이 다시 보내고 기록 · 행 지우기를 함께.
    common::db(&a)
        .execute_batch("DROP TRIGGER no_events;")
        .unwrap();
    within(10, || {
        sync_failed(&a) == vec!["편집 — 고친 질의".to_string()]
    })
    .await;
    within(5, || rows(&a) == 0).await;
}

/// 완전 삭제(적대적 평가 라운드 2) — 편집 반영 실패 기록(본문 미리보기)은 그 글을 지우면 함께 사라지고, 삭제 반영 실패 기록은 남는다.
#[tokio::test(flavor = "multi_thread")]
async fn deleting_message_removes_its_edit_failure_note() {
    let (a, b, b_id, _) = pair_review("purge", Duration::ZERO).await;
    a.chat.set_retry(1, 1, 600);
    let (conv, q) = a
        .chat
        .send_with(b_id, "review", None, "질의", ask(false))
        .await
        .unwrap();
    common::eventually(|| answer_of(&a, conv.id).is_some()).await;
    common::db(&b)
        .execute("DELETE FROM messages WHERE uuid = ?1", [&q.uuid])
        .unwrap();
    a.chat
        .edit_message(q.id, "비밀 본문 고침", true)
        .await
        .unwrap();
    assert_eq!(sync_failed(&a), vec!["편집 — 비밀 본문 고침".to_string()]);
    // 상대에는 이미 없으니 삭제 반영은 `Ok` — 실패 기록이 생기지 않는다. 이 PC 에서 지우면 편집 실패 기록도 사라진다.
    a.chat.delete_messages(&[q.id], true).await.unwrap();
    within(5, || sync_failed(&a).is_empty()).await;
    let left: i64 = common::db(&a)
        .query_row(
            "SELECT count(*) FROM relay_events WHERE detail LIKE '%비밀 본문%'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(left, 0, "본문 미리보기가 DB 에 남지 않는다");
    // 삭제 반영 실패 기록(본문 없음)은 글이 지워진 뒤에 생겨 남는다.
    let (_, r, _) = a
        .chat
        .post_with(conv.id, "요청", true, ask(true))
        .await
        .unwrap();
    common::eventually(|| msg(&b, &r.uuid).is_some()).await;
    a.chat.set_retry(1, 1, 2);
    let _was = unplug(&a, b_id);
    a.chat.delete_messages(&[r.id], true).await.unwrap();
    within(15, || sync_failed(&a) == vec!["삭제".to_string()]).await;
}

/// 맨 끝 글을 지운 뒤 온 글은 안 읽음 · 안 본 것으로 보인다 — SQLite 가 지운 id 를 다시 주어도 읽음 · 본 표시가 그 id 를 덮지 않는다(개정 3).
#[tokio::test(flavor = "multi_thread")]
async fn reused_id_after_delete_stays_unread() {
    let (a, b, b_id, a_id) = pair_review("reuse", Duration::ZERO).await;
    let (conv, q) = a
        .chat
        .send_with(b_id, "review", None, "질의", ask(false))
        .await
        .unwrap();
    common::eventually(|| answer_of(&a, conv.id).is_some()).await;
    let (_, r, _) = a
        .chat
        .post_with(conv.id, "맨 끝 요청", true, ask(true))
        .await
        .unwrap();
    common::eventually(|| msg(&b, &r.uuid).is_some()).await;
    // A 가 대화 · 내 활동을 다 봤다 — 표시 = 맨 끝 글(내 요청) id.
    let db = common::db(&a);
    db.execute("UPDATE conversations SET read_id = ?1", [r.id])
        .unwrap();
    db.execute("UPDATE peers SET activity_seen_msg = ?1", [r.id])
        .unwrap();
    a.chat.delete_messages(&[r.id], false).await.unwrap();
    // B 가 쓴 글이 A 에 온다 — 지운 id 를 다시 받는다.
    let bc = chat::list(&common::db(&b)).unwrap().remove(0);
    let (_, bm, _) = b
        .chat
        .post_with(bc.id, "B 의 새 요청", true, ask(true))
        .await
        .unwrap();
    common::eventually(|| msg(&a, &bm.uuid).is_some()).await;
    let got = msg(&a, &bm.uuid).unwrap();
    let (read, seen): (i64, i64) = db
        .query_row(
            "SELECT c.read_id, p.activity_seen_msg FROM conversations c, peers p WHERE c.id = ?1 AND p.id = ?2",
            (conv.id, b_id),
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert!(
        got.id > read && got.id > seen,
        "새 글 {} · 읽음 {read} · 본 {seen}",
        got.id
    );
    assert!(
        chat::activity_unseen(&db, None).unwrap() >= 1,
        "안 본 수에 든다"
    );
    let _ = (q, a_id);
}
