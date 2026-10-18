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
        (
            empty.title.as_str(),
            empty.last_state,
            empty.last_at,
            &empty.last
        ),
        ("", None, None, &None)
    );
    // 대화방 미리보기(#1j) — 마지막 글의 본문 앞 PREVIEW_CHARS 자 · 말한 이 · 이 PC 쪽인지.
    let last = |s: &[chat::Summary], id| {
        s.iter()
            .find(|x| x.conversation.id == id)
            .unwrap()
            .last
            .clone()
            .unwrap()
    };
    let l = last(&s, b.id);
    assert_eq!(
        (l.text, l.author, l.mine),
        ("가".repeat(60), chat::Author::Human, false),
        "받은 대화의 사람 글 = 상대 것"
    );
    let l = last(&s, a.id);
    assert_eq!((l.text.as_str(), l.author), ("답2", chat::Author::Agent));
    chat::add_message(
        &c,
        a.id,
        "m3",
        chat::Author::Human,
        &"나".repeat(200),
        chat::State::Sent,
    )
    .unwrap();
    let l = last(&chat::summaries(&c).unwrap(), a.id);
    assert_eq!(
        (l.text, l.author, l.mine),
        ("나".repeat(chat::PREVIEW_CHARS), chat::Author::Human, true),
        "보낸 대화의 사람 글 = 내 것, 긴 본문은 잘린다"
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
    assert_eq!(db::user_version(&conn).unwrap(), db::SCHEMA_VERSION);
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

/// 스키마 16 → 17(#1h — 원장 #17 S1-P): `peers` 를 AUTOINCREMENT 로 다시 만든다 — 번호 · 열 값 그대로, 순번은 지운 상대가 남긴 대화 · 보낼 글 번호까지,
/// 지우고 다시 이어도 번호를 다시 쓰지 않는다.
#[test]
fn schema17_peer_ids_never_reused() {
    use an_core::peers;
    let p = std::env::temp_dir().join(format!("an-core-store-s17-{}.db", std::process::id()));
    let _ = std::fs::remove_file(&p);
    let _ = std::fs::remove_file(db::backup_path(&p, 16));
    let columns = |c: &db::Connection| {
        let mut st = c
            .prepare("SELECT name FROM pragma_table_info('peers') ORDER BY cid")
            .unwrap();
        st.query_map([], |r| r.get::<_, String>(0))
            .unwrap()
            .map(Result::unwrap)
            .collect::<Vec<_>>()
    };
    let before;
    {
        // 스키마 16 이 남긴 파일 — `peers` 에 AUTOINCREMENT 가 없다. 상대 1 · 3, 지운 상대 5 의 대화 · 지운 상대 7 의 보낼 글이 남았다.
        let conn = db::open(&p).unwrap();
        let sql: String = conn
            .query_row(
                "SELECT sql FROM sqlite_master WHERE name = 'peers'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        conn.execute_batch(&format!(
            "{}; DROP TABLE peers; ALTER TABLE peers_old RENAME TO peers; DELETE FROM sqlite_sequence WHERE name = 'peers';
             PRAGMA user_version = 16;",
            sql.replace("CREATE TABLE \"peers\"", "CREATE TABLE peers_old")
                .replace("CREATE TABLE peers ", "CREATE TABLE peers_old ")
                .replace(" AUTOINCREMENT", "")
        ))
        .unwrap();
        conn.execute_batch(
            "INSERT INTO peers (id, spki, name, host, addr, round, created_at, alias, icon, pos) VALUES
               (1, x'01', 'p1', 'h1', '1.2.3.4:1', 'r', 0, '별칭', 'data:x', 4), (3, x'03', 'p3', 'h3', '1.2.3.4:3', 'r', 0, NULL, NULL, 2);
             INSERT INTO conversations (id, uuid, peer_id, agent, dir, created_at) VALUES (1, 'c1', 5, 'a', 'in', 10);
             INSERT INTO outbox (peer_id, kind, frame, created_at) VALUES (7, 'k', '{}', 0);",
        )
        .unwrap();
        assert!(
            !conn
                .query_row::<String, _, _>(
                    "SELECT sql FROM sqlite_master WHERE name = 'peers'",
                    [],
                    |r| r.get(0)
                )
                .unwrap()
                .contains("AUTOINCREMENT")
        );
        before = columns(&conn);
    }
    let conn = db::open(&p).unwrap();
    assert_eq!(db::user_version(&conn).unwrap(), db::SCHEMA_VERSION);
    assert!(db::backup_path(&p, 16).exists(), "올리기 전 사본");
    drop(conn);
    // 접힘 열쇠의 번호도 흔적이다(라운드 2) — 스키마 16 사본으로 돌아가 접힘에 지운 상대 12 를 남기고 다시 올린다.
    std::fs::copy(db::backup_path(&p, 16), &p).unwrap();
    {
        let c = rusqlite::Connection::open(&p).unwrap();
        an_core::settings::set(
            &c,
            an_core::run_opts::FOLDED_KEY,
            "convs:12,saved:me,peer:x",
        )
        .unwrap();
    }
    let conn = db::open(&p).unwrap();
    let seq: i64 = conn
        .query_row(
            "SELECT seq FROM sqlite_sequence WHERE name = 'peers'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(
        seq, 12,
        "접힘 열쇠의 지운 상대 12 까지 — 'me' · 숫자 아닌 열쇠는 건너뜀"
    );
    an_core::settings::set(&conn, an_core::run_opts::FOLDED_KEY, "").unwrap();
    conn.execute(
        "UPDATE sqlite_sequence SET seq = 7 WHERE name = 'peers'",
        [],
    )
    .unwrap();
    let sql: String = conn
        .query_row(
            "SELECT sql FROM sqlite_master WHERE name = 'peers'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert!(
        sql.contains("id INTEGER PRIMARY KEY AUTOINCREMENT"),
        "{sql}"
    );
    assert_eq!(columns(&conn), before, "열 전부 · 순서 그대로");
    assert_eq!(before.len(), 19, "peers 열 수(스키마 16 기준)");
    let p1 = peers::by_id(&conn, 1).unwrap().unwrap();
    assert_eq!(
        (p1.alias.as_deref(), p1.icon.as_deref()),
        (Some("별칭"), Some("data:x")),
        "열 값 그대로"
    );
    let pos: i64 = conn
        .query_row("SELECT pos FROM peers WHERE id = 1", [], |r| r.get(0))
        .unwrap();
    assert_eq!(pos, 4);
    assert!(peers::by_id(&conn, 3).unwrap().is_some());
    // 순번 = max(상대 3, 남은 대화의 상대 5, 보낼 글의 상대 7) — 새 상대는 8.
    let a = peers::upsert(&conn, b"new-a", "a", "h", "1.2.3.4:9", "r").unwrap();
    assert_eq!(a.id, 8);
    assert!(peers::remove(&conn, a.id).unwrap());
    let b = peers::upsert(&conn, b"new-b", "b", "h", "1.2.3.4:9", "r").unwrap();
    assert_eq!(b.id, 9, "지운 가장 큰 번호를 다시 쓰지 않는다");
    let again = peers::upsert(&conn, b"new-a", "a", "h", "1.2.3.4:9", "r").unwrap();
    assert_eq!(
        again.id, 10,
        "지운 상대를 다시 이어도 새 번호 — 옛 대화는 지운 상대 몫으로 남는다"
    );
    assert_eq!(
        peers::upsert(&conn, b"new-b", "b2", "h", "1.2.3.4:9", "r")
            .unwrap()
            .id,
        9,
        "있는 상대(같은 키)는 번호 유지"
    );
    drop(conn);
    let conn = db::open(&p).unwrap();
    // 있는 상대의 upsert(충돌 → 갱신)도 순번을 하나 쓴다(SQLite) — 빈 번호는 생겨도 다시 쓰지는 않는다.
    assert!(
        peers::upsert(&conn, b"new-c", "c", "h", "1.2.3.4:9", "r")
            .unwrap()
            .id
            > 10,
        "다시 열어도 순번 유지"
    );
}

/// 새 DB(스키마 17 로 바로) — 처음부터 AUTOINCREMENT, 지운 번호를 다시 쓰지 않는다.
#[test]
fn schema17_fresh_db_no_reuse() {
    use an_core::peers;
    let c = conn("s17-fresh");
    let a = peers::upsert(&c, b"a", "a", "h", "1.2.3.4:1", "r").unwrap();
    assert!(peers::remove(&c, a.id).unwrap());
    let b = peers::upsert(&c, b"b", "b", "h", "1.2.3.4:2", "r").unwrap();
    assert_eq!(b.id, a.id + 1);
}

#[test]
fn schema18_peer_groups_from_old_text() {
    use an_core::{agents, groups, peers};
    let p = std::env::temp_dir().join(format!("an-core-store-s18-{}.db", std::process::id()));
    let _ = std::fs::remove_file(&p);
    let _ = std::fs::remove_file(db::backup_path(&p, 17));
    let long = "가".repeat(45);
    let mut ids = Vec::new();
    {
        // 스키마 17 이 남긴 파일 — `group_peers` 가 없고 상대마다 자유 입력 그룹(`grp`), 같은 이름 레일 그룹 하나는 이미 있다.
        let c = db::open(&p).unwrap();
        c.execute_batch(
            "DROP TABLE group_peers; INSERT INTO groups (name, pos) VALUES ('결제 모듈', 0); PRAGMA user_version = 17;",
        )
        .unwrap();
        let olds = [
            Some("QA"),
            Some("  결제 모듈 "),
            Some("QA"),
            Some("   "),
            None,
            Some(long.as_str()),
            Some("a\u{7}b"),
        ];
        for (i, grp) in olds.iter().enumerate() {
            let pr = peers::upsert(
                &c,
                format!("k{i}").as_bytes(),
                &format!("p{i}"),
                "h",
                &format!("1.2.3.4:{i}"),
                "r",
            )
            .unwrap();
            c.execute(
                "UPDATE peers SET grp = ?2 WHERE id = ?1",
                rusqlite::params![pr.id, grp],
            )
            .unwrap();
            ids.push(pr.id);
        }
        let a = agents::add(&c, "dev", ".", "").unwrap();
        c.execute(
            "UPDATE agents SET grp = '에이전트 묶음' WHERE id = ?1",
            [a.id],
        )
        .unwrap();
    }
    let c = db::open(&p).unwrap();
    assert_eq!(db::user_version(&c).unwrap(), db::SCHEMA_VERSION);
    assert!(db::backup_path(&p, 17).exists(), "올리기 전 사본");
    // 같은 이름은 한 그룹(있던 그룹 포함) · 앞뒤 공백 · 제어 문자 뺌 · 40자 · 빈 이름 · 없음은 건너뜀, 새 그룹은 상대 순서대로 끝에.
    assert_eq!(
        groups::list(&c)
            .unwrap()
            .into_iter()
            .map(|g| (g.name, g.peers, g.convs))
            .collect::<Vec<_>>(),
        vec![
            ("결제 모듈".to_string(), vec![ids[1]], vec![]),
            ("QA".to_string(), vec![ids[0], ids[2]], vec![]),
            ("가".repeat(40), vec![ids[5]], vec![]),
            ("ab".to_string(), vec![ids[6]], vec![]),
        ]
    );
    let agent_grp: String = c
        .query_row("SELECT grp FROM agents WHERE name = 'dev'", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(
        agent_grp, "에이전트 묶음",
        "agents.grp 는 그대로 · 그룹을 만들지 않는다"
    );
    assert_eq!(groups::fit_name(" \u{1}\t "), None);
    assert_eq!(groups::fit_name(" x "), Some("x".to_string()));
}

#[test]
fn schema18_group_includes_peer_conversations() {
    use an_core::{groups, peers};
    let c = conn("s18-scope");
    let a = peers::upsert(&c, b"a", "a", "h", "1.2.3.4:1", "r").unwrap();
    let b = peers::upsert(&c, b"b", "b", "h", "1.2.3.4:2", "r").unwrap();
    let say = |conv: i64, uuid: &str| {
        chat::add_message(&c, conv, uuid, chat::Author::Human, "글", chat::State::Sent)
            .unwrap()
            .id
    };
    let ca = chat::open_conversation(&c, "u-a", a.id, "x", chat::Dir::Out).unwrap();
    let cb = chat::open_conversation(&c, "u-b", b.id, "x", chat::Dir::Out).unwrap();
    let cb2 = chat::open_conversation(&c, "u-b2", b.id, "y", chat::Dir::Out).unwrap();
    let (ma, mb, mb2) = (say(ca.id, "m-a"), say(cb.id, "m-b"), say(cb2.id, "m-b2"));
    let g = groups::add(&c, "묶음").unwrap();
    assert!(groups::change(&c, g.id, &[cb.id], &[]).unwrap());
    assert!(groups::change_peers(&c, g.id, &[a.id], &[]).unwrap());
    assert!(
        !groups::change_peers(&c, 999, &[a.id], &[]).unwrap(),
        "없는 그룹"
    );
    // 넣은 뒤 생긴 그 상대의 대화도 그룹에.
    let ca2 = chat::open_conversation(&c, "u-a2", a.id, "y", chat::Dir::In).unwrap();
    let ma2 = say(ca2.id, "m-a2");
    let in_group = |c: &db::Connection| {
        let mut ids: Vec<i64> =
            chat::messages_across(c, None, chat::Scope::Group(g.id), None, None, None, 100)
                .unwrap()
                .into_iter()
                .map(|m| m.id)
                .collect();
        ids.sort();
        ids
    };
    assert_eq!(in_group(&c), vec![ma, mb, ma2]);
    assert!(
        !in_group(&c).contains(&mb2),
        "하나씩 넣지 않은 다른 상대 대화는 아니다"
    );
    assert_eq!(groups::by_id(&c, g.id).unwrap().unwrap().peers, vec![a.id]);
    assert_eq!(
        groups::unknown_peers(&c, &[a.id, 77, 77]).unwrap(),
        vec![77]
    );
    // 상대를 빼도 하나씩 넣은 대화는 남는다 · 그룹을 지우면 상대 소속도 지운다.
    assert!(groups::change(&c, g.id, &[ca.id], &[]).unwrap());
    assert!(groups::change_peers(&c, g.id, &[], &[a.id]).unwrap());
    assert_eq!(in_group(&c), vec![ma, mb]);
    assert!(groups::change_peers(&c, g.id, &[b.id], &[]).unwrap());
    assert!(groups::remove(&c, g.id).unwrap());
    let left: i64 = c
        .query_row("SELECT count(*) FROM group_peers", [], |r| r.get(0))
        .unwrap();
    assert_eq!(left, 0);
}

/// 대화방 미리보기(#1j 라운드 1) — 마지막 글 = 대화 화면과 같은 `(lc, uuid)` 순서의 마지막(로컬 `id` 가 더 커도 공통 순서가 앞이면 아니다).
#[test]
fn summary_last_follows_canonical_order() {
    let c = conn("last-order");
    let conv = chat::open_conversation(&c, "u-o", 1, "review", chat::Dir::Out).unwrap();
    let early = chat::add_message(
        &c,
        conv.id,
        "m-1",
        chat::Author::Human,
        "먼저 쓴 글",
        chat::State::Sent,
    )
    .unwrap();
    let late = chat::add_message(
        &c,
        conv.id,
        "m-2",
        chat::Author::Human,
        "나중 id",
        chat::State::Sent,
    )
    .unwrap();
    assert!(late.id > early.id);
    // 상대 쪽 시계가 앞서 먼저 저장된 글의 공통 순서가 뒤 — 화면 마지막 말풍선은 `early`.
    c.execute("UPDATE messages SET lc = 99 WHERE id = ?1", [early.id])
        .unwrap();
    let shown = chat::messages(&c, conv.id).unwrap();
    assert_eq!(shown.last().unwrap().id, early.id, "화면 순서");
    let s = chat::summaries(&c).unwrap();
    let last = s[0].last.clone().unwrap();
    assert_eq!(last.text, "먼저 쓴 글");
    // 라운드 2 — 마지막 상태 · 시각 · 출처(CLI)도 같은 글에서.
    c.execute(
        "UPDATE messages SET state = 'running', created_at = 1000, via = 'cli' WHERE id = ?1",
        [early.id],
    )
    .unwrap();
    c.execute(
        "UPDATE messages SET created_at = 2000 WHERE id = ?1",
        [late.id],
    )
    .unwrap();
    let s = chat::summaries(&c).unwrap();
    assert_eq!(
        (s[0].last_state, s[0].last_at),
        (Some(chat::State::Running), Some(1000))
    );
    assert_eq!(s[0].last.as_ref().unwrap().via.as_deref(), Some("cli"));
}

#[test]
fn schema20_share_tables_from_19() {
    use an_core::{peers, share};
    let p = std::env::temp_dir().join(format!("an-core-store-s20-{}.db", std::process::id()));
    let _ = std::fs::remove_file(&p);
    let _ = std::fs::remove_file(db::backup_path(&p, 19));
    let (owner, out, inn) = {
        // 스키마 19 가 남긴 파일 — 공유 표 · 열이 없다.
        let c = db::open(&p).unwrap();
        let pr = peers::upsert(&c, b"k1", "p1", "h", "1.2.3.4:1", "r").unwrap();
        let out = chat::open_conversation(&c, "u-out", pr.id, "dev", chat::Dir::Out).unwrap();
        let inn = chat::open_conversation(&c, "u-in", pr.id, "dev", chat::Dir::In).unwrap();
        chat::add_message(
            &c,
            out.id,
            "m-out",
            chat::Author::Human,
            "안녕",
            chat::State::Done,
        )
        .unwrap();
        c.execute_batch(
            "DROP TABLE shares;
             ALTER TABLE conversations DROP COLUMN share_other; ALTER TABLE conversations DROP COLUMN share_dir;
             ALTER TABLE conversations DROP COLUMN share_ended; ALTER TABLE conversations DROP COLUMN watchers;
             ALTER TABLE messages DROP COLUMN owner; PRAGMA user_version = 19;",
        )
        .unwrap();
        (pr.id, out.id, inn.id)
    };
    let c = db::open(&p).unwrap();
    assert_eq!(db::SCHEMA_VERSION, 20);
    assert_eq!(db::user_version(&c).unwrap(), 20);
    assert!(db::backup_path(&p, 19).exists(), "올리기 전 사본");
    // 기존 대화 dir · 글 그대로.
    assert_eq!(chat::by_id(&c, out).unwrap().unwrap().dir, chat::Dir::Out);
    assert_eq!(chat::by_id(&c, inn).unwrap().unwrap().dir, chat::Dir::In);
    let m = chat::messages(&c, out).unwrap().remove(0);
    assert_eq!((m.text.as_str(), m.mine, m.owner), ("안녕", true, false));
    // 새 표 · 열.
    let has = |t: &str, col: &str| {
        c.prepare(&format!(
            "SELECT 1 FROM pragma_table_info('{t}') WHERE name = ?1"
        ))
        .unwrap()
        .exists([col])
        .unwrap()
    };
    for (t, col) in [
        ("shares", "conversation_id"),
        ("shares", "peer_id"),
        ("shares", "sent_upto"),
        ("shares", "created_at"),
        ("conversations", "share_other"),
        ("conversations", "share_dir"),
        ("conversations", "share_ended"),
        ("conversations", "watchers"),
        ("messages", "owner"),
    ] {
        assert!(has(t, col), "{t}.{col}");
    }

    // 관람자(A) — 한 번만, 커서 0, 페어링 해제로 지워진다.
    let viewer = peers::upsert(&c, b"k2", "p2", "h", "1.2.3.4:2", "r").unwrap();
    assert!(share::add(&c, out, viewer.id).unwrap());
    assert!(!share::add(&c, out, viewer.id).unwrap());
    assert_eq!(share::watchers(&c, out).unwrap()[0].sent_upto, 0);
    share::set_sent(&c, out, viewer.id, 5).unwrap();
    share::set_sent(&c, out, viewer.id, 3).unwrap();
    assert_eq!(
        share::watchers(&c, out).unwrap()[0].sent_upto,
        5,
        "뒤로 가지 않음"
    );
    peers::remove(&c, viewer.id).unwrap();
    assert!(share::watchers(&c, out).unwrap().is_empty());

    // 사본(C) — 상대 id = 대화를 가진 PC, 내 대화 · 다른 상대의 uuid 는 사본으로 덮지 못한다.
    let copy = share::open_copy(&c, "u-copy", owner, "rev", Some("제목"), ("원 상대", "in"))
        .unwrap()
        .unwrap();
    assert_eq!((copy.dir, copy.peer_id), (chat::Dir::Shared, owner));
    assert!(
        share::open_copy(&c, "u-out", owner, "x", None, ("x", "out"))
            .unwrap()
            .is_none()
    );
    assert!(
        share::open_copy(&c, "u-copy", 99, "x", None, ("x", "out"))
            .unwrap()
            .is_none()
    );
    let post = |uuid: &'static str, text: &'static str, edited| share::Post {
        uuid,
        author: chat::Author::Agent,
        text,
        lc: Some(3),
        at: 1_700_000_000,
        owner: true,
        via: None,
        agent: None,
        ok: true,
        edited,
    };
    assert!(share::upsert(&c, copy.id, &post("m-copy", "원문", false)).unwrap());
    assert!(share::upsert(&c, copy.id, &post("m-copy", "고침", true)).unwrap());
    assert!(
        !share::upsert(&c, copy.id, &post("m-out", "덮기", false)).unwrap(),
        "남의 글"
    );
    let ms = chat::messages(&c, copy.id).unwrap();
    assert_eq!(ms.len(), 1);
    let m = &ms[0];
    assert_eq!(
        (m.text.as_str(), m.mine, m.exec, m.owner, m.lc, m.created_at),
        ("고침", false, false, true, 3, 1_700_000_000)
    );
    assert!(m.edited_at.is_some());
    assert_eq!(
        chat::message_by_uuid(&c, "m-out").unwrap().unwrap().text,
        "안녕"
    );
    // 사본 글은 내 활동 · 질의 목록에 들지 않는다.
    assert!(
        chat::activity(&c, None, chat::ActivityFilter::All, None, 50)
            .unwrap()
            .iter()
            .all(|a| a.conversation_id != copy.id)
    );
    // 빠짐 — 한 번만, 다시 초대하면 풀리고 글은 비운다.
    assert!(share::end(&c, copy.id).unwrap());
    assert!(!share::end(&c, copy.id).unwrap());
    assert!(share::copy(&c, copy.id).unwrap().unwrap().ended);
    share::open_copy(&c, "u-copy", owner, "rev", None, ("원 상대", "in"))
        .unwrap()
        .unwrap();
    assert!(!share::copy(&c, copy.id).unwrap().unwrap().ended);
    assert!(chat::messages(&c, copy.id).unwrap().is_empty());
    assert!(share::copy(&c, out).unwrap().is_none(), "사본이 아니다");
    // 원 상대(B) 쪽 관람자 수.
    assert!(share::set_watched(&c, inn, 2).unwrap());
    assert!(!share::set_watched(&c, inn, 2).unwrap());
    assert_eq!(share::watched(&c, inn).unwrap(), 2);
}

/// 내 활동 범위(#1q) — 받은 요청 · 받은 답(내 사람 글에 단 답만) · 내 요청 · 실패만. 상대 사람 일반 메시지 · 자동 왕복 기록 · 내 에이전트 글에
/// 단 상대 답은 빠지고, 안 본 수도 같은 범위다. 내 요청은 상태가 바뀐 기록(`req_state`)이 있을 때만 안 본 것.
#[test]
fn activity_scope() {
    use chat::{ActivityFilter as F, ActivityKind as K, Author, Dir, State};
    let c = conn("activity-scope");
    let p = an_core::peers::upsert(&c, &[7u8], "p7", "h", "1.2.3.4:7", "r").unwrap().id;
    let out = chat::open_conversation(&c, "u-out", p, "guide", Dir::Out).unwrap();
    let inn = chat::open_conversation(&c, "u-in", p, "review", Dir::In).unwrap();
    // 받은 대화 — 상대 요청 · 상대 일반 메시지.
    let req = chat::add_human(&c, inn.id, "req", "로그 형식 바꿔 주세요", State::Done, false, false).unwrap();
    chat::set_request(&c, req.id, true).unwrap();
    chat::add_human(&c, inn.id, "chat", "점심 뒤에 볼까요", State::Done, false, false).unwrap();
    // 보낸 대화 — 내 질의 → 상대 답 · 내 요청 · 실패한 내 글 · 자동 왕복(내 에이전트 글에 단 상대 답 · 시작 · 끝 기록).
    let q = chat::add_human(&c, out.id, "q", "배포 절차?", State::Done, true, true).unwrap();
    chat::add_reply(&c, out.id, "ans", "절차는 …", State::Done, q.id, None, None, None, None).unwrap();
    let mreq = chat::add_human(&c, out.id, "mreq", "QA 서버 올려 주세요", State::Done, true, false).unwrap();
    chat::set_request(&c, mreq.id, true).unwrap();
    let fail = chat::add_human(&c, out.id, "fail", "못 간 글", State::Sent, true, true).unwrap();
    chat::set_state(&c, fail.id, State::Error).unwrap();
    let mine_agent = chat::add_message(&c, out.id, "relay-mine", Author::Agent, "내 에이전트 말", State::Done).unwrap();
    chat::set_flags(&c, mine_agent.id, true, true, State::Done).unwrap();
    chat::add_reply(&c, out.id, "relay-ans", "상대 에이전트 말", State::Done, mine_agent.id, None, None, None, None).unwrap();
    chat::add_relay_event(&c, out.id, chat::RelayMark::Start, "시작").unwrap();
    chat::add_relay_event(&c, out.id, chat::RelayMark::End, "상한").unwrap();
    // 공유 사본(관람자 — #1o)의 사람 요청 · 에이전트 답도 빠진다.
    let copy = an_core::share::open_copy(&c, "u-copy", p, "rev", None, ("원 상대", "in"))
        .unwrap()
        .unwrap();
    let post = |uuid: &'static str, author| an_core::share::Post {
        uuid,
        author,
        text: "사본 글",
        lc: Some(1),
        at: 1_700_000_000,
        owner: true,
        via: None,
        agent: None,
        ok: true,
        edited: false,
    };
    assert!(an_core::share::upsert(&c, copy.id, &post("copy-h", Author::Human)).unwrap());
    assert!(an_core::share::upsert(&c, copy.id, &post("copy-a", Author::Agent)).unwrap());
    let copied = chat::message_by_uuid(&c, "copy-h").unwrap().unwrap();
    chat::set_request(&c, copied.id, true).unwrap();

    let got = |f| {
        let mut v: Vec<(K, String)> = chat::activity(&c, None, f, None, 50)
            .unwrap()
            .into_iter()
            .map(|a| (a.kind, a.message.map(|m| m.uuid).unwrap_or_default()))
            .collect();
        v.sort_by(|a, b| a.1.cmp(&b.1));
        v
    };
    let s = |k, u: &str| (k, u.to_string());
    assert_eq!(
        got(F::All),
        [s(K::Message, "ans"), s(K::Failed, "fail"), s(K::MyRequest, "mreq"), s(K::Message, "req")],
        "일반 메시지 · 자동 왕복 답 · 시작 · 끝 기록 없음"
    );
    assert_eq!(got(F::Request), [s(K::Message, "req")]);
    assert_eq!(got(F::Answer), [s(K::Message, "ans")]);
    assert_eq!(got(F::MyRequest), [s(K::MyRequest, "mreq")]);
    assert_eq!(got(F::Failed), [s(K::Failed, "fail")]);
    // 안 본 수 = 받은 요청 · 받은 답 · 실패(막 보낸 내 요청은 아니다).
    assert_eq!(chat::activity_unseen(&c, None).unwrap(), 3);
    assert_eq!(chat::activity_unseen(&c, Some(p)).unwrap(), 3);
    let seen = |c: &db::Connection| chat::mark_activity_seen(c, None, chat::activity_upto(c).unwrap()).unwrap();
    seen(&c);
    assert_eq!(chat::activity_unseen(&c, None).unwrap(), 0);
    // 상대가 내 요청 상태를 바꿨다 → 안 본 것 하나(같은 요청이 두 번 바뀌어도 하나).
    chat::set_req_state(&c, mreq.id, chat::ReqState::Accepted, Some("오늘")).unwrap();
    chat::note_req_state(&c, mreq.id).unwrap();
    chat::set_req_state(&c, mreq.id, chat::ReqState::Done, None).unwrap();
    chat::note_req_state(&c, mreq.id).unwrap();
    assert_eq!(chat::activity_unseen(&c, None).unwrap(), 1);
    let mine = chat::activity(&c, None, F::MyRequest, None, 50).unwrap();
    assert!(mine[0].unseen);
    assert_eq!(mine[0].message.as_ref().unwrap().req_state, Some(chat::ReqState::Done));
    seen(&c);
    assert_eq!(chat::activity_unseen(&c, None).unwrap(), 0);
    assert!(!chat::activity(&c, None, F::MyRequest, None, 50).unwrap()[0].unseen);
    // 내 요청이 아닌 글에는 기록하지 않는다.
    chat::note_req_state(&c, req.id).unwrap();
    chat::note_req_state(&c, q.id).unwrap();
    assert_eq!(chat::activity_unseen(&c, None).unwrap(), 0);
}

/// 주고받은 메시지 순서(#1q 적대적 평가 R2) — 페이지 · 상대별 마지막 메시지 모두 `(created_at, id)` 최신순. 나중에 들어온 공유 사본의 옛 글(상대 시각)이
/// 맨 위 · 마지막 메시지가 되지 않고, 복합 커서로 한 건씩 넘겨도 빠짐 · 겹침이 없다.
#[test]
fn peer_pages_order() {
    use chat::{Dir, State};
    let c = conn("peer-pages");
    let p = an_core::peers::upsert(&c, &[8u8], "p8", "h", "1.2.3.4:8", "r").unwrap().id;
    let out = chat::open_conversation(&c, "u-pp", p, "guide", Dir::Out).unwrap();
    for (i, t) in ["첫", "둘", "셋"].iter().enumerate() {
        let m = chat::add_human(&c, out.id, &format!("pp-{i}"), t, State::Done, true, true).unwrap();
        c.execute("UPDATE messages SET created_at = ?2 WHERE id = ?1", (m.id, 1_800_000_000 + i as i64)).unwrap();
    }
    // 나중에 들어온 공유 사본의 옛 글.
    let copy = an_core::share::open_copy(&c, "u-pp-copy", p, "rev", None, ("원 상대", "in")).unwrap().unwrap();
    let post = an_core::share::Post {
        uuid: "pp-old",
        author: chat::Author::Agent,
        text: "옛 사본 글",
        lc: Some(1),
        at: 1_700_000_000,
        owner: true,
        via: None,
        agent: None,
        ok: true,
        edited: false,
    };
    assert!(an_core::share::upsert(&c, copy.id, &post).unwrap());
    let page = |before| chat::messages_page(&c, chat::Scope::All, before, 10).unwrap();
    assert_eq!(
        page(None).iter().map(|m| m.text.as_str()).collect::<Vec<_>>(),
        ["셋", "둘", "첫", "옛 사본 글"]
    );
    let (_, n, last) = chat::peer_message_counts(&c, chat::Scope::All).unwrap()[0];
    assert_eq!(n, 4);
    assert_eq!(chat::message_by_id(&c, last).unwrap().unwrap().text, "셋", "마지막 메시지 = 가장 최근 시각");
    // 한 건씩 넘기기.
    let mut seen = Vec::new();
    let mut cur = None;
    loop {
        let one = chat::messages_page(&c, chat::Scope::All, cur, 1).unwrap();
        let Some(m) = one.first() else { break };
        seen.push(m.text.clone());
        cur = Some((m.created_at, m.id));
    }
    assert_eq!(seen, ["셋", "둘", "첫", "옛 사본 글"]);
}
