//! 컨텍스트 압축 · 비우기(#1m) — 묻는 쪽 요청 → 상대 처리 → 양쪽 구분선, 요약 주입, 비우기, 자동 압축, 꺼진 상대 보관, 자동 왕복 연동.
mod common;
use an_agent::chat::Run;
use an_agent::runner::{RunInput, RunOutput, SUMMARY_PROMPT, Session};
use an_core::chat::{self, Author, Message, State};
use an_core::{agents, peers, settings};
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// 실행기 대역 — 요약 지시면 "요약: {세션}" , 아니면 "답: {본문}". 창 사용량은 `ctx` 값.
fn ctx_run(runs: Arc<Mutex<Vec<RunInput>>>, ctx: Arc<AtomicI64>) -> Run {
    Arc::new(move |input: RunInput| {
        // 창 사용량 -1 = 요약 실행 실패(라운드 1 — 상대 쪽 실패 시험).
        let fail = input.text == SUMMARY_PROMPT && ctx.load(Ordering::SeqCst) < 0;
        runs.lock().unwrap().push(input.clone());
        let session = match &input.session {
            Session::New(id) | Session::Resume(id) => id.clone(),
        };
        let text = if input.text == SUMMARY_PROMPT {
            format!("요약: {session}")
        } else {
            format!("답: {}", common::body(&input.text))
        };
        Ok(RunOutput {
            text,
            session,
            is_error: fail,
            turns: Some(1),
            duration_ms: Some(10),
            ctx_tokens: Some(ctx.load(Ordering::SeqCst)),
        })
    })
}

struct Two {
    a: common::Side,
    b: common::Side,
    b_id: i64,
    a_ctx: Arc<AtomicI64>,
    b_ctx: Arc<AtomicI64>,
}

async fn two(tag: &str) -> Two {
    let a_ctx = Arc::new(AtomicI64::new(50_000));
    let b_ctx = Arc::new(AtomicI64::new(50_000));
    let (ac, bc) = (a_ctx.clone(), b_ctx.clone());
    let a = common::side_with(&format!("{tag}-a"), move |r| ctx_run(r, ac)).await;
    let b = common::side_with(&format!("{tag}-b"), move |r| ctx_run(r, bc)).await;
    let (b_id, _) = common::pair(&a, &b).await;
    agents::add(&common::db(&a), "mine", ".", "").unwrap();
    agents::add(&common::db(&b), "review", ".", "").unwrap();
    Two {
        a,
        b,
        b_id,
        a_ctx,
        b_ctx,
    }
}

fn markers(s: &common::Side, conv: i64) -> Vec<Message> {
    chat::messages(&common::db(s), conv)
        .unwrap()
        .into_iter()
        .filter(|m| m.author == Author::Context)
        .collect()
}

fn b_conv(b: &common::Side) -> chat::Conversation {
    chat::list(&common::db(b)).unwrap().remove(0)
}

fn answered(s: &common::Side, conv: i64, n: usize) -> bool {
    chat::messages(&common::db(s), conv)
        .unwrap()
        .iter()
        .filter(|m| m.author == Author::Agent && m.state == State::Done)
        .count()
        >= n
}

fn last_run(s: &common::Side) -> RunInput {
    s.runs.lock().unwrap().last().unwrap().clone()
}

#[tokio::test(flavor = "multi_thread")]
async fn compact_from_asker_runs_on_peer_and_seeds_next_session() {
    let t = two("cx-compact").await;
    let (conv, _) =
        t.a.chat
            .send(t.b_id, "review", None, "첫 질문")
            .await
            .unwrap();
    common::eventually(|| answered(&t.a, conv.id, 1)).await;
    // 답의 창 사용량이 묻는 쪽에도 남는다.
    let reply = chat::messages(&common::db(&t.a), conv.id)
        .unwrap()
        .pop()
        .unwrap();
    assert_eq!(reply.ctx_tokens, Some(50_000));
    let first = match last_run(&t.b).session {
        Session::New(id) => id,
        s => panic!("{s:?}"),
    };

    let m = t.a.chat.context(conv.id, "compact").unwrap();
    assert_eq!((m.state, m.exec, m.mine), (State::Sent, false, true));
    assert!(
        t.a.chat.context(conv.id, "clear").is_err(),
        "진행 중이면 거절"
    );
    common::eventually(|| markers(&t.a, conv.id)[0].state == State::Done).await;
    let mine = &markers(&t.a, conv.id)[0];
    assert_eq!(mine.text, format!("요약: {first}"));
    assert_eq!(mine.ctx_op.as_deref(), Some("compact"));
    // 받은 쪽 — 그 세션을 이어 요약하고 세션을 비웠다. 같은 uuid 의 구분선, 상대가 누른 글.
    let run = last_run(&t.b);
    assert_eq!(
        (run.text.as_str(), run.session),
        (SUMMARY_PROMPT, Session::Resume(first.clone()))
    );
    let theirs = &markers(&t.b, b_conv(&t.b).id)[0];
    assert_eq!(
        (theirs.uuid.as_str(), theirs.mine, theirs.state),
        (mine.uuid.as_str(), false, State::Done)
    );
    assert_eq!(b_conv(&t.b).session, None);

    // 다음 글 — 새 세션, 첫 입력 앞에 요약. 한 번 쓴 요약은 지운다.
    t.a.chat
        .send(t.b_id, "review", Some(conv.id), "둘째")
        .await
        .unwrap();
    common::eventually(|| answered(&t.a, conv.id, 2)).await;
    let run = last_run(&t.b);
    assert!(
        matches!(run.session, Session::New(ref id) if *id != first),
        "{:?}",
        run.session
    );
    assert!(
        run.text
            .contains(&format!("<대화 요약>\n요약: {first}\n</대화 요약>")),
        "{}",
        run.text
    );
    assert_eq!(
        chat::ctx_seed(&common::db(&t.b), b_conv(&t.b).id, "").unwrap(),
        None
    );
    // 구분선은 메모 · 안 읽음 · 미리보기에 들지 않는다.
    t.a.chat
        .send(t.b_id, "review", Some(conv.id), "셋째")
        .await
        .unwrap();
    common::eventually(|| answered(&t.a, conv.id, 3)).await;
    assert!(!last_run(&t.b).text.contains("<대화 요약>"));
}

#[tokio::test(flavor = "multi_thread")]
async fn clear_drops_sessions_without_summary() {
    let t = two("cx-clear").await;
    let (conv, _) =
        t.a.chat
            .send(t.b_id, "review", None, "첫 질문")
            .await
            .unwrap();
    common::eventually(|| answered(&t.a, conv.id, 1)).await;
    let before = t.b.runs.lock().unwrap().len();
    t.a.chat.context(conv.id, "clear").unwrap();
    common::eventually(|| markers(&t.a, conv.id)[0].state == State::Done).await;
    assert_eq!(markers(&t.a, conv.id)[0].text, "");
    assert_eq!(t.b.runs.lock().unwrap().len(), before, "비우기는 실행 없음");
    assert_eq!(b_conv(&t.b).session, None);
    t.a.chat
        .send(t.b_id, "review", Some(conv.id), "둘째")
        .await
        .unwrap();
    common::eventually(|| answered(&t.a, conv.id, 2)).await;
    let run = last_run(&t.b);
    assert!(matches!(run.session, Session::New(_)));
    assert!(!run.text.contains("<대화 요약>"));
    assert!(t.a.chat.context(conv.id, "nope").is_err());
}

#[tokio::test(flavor = "multi_thread")]
async fn auto_compact_over_80_percent_unless_off() {
    let t = two("cx-auto").await;
    t.b_ctx.store(settings::AUTO_COMPACT_AT, Ordering::SeqCst);
    let (conv, _) =
        t.a.chat
            .send(t.b_id, "review", None, "큰 질문")
            .await
            .unwrap();
    let bc = || b_conv(&t.b).id;
    common::eventually(|| {
        markers(&t.b, bc())
            .first()
            .is_some_and(|m| m.state == State::Done)
    })
    .await;
    let m = &markers(&t.b, bc())[0];
    assert_eq!((m.ctx_op.as_deref(), m.mine), (Some("auto_compact"), true));
    assert_eq!(last_run(&t.b).text, SUMMARY_PROMPT);
    assert!(
        markers(&t.a, conv.id).is_empty(),
        "보낸 대화에는 넘기지 않는다(자동 왕복 아님)"
    );

    // 끄면 넘어도 압축하지 않는다.
    settings::set(&common::db(&t.b), settings::AUTO_COMPACT_KEY, "0").unwrap();
    t.a.chat
        .send(t.b_id, "review", Some(conv.id), "또")
        .await
        .unwrap();
    common::eventually(|| answered(&t.a, conv.id, 2)).await;
    tokio::time::sleep(Duration::from_millis(500)).await;
    assert_eq!(markers(&t.b, bc()).len(), 1);
    // 문턱 아래는 압축하지 않는다.
    settings::set(&common::db(&t.b), settings::AUTO_COMPACT_KEY, "1").unwrap();
    t.b_ctx
        .store(settings::AUTO_COMPACT_AT - 1, Ordering::SeqCst);
    t.a.chat
        .send(t.b_id, "review", Some(conv.id), "작게")
        .await
        .unwrap();
    common::eventually(|| answered(&t.a, conv.id, 3)).await;
    tokio::time::sleep(Duration::from_millis(500)).await;
    assert_eq!(markers(&t.b, bc()).len(), 1);
}

#[tokio::test(flavor = "multi_thread")]
async fn request_kept_while_peer_offline() {
    let t = two("cx-offline").await;
    t.a.chat.set_retry(1, 1, 3600);
    let (conv, _) =
        t.a.chat
            .send(t.b_id, "review", None, "첫 질문")
            .await
            .unwrap();
    common::eventually(|| answered(&t.a, conv.id, 1)).await;
    // 상대 주소를 닿지 않는 곳으로 — 요청은 보관된다.
    let real = peers::by_id(&common::db(&t.a), t.b_id)
        .unwrap()
        .unwrap()
        .addr;
    let dead = std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap();
    peers::set_addr(&common::db(&t.a), t.b_id, &dead.to_string()).unwrap();
    t.a.chat.context(conv.id, "compact").unwrap();
    tokio::time::sleep(Duration::from_millis(1500)).await;
    assert_eq!(markers(&t.a, conv.id)[0].state, State::Sent);
    assert!(markers(&t.b, b_conv(&t.b).id).is_empty());
    // 돌아오면 줄이 보낸다.
    peers::set_addr(&common::db(&t.a), t.b_id, &real).unwrap();
    common::eventually(|| markers(&t.a, conv.id)[0].state == State::Done).await;
    assert_eq!(markers(&t.b, b_conv(&t.b).id).len(), 1);
}

#[tokio::test(flavor = "multi_thread")]
async fn relay_compacts_both_sessions_once() {
    let t = two("cx-relay").await;
    let (conv, _) =
        t.a.chat
            .start_relay(t.b_id, "review", "mine", 1, "시작", None)
            .await
            .unwrap();
    common::eventually(|| {
        t.a.runs.lock().unwrap().len() == 1 && !t.b.runs.lock().unwrap().is_empty()
    })
    .await;
    tokio::time::sleep(Duration::from_millis(800)).await;
    let (a_runs, b_runs) = (
        t.a.runs.lock().unwrap().len(),
        t.b.runs.lock().unwrap().len(),
    );
    // 연 쪽 자동 압축 — 내 세션을 압축하고 상대에도 보낸다(받은 쪽은 다시 넘기지 않는다).
    t.a_ctx.store(0, Ordering::SeqCst);
    let m = t.a.chat.context(conv.id, "compact").unwrap();
    assert!(m.exec && m.mine);
    common::eventually(|| {
        markers(&t.a, conv.id)[0].state == State::Done
            && markers(&t.b, b_conv(&t.b).id)
                .first()
                .is_some_and(|m| m.state == State::Done)
    })
    .await;
    assert_eq!(t.a.runs.lock().unwrap().len(), a_runs + 1);
    assert_eq!(t.b.runs.lock().unwrap().len(), b_runs + 1);
    assert_eq!(last_run(&t.a).text, SUMMARY_PROMPT);
    assert_eq!(last_run(&t.b).text, SUMMARY_PROMPT);
    tokio::time::sleep(Duration::from_millis(500)).await;
    assert_eq!(markers(&t.a, conv.id).len(), 1, "되넘김 없음");
    assert_eq!(markers(&t.b, b_conv(&t.b).id).len(), 1);
    assert_eq!(
        chat::by_id(&common::db(&t.a), conv.id)
            .unwrap()
            .unwrap()
            .session,
        None
    );
    assert_eq!(b_conv(&t.b).session, None);
}

#[test]
fn old_pong_has_no_context() {
    let old: an_net::proto::Response =
        serde_json::from_str(r#"{"type":"pong","focus":true}"#).unwrap();
    assert_eq!(
        old,
        an_net::proto::Response::Pong {
            focus: true,
            ctx: false,
            share: false,
            leave: false,
            clear: false
        }
    );
    // 0.4.0 이 보낸 답 — 크기 없음.
    let reply: an_net::proto::Request = serde_json::from_str(
        r#"{"type":"reply","conv":"c","msg":"m","reply_to":"r","text":"t","ok":true}"#,
    )
    .unwrap();
    assert!(matches!(
        reply,
        an_net::proto::Request::Reply { ctx: None, .. }
    ));
}

#[tokio::test(flavor = "multi_thread")]
async fn seed_dropped_when_agent_folder_changes() {
    let t = two("cx-seed").await;
    let (conv, _) =
        t.a.chat
            .send(t.b_id, "review", None, "첫 질문")
            .await
            .unwrap();
    common::eventually(|| answered(&t.a, conv.id, 1)).await;
    t.a.chat.context(conv.id, "compact").unwrap();
    common::eventually(|| markers(&t.a, conv.id)[0].state == State::Done).await;
    let bc = b_conv(&t.b).id;
    assert!(chat::ctx_seed(&common::db(&t.b), bc, "").unwrap().is_some());
    // 폴더를 바꾸면(새 세대) 옛 요약은 새 세션에 들어가지 않는다.
    let id = agents::by_name(&common::db(&t.b), "review")
        .unwrap()
        .unwrap()
        .id;
    let other = std::env::temp_dir();
    agents::update(&common::db(&t.b), id, other.to_str().unwrap(), "", None, false, false).unwrap();
    assert_eq!(chat::ctx_seed(&common::db(&t.b), bc, "").unwrap(), None);
    t.a.chat
        .send(t.b_id, "review", Some(conv.id), "둘째")
        .await
        .unwrap();
    common::eventually(|| answered(&t.a, conv.id, 2)).await;
    assert!(!last_run(&t.b).text.contains("<대화 요약>"));
}

#[tokio::test(flavor = "multi_thread")]
async fn relay_peer_failure_not_hidden() {
    let t = two("cx-partial").await;
    let (conv, _) =
        t.a.chat
            .start_relay(t.b_id, "review", "mine", 1, "시작", None)
            .await
            .unwrap();
    common::eventually(|| {
        t.a.runs.lock().unwrap().len() == 1 && !t.b.runs.lock().unwrap().is_empty()
    })
    .await;
    tokio::time::sleep(Duration::from_millis(800)).await;
    t.b_ctx.store(-1, Ordering::SeqCst); // 상대 요약 실패
    t.a.chat.context(conv.id, "compact").unwrap();
    common::eventually(|| {
        let m = &markers(&t.a, conv.id)[0];
        m.state == State::Done && m.refused.as_deref() == Some("peer_compact_failed")
    })
    .await;
    let theirs = &markers(&t.b, b_conv(&t.b).id)[0];
    assert_eq!(
        (theirs.state, theirs.refused.as_deref()),
        (State::Error, Some("compact_failed"))
    );
    assert!(b_conv(&t.b).session.is_some(), "실패한 쪽 세션은 그대로");
}

#[tokio::test(flavor = "multi_thread")]
async fn done_kept_while_requester_away_past_give_up() {
    let t = two("cx-away").await;
    let (conv, _) =
        t.a.chat
            .send(t.b_id, "review", None, "첫 질문")
            .await
            .unwrap();
    common::eventually(|| answered(&t.a, conv.id, 1)).await;
    let bc = b_conv(&t.b).id;
    // 받은 쪽(B)에서 요청자(A)가 포기 한도(1초)보다 오래 닿지 않는다 — 처리 결과 줄은 포기하지 않는다(라운드 2).
    t.b.chat.set_retry(1, 1, 1);
    let a_on_b = peers::list(&common::db(&t.b)).unwrap().remove(0);
    let dead = std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap();
    peers::set_addr(&common::db(&t.b), a_on_b.id, &dead.to_string()).unwrap();
    chat::add_context(
        &common::db(&t.b),
        bc,
        &an_agent::chat::new_uuid(),
        "clear",
        State::Queued,
        false,
        true,
    )
    .unwrap();
    t.b.chat.recover().unwrap();
    common::eventually(|| markers(&t.b, bc)[0].state == State::Done).await;
    tokio::time::sleep(Duration::from_secs(9)).await; // Windows 는 닫힌 포트 연결 실패에 약 2초 — 포기 한도를 넘길 만큼 여러 번
    let head = an_core::outbox::head(&common::db(&t.b), a_on_b.id).unwrap();
    assert_eq!(
        head.map(|i| (i.kind, i.state != "failed")),
        Some(("context_done".to_string(), true)),
        "포기 한도를 넘어도 줄에 남는다"
    );
    // 돌아오면 보낸다.
    peers::set_addr(&common::db(&t.b), a_on_b.id, &a_on_b.addr).unwrap();
    common::eventually(|| {
        an_core::outbox::queued_for(&common::db(&t.b), a_on_b.id)
            .unwrap()
            .is_empty()
    })
    .await;
}

#[tokio::test(flavor = "multi_thread")]
async fn manual_waits_for_runs_and_only_auto_jumps_queue() {
    let t = two("cx-order").await;
    let (conv, _) =
        t.a.chat
            .send(t.b_id, "review", None, "첫 질문")
            .await
            .unwrap();
    common::eventually(|| answered(&t.a, conv.id, 1)).await;
    let bc = b_conv(&t.b).id;
    // 깨우지 않고 실행 대기 글을 하나 둔다.
    let waiting = chat::add_human(
        &common::db(&t.b),
        bc,
        &an_agent::chat::new_uuid(),
        "대기",
        State::Queued,
        false,
        true,
    )
    .unwrap();
    assert!(
        t.b.chat.context(bc, "clear").is_err(),
        "실행 대기 글이 있으면 수동은 거절"
    );
    let manual = chat::add_context(
        &common::db(&t.b),
        bc,
        &an_agent::chat::new_uuid(),
        "clear",
        State::Queued,
        false,
        true,
    )
    .unwrap();
    let auto = chat::add_context(
        &common::db(&t.b),
        bc,
        &an_agent::chat::new_uuid(),
        "auto_compact",
        State::Queued,
        true,
        true,
    )
    .unwrap();
    let order: Vec<i64> = chat::pending(&common::db(&t.b), bc)
        .unwrap()
        .iter()
        .map(|m| m.id)
        .collect();
    assert_eq!(
        order,
        vec![auto.id, waiting.id, manual.id],
        "자동 압축만 앞자리, 나머지는 id 순"
    );
}

/// 압축이 앞 세션을 요약하는 동안 다른 에이전트의 상위 읽기(#05-2)를 바꾸면 그 에이전트 세션은 비었다 — 처음 읽어 둔 옛 세션을 새 세대로 이어
/// 요약하지 않는다(#05-2 적대적 평가 R1).
#[tokio::test(flavor = "multi_thread")]
async fn compact_skips_session_cleared_while_waiting() {
    let tag = "cx-toggle";
    let b_db = an_core::paths::Paths::under(std::env::temp_dir().join(format!("an-agent-{tag}-b-{}", std::process::id()))).db_file();
    let a_ctx = Arc::new(AtomicI64::new(50_000));
    let b_ctx = Arc::new(AtomicI64::new(50_000));
    let a = common::side_with(&format!("{tag}-a"), move |r| ctx_run(r, a_ctx)).await;
    let b = common::side_with(&format!("{tag}-b"), move |r| -> Run {
        let inner = ctx_run(r, b_ctx);
        Arc::new(move |input: RunInput| {
            if input.text == SUMMARY_PROMPT && matches!(&input.session, Session::Resume(s) if s != "s-lint") {
                let db = an_core::db::open(&b_db).unwrap();
                let lint = agents::by_name(&db, "lint").unwrap().unwrap();
                agents::update(&db, lint.id, &lint.folder, &lint.prompt, None, true, false).unwrap();
            }
            inner(input)
        })
    })
    .await;
    let (b_id, _) = common::pair(&a, &b).await;
    agents::add(&common::db(&b), "review", ".", "").unwrap();
    agents::add(&common::db(&b), "lint", ".", "").unwrap();
    let (conv, _) = a.chat.send(b_id, "review", None, "첫 질문").await.unwrap();
    common::eventually(|| answered(&a, conv.id, 1)).await;
    let bc = b_conv(&b).id;
    chat::set_agent_session(&common::db(&b), bc, "lint", "s-lint").unwrap();
    a.chat.context(conv.id, "compact").unwrap();
    common::eventually(|| markers(&a, conv.id)[0].state == State::Done).await;
    let resumed = b.runs.lock().unwrap().iter().any(|r| matches!(&r.session, Session::Resume(s) if s == "s-lint"));
    assert!(!resumed, "비운 세션을 이어 요약했다");
    let db = common::db(&b);
    assert!(chat::ctx_seed(&db, bc, "").unwrap().is_some(), "대화 에이전트 요약은 남는다");
    assert_eq!(chat::ctx_seed(&db, bc, "lint").unwrap(), None);
}
