mod common;
use an_app::{handoff, server};
use an_core::instance::InstanceInfo;

async fn open_with(port: u16, token: String) -> u16 {
    tokio::task::spawn_blocking(move || {
        handoff::request_open(&InstanceInfo { port, token }).unwrap()
    })
    .await
    .unwrap()
}

#[tokio::test]
async fn valid_token_opens_once() {
    let r = common::start().await;
    assert_eq!(open_with(r.port, r.token.clone()).await, 204);
    assert_eq!(r.opened(), 1);
}

#[tokio::test]
async fn wrong_token_401_opens_none() {
    let r = common::start().await;
    assert_eq!(open_with(r.port, server::new_token()).await, 401);
    assert_eq!(r.opened(), 0);
}
