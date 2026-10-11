use an_app::handoff;
use an_core::instance::InstanceInfo;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::time::{Duration, Instant};

/// 상태 줄을 끝내지 않고 바이트를 조금씩 흘리는 상대 — 전체 상한 안에 실패해야 한다.
#[test]
fn slow_drip_peer_times_out_within_deadline() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    std::thread::spawn(move || {
        let (mut s, _) = listener.accept().unwrap();
        let mut buf = [0u8; 512];
        let _ = s.read(&mut buf);
        for _ in 0..50 {
            if s.write_all(b"H").is_err() {
                break;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
    });
    let info = InstanceInfo {
        port,
        token: "ab".repeat(32),
    };
    let started = Instant::now();
    let err = handoff::request_open_within(&info, Duration::from_millis(800)).unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::TimedOut);
    assert!(
        started.elapsed() < Duration::from_secs(2),
        "{:?}",
        started.elapsed()
    );
}
