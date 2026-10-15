use an_core::{agents, chat, db};

fn conn(tag: &str) -> db::Connection {
    let p = std::env::temp_dir().join(format!("an-core-store-{tag}-{}.db", std::process::id()));
    let _ = std::fs::remove_file(&p);
    db::open(&p).unwrap()
}

#[test]
fn agent_profile_store() {
    let c = conn("agents");
    let a = agents::add(&c, "review", r"C:\work\proj", "").unwrap();
    let b = agents::add(&c, "dev", "/tmp/x", "짧게 답해").unwrap();
    assert_eq!(b.prompt, "짧게 답해");
    assert!(agents::add(&c, "review", "/y", "").is_err(), "이름은 유일");
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

/// 스키마 16(#1i) — 첫 자리 = 지금 보이는 순서(상대 id 순 · 대화 최근 활동순), 새 대화 맨 위 · 새 상대 맨 뒤, 그룹 넣고 빼기 · 지우기, 순서 바꾸기.
#[test]
fn schema16_groups_and_order() {
    use an_core::groups::{self, Order};
    use an_core::peers;
    let p = std::env::temp_dir().join(format!("an-core-store-s16-{}.db", std::process::id()));
    let _ = std::fs::remove_file(&p);
    let _ = std::fs::remove_file(db::backup_path(&p, 15));
    {
        // 스키마 15 가 남긴 파일 — 자리 열 · 그룹 표가 없다. 대화 1 은 오래된 글, 2 는 최근 글, 3 은 글 없음(생성 시각 중간).
        let conn = db::open(&p).unwrap();
        conn.execute_batch(
            "DROP TABLE group_convs; DROP TABLE groups;
             ALTER TABLE peers DROP COLUMN pos; ALTER TABLE conversations DROP COLUMN pos; PRAGMA user_version = 15;
             INSERT INTO peers (id, spki, name, host, addr, round, created_at) VALUES (5, x'05', 'p5', 'h', '1.2.3.4:5', 'r', 0);
             INSERT INTO peers (id, spki, name, host, addr, round, created_at) VALUES (2, x'02', 'p2', 'h', '1.2.3.4:2', 'r', 0);
             INSERT INTO conversations (id, uuid, peer_id, agent, dir, created_at) VALUES (1, 'c1', 2, 'a', 'out', 10),
               (2, 'c2', 2, 'a', 'out', 20), (3, 'c3', 5, 'a', 'in', 50);
             INSERT INTO messages (id, uuid, conversation_id, author, text, state, created_at) VALUES
               (1, 'm1', 1, 'human', '옛', 'done', 30), (2, 'm2', 2, 'human', '새', 'done', 90);",
        )
        .unwrap();
    }
    let conn = db::open(&p).unwrap();
    assert_eq!(db::user_version(&conn).unwrap(), 16);
    assert!(db::backup_path(&p, 15).exists(), "올리기 전 사본");
    let peer_order = |c: &db::Connection| {
        peers::list(c)
            .unwrap()
            .iter()
            .map(|p| p.id)
            .collect::<Vec<_>>()
    };
    assert_eq!(peer_order(&conn), [2, 5], "상대 = id 순");
    let conv_order = |c: &db::Connection| {
        let mut s = chat::summaries(c).unwrap();
        s.sort_by_key(|s| (s.pos, s.conversation.id));
        s.iter().map(|s| s.conversation.id).collect::<Vec<_>>()
    };
    assert_eq!(
        conv_order(&conn),
        [2, 3, 1],
        "대화 = 최근 활동순(글 없으면 생성 시각)"
    );

    // 새 대화는 맨 위, 새 상대는 맨 뒤.
    let n = chat::open_conversation(&conn, "c4", 5, "a", chat::Dir::In).unwrap();
    assert_eq!(conv_order(&conn), [n.id, 2, 3, 1]);
    peers::upsert(&conn, &[9u8], "p9", "h", "1.2.3.4:9", "r").unwrap();
    let p9 = peers::by_spki(&conn, &[9u8]).unwrap().unwrap().id;
    assert_eq!(peer_order(&conn), [2, 5, p9]);
    // 새 글이 와도 자리는 그대로.
    chat::add_human(&conn, 1, "m3", "더 새", chat::State::Done, true, true).unwrap();
    assert_eq!(conv_order(&conn), [n.id, 2, 3, 1]);

    // 순서 바꾸기 — 준 순서대로, 겹침 · 없는 id 는 거절(아무것도 안 바뀜).
    groups::reorder(&conn, Order::Peers, &[p9, 2, 5])
        .unwrap()
        .unwrap();
    assert_eq!(peer_order(&conn), [p9, 2, 5]);
    groups::reorder(&conn, Order::Conversations, &[1, 2, 3, n.id])
        .unwrap()
        .unwrap();
    assert_eq!(conv_order(&conn), [1, 2, 3, n.id]);
    assert!(
        groups::reorder(&conn, Order::Conversations, &[2, 2])
            .unwrap()
            .is_err()
    );
    assert!(
        groups::reorder(&conn, Order::Conversations, &[3, 999])
            .unwrap()
            .is_err()
    );
    assert_eq!(conv_order(&conn), [1, 2, 3, n.id], "거절이면 그대로");

    // 그룹 — 이름 규칙 · 같은 이름 · 여러 그룹 소속 · 넣고 빼기 · 지워도 대화는 남는다.
    assert!(groups::clean_name("  ").is_err());
    assert!(groups::clean_name(&"가".repeat(41)).is_err());
    assert_eq!(
        groups::clean_name(&format!(" {} ", "가".repeat(40))).unwrap(),
        "가".repeat(40)
    );
    let g1 = groups::add(&conn, "결제").unwrap();
    let g2 = groups::add(&conn, "배포").unwrap();
    assert!(groups::name_taken(&conn, "결제", None).unwrap());
    assert!(
        !groups::name_taken(&conn, "결제", Some(g1.id)).unwrap(),
        "자기 이름은 겹침 아님"
    );
    assert!(groups::add(&conn, "결제").is_err(), "이름은 유일");
    assert!(groups::change(&conn, g1.id, &[1, 2, 2], &[]).unwrap());
    assert!(groups::change(&conn, g2.id, &[2], &[]).unwrap());
    assert!(
        groups::change(&conn, g1.id, &[], &[1, 7]).unwrap(),
        "없는 소속 빼기는 그대로"
    );
    assert!(!groups::change(&conn, 999, &[1], &[]).unwrap(), "없는 그룹");
    assert_eq!(
        groups::unknown_conversations(&conn, &[1, 77, 77, 78]).unwrap(),
        [77, 78]
    );
    let convs = |c: &db::Connection, id| groups::by_id(c, id).unwrap().unwrap().convs;
    assert_eq!(
        (convs(&conn, g1.id), convs(&conn, g2.id)),
        (vec![2], vec![2]),
        "한 대화가 두 그룹에"
    );
    let tx = db::write_tx(&conn).unwrap();
    assert!(groups::join(&tx, g2.id, 3).unwrap());
    assert!(!groups::join(&tx, 999, 3).unwrap());
    tx.commit().unwrap();
    assert_eq!(convs(&conn, g2.id), [2, 3]);
    assert!(groups::remove(&conn, g2.id).unwrap());
    let left: i64 = conn
        .query_row(
            "SELECT count(*) FROM group_convs WHERE group_id = ?1",
            [g2.id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(left, 0, "그룹을 지우면 소속도");
    assert_eq!(chat::summaries(&conn).unwrap().len(), 4, "대화는 남는다");
    let g3 = groups::add(&conn, "새").unwrap();
    assert!(g3.id > g2.id, "지운 그룹 번호를 다시 쓰지 않는다");
    drop(conn);
    // 다시 열어도 남는다.
    let conn = db::open(&p).unwrap();
    assert_eq!(conv_order(&conn), [1, 2, 3, n.id]);
    assert_eq!(
        groups::list(&conn)
            .unwrap()
            .iter()
            .map(|g| (g.name.clone(), g.convs.clone()))
            .collect::<Vec<_>>(),
        [("결제".to_string(), vec![2]), ("새".to_string(), vec![])]
    );
}

/// 순서 바꾸기에 개수 상한이 없다(#1i 라운드 2) — 웹UI 는 전체 순서를 보내고 대화는 지워지지 않는다. 10,001개도 그대로 저장.
#[test]
fn reorder_more_than_ten_thousand() {
    use an_core::groups::{self, Order};
    let c = conn("many");
    let tx = db::write_tx(&c).unwrap();
    for i in 0..10_001 {
        chat::open_conversation(&tx, &format!("u{i}"), 1, "a", chat::Dir::Out).unwrap();
    }
    tx.commit().unwrap();
    let mut ids: Vec<i64> = chat::list(&c).unwrap().iter().map(|x| x.id).collect();
    assert_eq!(ids.len(), 10_001);
    ids.sort();
    groups::reorder(&c, Order::Conversations, &ids)
        .unwrap()
        .unwrap();
    let first: i64 = c
        .query_row(
            "SELECT id FROM conversations ORDER BY pos, id LIMIT 1",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(first, ids[0], "준 순서대로");
}
