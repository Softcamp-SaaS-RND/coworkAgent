use an_core::{agents, chat, db};

fn conn(tag: &str) -> db::Connection {
    let p = std::env::temp_dir().join(format!("an-core-store-{tag}-{}.db", std::process::id()));
    let _ = std::fs::remove_file(&p);
    db::open(&p).unwrap()
}

#[test]
fn agent_defaults_read_only_tools() {
    let c = conn("agents");
    assert_eq!(agents::DEFAULT_TOOLS, "Read,Grep,Glob");
    let a = agents::add(&c, "review", r"C:\work\proj", None, "").unwrap();
    assert_eq!(a.tools, "Read,Grep,Glob");
    let b = agents::add(&c, "dev", "/tmp/x", Some("Read,Edit"), "짧게 답해").unwrap();
    assert_eq!(
        (b.tools.as_str(), b.prompt.as_str()),
        ("Read,Edit", "짧게 답해")
    );
    assert!(
        agents::add(&c, "review", "/y", None, "").is_err(),
        "이름은 유일"
    );
    assert_eq!(
        agents::by_name(&c, "dev").unwrap().unwrap().folder,
        "/tmp/x"
    );
    assert_eq!(
        agents::list(&c)
            .unwrap()
            .iter()
            .map(|a| a.name.as_str())
            .collect::<Vec<_>>(),
        ["review", "dev"]
    );
    assert!(agents::remove(&c, a.id).unwrap());
    assert!(agents::by_name(&c, "review").unwrap().is_none());
}

#[test]
fn conversation_by_uuid_per_peer() {
    let c = conn("conv");
    let a = chat::open_conversation(&c, "u-1", 7, "review", chat::Dir::Out).unwrap();
    // 같은 uuid · 같은 상대면 같은 대화.
    let again = chat::open_conversation(&c, "u-1", 7, "review", chat::Dir::Out).unwrap();
    assert_eq!(a.id, again.id);
    // 다른 상대가 같은 uuid 를 대면 거부 — 남의 대화에 끼어들지 못한다.
    assert!(chat::open_conversation(&c, "u-1", 8, "review", chat::Dir::Out).is_err());
    assert_eq!(chat::find(&c, "u-1", 7).unwrap().unwrap().id, a.id);
    assert!(chat::find(&c, "u-1", 8).unwrap().is_none());
    assert_eq!(a.session, None);
    chat::set_session(&c, a.id, "sess-1").unwrap();
    assert_eq!(
        chat::by_id(&c, a.id).unwrap().unwrap().session.as_deref(),
        Some("sess-1")
    );
    assert_eq!(chat::list(&c).unwrap().len(), 1);
}

#[test]
fn messages_in_order_with_state() {
    let c = conn("msgs");
    let conv = chat::open_conversation(&c, "u-2", 1, "dev", chat::Dir::In).unwrap();
    let m1 = chat::add_message(
        &c,
        conv.id,
        "m-1",
        chat::Author::Human,
        "안녕",
        chat::State::Queued,
    )
    .unwrap();
    chat::add_message(
        &c,
        conv.id,
        "m-2",
        chat::Author::Agent,
        "반가워",
        chat::State::Done,
    )
    .unwrap();
    assert!(
        chat::add_message(
            &c,
            conv.id,
            "m-1",
            chat::Author::Human,
            "중복",
            chat::State::Queued
        )
        .is_err(),
        "메시지 uuid 는 유일"
    );
    chat::set_state(&c, m1.id, chat::State::Done).unwrap();
    let ms = chat::messages(&c, conv.id).unwrap();
    assert_eq!(ms.len(), 2);
    assert_eq!(
        (ms[0].text.as_str(), ms[0].author, ms[0].state),
        ("안녕", chat::Author::Human, chat::State::Done)
    );
    assert_eq!(
        (ms[1].text.as_str(), ms[1].author),
        ("반가워", chat::Author::Agent)
    );
}

#[test]
fn reply_meta_and_summaries() {
    let c = conn("reply");
    let a = chat::open_conversation(&c, "u-a", 1, "review", chat::Dir::Out).unwrap();
    let b = chat::open_conversation(&c, "u-b", 2, "dev", chat::Dir::In).unwrap();
    let q = chat::add_message(
        &c,
        a.id,
        "m1",
        chat::Author::Human,
        "첫 질문입니다",
        chat::State::Sent,
    )
    .unwrap();
    assert!(q.created_at > 0);
    assert_eq!((q.reply_to, q.turns, q.duration_ms), (None, None, None));
    // 답은 원 요청 · 실행 정보와 함께.
    let r = chat::add_reply(
        &c,
        a.id,
        "r1",
        "답",
        chat::State::Done,
        q.id,
        Some(3),
        Some(18400),
        Some("opus"),
        Some("high"),
    )
    .unwrap();
    assert_eq!(
        (r.author, r.reply_to, r.turns, r.duration_ms),
        (chat::Author::Agent, Some(q.id), Some(3), Some(18400))
    );
    assert_eq!(
        (r.model.as_deref(), r.effort.as_deref()),
        (Some("opus"), Some("high"))
    );
    // 실행 정보 없는 답(이전 버전 상대)도.
    let r2 = chat::add_reply(
        &c,
        a.id,
        "r2",
        "답2",
        chat::State::Error,
        q.id,
        None,
        None,
        None,
        None,
    )
    .unwrap();
    assert_eq!((r2.turns, r2.duration_ms), (None, None));
    chat::add_message(
        &c,
        b.id,
        "m2",
        chat::Author::Human,
        &"가".repeat(60),
        chat::State::Queued,
    )
    .unwrap();
    // 요약: 제목 = 첫 사람 메시지 앞 40자, 마지막 상태 · 시각, 최근 것 먼저(같은 초면 id 큰 것 먼저).
    let s = chat::summaries(&c).unwrap();
    assert_eq!(s.len(), 2);
    assert_eq!(s[0].conversation.id, b.id);
    assert_eq!(s[0].title, "가".repeat(40));
    assert_eq!(s[0].last_state, Some(chat::State::Queued));
    assert_eq!(s[1].title, "첫 질문입니다");
    assert_eq!(s[1].last_state, Some(chat::State::Error));
    assert!(s[1].last_at.unwrap() > 0);
    // 메시지 없는 대화는 빈 제목.
    chat::open_conversation(&c, "u-c", 1, "x", chat::Dir::Out).unwrap();
    let s = chat::summaries(&c).unwrap();
    let empty = s.iter().find(|x| x.conversation.uuid == "u-c").unwrap();
    assert_eq!(
        (empty.title.as_str(), empty.last_state, empty.last_at),
        ("", None, None)
    );
}

#[test]
fn human_posts_and_notes() {
    let c = conn("notes");
    let conv = chat::open_conversation(&c, "u-n", 1, "review", chat::Dir::In).unwrap();
    // 상대의 실행 요청(방향 기본: 받은 대화의 사람 글 = 상대 것 · 실행 요청).
    let q1 = chat::add_message(
        &c,
        conv.id,
        "q1",
        chat::Author::Human,
        "첫 요청",
        chat::State::Done,
    )
    .unwrap();
    assert_eq!((q1.mine, q1.exec), (false, true));
    let r1 = chat::add_reply(
        &c,
        conv.id,
        "r1",
        "답",
        chat::State::Done,
        q1.id,
        None,
        None,
        None,
        None,
    )
    .unwrap();
    assert_eq!(
        (r1.mine, r1.exec),
        (true, false),
        "받은 대화의 에이전트 답 = 내 것"
    );
    // 그 사이 양쪽 사람 글(실행 없음).
    let n1 = chat::add_human(
        &c,
        conv.id,
        "n1",
        "상대 메모",
        chat::State::Done,
        false,
        false,
    )
    .unwrap();
    let n2 = chat::add_human(&c, conv.id, "n2", "내 메모", chat::State::Done, true, false).unwrap();
    assert_eq!((n2.mine, n2.exec), (true, false));
    // 내 실행 요청.
    let q2 = chat::add_human(
        &c,
        conv.id,
        "q2",
        "둘째 요청",
        chat::State::Queued,
        true,
        true,
    )
    .unwrap();
    assert_eq!(
        chat::last_exec_before(&c, conv.id, &q2)
            .unwrap()
            .unwrap()
            .id,
        q1.id
    );
    let notes = chat::notes_between(&c, conv.id, Some(&q1), &q2).unwrap();
    assert_eq!(
        notes.iter().map(|m| m.id).collect::<Vec<_>>(),
        [n1.id, n2.id]
    );
    // 대기열은 실행 요청만.
    assert_eq!(chat::pending(&c, conv.id).unwrap()[0].id, q2.id);
    chat::add_human(
        &c,
        conv.id,
        "n3",
        "메모 대기?",
        chat::State::Queued,
        true,
        false,
    )
    .unwrap();
    assert_eq!(
        chat::pending(&c, conv.id).unwrap().len(),
        1,
        "실행 없는 글은 대기열에 없다"
    );
}

#[test]
fn read_only_tools_filter() {
    assert_eq!(agents::read_only("Read, Grep,Edit,Bash,LS"), "Read,Grep,LS");
    assert_eq!(
        agents::read_only("Edit,Write,Bash"),
        agents::DEFAULT_TOOLS,
        "남는 것이 없으면 기본"
    );
    assert_eq!(agents::read_only(""), agents::DEFAULT_TOOLS);
    assert_eq!(
        agents::unknown_tools("Read,Edit, Bash ,Glob"),
        ["Edit", "Bash"]
    );
    assert!(agents::unknown_tools("Read,Grep,Glob,LS,NotebookRead").is_empty());
    assert!(
        agents::unknown_tools("read").len() == 1,
        "대소문자도 정확히"
    );
}

#[test]
fn remote_order_value_bounded() {
    let c = conn("lc");
    let conv = chat::open_conversation(&c, "u-l", 1, "review", chat::Dir::In).unwrap();
    let a = chat::add_message(
        &c,
        conv.id,
        "a",
        chat::Author::Human,
        "x",
        chat::State::Done,
    )
    .unwrap();
    // 범위 밖 원격 값은 무시 — 이 PC 가 매긴 값 그대로.
    for bad in [i64::MAX, i64::MAX - 1, chat::MAX_LC + 1, 0, -5] {
        assert_eq!(
            chat::observe(&c, a.id, Some(bad)).unwrap().lc,
            a.lc,
            "{bad}"
        );
    }
    let b = chat::add_message(
        &c,
        conv.id,
        "b",
        chat::Author::Human,
        "y",
        chat::State::Done,
    )
    .unwrap();
    assert_eq!(
        chat::observe(&c, b.id, Some(chat::MAX_LC)).unwrap().lc,
        chat::MAX_LC
    );
    // 상한 값을 받은 뒤에도 다음 값은 정수로 매겨진다.
    let n = chat::add_message(
        &c,
        conv.id,
        "n",
        chat::Author::Human,
        "z",
        chat::State::Done,
    )
    .unwrap();
    assert_eq!(n.lc, chat::MAX_LC + 1);
    assert_eq!(chat::messages(&c, conv.id).unwrap().len(), 3);
}
