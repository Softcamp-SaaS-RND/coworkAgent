mod common;
use an_app::handoff;
use an_core::instance::InstanceInfo;

#[tokio::test]
async fn browser_failure_is_500_not_204() {
    let r = common::start_with(false).await;
    let info = InstanceInfo {
        port: r.port,
        token: r.token.clone(),
    };
    let code = tokio::task::spawn_blocking(move || handoff::request_open(&info).unwrap())
        .await
        .unwrap();
    assert_eq!(code, 500);
    assert_eq!(r.opened(), 1);
}
