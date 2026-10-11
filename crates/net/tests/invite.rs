mod common;
use an_net::invite::Invite;
use an_net::node::PAIRING_WINDOW;
use std::net::Ipv4Addr;

#[tokio::test]
async fn invite_roundtrip_through_messenger_text() {
    let b = common::start("inv-rt-b").await;
    let inv = b
        .node
        .open_invite(Ipv4Addr::LOCALHOST.into(), PAIRING_WINDOW);
    assert_eq!(inv.addr, b.addr);
    let text = inv.to_text("이도윤", "DYOON-PC");
    // 메신저가 붙이는 머리말 · 줄바꿈(CRLF) · 꼬리말을 견딘다.
    let pasted = format!(
        "[오후 2:01] 이도윤 : {}\r\n\r\n(읽음)",
        text.replace('\n', "\r\n")
    );
    assert_eq!(Invite::parse(&pasted), Some(inv.clone()));
    assert_eq!(inv.fingerprint.len(), 32);
    assert_eq!(
        inv.fingerprint,
        b.node.identity().fingerprint().replace(' ', "")
    );
}

#[test]
fn invite_parse_rejects_malformed() {
    let fp = "6FEECC8C16C5551D9FEB3EB5F77E2DA7";
    let ok = format!("coworkagent-invite:v1/10.0.0.5:47100/ABCD-EFGH/{fp}");
    assert!(Invite::parse(&ok).is_some());
    for bad in [
        "초대 없음".to_string(),
        format!("coworkagent-invite:v1/10.0.0.5/ABCD-EFGH/{fp}"), // 포트 없음
        format!("coworkagent-invite:v1/10.0.0.5:47100//{fp}"),    // 코드 없음
        "coworkagent-invite:v1/10.0.0.5:47100/ABCD-EFGH/6FEE".to_string(), // 지문 짧음
        format!("coworkagent-invite:v1/10.0.0.5:47100/ABCD-EFGH/{fp}/x"), // 조각 초과
        format!("coworkagent-invite:v2/10.0.0.5:47100/ABCD-EFGH/{fp}"), // 다른 판
    ] {
        assert_eq!(Invite::parse(&bad), None, "{bad}");
    }
}

#[tokio::test]
async fn invite_add_pins_fingerprint() {
    let (a, b) = (
        common::start("inv-pin-a").await,
        common::start("inv-pin-b").await,
    );
    let inv = b
        .node
        .open_invite(Ipv4Addr::LOCALHOST.into(), PAIRING_WINDOW);
    // 지문을 바꾼 초대(중간자 · 오타) — 연결 단계에서 끊겨 hello 가 가지 않는다.
    let mut forged = inv.clone();
    forged.fingerprint = a.node.identity().fingerprint().replace(' ', "");
    let err = a.node.add_invite(&forged).await.unwrap_err();
    assert!(err.to_string().contains("지문"), "{err}");
    assert!(a.node.peers().unwrap().is_empty());
    assert!(b.node.peers().unwrap().is_empty());
    // 진짜 초대는 진행된다(위 시도가 창을 쓰지 않았다).
    let p = a.node.add_invite(&inv).await.unwrap();
    assert_eq!(p.spki, b.node.identity().spki());
    assert_eq!(b.only_peer().spki, a.node.identity().spki());
}
