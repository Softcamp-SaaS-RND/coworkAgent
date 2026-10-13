use an_core::{db, paths::Paths};
use std::path::PathBuf;

fn temp_root(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("an-core-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

#[test]
fn layout_created_under_root() {
    let root = temp_root("layout");
    let paths = Paths::under(&root);
    paths.ensure().unwrap();
    db::open(&paths.db_file()).unwrap();

    assert!(paths.log_dir().is_dir());
    assert!(paths.db_file().is_file());
    for p in [
        paths.db_file(),
        paths.log_file(),
        paths.lock_file(),
        paths.instance_file(),
        paths.identity_file(),
    ] {
        assert!(p.starts_with(&root), "{p:?} 가 루트 밖");
    }
    // 루트 바로 아래에는 정확히 logs 폴더와 DB 파일만 생긴다.
    let mut names: Vec<_> = std::fs::read_dir(&root)
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .collect();
    names.sort();
    assert_eq!(names, ["coworkAgent.db", "logs"]);
}

#[test]
fn db_schema_version_kept_on_reopen() {
    let root = temp_root("dbver");
    let paths = Paths::under(&root);
    paths.ensure().unwrap();
    assert_eq!(db::SCHEMA_VERSION, 13);
    {
        // 00 이 남긴 스키마 1 파일 — 열면 2 로 오른다.
        let conn = rusqlite::Connection::open(paths.db_file()).unwrap();
        conn.pragma_update(None, "user_version", 1).unwrap();
    }
    {
        let conn = db::open(&paths.db_file()).unwrap();
        assert_eq!(db::user_version(&conn).unwrap(), 13);
        assert!(an_core::peers::list(&conn).unwrap().is_empty());
        assert_eq!(an_core::settings::get(&conn, "x").unwrap(), None);
    }
    let conn = db::open(&paths.db_file()).unwrap();
    assert_eq!(db::user_version(&conn).unwrap(), 13);
    conn.pragma_update(None, "user_version", 14).unwrap();
    drop(conn);
    assert!(
        db::open(&paths.db_file()).is_err(),
        "더 새 스키마는 열지 않는다"
    );
}

#[test]
fn schema5_migration_survives_partial_state() {
    let root = temp_root("partial5");
    let paths = Paths::under(&root);
    paths.ensure().unwrap();
    {
        // 스키마 4 까지 올린 뒤, 열 하나만 붙고 버전은 4 로 남은 상태(중간에 끊긴 이전 실행).
        let conn = db::open(&paths.db_file()).unwrap();
        conn.execute_batch(
            "ALTER TABLE messages DROP COLUMN duration_ms; ALTER TABLE messages DROP COLUMN turns; PRAGMA user_version = 4;",
        )
        .unwrap();
    }
    let conn = db::open(&paths.db_file()).unwrap();
    assert_eq!(db::user_version(&conn).unwrap(), 13);
    let cols: Vec<String> = conn
        .prepare("SELECT name FROM pragma_table_info('messages')")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    for c in ["reply_to", "turns", "duration_ms"] {
        assert!(cols.iter().any(|x| x == c), "{c} 열: {cols:?}");
    }
}

#[test]
fn migration_keeps_backup_for_rollback() {
    let root = temp_root("backup");
    let paths = Paths::under(&root);
    paths.ensure().unwrap();
    {
        let conn = db::open(&paths.db_file()).unwrap();
        // 스키마 4 DB 에 설정 한 줄 — 이전 실행 파일이 쓰던 상태.
        conn.execute_batch(
            "ALTER TABLE messages DROP COLUMN reply_to; ALTER TABLE messages DROP COLUMN turns;
             ALTER TABLE messages DROP COLUMN duration_ms; PRAGMA user_version = 4;
             INSERT INTO settings (key, value) VALUES ('k', 'v4');",
        )
        .unwrap();
    }
    assert!(
        !db::backup_path(&paths.db_file(), 0).exists(),
        "새 DB 는 사본이 없다"
    );
    db::open(&paths.db_file()).unwrap();
    // 올리기 전 스키마 4 사본 — 이전 실행 파일이 열 수 있는 그대로.
    let old = rusqlite::Connection::open(db::backup_path(&paths.db_file(), 4)).unwrap();
    assert_eq!(db::user_version(&old).unwrap(), 4);
    let v: String = old
        .query_row("SELECT value FROM settings WHERE key = 'k'", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(v, "v4");
}

#[test]
fn schema6_fills_mine_exec() {
    let root = temp_root("schema6");
    let paths = Paths::under(&root);
    paths.ensure().unwrap();
    {
        // 스키마 5 이력: 보낸 대화(사람 글 → 답) · 받은 대화(사람 글 → 답).
        let conn = db::open(&paths.db_file()).unwrap();
        conn.execute_batch(
            "ALTER TABLE messages DROP COLUMN mine; ALTER TABLE messages DROP COLUMN exec; PRAGMA user_version = 5;
             INSERT INTO conversations (id, uuid, peer_id, agent, dir, created_at) VALUES (1, 'o', 1, 'a', 'out', 0), (2, 'i', 1, 'a', 'in', 0);
             INSERT INTO messages (id, uuid, conversation_id, author, text, state, created_at) VALUES
               (1, 'm1', 1, 'human', 'q', 'done', 0), (2, 'm2', 1, 'agent', 'a', 'done', 0),
               (3, 'm3', 2, 'human', 'q', 'done', 0), (4, 'm4', 2, 'agent', 'a', 'done', 0);",
        )
        .unwrap();
    }
    let conn = db::open(&paths.db_file()).unwrap();
    assert_eq!(db::user_version(&conn).unwrap(), 13);
    let flags = |conv| -> Vec<(bool, bool)> {
        an_core::chat::messages(&conn, conv)
            .unwrap()
            .iter()
            .map(|m| (m.mine, m.exec))
            .collect()
    };
    // 보낸 쪽: 내 사람 글(실행 요청) · 상대 에이전트 답. 받은 쪽: 상대 사람 글(실행 요청) · 내 에이전트 답.
    assert_eq!(flags(1), [(true, true), (false, false)]);
    assert_eq!(flags(2), [(false, true), (true, false)]);
}

#[test]
fn schema7_relay_columns() {
    let root = temp_root("schema7");
    let paths = Paths::under(&root);
    paths.ensure().unwrap();
    {
        let conn = db::open(&paths.db_file()).unwrap();
        conn.execute_batch(
            "ALTER TABLE conversations DROP COLUMN relay_blocked; ALTER TABLE conversations DROP COLUMN relay_on;
             PRAGMA user_version = 6;
             INSERT INTO conversations (id, uuid, peer_id, agent, dir, created_at) VALUES (1, 'o', 1, 'a', 'out', 0);",
        )
        .unwrap();
    }
    let conn = db::open(&paths.db_file()).unwrap();
    assert_eq!(db::user_version(&conn).unwrap(), 13);
    let c = an_core::chat::by_id(&conn, 1).unwrap().unwrap();
    assert_eq!(
        (c.relay, c.relay_blocked),
        (None, false),
        "이전 대화는 자동 왕복 아님"
    );
    an_core::chat::set_relay(&conn, 1, "mine", 3).unwrap();
    assert_eq!(an_core::chat::relay_step(&conn, 1).unwrap(), 1);
    an_core::chat::relay_on(&conn, 1, false, false).unwrap();
    let r = an_core::chat::by_id(&conn, 1)
        .unwrap()
        .unwrap()
        .relay
        .unwrap();
    assert_eq!(
        (r.agent.as_str(), r.cap, r.count, r.on),
        ("mine", 3, 1, false)
    );
    an_core::chat::relay_on(&conn, 1, true, true).unwrap();
    let r = an_core::chat::by_id(&conn, 1)
        .unwrap()
        .unwrap()
        .relay
        .unwrap();
    assert_eq!((r.count, r.on), (0, true), "재개는 카운트 0");
}

#[test]
fn schema8_via_column() {
    let root = temp_root("schema8");
    let paths = Paths::under(&root);
    paths.ensure().unwrap();
    {
        let conn = db::open(&paths.db_file()).unwrap();
        conn.execute_batch(
            "ALTER TABLE messages DROP COLUMN via; PRAGMA user_version = 7;
             INSERT INTO conversations (id, uuid, peer_id, agent, dir, created_at) VALUES (1, 'o', 1, 'a', 'out', 0);
             INSERT INTO messages (id, uuid, conversation_id, author, text, state, created_at, mine, exec, lc)
             VALUES (1, 'm', 1, 'human', 'x', 'done', 0, 1, 1, 1);",
        )
        .unwrap();
    }
    let conn = db::open(&paths.db_file()).unwrap();
    assert_eq!(db::user_version(&conn).unwrap(), 13);
    assert_eq!(
        an_core::chat::messages(&conn, 1).unwrap()[0].via,
        None,
        "이전 글은 웹UI(사람)"
    );
    an_core::chat::set_via(&conn, 1, Some("cli")).unwrap();
    assert_eq!(
        an_core::chat::messages(&conn, 1).unwrap()[0].via.as_deref(),
        Some("cli")
    );
}

#[test]
fn schema9_group_model_columns() {
    let root = temp_root("schema9");
    let paths = Paths::under(&root);
    paths.ensure().unwrap();
    {
        let conn = db::open(&paths.db_file()).unwrap();
        conn.execute_batch(
            "ALTER TABLE peers DROP COLUMN effort; ALTER TABLE peers DROP COLUMN model; ALTER TABLE peers DROP COLUMN grp;
             ALTER TABLE agents DROP COLUMN grp; PRAGMA user_version = 8;
             INSERT INTO peers (id, spki, name, host, addr, round, created_at) VALUES (1, x'01', 'p', 'h', '1.2.3.4:1', 'r', 0);
             INSERT INTO agents (id, name, folder, tools, created_at) VALUES (1, 'a', 'D:\\x', 'Read', 0);",
        )
        .unwrap();
    }
    let conn = db::open(&paths.db_file()).unwrap();
    assert_eq!(db::user_version(&conn).unwrap(), 13);
    assert!(
        db::backup_path(&paths.db_file(), 8).exists(),
        "올리기 전 사본"
    );
    let p = an_core::peers::by_id(&conn, 1).unwrap().unwrap();
    assert_eq!(
        (p.group, p.model, p.effort),
        (None, None, None),
        "이전 피어는 그룹 · 덮어쓰기 없음"
    );
    assert!(an_core::peers::set_meta(&conn, 1, Some("개발팀"), Some("opus"), Some("max")).unwrap());
    let p = an_core::peers::by_id(&conn, 1).unwrap().unwrap();
    assert_eq!(
        (p.group.as_deref(), p.model.as_deref(), p.effort.as_deref()),
        (Some("개발팀"), Some("opus"), Some("max"))
    );
    let a = an_core::agents::by_name(&conn, "a").unwrap().unwrap();
    assert_eq!(a.group, None);
    // 받은 대화의 세션 — 폴더가 바뀌면 비워진다(세션은 폴더에 묶여 있다). 같은 이름의 원격 에이전트 대화(out)는 그대로.
    use an_core::chat::{self, Dir};
    let inbound = chat::open_conversation(&conn, "u-in", 1, "a", Dir::In).unwrap();
    let outbound = chat::open_conversation(&conn, "u-out", 1, "a", Dir::Out).unwrap();
    for c in [&inbound, &outbound] {
        chat::set_session(&conn, c.id, "s1").unwrap();
    }
    let session = |id| chat::by_id(&conn, id).unwrap().unwrap().session;
    an_core::agents::update(&conn, a.id, "D:\\x", "Read", "", None).unwrap();
    assert_eq!(
        session(inbound.id).as_deref(),
        Some("s1"),
        "폴더 그대로면 유지"
    );
    let a = an_core::agents::update(&conn, a.id, "D:\\y", "Read,Grep", "지시", Some("문서"))
        .unwrap()
        .unwrap();
    assert_eq!(
        (
            a.name.as_str(),
            a.folder.as_str(),
            a.tools.as_str(),
            a.prompt.as_str(),
            a.group.as_deref()
        ),
        ("a", "D:\\y", "Read,Grep", "지시", Some("문서")),
        "이름은 그대로"
    );
    assert_eq!(session(inbound.id), None, "폴더가 바뀌면 새 세션");
    assert_eq!(session(outbound.id).as_deref(), Some("s1"));
    assert_eq!(
        an_core::agents::update(&conn, 999, "D:\\y", "Read", "", None).unwrap(),
        None
    );
}

#[test]
fn schema10_title_alias_icon() {
    let root = temp_root("schema10");
    let paths = Paths::under(&root);
    paths.ensure().unwrap();
    {
        let conn = db::open(&paths.db_file()).unwrap();
        conn.execute_batch(
            "ALTER TABLE conversations DROP COLUMN title; ALTER TABLE peers DROP COLUMN icon; ALTER TABLE peers DROP COLUMN alias;
             PRAGMA user_version = 9;
             INSERT INTO peers (id, spki, name, host, addr, round, created_at) VALUES (1, x'01', 'p', 'h', '1.2.3.4:1', 'r', 0);",
        )
        .unwrap();
    }
    let conn = db::open(&paths.db_file()).unwrap();
    assert_eq!(db::user_version(&conn).unwrap(), 13);
    assert!(
        db::backup_path(&paths.db_file(), 9).exists(),
        "올리기 전 사본"
    );
    use an_core::peers;
    let p = peers::by_id(&conn, 1).unwrap().unwrap();
    assert_eq!(
        (p.alias, p.icon),
        (None, None),
        "이전 피어는 표시 이름 · 아이콘 없음"
    );
    let icon = format!("{}AAAA", an_core::run_opts::ICON_PREFIX);
    assert!(peers::set_tile(&conn, 1, Some("세일즈"), Some(&icon)).unwrap());
    let p = peers::by_id(&conn, 1).unwrap().unwrap();
    assert_eq!(
        (p.alias.as_deref(), p.icon.as_deref(), p.name.as_str()),
        (Some("세일즈"), Some(icon.as_str()), "p"),
        "원래 이름은 그대로"
    );
    assert!(!peers::set_tile(&conn, 9, None, None).unwrap(), "없는 피어");
    // 다시 hello(upsert)해도 이 PC 의 표시는 남는다.
    peers::upsert(&conn, &[1u8], "p2", "h2", "1.2.3.4:2", "r2").unwrap();
    let p = peers::by_id(&conn, 1).unwrap().unwrap();
    assert_eq!(
        (p.alias.as_deref(), p.name.as_str()),
        (Some("세일즈"), "p2")
    );
    // 대화 제목 — 붙인 제목이 첫 사람 글 앞부분보다 먼저, 지우면 다시 앞부분.
    use an_core::chat::{self, Dir, State};
    let c = chat::open_conversation(&conn, "u1", 1, "guide", Dir::Out).unwrap();
    chat::add_human(&conn, c.id, "m1", &"배".repeat(50), State::Done, true, true).unwrap();
    let title = |conn: &db::Connection| chat::summaries(conn).unwrap()[0].title.clone();
    assert_eq!(
        title(&conn),
        "배".repeat(chat::TITLE_CHARS),
        "붙인 제목 없음 = 앞부분"
    );
    let long = "제".repeat(an_core::run_opts::MAX_TITLE_CHARS);
    assert!(chat::set_title(&conn, c.id, Some(&long)).unwrap());
    assert_eq!(
        title(&conn),
        long,
        "붙인 제목은 앞부분 길이로 자르지 않는다"
    );
    assert!(chat::set_title(&conn, c.id, None).unwrap());
    assert_eq!(title(&conn), "배".repeat(chat::TITLE_CHARS));
    assert!(
        !chat::set_title(&conn, 999, Some("x")).unwrap(),
        "없는 대화"
    );
}

/// 스키마 11 — ★ · 읽음 · 저장 · 답의 실행 옵션 · 내 활동을 본 마지막 항목 · 자동 왕복 기록 · 대화별 색인. 이전 대화는 다 읽은 것, 이전 글은
/// 본 것으로.
#[test]
fn schema11_workspace() {
    let root = temp_root("schema11");
    let paths = Paths::under(&root);
    paths.ensure().unwrap();
    {
        let conn = db::open(&paths.db_file()).unwrap();
        conn.execute_batch(
            "ALTER TABLE conversations DROP COLUMN starred_at; ALTER TABLE conversations DROP COLUMN read_id;
             ALTER TABLE messages DROP COLUMN saved_at; ALTER TABLE messages DROP COLUMN model;
             ALTER TABLE messages DROP COLUMN effort; ALTER TABLE peers DROP COLUMN activity_seen_msg;
             ALTER TABLE peers DROP COLUMN activity_seen_event; DROP INDEX messages_by_conv;
             DROP TABLE relay_events;
             PRAGMA user_version = 10;
             INSERT INTO peers (id, spki, name, host, addr, round, created_at) VALUES (1, x'01', 'p', 'h', '1.2.3.4:1', 'r', 0);
             INSERT INTO conversations (id, uuid, peer_id, agent, dir, created_at) VALUES (1, 'c1', 1, 'review', 'out', 0);
             INSERT INTO messages (id, uuid, conversation_id, author, text, state, created_at, mine, exec, lc)
               VALUES (1, 'm1', 1, 'human', '질문', 'done', 0, 1, 1, 1), (2, 'm2', 1, 'agent', '답', 'done', 0, 0, 0, 2);",
        )
        .unwrap();
    }
    let conn = db::open(&paths.db_file()).unwrap();
    assert_eq!(db::user_version(&conn).unwrap(), db::SCHEMA_VERSION);
    assert!(
        db::backup_path(&paths.db_file(), 10).exists(),
        "올리기 전 사본"
    );
    use an_core::chat;
    let s = chat::summaries(&conn).unwrap().remove(0);
    assert_eq!(
        (s.starred, s.unread),
        (false, 0),
        "이전 대화는 다 읽은 것으로"
    );
    let seen: (Option<i64>, Option<i64>) = conn
        .query_row(
            "SELECT activity_seen_msg, activity_seen_event FROM peers WHERE id = 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!(
        seen,
        (Some(2), Some(0)),
        "내 활동은 올린 때의 마지막 글 · 기록까지 본 것으로"
    );
    assert_eq!(
        chat::activity_unseen(&conn, None).unwrap(),
        0,
        "이전 글은 본 것으로"
    );
    // 본 뒤 실패로 바뀐 내 글 — 글 id 는 그대로라 실패 기록(`relay_events.message_id`)으로 다시 센다, 한 번만.
    chat::set_state(&conn, 1, chat::State::Error).unwrap();
    chat::set_state(&conn, 1, chat::State::Error).unwrap();
    assert_eq!(chat::activity_unseen(&conn, None).unwrap(), 1, "늦은 실패");
    let m = chat::messages(&conn, 1).unwrap().remove(1);
    assert_eq!((m.saved_at, m.model, m.effort), (None, None, None));
    chat::add_relay_event(&conn, 1, chat::RelayMark::End, "상한 도달").unwrap();
    let items = chat::activity(&conn, None, chat::ActivityFilter::Relay, None, 10).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        (
            items[0].kind,
            items[0].detail.as_deref(),
            items[0].conversation_id
        ),
        (chat::ActivityKind::RelayEnd, Some("상한 도달"), 1)
    );
    // 대화별 글 · 자동 왕복 기록 색인 — 5초 새로 고침의 대화 목록 · 안 읽음 · 내 활동이 표 전체를 훑지 않게.
    let names: Vec<String> = conn
        .prepare(
            "SELECT name FROM sqlite_master WHERE type = 'index'
               AND name IN ('messages_by_conv', 'relay_events_by_conv', 'relay_events_by_message') ORDER BY name",
        )
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    assert_eq!(
        names,
        [
            "messages_by_conv",
            "relay_events_by_conv",
            "relay_events_by_message"
        ]
    );
}

/// 스키마 12 — 질의 · 요청 · 완료 · 멘션 · 거절 사유(글) · 접속 상태(상대) · 멘션한 에이전트 세션 표. 이전 글은 질의 · 대화 에이전트, 이전 상대는 접속
/// 기록 없음. 집중 설정 · 접속 기록 · 에이전트 세션의 읽고 쓰기, 폴더가 바뀌면 그 에이전트의 세션을 두 곳 모두 지운다.
#[test]
fn schema12_focus_mention() {
    let root = temp_root("schema12");
    let paths = Paths::under(&root);
    paths.ensure().unwrap();
    {
        let conn = db::open(&paths.db_file()).unwrap();
        conn.execute_batch(
            "ALTER TABLE messages DROP COLUMN request; ALTER TABLE messages DROP COLUMN done_at;
             ALTER TABLE messages DROP COLUMN agent; ALTER TABLE messages DROP COLUMN refused;
             ALTER TABLE peers DROP COLUMN seen_at; ALTER TABLE peers DROP COLUMN focus;
             DROP TABLE agent_sessions; ALTER TABLE agents DROP COLUMN uid;
             PRAGMA user_version = 11;
             INSERT INTO agents (id, name, folder, tools, prompt, created_at) VALUES (5, 'old', '/o', 'Read', '', 0);
             INSERT INTO peers (id, spki, name, host, addr, round, created_at) VALUES (1, x'01', 'p', 'h', '1.2.3.4:1', 'r', 0);
             INSERT INTO conversations (id, uuid, peer_id, agent, dir, session, created_at) VALUES (1, 'c1', 1, 'review', 'in', 's1', 0);
             INSERT INTO messages (id, uuid, conversation_id, author, text, state, created_at, mine, exec, lc)
               VALUES (1, 'm1', 1, 'human', '질문', 'done', 0, 0, 1, 1);",
        )
        .unwrap();
    }
    let conn = db::open(&paths.db_file()).unwrap();
    assert_eq!(db::user_version(&conn).unwrap(), 13);
    assert!(
        db::backup_path(&paths.db_file(), 11).exists(),
        "올리기 전 사본"
    );
    use an_core::{agents, chat, peers, settings};
    let m = chat::messages(&conn, 1).unwrap().remove(0);
    assert_eq!(
        (
            m.request,
            m.done_at,
            m.agent.as_deref(),
            m.refused.as_deref()
        ),
        (false, None, None, None),
        "이전 글 = 질의 · 대화 에이전트"
    );
    let p = peers::by_id(&conn, 1).unwrap().unwrap();
    assert_eq!((p.seen_at, p.focus), (None, false));
    // 이전 에이전트에도 세대(uid)를 채운다.
    let old = agents::by_name(&conn, "old").unwrap().unwrap();
    assert_eq!(old.uid.len(), 32, "{old:?}");
    assert!(peers::set_presence(&conn, 1, true).unwrap());
    let p = peers::by_id(&conn, 1).unwrap().unwrap();
    assert!(p.seen_at.is_some_and(|t| t > 0) && p.focus, "{p:?}");
    assert!(!peers::set_presence(&conn, 9, true).unwrap(), "없는 상대");
    // 집중 — 이 PC 설정, `1` 만 켜짐.
    assert!(!settings::focus(&conn).unwrap());
    settings::set(&conn, settings::FOCUS_KEY, "1").unwrap();
    assert!(settings::focus(&conn).unwrap());
    settings::set(&conn, settings::FOCUS_KEY, "0").unwrap();
    assert!(!settings::focus(&conn).unwrap());
    // 멘션한 에이전트의 세션은 따로, 대화 에이전트 세션(`conversations.session`)은 그대로. 폴더가 바뀌면 둘 다 지운다.
    let review = agents::add(&conn, "review", "/a", None, "").unwrap();
    agents::add(&conn, "lint", "/b", None, "").unwrap();
    assert_eq!(chat::agent_session(&conn, 1, "lint").unwrap(), None);
    chat::set_agent_session(&conn, 1, "lint", "s2").unwrap();
    chat::set_agent_session(&conn, 1, "review", "s3").unwrap();
    assert_eq!(
        chat::agent_session(&conn, 1, "lint").unwrap().as_deref(),
        Some("s2")
    );
    assert_eq!(
        chat::by_id(&conn, 1).unwrap().unwrap().session.as_deref(),
        Some("s1")
    );
    let moved = agents::update(&conn, review.id, "/c", &review.tools, "", None)
        .unwrap()
        .unwrap();
    assert_ne!(moved.uid, review.uid, "폴더를 바꾸면 새 세대");
    let same = agents::update(&conn, review.id, "/c", &review.tools, "지시", None)
        .unwrap()
        .unwrap();
    assert_eq!(same.uid, moved.uid, "폴더 그대로면 세대 그대로");
    assert_eq!(chat::by_id(&conn, 1).unwrap().unwrap().session, None);
    assert_eq!(chat::agent_session(&conn, 1, "review").unwrap(), None);
    assert_eq!(
        chat::agent_session(&conn, 1, "lint").unwrap().as_deref(),
        Some("s2"),
        "다른 에이전트 세션은 그대로"
    );
    // 지워도 그 에이전트 세션을 비운다(같은 이름을 다시 등록해도 옛 세션을 잇지 않게).
    let lint = agents::by_name(&conn, "lint").unwrap().unwrap();
    assert!(agents::remove(&conn, lint.id).unwrap());
    assert_eq!(chat::agent_session(&conn, 1, "lint").unwrap(), None);
    assert!(!agents::remove(&conn, lint.id).unwrap(), "없는 id");
}

/// 옛 루트 — DB · 신원 · 로그 · 잠금.
fn legacy_root(base: &std::path::Path) -> PathBuf {
    let old = base.join("agentNetwork");
    std::fs::create_dir_all(old.join("logs")).unwrap();
    std::fs::write(old.join("agentNetwork.db"), b"db").unwrap();
    std::fs::write(old.join("identity.key"), b"key").unwrap();
    std::fs::write(old.join("logs").join("agentNetwork.log"), b"log").unwrap();
    std::fs::write(old.join("instance.lock"), b"").unwrap();
    old
}

#[test]
fn legacy_data_adopted() {
    let base = temp_root("legacy");
    let old = legacy_root(&base);
    let paths = Paths::under(base.join("coworkAgent"));
    assert!(paths.adopt_legacy(&old).unwrap(), "옮겼다");
    assert!(!old.exists(), "옛 루트는 지운다");
    assert_eq!(std::fs::read(paths.db_file()).unwrap(), b"db");
    assert_eq!(
        std::fs::read(paths.identity_file()).unwrap(),
        b"key",
        "신원 그대로"
    );
    assert_eq!(std::fs::read(paths.log_file()).unwrap(), b"log");
    assert!(!paths.lock_file().exists(), "잠금 파일은 옮기지 않는다");
    // 다시 불러도 그대로.
    assert!(!paths.adopt_legacy(&old).unwrap());
    // 이전 판(폴더째 옮기기)이 남긴 옛 DB 이름 — 마저 바꾼다.
    std::fs::rename(paths.db_file(), paths.root().join("agentNetwork.db")).unwrap();
    paths.adopt_legacy(&old).unwrap();
    assert_eq!(std::fs::read(paths.db_file()).unwrap(), b"db");
}

#[test]
fn legacy_with_empty_new_root_adopted() {
    // 실패 뒤 남은 빈 새 루트(로그만) — 옛 데이터를 그리로 옮긴다.
    let base = temp_root("legacy-empty");
    let old = legacy_root(&base);
    let paths = Paths::under(base.join("coworkAgent"));
    paths.ensure().unwrap();
    std::fs::write(paths.log_file(), b"new log").unwrap();
    assert!(paths.adopt_legacy(&old).unwrap());
    assert_eq!(std::fs::read(paths.identity_file()).unwrap(), b"key");
    assert_eq!(
        std::fs::read(paths.log_file()).unwrap(),
        b"new log",
        "있는 것은 덮어쓰지 않는다"
    );
}

#[test]
fn legacy_conflict_and_running_refused() {
    // 두 루트 모두 데이터 — 고르지 않고 오류, 아무것도 안 바뀐다.
    let base = temp_root("legacy-conflict");
    let old = legacy_root(&base);
    let paths = Paths::under(base.join("coworkAgent"));
    paths.ensure().unwrap();
    std::fs::write(paths.identity_file(), b"new key").unwrap();
    assert!(paths.adopt_legacy(&old).is_err());
    assert_eq!(std::fs::read(old.join("identity.key")).unwrap(), b"key");
    assert_eq!(std::fs::read(paths.identity_file()).unwrap(), b"new key");
    // 옛 앱 실행 중(옛 잠금을 쥠) — 아무것도 안 바뀐다.
    let base = temp_root("legacy-running");
    let old = legacy_root(&base);
    let held = an_core::instance::InstanceLock::acquire(&old.join("instance.lock")).unwrap();
    assert!(matches!(held, an_core::instance::Acquired::Primary(_)));
    let paths = Paths::under(base.join("coworkAgent"));
    let err = paths.adopt_legacy(&old).unwrap_err();
    assert!(err.to_string().contains("실행 중"), "{err}");
    assert!(old.join("agentNetwork.db").exists() && !paths.db_file().exists());
    drop(held);
    assert!(paths.adopt_legacy(&old).unwrap(), "끄면 옮긴다");
}
