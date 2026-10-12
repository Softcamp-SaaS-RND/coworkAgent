//! 두 번째 실행 → 주 인스턴스에 "웹UI 열기" 를 넘긴다(D14). CLI(`cli`)도 같은 경로로 주 인스턴스 로컬 API 를 부른다.
use an_core::instance::InstanceInfo;
use std::io::{self, Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::{Duration, Instant};

/// 요청 전체(연결 · 쓰기 · 응답 읽기)의 상한.
pub const HANDOFF_DEADLINE: Duration = Duration::from_secs(5);

/// `POST /api/open` 을 보내고 HTTP 상태 코드를 돌려준다.
pub fn request_open(info: &InstanceInfo) -> io::Result<u16> {
    request_open_within(info, HANDOFF_DEADLINE)
}

/// 바이트를 조금씩 흘리는 상대에게도 `deadline` 안에 끝난다.
pub fn request_open_within(info: &InstanceInfo, deadline: Duration) -> io::Result<u16> {
    local_request(info, "POST", "/api/open", None, deadline).map(|(code, _)| code)
}

/// 주 인스턴스 로컬 API 요청 한 번 — bearer 토큰 · `Connection: close` 로 보내고 응답을 끝까지 읽는다(전체가 `deadline` 안).
/// 돌려주는 값: (HTTP 상태 코드, 본문).
pub fn local_request(
    info: &InstanceInfo,
    method: &str,
    path: &str,
    body: Option<&str>,
    deadline: Duration,
) -> io::Result<(u16, String)> {
    let until = Instant::now() + deadline;
    let addr = SocketAddr::from(([127, 0, 0, 1], info.port));
    let mut stream = TcpStream::connect_timeout(&addr, deadline)?;
    stream.set_write_timeout(Some(deadline))?;
    let body = body.unwrap_or("");
    write!(
        stream,
        "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nAuthorization: Bearer {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        info.port,
        info.token,
        body.len()
    )?;
    let mut raw = Vec::new();
    let mut buf = [0u8; 4096];
    loop {
        let left = until.saturating_duration_since(Instant::now());
        if left.is_zero() {
            return Err(timed_out());
        }
        stream.set_read_timeout(Some(left))?;
        match stream.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => raw.extend_from_slice(&buf[..n]),
            Err(e)
                if matches!(
                    e.kind(),
                    io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                ) =>
            {
                return Err(timed_out());
            }
            // 요청은 이미 다 보냈다 — 연결 거부와 달리 "앱이 받았는지 모른다". 호출자가 구분할 수 있게 종류를 바꾼다.
            Err(e) => return Err(cut_off(e)),
        }
    }
    if raw.is_empty() {
        return Err(cut_off("응답 없이 닫혔다"));
    }
    let text = String::from_utf8_lossy(&raw);
    let code = text
        .lines()
        .next()
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|c| c.parse().ok())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "HTTP 응답이 아니다"))?;
    let body = text.split_once("\r\n\r\n").map_or("", |(_, b)| b);
    Ok((code, body.to_string()))
}

/// 요청을 보낸 뒤 응답 전에 연결이 끊겼다(`UnexpectedEof`).
fn cut_off(why: impl std::fmt::Display) -> io::Error {
    io::Error::new(
        io::ErrorKind::UnexpectedEof,
        format!("응답을 받기 전에 연결이 끊겼다: {why}"),
    )
}

fn timed_out() -> io::Error {
    io::Error::new(io::ErrorKind::TimedOut, "주 인스턴스 응답 시간 초과")
}
