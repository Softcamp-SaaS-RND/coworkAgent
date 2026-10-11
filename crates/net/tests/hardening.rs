mod common;
use an_net::node::{INBOUND_DEADLINE, MAX_CONNS, Me, Node};
use an_net::proto::{Request, Response};
use std::time::{Duration, Instant};
use tokio::io::AsyncReadExt;
use tokio::net::TcpSocket;

fn stranger() -> std::sync::Arc<Node> {
    Node::new(
        common::bare_identity(),
        an_core::db::Connection::open_in_memory().unwrap(),
        Me {
            name: "n".into(),
            host: "h".into(),
            port: 1,
        },
    )
}

#[tokio::test]
async fn wrong_code_does_not_consume_window() {
    let b = common::start("h-code-b").await;
    let a = common::start("h-code-a").await;
    let code = b.open();
    // 창이 열린 동안 코드 없는 · 틀린 코드의 선점 시도.
    for guess in [None, Some("AAAA-AAAA".to_string()), Some(String::new())] {
        let (resp, _) = stranger()
            .send(b.addr, None, &common::hello("n", "h", 1, guess))
            .await
            .unwrap();
        assert_eq!(resp, Response::error("closed"));
    }
    assert!(b.node.peers().unwrap().is_empty());
    // 창은 그대로 — 코드를 아는 상대는 진행된다(소문자 · 공백으로 적어도).
    let typed = code.unwrap().to_lowercase().replace('-', " ");
    a.node.add(b.addr, Some(typed)).await.unwrap();
    assert_eq!(b.only_peer().spki, a.node.identity().spki());
}

#[tokio::test]
async fn stale_approve_rejected() {
    let (a, b) = (
        common::start("h-stale-a").await,
        common::start("h-stale-b").await,
    );
    let code = b.open();
    a.node.add(b.addr, code).await.unwrap();
    let old_round = b.only_peer().round;
    // A 가 다시 추가 → 새 세대. 그 뒤 이전 세대 승인이 늦게 도착.
    let pb = a.node.add(b.addr, None).await.unwrap();
    assert_ne!(pb.round, old_round);
    assert_eq!(b.only_peer().round, pb.round, "양쪽이 같은 세대");
    let (resp, _) = b
        .node
        .send(
            a.addr,
            Some(a.node.identity().spki().to_vec()),
            &Request::Approve { round: old_round },
        )
        .await
        .unwrap();
    assert_eq!(resp, Response::error("stale"));
    assert!(!a.only_peer().their_ok);
    // 현재 세대 승인은 적용되어 양쪽이 수렴한다.
    let pa = b.only_peer();
    b.node.approve(pa.id).await.unwrap();
    a.node.approve(pb.id).await.unwrap();
    assert!(a.only_peer().paired() && b.only_peer().paired());
}

#[tokio::test]
async fn slow_clients_do_not_block_others() {
    let b = common::start("h-slow-b").await;
    // 다른 출처(127.0.0.2)에서 인증 없이 붙잡고만 있는 연결 — 전체 자리보다 많이.
    let mut slow = Vec::new();
    for _ in 0..MAX_CONNS + 8 {
        let s = TcpSocket::new_v4().unwrap();
        s.bind("127.0.0.2:0".parse().unwrap()).unwrap();
        slow.push(s.connect(b.addr).await.unwrap());
    }
    tokio::time::sleep(Duration::from_millis(100)).await;
    // 127.0.0.1 의 정상 요청은 곧바로 처리된다.
    let started = Instant::now();
    let (resp, _) = stranger()
        .send(b.addr, None, &common::hello("n", "h", 1, None))
        .await
        .unwrap();
    assert_eq!(resp, Response::error("closed"));
    assert!(started.elapsed() < Duration::from_secs(1));
    // 붙잡힌 느린 연결도 수신 상한 안에 끊긴다.
    let mut first = slow.remove(0);
    let mut buf = [0u8; 1];
    let cut = tokio::time::timeout(
        INBOUND_DEADLINE + Duration::from_millis(500),
        first.read(&mut buf),
    )
    .await;
    assert!(matches!(cut, Ok(Ok(0)) | Ok(Err(_))), "{cut:?}");
}
