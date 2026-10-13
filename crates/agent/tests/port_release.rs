//! 포트 풀림(1b A8) — `claude -p` 자식이 살아 있어도 앱이 닫은 수신 소켓의 포트는 바로 다시 열린다(자식에 소켓이 물려가지 않는다).
use an_agent::runner::run_process;
use std::time::Duration;

#[tokio::test(flavor = "multi_thread")]
async fn closed_listener_port_reopens_while_child_runs() {
    // 운영과 같은 피어 수신 바인드(0 = 임의 포트).
    let (listener, _) = an_net::node::bind_peer_port_on(0).await.unwrap();
    let port = listener.local_addr().unwrap().port();
    // 소켓을 연 채 3초 도는 자식을 띄운다.
    let (program, args): (&str, Vec<String>) = if cfg!(windows) {
        ("ping", vec!["-n".into(), "4".into(), "127.0.0.1".into()])
    } else {
        ("sleep", vec!["3".into()])
    };
    let child = std::thread::spawn(move || {
        run_process(
            std::path::Path::new(program),
            &args,
            &std::env::temp_dir(),
            "",
            Duration::from_secs(20),
        )
    });
    std::thread::sleep(Duration::from_millis(500));
    assert!(!child.is_finished(), "자식이 살아 있는 동안 잰다");
    drop(listener);
    let again = std::net::TcpListener::bind(("0.0.0.0", port));
    assert!(again.is_ok(), "자식이 살아 있는데 포트 {port} 를 다시 못 연다: {again:?}");
    assert!(!child.is_finished(), "자식이 살아 있는 동안 다시 열었다");
    child.join().unwrap().unwrap();
}
