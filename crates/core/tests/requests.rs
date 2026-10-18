//! #1d — 요청 목록 저장(스키마 14 · 상태 옮김 · 메모 · 거르기).
use an_core::db;

fn path(tag: &str) -> std::path::PathBuf {
    let p = std::env::temp_dir().join(format!("an-core-req-{tag}-{}.db", std::process::id()));
    let _ = std::fs::remove_file(&p);
    p
}

#[test]
fn schema14_moves_done_and_models() {
    let p = path("mig");
    {
        // 스키마 13 이 남긴 파일 — 요청 상태 열이 없고 완료 표시는 `done_at`, 모델은 아무 글자.
        let conn = db::open(&p).unwrap();
        conn.execute_batch(
            "ALTER TABLE messages DROP COLUMN req_state; ALTER TABLE messages DROP COLUMN req_note;
             ALTER TABLE messages DROP COLUMN req_at; PRAGMA user_version = 13;
             INSERT INTO peers (id, spki, name, host, addr, round, created_at, model) VALUES (1, x'01', 'p', 'h', '1.2.3.4:1', 'r', 0, 'sonnet[1m]');
             INSERT INTO peers (id, spki, name, host, addr, round, created_at, model) VALUES (2, x'02', 'q', 'h', '1.2.3.4:2', 'r', 0, 'opus');
             INSERT INTO settings (key, value) VALUES ('claude_model', 'claude-sonnet-5-5');
             INSERT INTO conversations (id, uuid, peer_id, agent, dir, created_at) VALUES (1, 'c', 1, 'a', 'in', 0);
             INSERT INTO messages (id, uuid, conversation_id, author, text, state, created_at, request, done_at)
               VALUES (1, 'm1', 1, 'human', '고쳐 주세요', 'done', 0, 1, 77),
                      (2, 'm2', 1, 'human', '이것도', 'done', 0, 1, NULL),
                      (3, 'm3', 1, 'human', '질의', 'done', 0, 0, NULL);
             INSERT INTO outbox (id, peer_id, kind, frame, created_at) VALUES
               (1, 1, 'done', '{\"type\":\"done\",\"conv\":\"c\",\"msg\":\"m1\",\"done\":true}', 0),
               (2, 1, 'done', '{\"type\":\"done\",\"conv\":\"c\",\"msg\":\"m2\",\"done\":false}', 0),
               (3, 1, 'note', '{\"type\":\"post\",\"msg\":\"m3\"}', 0);",
        )
        .unwrap();
    }
    let conn = db::open(&p).unwrap();
    assert_eq!(db::user_version(&conn).unwrap(), db::SCHEMA_VERSION);
    let states: Vec<(i64, Option<String>, Option<i64>)> = conn
        .prepare("SELECT id, req_state, req_at FROM messages ORDER BY id")
        .unwrap()
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    assert_eq!(
        states,
        [
            (1, Some("done".into()), Some(77)),
            (2, Some("wait".into()), None),
            (3, None, None)
        ],
        "완료 표시 → 완료 · 나머지 요청 → 대기 · 질의는 없음"
    );
    let model = |id: i64| an_core::peers::by_id(&conn, id).unwrap().unwrap().model;
    assert_eq!(
        (model(1), model(2)),
        (None, Some("opus".into())),
        "목록 밖 상대별 모델은 기본값"
    );
    assert_eq!(an_core::settings::get(&conn, "claude_model").unwrap(), None);
    assert_eq!(
        an_core::run_opts::defaults(&conn).unwrap().0,
        "sonnet",
        "전역 목록 밖 → 기본"
    );
    // 아직 못 보낸 옛 완료 알림은 같은 자리에서 ReqState 프레임으로(완료 → done · 취소 → wait), 그 요청 글에 잇는다(#1d R1).
    type Row = (i64, String, Option<i64>, String, Option<String>);
    let rows: Vec<Row> = conn
        .prepare(
            "SELECT id, kind, msg_id, json_extract(frame, '$.type') || ':' || json_extract(frame, '$.msg') || ':' ||
               coalesce(json_extract(frame, '$.state'), ''), json_type(frame, '$.note') FROM outbox ORDER BY id",
        )
        .unwrap()
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    assert_eq!(
        rows,
        [
            (
                1,
                "req_state".into(),
                Some(1),
                "req_state:m1:done".into(),
                Some("null".into())
            ),
            (
                2,
                "req_state".into(),
                Some(2),
                "req_state:m2:wait".into(),
                Some("null".into())
            ),
            (3, "note".into(), None, "post:m3:".into(), None),
        ]
    );
}

#[test]
fn transitions_only_forward() {
    use an_core::chat::ReqState::{self, *};
    let allowed = [(Wait, Accepted), (Wait, Rejected), (Accepted, Done)];
    for from in ReqState::ALL {
        for to in ReqState::ALL {
            assert_eq!(
                from.can_become(to),
                allowed.contains(&(from, to)),
                "{from:?} → {to:?}"
            );
        }
    }
    for s in ReqState::ALL {
        assert_eq!(ReqState::parse(s.as_str()), Some(s));
    }
    assert_eq!(ReqState::parse("Wait"), None);
}

#[test]
fn note_one_line_200() {
    use an_core::chat::{MAX_REQ_NOTE, clean_req_note};
    assert_eq!(clean_req_note("  "), Ok(None));
    assert_eq!(clean_req_note(" 확인 ").unwrap().as_deref(), Some("확인"));
    assert!(clean_req_note(&"가".repeat(MAX_REQ_NOTE)).is_ok());
    for bad in [
        "가".repeat(MAX_REQ_NOTE + 1),
        "두\n줄".into(),
        "탭\t가".into(),
    ] {
        assert!(clean_req_note(&bad).is_err(), "{bad:?}");
    }
}

#[test]
fn list_filters_by_kind_and_state() {
    use an_core::chat::{self, AskFilter, ReqState, State};
    let conn = db::open(&path("list")).unwrap();
    conn.execute_batch(
        "INSERT INTO peers (id, spki, name, host, addr, round, created_at) VALUES (1, x'01', 'p', 'h', '1.2.3.4:1', 'r', 0);
         INSERT INTO conversations (id, uuid, peer_id, agent, dir, created_at) VALUES (1, 'c', 1, 'a', 'in', 0);",
    )
    .unwrap();
    let add = |uuid: &str, text: &str, request: bool| {
        let m = chat::add_human(&conn, 1, uuid, text, State::Done, false, false).unwrap();
        chat::set_request(&conn, m.id, request).unwrap();
        m.id
    };
    let r1 = add("r1", "요청 하나", true);
    add("r2", "요청 둘", true);
    add("q1", "질의", false);
    chat::set_req_state(&conn, r1, ReqState::Accepted, Some("곧")).unwrap();
    let texts = |ask| {
        chat::messages_across(
            &conn,
            Some(false),
            chat::Scope::All,
            None,
            Some(ask),
            None,
            10,
        )
        .unwrap()
        .into_iter()
        .map(|m| m.text)
        .collect::<Vec<_>>()
    };
    assert_eq!(texts(AskFilter::Request(None)), ["요청 둘", "요청 하나"]);
    assert_eq!(texts(AskFilter::Request(Some(ReqState::Wait))), ["요청 둘"]);
    assert_eq!(
        texts(AskFilter::Request(Some(ReqState::Accepted))),
        ["요청 하나"]
    );
    assert!(texts(AskFilter::Request(Some(ReqState::Done))).is_empty());
    assert_eq!(texts(AskFilter::Query), ["질의"]);
    let m = chat::message_by_id(&conn, r1).unwrap().unwrap();
    assert_eq!(
        (m.req_state, m.req_note.as_deref()),
        (Some(ReqState::Accepted), Some("곧"))
    );
    assert!(m.req_at.is_some());
    // 질의로 바꾸면 상태가 없다.
    chat::set_request(&conn, r1, false).unwrap();
    assert_eq!(
        chat::message_by_id(&conn, r1).unwrap().unwrap().req_state,
        None
    );
}

/// 요청 상태별 건수는 범위 전체(#1q 적대적 평가 R1) — 한 쪽 목록(최대 500)을 넘는 요청도 세고, 상태로 거르면 오래된 대기 요청도 나온다.
#[test]
fn req_counts_beyond_page() {
    use an_core::chat::{self, AskFilter, ReqState};
    let conn = db::open(&path("counts")).unwrap();
    conn.execute_batch(
        "INSERT INTO peers (id, spki, name, host, addr, round, created_at) VALUES (1, x'01', 'p', 'h', '1.2.3.4:1', 'r', 0);
         INSERT INTO conversations (id, uuid, peer_id, agent, dir, created_at) VALUES (1, 'c', 1, 'a', 'in', 0), (2, 'o', 1, 'a', 'out', 0);
         INSERT INTO messages (uuid, conversation_id, author, text, state, created_at, mine, request, req_state)
           VALUES ('old-wait', 1, 'human', '오래된 대기', 'done', 1, 0, 1, 'wait');
         WITH RECURSIVE n(i) AS (SELECT 1 UNION ALL SELECT i + 1 FROM n WHERE i < 501)
         INSERT INTO messages (uuid, conversation_id, author, text, state, created_at, mine, request, req_state)
           SELECT 'done-' || i, 1, 'human', '완료 ' || i, 'done', 100 + i, 0, 1, 'done' FROM n;
         INSERT INTO messages (uuid, conversation_id, author, text, state, created_at, mine, request, req_state)
           VALUES ('mine-acc', 2, 'human', '내 요청', 'done', 5, 1, 1, 'accepted'),
                  ('query', 1, 'human', '질의', 'done', 6, 0, 0, NULL);",
    )
    .unwrap();
    let c = |mine| chat::req_counts(&conn, mine, chat::Scope::All).unwrap();
    assert_eq!(
        c(false),
        chat::ReqCounts { wait: 1, accepted: 0, rejected: 0, done: 501 },
        "질의는 세지 않는다"
    );
    assert_eq!(c(false).total(), 502);
    assert_eq!(c(true), chat::ReqCounts { wait: 0, accepted: 1, rejected: 0, done: 0 });
    assert_eq!(chat::req_counts(&conn, false, chat::Scope::Peer(9)).unwrap().total(), 0);
    let waits = chat::messages_across(
        &conn,
        Some(false),
        chat::Scope::All,
        None,
        Some(AskFilter::Request(Some(ReqState::Wait))),
        None,
        500,
    )
    .unwrap();
    assert_eq!(waits.iter().map(|m| m.uuid.as_str()).collect::<Vec<_>>(), ["old-wait"]);
}
