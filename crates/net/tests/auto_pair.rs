mod common;
use an_net::node::{Node, PeerEvent};
use std::sync::{Arc, Mutex};

fn collect(node: &Node) -> Arc<Mutex<Vec<PeerEvent>>> {
    let seen = Arc::new(Mutex::new(Vec::new()));
    let sink = seen.clone();
    node.on_event(move |e| sink.lock().unwrap().push(e));
    seen
}

/// 초대 생성 측(B)은 초대 코드로 들어온 처음 보는 키를 승인 없이 등록한다 — 수락 측(A)의 승인 하나로 페어링.
#[tokio::test]
async fn invite_hello_auto_pairs_both_sides() {
    let (a, b) = (common::start("auto-a").await, common::start("auto-b").await);
    let seen = collect(&b.node);
    let code = b.open();
    let pb = a.node.add(b.addr, code).await.unwrap();
    assert!(
        pb.their_ok && !pb.my_ok,
        "수락 측은 상대 승인을 받은 채 내 승인 대기"
    );
    let pa = b.only_peer();
    assert!(pa.my_ok && !pa.their_ok, "생성 측은 자동 승인");
    assert_eq!(
        *seen.lock().unwrap(),
        vec![PeerEvent::AutoPaired {
            name: "user-auto-a".into(),
            host: "PC-auto-a".into(),
            fingerprint: a.node.identity().fingerprint(),
        }],
        "등록 즉시 알림"
    );

    let pb = a.node.approve(pb.id).await.unwrap();
    assert!(pb.paired());
    assert!(b.only_peer().paired());
    a.node.ping(pb.id).await.unwrap();
    b.node.ping(pa.id).await.unwrap();
}

/// 아는 키의 재 hello(다시 추가 — 코드 없음)는 자동 승인하지 않는다.
#[tokio::test]
async fn rehello_without_code_needs_approval() {
    let (a, b) = (common::start("reh-a").await, common::start("reh-b").await);
    let code = b.open();
    let pb = a.node.add(b.addr, code).await.unwrap();
    a.node.approve(pb.id).await.unwrap();
    assert!(b.only_peer().paired());

    let seen = collect(&b.node);
    let pb = a.node.add(b.addr, None).await.unwrap();
    assert!(!pb.my_ok && !pb.their_ok);
    let pa = b.only_peer();
    assert!(!pa.my_ok && !pa.their_ok, "사람 승인이 다시 필요");
    assert!(matches!(
        seen.lock().unwrap().as_slice(),
        [PeerEvent::PairRequest { .. }]
    ));

    // 창을 열어 둔 채 아는 키가 코드를 담아 와도 자동 승인은 처음 보는 키에만 — 창도 쓰지 않는다.
    let code = b.open();
    a.node.add(b.addr, code.clone()).await.unwrap();
    assert!(!b.only_peer().my_ok);
    let c = common::start("reh-c").await;
    c.node.add(b.addr, code).await.unwrap();
    let pc = b
        .node
        .peers()
        .unwrap()
        .into_iter()
        .find(|p| p.name == "user-reh-c")
        .unwrap();
    assert!(pc.my_ok, "같은 창의 코드로 처음 보는 키는 자동 등록");
}
