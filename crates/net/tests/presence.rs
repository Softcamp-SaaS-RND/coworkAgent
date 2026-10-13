//! 접속 상태 = 통신할 때만(#1c A8 · A9) — 주기 확인 없음 · 보내기 · 받기가 접속 시각 갱신 · 시작 알림으로 바뀐 포트를 따라감 · 화면 확인은 60초 안 중복 없음.
mod common;
use an_core::{db, peers};
use an_net::identity::Identity;
use an_net::node::{Me, Node};
use an_net::proto::Request;
use std::time::{Duration, Instant};
use tokio::net::TcpListener;

async fn paired(a: &common::Running, b: &common::Running) -> (i64, i64) {
    let pb = a.node.add(b.addr, b.open()).await.unwrap();
    let pa = b.only_peer();
    a.node.approve(pb.id).await.unwrap();
    b.node.approve(pa.id).await.unwrap();
    (pb.id, pa.id)
}

fn forget_seen(s: &common::Running, id: i64) {
    s.db()
        .execute("UPDATE peers SET seen_at = NULL WHERE id = ?1", [id])
        .unwrap();
}

#[tokio::test]
async fn idle_pair_opens_no_connections() {
    let (a, b) = (common::start("idle-a").await, common::start("idle-b").await);
    paired(&a, &b).await;
    tokio::spawn(a.node.clone().announce());
    tokio::spawn(b.node.clone().announce());
    // 시작 알림이 끝난 뒤 3초 — 어느 쪽에도 피어 연결이 생기지 않는다(30초 주기 확인 없음).
    tokio::time::sleep(Duration::from_millis(800)).await;
    let until = Instant::now() + Duration::from_secs(3);
    while Instant::now() < until {
        assert_eq!(
            (a.node.open_connections(), b.node.open_connections()),
            (0, 0)
        );
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

#[tokio::test]
async fn sending_and_receiving_touch_seen_at() {
    let (a, b) = (common::start("tch-a").await, common::start("tch-b").await);
    let (b_id, a_id) = paired(&a, &b).await;
    forget_seen(&a, b_id);
    forget_seen(&b, a_id);
    // 보내기 성공(응답을 받음 — 처리기가 없어 `unsupported` 여도 통신이다).
    let pb = peers::by_id(&a.db(), b_id).unwrap().unwrap();
    a.node
        .send(b.addr, Some(pb.spki.clone()), &Request::Agents)
        .await
        .unwrap();
    assert!(a.only_peer().seen_at.is_some(), "보낸 쪽");
    // 받기 — Ping 을 받은 쪽도 보낸 상대의 접속 시각을 남긴다.
    forget_seen(&b, a_id);
    a.node.ping(b_id).await.unwrap();
    assert!(b.only_peer().seen_at.is_some(), "받은 쪽");
}

#[tokio::test]
async fn announce_lets_peer_follow_new_port() {
    let (a, b) = (common::start("ann-a").await, common::start("ann-b").await);
    let (b_id, _) = paired(&a, &b).await;
    let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
    let new = listener.local_addr().unwrap();
    let b2 = Node::new(
        Identity::load_or_create(&b.paths.identity_file()).unwrap(),
        db::open(&b.paths.db_file()).unwrap(),
        Me {
            name: "user-ann-b".into(),
            host: "PC-ann-b".into(),
            port: new.port(),
        },
    );
    tokio::spawn(b2.clone().serve(listener));
    tokio::spawn(b2.announce());
    let until = Instant::now() + Duration::from_secs(5);
    loop {
        let pb = peers::by_id(&a.db(), b_id).unwrap().unwrap();
        if pb.addr == new.to_string() {
            assert!(pb.paired(), "승인 그대로");
            break;
        }
        assert!(Instant::now() < until, "새 주소를 따라가지 않았다: {}", pb.addr);
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}

#[tokio::test]
async fn refresh_skips_when_seen_recently() {
    let (a, b) = (common::start("rf-a").await, common::start("rf-b").await);
    let (b_id, _) = paired(&a, &b).await;
    a.node.ping(b_id).await.unwrap();
    // 상대 주소를 닫힌 포트로 — 보냈다면 실패했을 것이다.
    let closed = std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap();
    a.db()
        .execute(
            "UPDATE peers SET addr = ?1 WHERE id = ?2",
            (closed.to_string(), b_id),
        )
        .unwrap();
    assert!(a.node.refresh(b_id, 60).await.is_ok(), "60초 안 — 보내지 않는다");
    assert!(a.node.refresh(b_id, 0).await.is_err(), "기한이 지나면 보낸다");
}
