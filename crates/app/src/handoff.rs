//! 두 번째 실행 → 주 인스턴스에 "웹UI 열기" 를 넘긴다(D14).
use an_core::instance::InstanceInfo;
use std::io::{self, Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::{Duration, Instant};

/// 요청 전체(연결 · 쓰기 · 상태 줄 읽기)의 상한.
pub const HANDOFF_DEADLINE: Duration = Duration::from_secs(5);

/// `POST /api/open` 을 보내고 HTTP 상태 코드를 돌려준다.
pub fn request_open(info: &InstanceInfo) -> io::Result<u16> {
    request_open_within(info, HANDOFF_DEADLINE)
}

/// 상태 줄(첫 줄)만 읽는다 — 바이트를 조금씩 흘리는 상대에게도 `deadline` 안에 끝난다.
pub fn request_open_within(info: &InstanceInfo, deadline: Duration) -> io::Result<u16> {
    let until = Instant::now() + deadline;
    let addr = SocketAddr::from(([127, 0, 0, 1], info.port));
    let mut stream = TcpStream::connect_timeout(&addr, deadline)?;
    stream.set_write_timeout(Some(deadline))?;
    write!(
        stream,
        "POST /api/open HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nAuthorization: Bearer {}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
        info.port, info.token
    )?;
    let mut line = Vec::new();
    let mut byte = [0u8; 1];
    while !line.ends_with(b"\n") {
        let left = until.saturating_duration_since(Instant::now());
        if left.is_zero() {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "핸드오프 응답 시간 초과",
            ));
        }
        stream.set_read_timeout(Some(left))?;
        match stream.read(&mut byte)? {
            0 => break,
            _ => line.push(byte[0]),
        }
    }
    String::from_utf8_lossy(&line)
        .split_whitespace()
        .nth(1)
        .and_then(|code| code.parse().ok())
        .ok_or_else(|| io::Error::other("HTTP 응답이 아니다"))
}
