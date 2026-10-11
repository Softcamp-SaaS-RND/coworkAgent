mod common;
use an_net::node::{Me, Node, PeerEvent};
use an_net::proto::{Request, Response};
use std::sync::{Arc, Mutex};

/// 노드가 낸 이벤트를 모으는 대역.
fn collect(node: &Node) -> Arc<Mutex<Vec<PeerEvent>>> {
    let seen = Arc::new(Mutex::new(Vec::new()));
    let sink = seen.clone();
    node.on_event(move |e| sink.lock().unwrap().push(e));
    seen
}

fn stranger() -> Arc<Node> {
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
async fn accepted_hello_emits_pair_request() {
    let (a, b) = (
        common::start("ev-req-a").await,
        common::start("ev-req-b").await,
    );
    let seen = collect(&b.node);
    // 거절된 hello(창 닫힘)는 이벤트가 없다.
    let (resp, _) = stranger()
        .send(b.addr, None, &common::hello("n", "h", 1, None))
        .await
        .unwrap();
    assert_eq!(resp, Response::error("closed"));
    assert!(seen.lock().unwrap().is_empty());

    let code = b.open();
    a.node.add(b.addr, code).await.unwrap();
    let want = PeerEvent::PairRequest {
        name: "user-ev-req-a".into(),
        host: "PC-ev-req-a".into(),
        fingerprint: a.node.identity().fingerprint(),
    };
    assert_eq!(*seen.lock().unwrap(), vec![want.clone()]);
    // 아는 PC 가 다시 추가해도 요청이다(승인이 초기화된다).
    a.node.add(b.addr, None).await.unwrap();
    assert_eq!(*seen.lock().unwrap(), vec![want.clone(), want]);
}

#[tokio::test]
async fn applied_approve_emits_approved_only() {
    let (a, b) = (
        common::start("ev-ok-a").await,
        common::start("ev-ok-b").await,
    );
    let code = b.open();
    let pb = a.node.add(b.addr, code).await.unwrap();
    let seen = collect(&b.node);
    // 기록 없는 키 · 이전 세대 승인은 이벤트가 없다.
    for (who, round) in [
        (stranger(), pb.round.clone()),
        (a.node.clone(), "old".into()),
    ] {
        let (resp, _) = who
            .send(b.addr, None, &Request::Approve { round })
            .await
            .unwrap();
        assert!(matches!(resp, Response::Error { .. }), "{resp:?}");
    }
    assert!(seen.lock().unwrap().is_empty());

    a.node.approve(pb.id).await.unwrap();
    // 같은 세대 재통지("다시 알리기") — 응답은 ok, 이벤트는 한 번뿐.
    let (resp, _) = a
        .node
        .send(
            b.addr,
            None,
            &Request::Approve {
                round: pb.round.clone(),
            },
        )
        .await
        .unwrap();
    assert_eq!(resp, Response::Ok);
    assert_eq!(
        *seen.lock().unwrap(),
        vec![PeerEvent::Approved {
            name: "user-ev-ok-a".into(),
            host: "PC-ev-ok-a".into(),
        }]
    );
}
