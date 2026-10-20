//! 세션 생성(#1x) — 글 없이 이 PC 에 대화만 연다: 상대에는 첫 글이 갈 때 생기고, 자동 왕복 설정 대화는 첫 글부터 왕복한다.
mod common;
use an_agent::chat::Compose;
use an_core::agents;
use an_core::chat::{self, Author, State};
use std::time::Duration;

/// A · B 페어링 — A 에 내 에이전트 `mine`, B 에 상대 에이전트 `review`. (A, B, A 쪽 B 의 id)
async fn two(tag: &str) -> (common::Side, common::Side, i64) {
    let a = common::side(&format!("{tag}-a"), Duration::ZERO).await;
    let b = common::side(&format!("{tag}-b"), Duration::ZERO).await;
    let (b_id, _) = common::pair(&a, &b).await;
    agents::add(&common::db(&a), "mine", ".", "").unwrap();
    agents::add(&common::db(&b), "review", ".", "").unwrap();
    (a, b, b_id)
}

fn on_b(b: &common::Side, uuid: &str) -> Option<chat::Conversation> {
    chat::list(&common::db(b))
        .unwrap()
        .into_iter()
        .find(|c| c.uuid == uuid)
}

#[tokio::test(flavor = "multi_thread")]
async fn session_open_empty() {
    let (a, b, b_id) = two("so-empty").await;
    let opts = Compose {
        title: Some("주제"),
        ..Compose::default()
    };
    let c = a.chat.open_session(b_id, "review", None, opts).unwrap();
    assert!(chat::messages(&common::db(&a), c.id).unwrap().is_empty(), "글 없음");
    assert!(c.relay.is_none());
    let title = chat::summaries(&common::db(&a))
        .unwrap()
        .into_iter()
        .find(|s| s.conversation.id == c.id)
        .unwrap()
        .title;
    assert_eq!(title, "주제");
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert!(on_b(&b, &c.uuid).is_none(), "상대에는 아직 없다");
    // 틀린 값 — 빈 에이전트 · 상한 · 없는 내 에이전트 · 없는 상대.
    let bad = |e: std::io::Error| e.kind();
    assert_eq!(
        bad(a.chat.open_session(b_id, " ", None, Compose::default()).unwrap_err()),
        std::io::ErrorKind::InvalidInput
    );
    assert_eq!(
        bad(a.chat.open_session(b_id, "review", Some(("mine", 51)), Compose::default()).unwrap_err()),
        std::io::ErrorKind::InvalidInput
    );
    assert_eq!(
        bad(a.chat.open_session(b_id, "review", Some(("없음", 3)), Compose::default()).unwrap_err()),
        std::io::ErrorKind::InvalidInput
    );
    assert!(a.chat.open_session(9999, "review", None, Compose::default()).is_err());
}

#[tokio::test(flavor = "multi_thread")]
async fn session_first_post_reaches_peer() {
    let (a, b, b_id) = two("so-first").await;
    let c = a.chat.open_session(b_id, "review", None, Compose::default()).unwrap();
    a.chat.post(c.id, "첫 질문", true).await.unwrap();
    common::eventually(|| {
        chat::messages(&common::db(&a), c.id)
            .unwrap()
            .iter()
            .any(|m| m.author == Author::Agent && m.state == State::Done && m.text == "답: 첫 질문")
    })
    .await;
    let at_b = on_b(&b, &c.uuid).expect("첫 글로 상대에 대화가 생긴다");
    assert_eq!(at_b.agent, "review");
}

#[tokio::test(flavor = "multi_thread")]
async fn session_relay_starts_on_first_post() {
    let (a, b, b_id) = two("so-relay").await;
    let c = a
        .chat
        .open_session(b_id, "review", Some(("mine", 2)), Compose::default())
        .unwrap();
    assert!(c.relay.as_ref().is_some_and(|r| r.on && r.cap == 2));
    a.chat.post(c.id, "시작", true).await.unwrap();
    // 사람 → B 답 → A 답 → B 답 → A 답(상한) — 내 에이전트가 이어 답했다.
    common::eventually(|| !a.runs.lock().unwrap().is_empty() && b.runs.lock().unwrap().len() >= 2).await;
    assert!(
        chat::by_id(&common::db(&a), c.id).unwrap().unwrap().relay.unwrap().count >= 1,
        "자동 왕복이 돌았다"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn session_relay_request_refused() {
    let (a, _b, b_id) = two("so-req").await;
    let c = a
        .chat
        .open_session(b_id, "review", Some(("mine", 2)), Compose::default())
        .unwrap();
    let opts = Compose {
        request: true,
        ..Compose::default()
    };
    let e = a.chat.post_with(c.id, "요청", true, opts).await.unwrap_err();
    assert_eq!(e.kind(), std::io::ErrorKind::InvalidInput, "{e}");
    assert!(chat::messages(&common::db(&a), c.id).unwrap().is_empty(), "글이 남지 않는다");
    // 글 있는 대화에 사람이 쓰면 지금처럼 자동 왕복을 끈다 — 다음 첫 글 규칙은 빈 대화에만.
}
