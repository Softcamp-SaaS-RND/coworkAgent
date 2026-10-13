//! 보내는 쪽 보관(#1c) — 행 추가 · 키 합침 · 상대 줄 맨 앞 · 재시도 · 실패 보관.
use an_core::{db, outbox};

fn conn(tag: &str) -> db::Connection {
    let dir = std::env::temp_dir().join(format!("an-core-outbox-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    db::open(&dir.join("t.db")).unwrap()
}

#[test]
fn add_and_head_per_peer_in_order() {
    let c = conn("head");
    let a1 = outbox::add(&c, 1, Some(1), Some(10), "message", "{}", None, "pending").unwrap();
    let b1 = outbox::add(&c, 2, Some(2), None, "done", "{}", None, "pending").unwrap();
    let a2 = outbox::add(&c, 1, Some(1), Some(11), "reply", "{}", None, "pending").unwrap();
    assert_eq!(outbox::head(&c, 1).unwrap().unwrap().id, a1);
    assert_eq!(outbox::head(&c, 2).unwrap().unwrap().id, b1);
    assert!(outbox::has_before(&c, 1, a2, &["progress"]).unwrap());
    assert!(!outbox::has_before(&c, 1, a1, &["progress"]).unwrap());
    // 앞질러도 되는 종류 · 다른 대화는 세지 않는다.
    assert!(!outbox::has_before(&c, 2, b1 + 100, &["done"]).unwrap());
    assert!(!outbox::has_before(&c, 9, a2 + 100, &[]).unwrap());
    outbox::remove(&c, a1).unwrap();
    assert_eq!(outbox::head(&c, 1).unwrap().unwrap().id, a2, "ack 뒤 다음 행");
}

#[test]
fn same_key_pending_is_replaced() {
    let c = conn("key");
    let first = outbox::add(&c, 1, Some(1), Some(5), "progress", "{\"v\":1}", Some("p:5"), "pending").unwrap();
    let again = outbox::add(&c, 1, Some(1), Some(5), "progress", "{\"v\":2}", Some("p:5"), "pending").unwrap();
    assert_eq!(first, again);
    assert_eq!(outbox::get(&c, first).unwrap().unwrap().frame, "{\"v\":2}");
    // 보내는 중인 행은 바꾸지 않는다 — 새 행.
    outbox::set_state(&c, first, "inflight").unwrap();
    let third = outbox::add(&c, 1, Some(1), Some(5), "progress", "{\"v\":3}", Some("p:5"), "pending").unwrap();
    assert_ne!(third, first);
}

#[test]
fn retry_adds_wait_and_returns_to_queue() {
    let c = conn("retry");
    let id = outbox::add(&c, 1, Some(1), Some(1), "message", "{}", None, "inflight").unwrap();
    // 기다린 합 = 지난 시도부터 실제로 흐른 시간(직전 간격까지) — 곧바로 다시 실패하면 거의 늘지 않는다.
    assert_eq!(outbox::retry(&c, id, 5, 5, "연결 거부").unwrap(), 0);
    assert!(outbox::retry(&c, id, 10, 5, "연결 거부").unwrap() <= 1);
    c.execute("UPDATE outbox SET last_try = last_try - 100 WHERE id = ?1", [id]).unwrap();
    let waited = outbox::retry(&c, id, 10, 10, "연결 거부").unwrap();
    assert!((10..=11).contains(&waited), "간격까지만 센다(앱을 꺼 둔 시간 제외): {waited}");
    let it = outbox::get(&c, id).unwrap().unwrap();
    assert_eq!((it.state.as_str(), it.tries), ("pending", 3));
    assert_eq!(it.last_error.as_deref(), Some("연결 거부"));
    outbox::hurry(&c, 1).unwrap();
    let now: i64 = c.query_row("SELECT unixepoch()", [], |r| r.get(0)).unwrap();
    assert!(outbox::get(&c, id).unwrap().unwrap().next_at <= now);
}

#[test]
fn failed_rows_leave_the_queue_but_stay_visible() {
    let c = conn("fail");
    let id = outbox::add(&c, 1, Some(1), Some(7), "message", "{}", None, "pending").unwrap();
    outbox::fail(&c, id, "24시간 닿지 않음").unwrap();
    assert!(outbox::head(&c, 1).unwrap().is_none());
    assert_eq!(outbox::for_message(&c, 7).unwrap().unwrap().state, "failed");
    outbox::clear_failed(&c, 7).unwrap();
    assert!(outbox::for_message(&c, 7).unwrap().is_none());
}

#[test]
fn restart_returns_inflight_to_pending() {
    let c = conn("restart");
    let id = outbox::add(&c, 3, Some(3), None, "reply", "{}", None, "inflight").unwrap();
    outbox::add(&c, 4, Some(4), None, "done", "{}", None, "pending").unwrap();
    assert_eq!(outbox::pending_peers(&c).unwrap(), vec![3, 4]);
    assert_eq!(outbox::get(&c, id).unwrap().unwrap().state, "pending");
}
