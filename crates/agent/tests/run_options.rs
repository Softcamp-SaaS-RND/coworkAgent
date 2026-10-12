mod common;
use an_agent::runner::{Session, claude_args};
use an_core::chat::{self, State};
use an_core::{agents, peers, run_opts, settings};
use std::time::Duration;

#[test]
fn claude_args_carry_model_effort() {
    let agent = agents::Agent {
        id: 1,
        name: "review".into(),
        folder: "/work".into(),
        tools: "Read".into(),
        prompt: String::new(),
        group: None,
        uid: String::new(),
    };
    let args = claude_args(&agent, &Session::New("u".into()), "opus", "max");
    let at = |f: &str| args.iter().position(|a| a == f).expect(f);
    assert_eq!(args[at("--model") + 1], "opus");
    assert_eq!(args[at("--effort") + 1], "max");
    // 가드 · 도구 규칙 뒤 끝에 붙는다.
    assert!(at("--model") > at("--append-system-prompt") && at("--effort") > at("--model"));
}

/// 그 상대 요청의 실행 — 상대별 덮어쓰기 → 전역 설정 → 기본값(sonnet · high).
#[tokio::test(flavor = "multi_thread")]
async fn peer_override_then_default() {
    let (a, b) = (
        common::side("ro-a", Duration::ZERO).await,
        common::side("ro-b", Duration::ZERO).await,
    );
    let (b_id, a_id) = common::pair(&a, &b).await;
    agents::add(&common::db(&b), "review", ".", None, "").unwrap();
    let ask = |n: usize| {
        let (a, b) = (&a, &b);
        async move {
            a.chat.send(b_id, "review", None, "질문").await.unwrap();
            common::eventually(|| b.runs.lock().unwrap().len() == n).await;
            let r = b.runs.lock().unwrap()[n - 1].clone();
            (r.model, r.effort)
        }
    };
    assert_eq!(ask(1).await, ("sonnet".into(), "high".into()), "기본값");

    let b_db = common::db(&b);
    settings::set(&b_db, run_opts::MODEL_KEY, "opus").unwrap();
    settings::set(&b_db, run_opts::EFFORT_KEY, "low").unwrap();
    assert_eq!(ask(2).await, ("opus".into(), "low".into()), "전역 설정");

    peers::set_meta(&b_db, a_id, None, Some("haiku"), None).unwrap();
    assert_eq!(
        ask(3).await,
        ("haiku".into(), "low".into()),
        "상대 model 만 덮어쓰기"
    );

    peers::set_meta(&b_db, a_id, None, Some("haiku"), Some("max")).unwrap();
    assert_eq!(ask(4).await, ("haiku".into(), "max".into()), "상대 둘 다");

    // 저장된 값이 틀리면(인자로 읽힐 값 포함) 쓰지 않고 그 아래 단계로.
    peers::set_meta(
        &b_db,
        a_id,
        None,
        Some("--dangerously-skip-permissions"),
        Some("turbo"),
    )
    .unwrap();
    settings::set(&b_db, run_opts::MODEL_KEY, "-x").unwrap();
    assert_eq!(
        ask(5).await,
        ("sonnet".into(), "low".into()),
        "틀린 값은 건너뛴다"
    );
}

/// 설정을 못 읽으면(DB 오류) 기본값으로 돌리지 않고 실행 실패로 답한다.
#[tokio::test(flavor = "multi_thread")]
async fn settings_read_error_fails_run() {
    let (a, b) = (
        common::side("re-a", Duration::ZERO).await,
        common::side("re-b", Duration::ZERO).await,
    );
    let (b_id, _) = common::pair(&a, &b).await;
    let b_db = common::db(&b);
    agents::add(&b_db, "review", ".", None, "").unwrap();
    b_db.execute_batch("DROP TABLE settings").unwrap();
    let (conv, _) = a.chat.send(b_id, "review", None, "질문").await.unwrap();
    let a_db = common::db(&a);
    common::eventually(|| chat::messages(&a_db, conv.id).unwrap().len() == 2).await;
    let reply = chat::messages(&a_db, conv.id).unwrap().remove(1);
    assert_eq!(reply.state, State::Error);
    assert!(reply.text.contains("settings"), "{}", reply.text);
    assert!(b.runs.lock().unwrap().is_empty(), "실행하지 않는다");
}
