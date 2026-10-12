mod common;
use an_core::agents;
use an_core::chat;
use an_net::proto::Request;
use std::time::Duration;

/// 대화의 글별 `via` — 이력 비교용.
fn vias(conn: &an_core::db::Connection, conv: i64) -> Vec<Option<String>> {
    chat::messages(conn, conv)
        .unwrap()
        .into_iter()
        .map(|m| m.via)
        .collect()
}

#[tokio::test(flavor = "multi_thread")]
async fn ask_via_cli_marks_both_sides() {
    let (a, b) = (
        common::side("cv-a", Duration::ZERO).await,
        common::side("cv-b", Duration::ZERO).await,
    );
    let (b_id, _) = common::pair(&a, &b).await;
    agents::add(&common::db(&b), "review", ".", None, "").unwrap();
    let (conv, asked) = a
        .chat
        .send_via(b_id, "review", None, "질문", Some("cli"), None)
        .await
        .unwrap();
    assert_eq!(asked.via.as_deref(), Some("cli"));
    let a_db = common::db(&a);
    common::eventually(|| chat::messages(&a_db, conv.id).unwrap().len() == 2).await;
    // 답은 CLI 글이 아니다.
    assert_eq!(vias(&a_db, conv.id), [Some("cli".into()), None]);
    let b_db = common::db(&b);
    let bc = chat::list(&b_db).unwrap()[0].id;
    common::eventually(|| chat::messages(&b_db, bc).unwrap().len() == 2).await;
    assert_eq!(
        vias(&b_db, bc),
        [Some("cli".into()), None],
        "답은 CLI 글이 아니다(받은 쪽)"
    );
    // 웹UI 글은 그대로 표시 없음.
    let (_, web) = a
        .chat
        .send(b_id, "review", Some(conv.id), "둘째")
        .await
        .unwrap();
    assert_eq!(web.via, None);
}

#[tokio::test(flavor = "multi_thread")]
async fn say_via_cli_marks_both_sides() {
    let (a, b) = (
        common::side("cs-a", Duration::ZERO).await,
        common::side("cs-b", Duration::ZERO).await,
    );
    let (b_id, _) = common::pair(&a, &b).await;
    agents::add(&common::db(&b), "review", ".", None, "").unwrap();
    let (conv, _) = a.chat.send(b_id, "review", None, "질문").await.unwrap();
    let a_db = common::db(&a);
    common::eventually(|| chat::messages(&a_db, conv.id).unwrap().len() == 2).await;
    let (_, note, _) = a
        .chat
        .post_via(conv.id, "메모", false, Some("cli"))
        .await
        .unwrap();
    assert_eq!((note.exec, note.via.as_deref()), (false, Some("cli")));
    let b_db = common::db(&b);
    let bc = chat::list(&b_db).unwrap()[0].id;
    assert_eq!(vias(&b_db, bc), [None, None, Some("cli".into())]);
    // 받은 쪽에서 CLI 로 쓴 글도 양쪽에.
    b.chat
        .post_via(bc, "답장", false, Some("cli"))
        .await
        .unwrap();
    assert_eq!(vias(&a_db, conv.id)[3].as_deref(), Some("cli"));
    // 받은 쪽이 CLI 로 자기 에이전트에게 — 실행 요청 미러도 양쪽에 CLI 로.
    let (_, ask, _) = b
        .chat
        .post_via(bc, "내 에이전트에", true, Some("cli"))
        .await
        .unwrap();
    assert_eq!((ask.exec, ask.via.as_deref()), (true, Some("cli")));
    let a_ask = chat::messages(&a_db, conv.id).unwrap().remove(4);
    assert_eq!(
        (a_ask.text.as_str(), a_ask.via.as_deref()),
        ("내 에이전트에", Some("cli"))
    );
}

#[test]
fn payload_without_via_is_human() {
    // 이전 버전 상대가 보낸 프레임 — via 가 없다.
    let old = r#"{"type":"post","conv":"c","msg":"m","author":"human","text":"t"}"#;
    let Request::Post { via, .. } = serde_json::from_str::<Request>(old).unwrap() else {
        panic!("post 여야 한다");
    };
    assert_eq!(via, None);
    let old = r#"{"type":"message","conv":"c","msg":"m","agent":"a","text":"t"}"#;
    let Request::Message { via, .. } = serde_json::from_str::<Request>(old).unwrap() else {
        panic!("message 여야 한다");
    };
    assert_eq!(via, None);
    // 새 버전 프레임의 via 는 이전 버전이 모르는 선택 필드 — 이름 그대로 싣는다.
    let new = serde_json::to_string(&Request::Message {
        conv: "c".into(),
        msg: "m".into(),
        agent: "a".into(),
        text: "t".into(),
        lc: None,
        by_agent: None,
        relay: None,
        via: Some("cli".into()),
        request: false,
        mention: None,
    })
    .unwrap();
    assert!(new.contains(r#""via":"cli""#), "{new}");
}
