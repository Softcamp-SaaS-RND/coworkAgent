//! 받는 쪽 실행 문 · 보낸 쪽 진행(1b A1 · A2 · A3) — 두 PC 통합.
mod common;
use an_agent::chat::{Progress, Run};
use an_agent::runner::{RunInput, RunOutput, Session};
use an_core::agents;
use an_core::chat::{self, Author, State};
use an_net::proto::{Request, Response};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// 실행기 대역 — 동시에 몇 개 도는지 · 시작 순서(본문)를 센다.
#[derive(Default)]
struct Meter {
    now: AtomicUsize,
    max: AtomicUsize,
    order: Mutex<Vec<String>>,
}

/// 실행 시간은 본문으로 정한다(`delay(본문)`).
type Delay = Arc<dyn Fn(&str) -> Duration + Send + Sync>;

fn metered(meter: Arc<Meter>, delay: Delay) -> impl FnOnce(Arc<Mutex<Vec<RunInput>>>) -> Run {
    move |runs| {
        Arc::new(move |input: RunInput| {
            let n = meter.now.fetch_add(1, Ordering::SeqCst) + 1;
            meter.max.fetch_max(n, Ordering::SeqCst);
            let body = common::body(&input.text).to_string();
            meter.order.lock().unwrap().push(body.clone());
            runs.lock().unwrap().push(input.clone());
            std::thread::sleep(delay(&body));
            meter.now.fetch_sub(1, Ordering::SeqCst);
            let session = match &input.session {
                Session::New(id) | Session::Resume(id) => id.clone(),
            };
            Ok(RunOutput {
                text: format!("답: {body}"),
                session,
                is_error: false,
                turns: Some(1),
                duration_ms: Some(10),
                ctx_tokens: None,
            })
        })
    }
}

/// a(묻는 쪽) · b(답하는 쪽, 계량 실행기) · a 쪽 b 의 id.
async fn two(tag: &str, delay: Duration) -> (common::Side, common::Side, Arc<Meter>, i64) {
    two_by(tag, Arc::new(move |_| delay)).await
}

/// [`two`] + 본문마다 다른 실행 시간.
async fn two_by(tag: &str, delay: Delay) -> (common::Side, common::Side, Arc<Meter>, i64) {
    let meter = Arc::new(Meter::default());
    let a = common::side(&format!("{tag}-a"), Duration::ZERO).await;
    let b = common::side_with(&format!("{tag}-b"), metered(meter.clone(), delay)).await;
    let (b_id, _) = common::pair(&a, &b).await;
    agents::add(&common::db(&b), "review", ".", "").unwrap();
    (a, b, meter, b_id)
}

/// a 가 보낸 대화들의 답(성공) 수.
fn answered(a: &common::Side) -> usize {
    let db = common::db(a);
    chat::list(&db)
        .unwrap()
        .iter()
        .filter(|c| {
            chat::messages(&db, c.id)
                .unwrap()
                .iter()
                .any(|m| m.author == Author::Agent && m.state == State::Done)
        })
        .count()
}

#[tokio::test(flavor = "multi_thread")]
async fn five_conversations_respect_cap_in_arrival_order() {
    // 실행 시간을 엇갈려 자리가 하나씩 빈다(q0 3초 · q1 4초 · 나머지 2초) — 기다린 셋의 시작이 1초 넘게 떨어져 시작 기록 순서가 곧 들어간
    // 순서다. 처음 둘은 보내기가 다 끝나기 전에 끝나지 않는다(부하 걸린 PC 에서 보내기가 느려도 상한 2 를 채운다).
    let (a, _b, meter, b_id) = two_by(
        "cap5",
        Arc::new(|body: &str| match body {
            "q0" => Duration::from_secs(3),
            "q1" => Duration::from_secs(4),
            _ => Duration::from_secs(2),
        }),
    )
    .await;
    let asks: Vec<String> = (0..5).map(|i| format!("q{i}")).collect();
    for q in &asks {
        a.chat.send(b_id, "review", None, q).await.unwrap();
    }
    common::eventually(|| answered(&a) == 5).await;
    assert_eq!(meter.max.load(Ordering::SeqCst), 2, "기본 상한 2");
    // 바로 들어간 둘은 함께 시작해 시작 순서가 섞인다 — 기다린 셋은 닿은 순서.
    let mut order = meter.order.lock().unwrap().clone();
    order[..2].sort();
    assert_eq!(order, asks, "닿은 순서");
}

#[tokio::test(flavor = "multi_thread")]
async fn thirty_requests_all_answered_without_refusal() {
    let (a, _b, meter, b_id) = two("cap30", Duration::from_millis(40)).await;
    for i in 0..30 {
        a.chat.send(b_id, "review", None, &format!("q{i}")).await.unwrap();
    }
    let until = std::time::Instant::now() + Duration::from_secs(20);
    while answered(&a) < 30 {
        assert!(std::time::Instant::now() < until, "30건이 20초 안에 다 답하지 않았다: {}", answered(&a));
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    assert!(meter.max.load(Ordering::SeqCst) <= 2);
    // 거절(실패) 글 없음.
    let db = common::db(&a);
    for c in chat::list(&db).unwrap() {
        assert!(chat::messages(&db, c.id).unwrap().iter().all(|m| m.state != State::Error));
    }
}

/// a 가 보낸 대화 `conv` 의 첫 글.
fn first(a: &common::Side, conv: i64) -> chat::Message {
    chat::messages(&common::db(a), conv).unwrap().remove(0)
}

#[tokio::test(flavor = "multi_thread")]
async fn waiting_request_shows_queue_then_running_then_clears() {
    let (a, b, _meter, b_id) = two("prog", Duration::from_secs(2)).await;
    b.chat.set_max_runs(1).unwrap();
    let mut convs = Vec::new();
    for i in 0..3 {
        convs.push(a.chat.send(b_id, "review", None, &format!("q{i}")).await.unwrap().0.id);
    }
    // 셋째는 앞에 한 사람(둘째)이 기다리는 동안 대기.
    common::eventually(|| {
        a.chat.progress_of(&first(&a, convs[2]))
            == Some(Progress {
                state: "queued".into(),
                ahead: 1,
                detail: None,
            })
    })
    .await;
    // 첫째는 바로 실행 — running(대화마다 따로 오는 알림이라 셋째의 queued 보다 늦게 닿을 수 있다).
    common::eventually(|| a.chat.progress_of(&first(&a, convs[0])).is_some_and(|p| p.state == "running")).await;
    // 셋째 차례가 오면 running, 답이 오면 지운다.
    let until = std::time::Instant::now() + Duration::from_secs(15);
    loop {
        let p = a.chat.progress_of(&first(&a, convs[2]));
        if p.as_ref().is_some_and(|p| p.state == "running") {
            break;
        }
        assert!(std::time::Instant::now() < until, "running 이 오지 않았다: {p:?}");
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    common::eventually(|| answered(&a) == 3).await;
    assert_eq!(a.chat.progress_of(&first(&a, convs[2])), None);
    // 받는 쪽 실행 문은 비었다 · 상한은 저장됐다.
    assert_eq!(b.chat.run_queue(), (1, vec![], vec![]));
    assert_eq!(an_agent::chat::max_runs(&common::db(&b)).unwrap(), 1);
}

#[tokio::test(flavor = "multi_thread")]
async fn cap_bounds_and_queue_snapshot() {
    let (a, b, _meter, b_id) = two("snap", Duration::from_secs(4)).await;
    assert!(b.chat.set_max_runs(0).is_err() && b.chat.set_max_runs(9).is_err());
    b.chat.set_max_runs(1).unwrap();
    let gone = agents::add(&common::db(&b), "gone", ".", "").unwrap();
    for i in 0..3 {
        a.chat.send(b_id, "review", None, &format!("q{i}")).await.unwrap();
    }
    let (late, _) = a.chat.send(b_id, "gone", None, "q3").await.unwrap();
    common::eventually(|| {
        let (cap, running, waiting) = b.chat.run_queue();
        cap == 1 && running.len() == 1 && waiting.len() == 3
    })
    .await;
    // 기다리는 동안 지운 에이전트 — 차례가 와도 실행하지 않는다(차례가 온 뒤의 상태로 판단).
    assert!(agents::remove(&common::db(&b), gone.id).unwrap());
    // 상한을 올리면 그 자리에서 기다리던 것이 들어간다.
    b.chat.set_max_runs(3).unwrap();
    common::eventually(|| b.chat.run_queue().1.len() == 3).await;
    common::eventually(|| answered(&a) == 3).await;
    common::eventually(|| {
        chat::messages(&common::db(&a), late.id)
            .unwrap()
            .iter()
            .any(|m| m.author == Author::Agent && m.state == State::Error && m.text.contains("삭제"))
    })
    .await;
}

#[tokio::test(flavor = "multi_thread")]
async fn same_conversation_followup_is_queued_and_listed() {
    let (a, b, _meter, b_id) = two("same", Duration::from_secs(2)).await;
    b.chat.set_max_runs(2).unwrap();
    let (conv, _) = a.chat.send(b_id, "review", None, "q0").await.unwrap();
    let (_, next) = a.chat.send(b_id, "review", Some(conv.id), "q1").await.unwrap();
    // 문에는 자리가 있지만 같은 대화의 앞 실행을 기다린다 — 닿자마자 queued · 받는 쪽 실행 목록의 기다림.
    common::eventually(|| {
        a.chat.progress_of(&first_of(&a, conv.id, &next.uuid))
            == Some(Progress {
                state: "queued".into(),
                ahead: 1,
                detail: None,
            })
    })
    .await;
    let (_, running, waiting) = b.chat.run_queue();
    assert_eq!((running.len(), waiting.len()), (1, 1));
    common::eventually(|| answered_msgs(&a, conv.id) == 2).await;
    assert_eq!(a.chat.progress_of(&first_of(&a, conv.id, &next.uuid)), None);
}

/// a 쪽 대화의 그 글.
fn first_of(a: &common::Side, conv: i64, uuid: &str) -> chat::Message {
    chat::messages(&common::db(a), conv)
        .unwrap()
        .into_iter()
        .find(|m| m.uuid == uuid)
        .unwrap()
}

/// a 쪽 대화의 성공 답 수.
fn answered_msgs(a: &common::Side, conv: i64) -> usize {
    chat::messages(&common::db(a), conv)
        .unwrap()
        .iter()
        .filter(|m| m.author == Author::Agent && m.state == State::Done)
        .count()
}

#[tokio::test(flavor = "multi_thread")]
async fn foreign_progress_refused() {
    let (a, b, _meter, b_id) = two("pfor", Duration::ZERO).await;
    let (conv, sent) = a.chat.send(b_id, "review", None, "q").await.unwrap();
    common::eventually(|| answered(&a) == 1).await;
    let pin = Some(a.node.identity().spki().to_vec());
    let reply = chat::messages(&common::db(&a), conv.id)
        .unwrap()
        .into_iter()
        .find(|m| m.author == Author::Agent)
        .unwrap();
    let progress = |conv: &str, msg: &str, state: &str| Request::Progress {
        conv: conv.into(),
        msg: msg.into(),
        state: state.into(),
        ahead: 0,
        detail: None,
    };
    for (req, want) in [
        // 답이 이미 온 글(sent 아님) · 내 글이 아님(상대 에이전트 답) · 없는 대화.
        (progress(&conv.uuid, &sent.uuid, "running"), "unknown"),
        (progress(&conv.uuid, &reply.uuid, "running"), "unknown"),
        (progress(&an_agent::chat::new_uuid(), &sent.uuid, "running"), "unknown"),
        (progress(&conv.uuid, &sent.uuid, "done"), "bad_request"),
    ] {
        let (resp, _) = b.node.send(a.addr, pin.clone(), &req).await.unwrap();
        assert_eq!(resp, Response::error(want), "{req:?}");
    }
}
