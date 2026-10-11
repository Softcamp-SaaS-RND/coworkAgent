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
    assert_eq!(db::SCHEMA_VERSION, 7);
    {
        // 00 이 남긴 스키마 1 파일 — 열면 2 로 오른다.
        let conn = rusqlite::Connection::open(paths.db_file()).unwrap();
        conn.pragma_update(None, "user_version", 1).unwrap();
    }
    {
        let conn = db::open(&paths.db_file()).unwrap();
        assert_eq!(db::user_version(&conn).unwrap(), 7);
        assert!(an_core::peers::list(&conn).unwrap().is_empty());
        assert_eq!(an_core::settings::get(&conn, "x").unwrap(), None);
    }
    let conn = db::open(&paths.db_file()).unwrap();
    assert_eq!(db::user_version(&conn).unwrap(), 7);
    conn.pragma_update(None, "user_version", 8).unwrap();
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
    assert_eq!(db::user_version(&conn).unwrap(), 7);
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
    assert_eq!(db::user_version(&conn).unwrap(), 7);
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
    assert_eq!(db::user_version(&conn).unwrap(), 7);
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
