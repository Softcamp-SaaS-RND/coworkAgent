mod common;
use an_net::proto::{Request, Response};

#[tokio::test]
async fn add_registers_both_sides() {
    let (a, b) = (common::start("add-a").await, common::start("add-b").await);
    let code = b.open();
    let pb = a.node.add(b.addr, code).await.unwrap();
    assert_eq!(pb.spki, b.node.identity().spki());
    assert_eq!(
        (pb.name.as_str(), pb.host.as_str()),
        ("user-add-b", "PC-add-b")
    );
    assert_eq!(pb.addr, b.addr.to_string());
    assert!(!pb.my_ok && pb.their_ok, "초대 생성 측 자동 승인(D8 개정)");

    let pa = b.only_peer();
    assert_eq!(pa.spki, a.node.identity().spki());
    assert_eq!(pa.name, "user-add-a");
    assert_eq!(
        pa.addr,
        a.addr.to_string(),
        "주소 = 접속 IP + hello 의 포트"
    );
    assert!(pa.my_ok && !pa.their_ok, "초대 수락 측은 아직 승인 전");

    // 자기 자신의 주소는 추가하지 않는다.
    assert!(a.node.add(a.addr, None).await.is_err());
    assert_eq!(a.node.peers().unwrap().len(), 1);
}

#[tokio::test]
async fn one_side_approval_is_not_paired() {
    let (a, b) = (common::start("one-a").await, common::start("one-b").await);
    let code = b.open();
    a.node.add(b.addr, code).await.unwrap();
    // 다시 추가(코드 없음)는 양쪽 승인이 지워진다 — 사람 승인 경로.
    let pb = a.node.add(b.addr, None).await.unwrap();
    let pb = a.node.approve(pb.id).await.unwrap();
    assert!(pb.my_ok && !pb.paired());
    let pa = b.only_peer();
    assert!(pa.their_ok && !pa.my_ok && !pa.paired());
    // A → B ping: B 는 아직 승인하지 않았다.
    assert!(a.node.ping(pb.id).await.is_err());
    // B → A ping: A 는 B 의 승인을 받지 못했다.
    assert!(b.node.ping(pa.id).await.is_err());
}

#[tokio::test]
async fn both_approvals_pair() {
    let (a, b) = (common::start("both-a").await, common::start("both-b").await);
    let code = b.open();
    let pb = a.node.add(b.addr, code).await.unwrap();
    let pa = b.only_peer();
    a.node.approve(pb.id).await.unwrap();
    let pa = b.node.approve(pa.id).await.unwrap();
    assert!(pa.paired());
    assert!(a.only_peer().paired());
    a.node.ping(pb.id).await.unwrap();
    b.node.ping(pa.id).await.unwrap();
}

#[tokio::test]
async fn unpaired_ping_rejected() {
    let (a, b) = (common::start("unp-a").await, common::start("unp-b").await);
    // 처음 보는 키.
    let stranger = common::start("unp-c").await;
    let (resp, _) = stranger
        .node
        .send(b.addr, None, &Request::Ping)
        .await
        .unwrap();
    assert_eq!(resp, Response::error("not_paired"));
    // 주소만 추가한(승인 전) 키.
    let code = b.open();
    a.node.add(b.addr, code).await.unwrap();
    let (resp, _) = a
        .node
        .send(
            b.addr,
            Some(b.node.identity().spki().to_vec()),
            &Request::Ping,
        )
        .await
        .unwrap();
    assert_eq!(resp, Response::error("not_paired"));
    // 기록 없는 키의 approve 는 아무것도 바꾸지 않는다.
    let (resp, _) = stranger
        .node
        .send(
            b.addr,
            None,
            &Request::Approve {
                round: "r-any".into(),
            },
        )
        .await
        .unwrap();
    assert_eq!(resp, Response::error("unknown"));
    assert_eq!(b.node.peers().unwrap().len(), 1);
}

#[tokio::test]
async fn removed_peer_rejected_again() {
    let (a, b) = (common::start("rm-a").await, common::start("rm-b").await);
    let code = b.open();
    let pb = a.node.add(b.addr, code).await.unwrap();
    let pa = b.only_peer();
    a.node.approve(pb.id).await.unwrap();
    b.node.approve(pa.id).await.unwrap();
    a.node.ping(pb.id).await.unwrap();

    b.node.remove(pa.id).await.unwrap();
    assert!(b.node.peers().unwrap().is_empty());
    assert!(
        a.node.peers().unwrap().is_empty(),
        "삭제 통지로 상대 기록도 지워진다"
    );
    let (resp, _) = a
        .node
        .send(
            b.addr,
            Some(b.node.identity().spki().to_vec()),
            &Request::Ping,
        )
        .await
        .unwrap();
    assert_eq!(resp, Response::error("not_paired"));

    // 다시 추가하면 처음부터 — 자동으로 페어링되지 않는다.
    let code = b.open();
    let pb = a.node.add(b.addr, code).await.unwrap();
    assert!(!pb.paired() && !b.only_peer().paired());
}

/// `Pong` 에 상대의 집중(설정 `focus`)을 싣고, Ping 한 쪽은 받은 시각 · 집중을 남긴다(접속 상태). 거절된 Ping 은 남기지 않는다.
#[tokio::test]
async fn pong_carries_focus_and_presence() {
    let (a, b) = (common::start("pong-a").await, common::start("pong-b").await);
    let code = b.open();
    let pb = a.node.add(b.addr, code).await.unwrap();
    a.node.approve(pb.id).await.unwrap();
    assert_eq!(a.only_peer().seen_at, None, "아직 Ping 전");
    // 주기 확인은 시작 직후 한 번 — 다음은 30초 뒤라 아래 단정과 겹치지 않는다.
    tokio::spawn(a.node.clone().watch_presence());
    let until = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while a.only_peer().seen_at.is_none() {
        assert!(
            std::time::Instant::now() < until,
            "watch_presence 가 Ping 하지 않았다"
        );
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    assert!(!a.node.ping(pb.id).await.unwrap(), "집중 꺼짐");
    let p = a.only_peer();
    assert!(p.seen_at.is_some_and(|t| t > 0) && !p.focus, "{p:?}");
    let b_db = an_core::db::open(&b.paths.db_file()).unwrap();
    an_core::settings::set(&b_db, an_core::settings::FOCUS_KEY, "1").unwrap();
    assert!(a.node.ping(pb.id).await.unwrap(), "집중 켜짐");
    assert!(a.only_peer().focus);
    an_core::settings::set(&b_db, an_core::settings::FOCUS_KEY, "0").unwrap();
    assert!(!a.node.ping(pb.id).await.unwrap());
    assert!(!a.only_peer().focus);
    // B 가 A 를 지우면 Ping 은 거절 — 접속 기록은 그대로.
    let before = a.only_peer().seen_at;
    let pa = b.only_peer();
    an_core::peers::remove(&b_db, pa.id).unwrap();
    assert!(a.node.ping(pb.id).await.is_err());
    assert_eq!(a.only_peer().seen_at, before);
}
