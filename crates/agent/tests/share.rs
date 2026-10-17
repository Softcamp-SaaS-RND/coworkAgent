//! 공유 대화(#1o) — 세 PC 통합: A(대화를 가진 PC) · B(원 상대) · C(관람자). 이력 전체 · 이후 글 · 편집 · 삭제가 C 사본에 따라가고, 꺼진 C 는
//! 보관했다 보내며, 빼기 · 나가기 뒤에는 가지 않고 사본은 읽기 전용으로 남는다. 사본은 실행 · 내 활동에 끼지 않고, 위조 · 상한 · 구버전은 거절.
mod common;
use an_core::chat::{self, Author, Conversation, Dir, Message, State};
use an_core::{agents, share};
use an_net::identity::Identity;
use an_net::node::{MessageKind, PeerEvent};
use an_net::proto::{Request, Response, ShareMsg};
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::TcpListener;

struct Three {
    a: common::Side,
    b: common::Side,
    c: common::Side,
    /// A 쪽 B · C 의 id.
    b_id: i64,
    c_id: i64,
    /// B · C 쪽 A 의 id.
    a_at_b: i64,
    a_at_c: i64,
}

async fn three(tag: &str) -> Three {
    let a = common::side(&format!("{tag}-a"), Duration::ZERO).await;
    let b = common::side(&format!("{tag}-b"), Duration::ZERO).await;
    let c = common::side(&format!("{tag}-c"), Duration::ZERO).await;
    let (b_id, a_at_b) = common::pair(&a, &b).await;
    let (c_id, a_at_c) = common::pair(&a, &c).await;
    agents::add(&common::db(&a), "mine", ".", "").unwrap();
    agents::add(&common::db(&b), "review", ".", "").unwrap();
    Three {
        a,
        b,
        c,
        b_id,
        c_id,
        a_at_b,
        a_at_c,
    }
}

/// 그 uuid 의 C 사본.
fn copy_of(s: &common::Side, uuid: &str) -> Option<Conversation> {
    chat::list(&common::db(s))
        .unwrap()
        .into_iter()
        .find(|c| c.uuid == uuid && c.dir == Dir::Shared)
}

fn texts(s: &common::Side, conv: i64) -> Vec<String> {
    chat::messages(&common::db(s), conv)
        .unwrap()
        .into_iter()
        .map(|m| m.text)
        .collect()
}

fn copy_texts(s: &common::Side, uuid: &str) -> Vec<String> {
    copy_of(s, uuid).map_or_else(Vec::new, |c| texts(s, c.id))
}

fn answered(s: &common::Side, conv: i64, n: usize) -> bool {
    chat::messages(&common::db(s), conv)
        .unwrap()
        .iter()
        .filter(|m| m.author == Author::Agent && m.state == State::Done)
        .count()
        >= n
}

/// A 가 B 의 review 에 묻고 답을 받은 대화 + 실행 없는 메모 하나.
async fn asked(t: &Three, q: &str) -> (Conversation, Message) {
    let (conv, _) = t.a.chat.send(t.b_id, "review", None, q).await.unwrap();
    common::eventually(|| answered(&t.a, conv.id, 1)).await;
    let (_, memo, _) = t.a.chat.post(conv.id, "메모", false).await.unwrap();
    common::eventually(|| {
        chat::message_by_id(&common::db(&t.a), memo.id)
            .unwrap()
            .is_some_and(|m| m.state == State::Done)
    })
    .await;
    (conv, memo)
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
async fn history_and_followups_reach_viewer() {
    let t = three("sh-hist").await;
    let (conv, memo) = asked(&t, "첫 질문").await;
    t.a.chat.share_add(conv.id, t.c_id).await.unwrap();
    // 이력 전체 — 사람 글 · 답 · 메모, A 대화 순서 그대로.
    common::eventually(|| copy_texts(&t.c, &conv.uuid).len() == 3).await;
    let copy = copy_of(&t.c, &conv.uuid).unwrap();
    assert_eq!((copy.peer_id, copy.agent.as_str()), (t.a_at_c, "review"));
    assert_eq!(
        copy_texts(&t.c, &conv.uuid),
        texts(&t.a, conv.id),
        "A 와 같은 글 · 같은 순서"
    );
    let info = share::copy(&common::db(&t.c), copy.id).unwrap().unwrap();
    assert_eq!(
        (info.other.as_str(), info.owner_dir.as_str(), info.ended),
        ("user-sh-hist-b", "out", false)
    );
    // 사본 글은 실행 · 요청 · 내 글이 아니다. A 쪽 글(사람 글 · 메모)만 `owner`.
    let ms = chat::messages(&common::db(&t.c), copy.id).unwrap();
    assert!(ms.iter().all(|m| !m.mine && !m.exec && !m.request));
    assert_eq!(
        ms.iter()
            .map(|m| (m.author, m.owner, m.state))
            .collect::<Vec<_>>(),
        vec![
            (Author::Human, true, State::Done),
            (Author::Agent, false, State::Done),
            (Author::Human, true, State::Done),
        ]
    );

    // 이후 글 · 답이 따라간다.
    t.a.chat
        .send(t.b_id, "review", Some(conv.id), "둘째")
        .await
        .unwrap();
    common::eventually(|| copy_texts(&t.c, &conv.uuid).len() == 5).await;
    assert!(copy_texts(&t.c, &conv.uuid).contains(&"답: 둘째".to_string()));
    // 편집 · 삭제도.
    t.a.chat
        .edit_message(memo.id, "고친 메모", true)
        .await
        .unwrap();
    common::eventually(|| copy_texts(&t.c, &conv.uuid).contains(&"고친 메모".to_string())).await;
    let edited = chat::messages(&common::db(&t.c), copy.id)
        .unwrap()
        .into_iter()
        .find(|m| m.text == "고친 메모")
        .unwrap();
    assert!(edited.edited_at.is_some());
    t.a.chat.delete_messages(&[memo.id], true).await.unwrap();
    common::eventually(|| copy_texts(&t.c, &conv.uuid).len() == 4).await;
    assert_eq!(copy_texts(&t.c, &conv.uuid), texts(&t.a, conv.id));
    // 맨 끝 글을 지운 뒤 새 글 — SQLite 가 같은 id 를 다시 줘도 관람자에게 간다(보낸 커서를 낮춘다).
    let (_, last, _) = t.a.chat.post(conv.id, "끝 글", false).await.unwrap();
    common::eventually(|| copy_texts(&t.c, &conv.uuid).len() == 5).await;
    common::eventually(|| {
        chat::message_by_id(&common::db(&t.a), last.id)
            .unwrap()
            .is_some_and(|m| m.state == State::Done)
    })
    .await;
    t.a.chat.delete_messages(&[last.id], true).await.unwrap();
    common::eventually(|| copy_texts(&t.c, &conv.uuid).len() == 4).await;
    let (_, again, _) = t.a.chat.post(conv.id, "다시 쓴 글", false).await.unwrap();
    assert_eq!(again.id, last.id, "같은 id 를 다시 받는다(시험 전제)");
    common::eventually(|| copy_texts(&t.c, &conv.uuid).contains(&"다시 쓴 글".to_string())).await;

    // 사본에는 실행이 없고 내 활동에 들지 않는다. 쓰기 · 컨텍스트 · 자동 왕복 · 다시 공유는 막힌다(읽기 전용).
    assert!(t.c.runs.lock().unwrap().is_empty(), "C 에서 실행 없음");
    let cdb = common::db(&t.c);
    assert!(
        chat::activity(&cdb, None, chat::ActivityFilter::All, None, 50)
            .unwrap()
            .is_empty()
    );
    assert_eq!(chat::activity_unseen(&cdb, None).unwrap(), 0);
    assert!(chat::pending_conversations(&cdb).unwrap().is_empty());
    let ro = std::io::ErrorKind::InvalidInput;
    assert_eq!(
        t.c.chat
            .post(copy.id, "끼어들기", false)
            .await
            .unwrap_err()
            .kind(),
        ro
    );
    assert_eq!(
        t.c.chat
            .post(copy.id, "실행", true)
            .await
            .unwrap_err()
            .kind(),
        ro
    );
    assert_eq!(t.c.chat.context(copy.id, "clear").unwrap_err().kind(), ro);
    assert_eq!(t.c.chat.relay(copy.id, false).await.unwrap_err().kind(), ro);
    assert_eq!(
        t.c.chat
            .send(t.a_at_c, "review", Some(copy.id), "CLI")
            .await
            .unwrap_err()
            .kind(),
        ro
    );
    assert_eq!(
        t.c.chat
            .share_add(copy.id, t.a_at_c)
            .await
            .unwrap_err()
            .kind(),
        ro
    );
    assert_eq!(texts(&t.a, conv.id).len(), 5, "A 대화에 끼어들지 않았다");
}

#[tokio::test(flavor = "multi_thread")]
async fn kept_while_viewer_offline() {
    let t = three("sh-off").await;
    t.a.chat.set_retry(1, 1, 600);
    let (conv, _) = asked(&t, "질문").await;
    t.a.chat.share_add(conv.id, t.c_id).await.unwrap();
    common::eventually(|| copy_texts(&t.c, &conv.uuid).len() == 3).await;
    let was = readdr(&t.a, t.c_id, &closed_port());
    let (_, note, _) = t.a.chat.post(conv.id, "꺼진 동안", false).await.unwrap();
    // 꺼진 관람자는 원 상대로 가는 글을 막지 않는다.
    common::eventually(|| {
        chat::message_by_id(&common::db(&t.a), note.id)
            .unwrap()
            .is_some_and(|m| m.state == State::Done)
    })
    .await;
    tokio::time::sleep(Duration::from_millis(1500)).await;
    assert_eq!(copy_texts(&t.c, &conv.uuid).len(), 3, "꺼진 동안 안 닿음");
    readdr(&t.a, t.c_id, &was);
    common::eventually(|| copy_texts(&t.c, &conv.uuid).contains(&"꺼진 동안".to_string())).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn removed_or_left_viewer_keeps_read_only_copy() {
    let t = three("sh-end").await;
    let (conv, _) = asked(&t, "질문").await;
    t.a.chat.share_add(conv.id, t.c_id).await.unwrap();
    common::eventually(|| copy_texts(&t.c, &conv.uuid).len() == 3).await;
    let ended = |s: &common::Side| {
        let c = copy_of(s, &conv.uuid).unwrap();
        share::copy(&common::db(s), c.id).unwrap().unwrap().ended
    };

    // 빼기 — 사본은 남고 "빠짐", 이후 글은 가지 않는다.
    t.a.chat.share_remove(conv.id, t.c_id).unwrap();
    common::eventually(|| ended(&t.c)).await;
    t.a.chat.post(conv.id, "뺀 뒤", false).await.unwrap();
    common::eventually(|| texts(&t.b, chat::list(&common::db(&t.b)).unwrap()[0].id).len() == 4)
        .await;
    tokio::time::sleep(Duration::from_millis(500)).await;
    assert_eq!(copy_texts(&t.c, &conv.uuid).len(), 3, "뺀 뒤 글은 안 감");
    assert!(t.a.chat.share_remove(conv.id, t.c_id).is_err(), "이미 뺐다");

    // 다시 초대 — 빠짐이 풀리고 그새 쓴 글까지 다시 받는다.
    t.a.chat.share_add(conv.id, t.c_id).await.unwrap();
    common::eventually(|| !ended(&t.c) && copy_texts(&t.c, &conv.uuid).len() == 4).await;

    // 나가기 — C 사본은 남고, A 관람자에서 빠지고, 이후 글은 가지 않는다.
    let copy = copy_of(&t.c, &conv.uuid).unwrap();
    t.c.chat.share_leave(copy.id).unwrap();
    assert!(ended(&t.c));
    common::eventually(|| {
        share::watchers(&common::db(&t.a), conv.id)
            .unwrap()
            .is_empty()
    })
    .await;
    t.a.chat.post(conv.id, "나간 뒤", false).await.unwrap();
    tokio::time::sleep(Duration::from_millis(500)).await;
    assert_eq!(copy_texts(&t.c, &conv.uuid).len(), 4, "나간 뒤 글은 안 감");
    assert!(
        t.c.chat.share_leave(copy.id).is_ok(),
        "이미 나갔다 — 그대로"
    );
    assert!(t.c.chat.post(copy.id, "쓰기", false).await.is_err());
}

#[tokio::test(flavor = "multi_thread")]
async fn forged_share_rejected() {
    let t = three("sh-forge").await;
    // C 가 A 에게 연 C 자기 대화 — A 는 이 uuid 를 사본으로 덮거나 글을 넣지 못한다.
    let (own, _) =
        t.c.chat
            .send(t.a_at_c, "mine", None, "C 질문")
            .await
            .unwrap();
    common::eventually(|| answered(&t.c, own.id, 1)).await;
    let before = texts(&t.c, own.id);
    let pin = Some(t.c.node.identity().spki().to_vec());
    let send = |req: Request| {
        let (node, addr, pin) = (t.a.node.clone(), t.c.addr, pin.clone());
        async move { node.send(addr, pin, &req).await.unwrap().0 }
    };
    let msg = |uuid: &str| ShareMsg {
        uuid: uuid.into(),
        author: "human".into(),
        text: "위조".into(),
        lc: Some(1),
        at: 1,
        owner: true,
        via: None,
        agent: None,
        ok: None,
        edited: false,
    };
    let head = Request::Share {
        conv: own.uuid.clone(),
        title: "위조".into(),
        agent: "x".into(),
        owner_dir: "out".into(),
        other: "x".into(),
    };
    assert_eq!(send(head).await, Response::error("exists"));
    let fresh = an_agent::chat::new_uuid();
    assert_eq!(
        send(Request::SharePost {
            conv: own.uuid.clone(),
            msg: msg(&fresh)
        })
        .await,
        Response::error("unknown")
    );
    let first = chat::messages(&common::db(&t.c), own.id).unwrap()[0]
        .uuid
        .clone();
    assert_eq!(
        send(Request::ShareDelete {
            conv: own.uuid.clone(),
            msg: first.clone()
        })
        .await,
        Response::error("unknown")
    );
    assert_eq!(texts(&t.c, own.id), before, "C 자기 대화 그대로");
    assert_eq!(
        chat::find(&common::db(&t.c), &own.uuid, t.a_at_c)
            .unwrap()
            .unwrap()
            .dir,
        Dir::Out
    );

    // 사본에 C 자기 글 uuid 를 넣지 못하고(`exists`), 일반 대화 프레임으로 사본에 쓰지 못한다(`unknown`).
    let (conv, _) = asked(&t, "질문").await;
    t.a.chat.share_add(conv.id, t.c_id).await.unwrap();
    common::eventually(|| copy_texts(&t.c, &conv.uuid).len() == 3).await;
    assert_eq!(
        send(Request::SharePost {
            conv: conv.uuid.clone(),
            msg: msg(&first)
        })
        .await,
        Response::error("exists")
    );
    let post = Request::Post {
        conv: conv.uuid.clone(),
        msg: an_agent::chat::new_uuid(),
        author: "human".into(),
        text: "일반 글".into(),
        exec: false,
        reply_to: None,
        ok: None,
        turns: None,
        duration_ms: None,
        lc: None,
        via: None,
        model: None,
        effort: None,
        request: false,
        ctx: None,
    };
    assert_eq!(send(post).await, Response::error("unknown"));
    assert_eq!(copy_texts(&t.c, &conv.uuid).len(), 3);
    // id 꼴이 아니면 처리기 전에 거절.
    assert_eq!(
        send(Request::SharePost {
            conv: conv.uuid.clone(),
            msg: msg("x")
        })
        .await,
        Response::error("invalid")
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn cap_five_and_bad_targets() {
    let t = three("sh-cap").await;
    let (conv, _) = asked(&t, "질문").await;
    // 다섯 자리가 차 있으면(관람자 행 — 상대 없이) 거절, 네트워크 전에.
    for p in 1000..1005 {
        assert!(share::add(&common::db(&t.a), conv.id, p).unwrap());
    }
    let full = t.a.chat.share_add(conv.id, t.c_id).await.unwrap_err();
    assert_eq!(full.kind(), std::io::ErrorKind::InvalidInput);
    assert!(full.to_string().contains("5명"), "{full}");
    for p in 1000..1005 {
        assert!(share::remove(&common::db(&t.a), conv.id, p).unwrap());
    }
    // 원 상대 · 없는 상대는 관람자가 아니다.
    assert_eq!(
        t.a.chat
            .share_add(conv.id, t.b_id)
            .await
            .unwrap_err()
            .kind(),
        std::io::ErrorKind::InvalidInput
    );
    assert!(t.a.chat.share_add(conv.id, 9999).await.is_err());
    // 넣으면 한 번만.
    t.a.chat.share_add(conv.id, t.c_id).await.unwrap();
    assert_eq!(
        t.a.chat
            .share_add(conv.id, t.c_id)
            .await
            .unwrap_err()
            .kind(),
        std::io::ErrorKind::AlreadyExists
    );
    assert_eq!(
        share::watchers(&common::db(&t.a), conv.id).unwrap().len(),
        1
    );
}

/// 공유 대화를 모르는 상대(0.5.0) 대역 — `id` 로 TLS 를 받고 Ping 에는 `share` 없는 Pong, 그 밖 프레임은 읽고 끊는다(모르는 종류).
async fn old_peer(id: Identity) -> SocketAddr {
    let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
    let addr = listener.local_addr().unwrap();
    let acceptor = tokio_rustls::TlsAcceptor::from(an_net::tls::server_config(&id));
    tokio::spawn(async move {
        loop {
            let Ok((tcp, _)) = listener.accept().await else {
                return;
            };
            let acceptor = acceptor.clone();
            tokio::spawn(async move {
                let Ok(mut s) = acceptor.accept(tcp).await else {
                    return;
                };
                let Ok(req) = an_net::proto::read_frame::<_, serde_json::Value>(&mut s).await
                else {
                    return;
                };
                if req["type"] == "ping" {
                    let pong = serde_json::json!({"type": "pong", "focus": false, "ctx": true});
                    let _ = an_net::proto::write_frame(&mut s, &pong).await;
                }
            });
        }
    });
    addr
}

#[tokio::test(flavor = "multi_thread")]
async fn old_viewer_rejected() {
    let t = three("sh-old").await;
    let (conv, _) = asked(&t, "질문").await;
    let old = old_peer(Identity::load_or_create(&t.c.paths.identity_file()).unwrap()).await;
    let real = readdr(&t.a, t.c_id, &old.to_string());
    // 초대 때 묻는다 — 모르면 거절(화면 "상대 앱 업데이트 필요").
    let refused = t.a.chat.share_add(conv.id, t.c_id).await.unwrap_err();
    assert_eq!(refused.kind(), std::io::ErrorKind::Unsupported, "{refused}");
    assert!(
        share::watchers(&common::db(&t.a), conv.id)
            .unwrap()
            .is_empty()
    );
    // 보내는 줄에서도 — 넣은 뒤 구버전으로 바뀐 관람자는 빼고 행을 버린다.
    readdr(&t.a, t.c_id, &real);
    t.a.chat.share_add(conv.id, t.c_id).await.unwrap();
    common::eventually(|| copy_texts(&t.c, &conv.uuid).len() == 3).await;
    readdr(&t.a, t.c_id, &old.to_string());
    t.a.chat.post(conv.id, "구버전에", false).await.unwrap();
    common::eventually(|| {
        share::watchers(&common::db(&t.a), conv.id)
            .unwrap()
            .is_empty()
    })
    .await;
    let left: i64 = common::db(&t.a)
        .query_row(
            "SELECT count(*) FROM outbox WHERE peer_id = ?1 AND kind = 'share'",
            [t.c_id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(left, 0, "관람자 앞 공유 행을 버렸다");
}

#[tokio::test(flavor = "multi_thread")]
async fn notice_tells_origin_watcher_count() {
    let t = three("sh-notice").await;
    let shares = |s: &common::Side| -> Vec<String> {
        s.events
            .lock()
            .unwrap()
            .iter()
            .filter_map(|e| match e {
                PeerEvent::Message {
                    kind: MessageKind::Share,
                    preview,
                    ..
                } => Some(preview.clone()),
                _ => None,
            })
            .collect()
    };
    // agent 대화 — B 가 A 에이전트에게 연 대화(A 쪽 `in`)를 A 가 공유한다. B 는 승인 없이 관람자 수(이름 없음)와 알림 한 번.
    let (b_conv, _) =
        t.b.chat
            .send(t.a_at_b, "mine", None, "B 질문")
            .await
            .unwrap();
    common::eventually(|| answered(&t.b, b_conv.id, 1)).await;
    let a_conv = chat::find(&common::db(&t.a), &b_conv.uuid, t.b_id)
        .unwrap()
        .unwrap();
    assert_eq!(a_conv.dir, Dir::In);
    t.a.chat.share_add(a_conv.id, t.c_id).await.unwrap();
    common::eventually(|| share::watched(&common::db(&t.b), b_conv.id).unwrap() == 1).await;
    assert_eq!(shares(&t.b), vec!["관람자 1".to_string()]);
    common::eventually(|| copy_texts(&t.c, &b_conv.uuid).len() == 2).await;
    let copy = copy_of(&t.c, &b_conv.uuid).unwrap();
    assert_eq!(
        share::copy(&common::db(&t.c), copy.id)
            .unwrap()
            .unwrap()
            .owner_dir,
        "in"
    );
    // 빼면 0 — 알림 한 번 더.
    t.a.chat.share_remove(a_conv.id, t.c_id).unwrap();
    common::eventually(|| share::watched(&common::db(&t.b), b_conv.id).unwrap() == 0).await;
    assert_eq!(shares(&t.b).len(), 2);

    // 내 대화(A 쪽 `out`)도 같은 알림(plan 해석 16).
    let (conv, _) = asked(&t, "A 질문").await;
    t.a.chat.share_add(conv.id, t.c_id).await.unwrap();
    let b_out = chat::find(&common::db(&t.b), &conv.uuid, t.a_at_b)
        .unwrap()
        .unwrap();
    common::eventually(|| share::watched(&common::db(&t.b), b_out.id).unwrap() == 1).await;
    assert_eq!(shares(&t.b).len(), 3);
    assert!(
        shares(&t.b).iter().all(|p| !p.contains("sh-notice-c")),
        "관람자 이름은 가지 않는다"
    );
}

/// 빼기 때 보내던 중(`inflight`)이던 공유 글 — 뒤에 다시 보낼 차례가 와도 관람자에게 가지 않는다(적대적 평가 라운드 1).
#[tokio::test(flavor = "multi_thread")]
async fn removed_mid_flight_not_delivered() {
    let t = three("sh-flight").await;
    t.a.chat.set_retry(1, 1, 600);
    let (conv, _) = asked(&t, "질문").await;
    t.a.chat.share_add(conv.id, t.c_id).await.unwrap();
    common::eventually(|| copy_texts(&t.c, &conv.uuid).len() == 3).await;
    let was = readdr(&t.a, t.c_id, &closed_port());
    t.a.chat.post(conv.id, "보내던 중", false).await.unwrap();
    let share_rows = || {
        common::db(&t.a)
            .query_row(
                &format!("SELECT count(*) FROM outbox WHERE peer_id = {} AND kind = '{}' AND frame LIKE '%share_post%'", t.c_id, an_core::outbox::SHARE),
                (),
                |r| r.get::<_, i64>(0),
            )
            .unwrap()
    };
    common::eventually(|| share_rows() >= 1).await;
    // 보내던 중으로 만든 뒤 빼고, 다시 보낼 차례로 되살린다(보내기 실패 뒤 `backoff` 와 같다).
    common::db(&t.a)
        .execute(
            "UPDATE outbox SET state = 'inflight' WHERE peer_id = ?1",
            [t.c_id],
        )
        .unwrap();
    t.a.chat.share_remove(conv.id, t.c_id).unwrap();
    common::db(&t.a)
        .execute(
            "UPDATE outbox SET state = 'pending', next_at = 0 WHERE peer_id = ?1",
            [t.c_id],
        )
        .unwrap();
    readdr(&t.a, t.c_id, &was);
    let copy = copy_of(&t.c, &conv.uuid).unwrap();
    common::eventually(|| {
        share::copy(&common::db(&t.c), copy.id)
            .unwrap()
            .unwrap()
            .ended
    })
    .await;
    common::eventually(|| share_rows() == 0).await;
    assert!(
        !copy_texts(&t.c, &conv.uuid).contains(&"보내던 중".to_string()),
        "뺀 뒤에는 보내던 글도 가지 않는다"
    );
}

/// 관람자에 한도만큼 닿지 않으면 관람을 끝낸다 — 공유 행이 끝없이 쌓이지 않는다(적대적 평가 라운드 1).
#[tokio::test(flavor = "multi_thread")]
async fn unreachable_viewer_dropped_after_give_up() {
    let t = three("sh-giveup").await;
    let (conv, _) = asked(&t, "질문").await;
    t.a.chat.share_add(conv.id, t.c_id).await.unwrap();
    common::eventually(|| copy_texts(&t.c, &conv.uuid).len() == 3).await;
    t.a.chat.set_retry(1, 1, 2);
    let was = readdr(&t.a, t.c_id, &closed_port());
    t.a.chat.post(conv.id, "꺼진 동안 1", false).await.unwrap();
    common::eventually(|| {
        share::watchers(&common::db(&t.a), conv.id)
            .unwrap()
            .is_empty()
    })
    .await;
    let left: i64 = common::db(&t.a)
        .query_row(
            &format!(
                "SELECT count(*) FROM outbox WHERE peer_id = {} AND kind = '{}'",
                t.c_id,
                an_core::outbox::SHARE
            ),
            (),
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(left, 1, "포기한 관람자의 공유 글은 지우고 빼기 표시 한 행만");
    // 빠진 뒤 쓴 글은 줄에 쌓이지 않는다.
    t.a.chat.post(conv.id, "꺼진 동안 2", false).await.unwrap();
    tokio::time::sleep(Duration::from_millis(500)).await;
    let left: i64 = common::db(&t.a)
        .query_row(
            &format!(
                "SELECT count(*) FROM outbox WHERE peer_id = {} AND kind = '{}'",
                t.c_id,
                an_core::outbox::SHARE
            ),
            (),
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(left, 1, "빼기 표시 한 행만 남는다");
    // 돌아온 관람자 — 사본은 "공유 해제됨" 이 되고 꺼진 동안 글은 받지 않는다(라운드 2).
    readdr(&t.a, t.c_id, &was);
    let copy = copy_of(&t.c, &conv.uuid).unwrap();
    common::eventually(|| {
        share::copy(&common::db(&t.c), copy.id)
            .unwrap()
            .unwrap()
            .ended
    })
    .await;
    assert_eq!(
        copy_texts(&t.c, &conv.uuid).len(),
        3,
        "꺼진 동안 글은 가지 않는다"
    );
}
