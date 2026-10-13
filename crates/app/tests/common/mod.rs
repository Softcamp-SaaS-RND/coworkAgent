#![allow(dead_code)]
use an_app::server::{self, Status};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub struct Running {
    pub port: u16,
    pub token: String,
    pub opened: Arc<AtomicUsize>,
}

impl Running {
    pub fn opened(&self) -> usize {
        self.opened.load(Ordering::SeqCst)
    }
}

/// 운영과 같은 `bind` · `router` 로 실제 TCP 서버를 띄운다. 브라우저 열기는 호출 횟수를 세는 대역.
pub async fn start() -> Running {
    start_with(true).await
}

/// `browser_ok = false` 면 브라우저 열기가 실패하는 대역(호출 횟수는 센다).
pub async fn start_with(browser_ok: bool) -> Running {
    let listener = server::bind().await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let token = server::new_token();
    let opened = Arc::new(AtomicUsize::new(0));
    let counter = opened.clone();
    let status = Status {
        version: "test".into(),
        data_dir: r"C:\tmp\a".into(),
        db: "ok".into(),
    };
    let app = server::router(port, token.clone(), status, move || {
        counter.fetch_add(1, Ordering::SeqCst);
        if browser_ok {
            Ok(())
        } else {
            Err(std::io::Error::other("브라우저 실행 실패(대역)"))
        }
    });
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    Running {
        port,
        token,
        opened,
    }
}

/// 원시 HTTP/1.1 요청 — 헤더를 그대로 통제하려고 클라이언트 라이브러리를 쓰지 않는다.
pub async fn send(
    port: u16,
    method: &str,
    path: &str,
    headers: &[(&str, String)],
) -> (u16, String) {
    send_body(port, method, path, headers, "").await
}

/// `send` + 본문.
pub async fn send_body(
    port: u16,
    method: &str,
    path: &str,
    headers: &[(&str, String)],
    body: &str,
) -> (u16, String) {
    let mut req = format!(
        "{method} {path} HTTP/1.1\r\nConnection: close\r\nContent-Length: {}\r\n",
        body.len()
    );
    for (k, v) in headers {
        req.push_str(&format!("{k}: {v}\r\n"));
    }
    req.push_str("\r\n");
    req.push_str(body);
    let mut s = tokio::net::TcpStream::connect(("127.0.0.1", port))
        .await
        .unwrap();
    s.write_all(req.as_bytes()).await.unwrap();
    // 본문이 그림(favicon)일 수 있다 — 바이트로 읽어 글자로 바꾼다.
    let mut raw = Vec::new();
    s.read_to_end(&mut raw).await.unwrap();
    let out = String::from_utf8_lossy(&raw).into_owned();
    let code = out.split_whitespace().nth(1).unwrap().parse().unwrap();
    (code, out)
}

/// 브라우저가 상태 변경 요청에 다는 우리 Origin.
pub fn origin(port: u16) -> (&'static str, String) {
    ("Origin", format!("http://127.0.0.1:{port}"))
}

pub fn host(port: u16) -> (&'static str, String) {
    ("Host", format!("127.0.0.1:{port}"))
}
