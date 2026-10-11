mod common;
use an_core::peers;
use an_net::node::{Me, Node};
use an_net::proto::Response;
use std::time::Duration;

/// 처음 보는 임의 키의 hello — 수신 없이 보내기만 하는 노드.
async fn stranger_hello(to: std::net::SocketAddr, code: Option<String>) -> Response {
    let n = Node::new(
        common::bare_identity(),
        an_core::db::Connection::open_in_memory().unwrap(),
        Me {
            name: "n".into(),
            host: "h".into(),
            port: 1,
        },
    );
    n.send(to, None, &common::hello("n", "h", 1, code))
        .await
        .unwrap()
        .0
}

#[tokio::test]
async fn unknown_hello_closed_without_window() {
    let b = common::start("g-closed-b").await;
    // 창 없이 처음 보는 키 40개 — 하나도 저장되지 않는다(슬롯 고갈 불가).
    for _ in 0..40 {
        assert_eq!(
            stranger_hello(b.addr, None).await,
            Response::error("closed")
        );
    }
    assert!(b.node.peers().unwrap().is_empty());
    // 그 뒤에도 정상 초대는 진행된다.
    let a = common::start("g-closed-a").await;
    let code = b.open();
    a.node.add(b.addr, code).await.unwrap();
    assert_eq!(b.only_peer().spki, a.node.identity().spki());
}

#[tokio::test]
async fn window_accepts_one_then_closes() {
    let b = common::start("g-one-b").await;
    let a = common::start("g-one-a").await;
    let code = b.open();
    a.node.add(b.addr, code.clone()).await.unwrap();
    // 창은 1건으로 닫힌다 — 같은 코드를 다시 써도.
    assert_eq!(
        stranger_hello(b.addr, code).await,
        Response::error("closed")
    );
    assert_eq!(b.node.peers().unwrap().len(), 1);
    // 아는 키의 hello(다시 추가)는 창과 무관.
    a.node.add(b.addr, None).await.unwrap();
    assert_eq!(b.node.peers().unwrap().len(), 1);
}

#[tokio::test]
async fn window_expires() {
    let b = common::start("g-exp-b").await;
    let code = b.node.open_window(Duration::from_millis(50));
    tokio::time::sleep(Duration::from_millis(150)).await;
    assert_eq!(
        stranger_hello(b.addr, Some(code)).await,
        Response::error("closed")
    );
    assert!(b.node.peers().unwrap().is_empty());
}

#[tokio::test]
async fn lost_remove_readd_converges() {
    let (a, b) = (
        common::start("g-lost-a").await,
        common::start("g-lost-b").await,
    );
    let code = b.open();
    let pb = a.node.add(b.addr, code).await.unwrap();
    let pa = b.only_peer();
    a.node.approve(pb.id).await.unwrap();
    b.node.approve(pa.id).await.unwrap();

    // B 가 A 를 지웠지만 통지가 유실됐다 — A 는 아직 "페어링됨".
    peers::remove(&b.db(), pa.id).unwrap();
    assert!(a.only_peer().paired());

    // B 가 A 를 다시 추가 → A 쪽 기록도 승인이 지워져 양쪽이 같은 출발점.
    let pa = b.node.add(a.addr, None).await.unwrap();
    let pb = a.only_peer();
    assert!(!pb.my_ok && !pb.their_ok, "{pb:?}");
    assert!(!pa.my_ok && !pa.their_ok, "{pa:?}");
    b.node.approve(pa.id).await.unwrap();
    a.node.approve(pb.id).await.unwrap();
    assert!(a.only_peer().paired() && b.only_peer().paired());
    a.node.ping(pb.id).await.unwrap();
    b.node.ping(pa.id).await.unwrap();

    // 통지가 닿지 않는 삭제는 `false` 로 드러난다(내 쪽은 삭제).
    let gone = common::start("g-lost-c").await;
    let id = peers::upsert(&gone.db(), b"dead-key", "x", "x", "127.0.0.1:1", "r")
        .unwrap()
        .id;
    assert!(!gone.node.remove(id).await.unwrap());
    assert!(gone.node.peers().unwrap().is_empty());
    assert!(a.node.remove(pb.id).await.unwrap(), "닿는 통지는 true");
}
