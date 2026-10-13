//! 로컬 웹UI 서버(D9) — 127.0.0.1 전용 · 실행마다 바뀌는 토큰 · Host/Origin 검사.
//! 토큰은 `Authorization: Bearer` 로만(탭 메모리 — 쿠키는 포트를 가리지 않아 같은 PC 의 다른 포트로도 실린다, 1a A2).
//! 정적 자산은 공개(헤더를 못 싣는 `<script src>` · `import`), 모든 응답에 보안 헤더(A3).
use axum::Router;
use axum::extract::{Request, State};
use axum::http::{HeaderMap, HeaderValue, Method, StatusCode, header};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use std::sync::Arc;
use tokio::net::TcpListener;

/// 모든 응답의 CSP. `style-src 'unsafe-inline'` · `img-src data:` 는 HTML 본문 iframe(`srcdoc` 은 부모 CSP 를 물려받는다) · data: 아이콘 때문.
/// `base-uri` · `form-action` 은 넣지 않는다 — `srcdoc` 이 물려받아 본문의 `<base target="_top">` 을 막는다.
pub const CSP: &str = "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'self'; frame-ancestors 'none'; object-src 'none'";

/// 상태 페이지(T2)에 보이는 값.
/// CLI 가 기대하는 로컬 API 판(`/api/status` 의 `cli_api`) — CLI 출처 표시(`via`)를 아는 주 인스턴스만 이 값을 낸다.
pub const CLI_API: u32 = 1;

pub struct Status {
    pub version: String,
    pub data_dir: String,
    pub db: String,
}

struct AppState {
    token: String,
    host: String,
    origin: String,
    status: Status,
    open_browser: Box<dyn Fn() -> std::io::Result<()> + Send + Sync>,
}

/// 루프백 전용 · 동적 포트.
pub async fn bind() -> std::io::Result<TcpListener> {
    TcpListener::bind(("127.0.0.1", 0)).await
}

/// 32바이트 난수 → hex 64자.
pub fn new_token() -> String {
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes).expect("OS 난수원을 쓸 수 없다");
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// `open_browser` 는 핸드오프(`POST /api/open`)가 부른다 — 운영은 OS 브라우저 열기. 실패하면 500.
pub fn router(
    port: u16,
    token: String,
    status: Status,
    open_browser: impl Fn() -> std::io::Result<()> + Send + Sync + 'static,
) -> Router {
    router_with(port, token, status, open_browser, Router::new())
}

/// `router` + 더 붙일 경로(`extra`) — 같은 guard 뒤에 놓인다.
pub fn router_with(
    port: u16,
    token: String,
    status: Status,
    open_browser: impl Fn() -> std::io::Result<()> + Send + Sync + 'static,
    extra: Router,
) -> Router {
    let state = Arc::new(AppState {
        token,
        host: format!("127.0.0.1:{port}"),
        origin: format!("http://127.0.0.1:{port}"),
        status,
        open_browser: Box::new(open_browser),
    });
    Router::new()
        .route("/", get(index))
        .route("/app.js", get(app_js))
        .route("/view.js", get(view_js))
        .route("/app.css", get(app_css))
        .route("/favicon.ico", get(favicon))
        .route("/api/status", get(api_status))
        .route("/api/open", post(api_open))
        .with_state(state.clone())
        .merge(extra)
        .layer(middleware::from_fn_with_state(state, guard))
}

/// 토큰 없이 주는 정적 자산(실행 파일에 박힌 상수 — 비밀 없음).
const PUBLIC: [&str; 5] = ["/", "/app.js", "/view.js", "/app.css", "/favicon.ico"];

/// 차례로: Host(DNS rebinding) → Origin(교차 출처) → 토큰. 응답에는 늘 보안 헤더.
async fn guard(State(s): State<Arc<AppState>>, req: Request, next: Next) -> Response {
    let res = match check(&s, &req) {
        Ok(()) => next.run(req).await,
        Err(code) => code.into_response(),
    };
    harden(res)
}

fn check(s: &AppState, req: &Request) -> Result<(), StatusCode> {
    let headers = req.headers();
    if header_str(headers, header::HOST) != Some(s.host.as_str()) {
        return Err(StatusCode::FORBIDDEN);
    }
    if headers.contains_key(header::ORIGIN)
        && header_str(headers, header::ORIGIN) != Some(s.origin.as_str())
    {
        return Err(StatusCode::FORBIDDEN);
    }
    let path = req.uri().path();
    let query = query_token(req.uri().query());
    if req.method() == Method::GET && PUBLIC.contains(&path) {
        // 진입 `/?t=` 는 페이지가 토큰을 탭 메모리로 가져간다 — 틀린 토큰은 여기서 끊는다.
        return match query {
            Some(t) if path == "/" && !ct_eq(t, &s.token) => Err(StatusCode::UNAUTHORIZED),
            _ => Ok(()),
        };
    }
    // `EventSource` 는 헤더를 못 싣는다 — 알림 스트림만 쿼리 토큰.
    let presented = bearer(headers).or(query.filter(|_| path == "/api/events"));
    if presented.is_some_and(|t| ct_eq(t, &s.token)) {
        Ok(())
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

fn harden(mut res: Response) -> Response {
    let h = res.headers_mut();
    h.insert(header::X_FRAME_OPTIONS, HeaderValue::from_static("DENY"));
    h.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    h.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("no-referrer"),
    );
    h.insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static(CSP),
    );
    res
}

fn header_str(headers: &HeaderMap, name: header::HeaderName) -> Option<&str> {
    headers.get(name).and_then(|v| v.to_str().ok())
}

fn bearer(headers: &HeaderMap) -> Option<&str> {
    header_str(headers, header::AUTHORIZATION).and_then(|v| v.strip_prefix("Bearer "))
}

fn query_token(query: Option<&str>) -> Option<&str> {
    query?.split('&').find_map(|kv| kv.strip_prefix("t="))
}

/// 길이가 같으면 내용과 무관하게 같은 시간에 비교한다.
fn ct_eq(a: &str, b: &str) -> bool {
    a.len() == b.len()
        && a.bytes()
            .zip(b.bytes())
            .fold(0u8, |acc, (x, y)| acc | (x ^ y))
            == 0
}

async fn index() -> impl IntoResponse {
    (
        [
            (header::CONTENT_TYPE, "text/html; charset=utf-8"),
            (header::CACHE_CONTROL, "no-store"),
        ],
        include_str!("../web/index.html"),
    )
}

async fn app_js() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/javascript; charset=utf-8")],
        include_str!("../web/app.js"),
    )
}

/// 화면 로직 모듈(DOM 없음) — `app.js` 가 import, `node --test` 가 시험.
async fn view_js() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/javascript; charset=utf-8")],
        include_str!("../web/view.js"),
    )
}

async fn app_css() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/css; charset=utf-8")],
        include_str!("../web/app.css"),
    )
}

/// 브라우저 탭 아이콘 — 실행 파일 · 트레이와 같은 에이전트 아이콘.
async fn favicon() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "image/x-icon")],
        include_bytes!("../assets/coworkAgent.ico").as_slice(),
    )
}

async fn api_status(State(s): State<Arc<AppState>>) -> impl IntoResponse {
    let st = &s.status;
    let body = format!(
        r#"{{"version":"{}","data_dir":"{}","db":"{}","cli_api":{CLI_API}}}"#,
        json_escape(&st.version),
        json_escape(&st.data_dir),
        json_escape(&st.db)
    );
    ([(header::CONTENT_TYPE, "application/json")], body)
}

async fn api_open(State(s): State<Arc<AppState>>) -> StatusCode {
    match (s.open_browser)() {
        Ok(()) => StatusCode::NO_CONTENT,
        Err(e) => {
            log::error!("브라우저 열기 실패: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{ct_eq, json_escape, new_token};

    #[test]
    fn token_is_64_hex_and_fresh() {
        let (a, b) = (new_token(), new_token());
        assert_eq!(a.len(), 64);
        assert!(a.bytes().all(|c| c.is_ascii_hexdigit()));
        assert_ne!(a, b);
    }

    #[test]
    fn ct_eq_requires_same_len_and_bytes() {
        assert!(ct_eq("abc", "abc"));
        assert!(!ct_eq("abc", "abd"));
        assert!(!ct_eq("abc", "ab"));
    }

    #[test]
    fn json_escape_quotes_backslash_control() {
        assert_eq!(json_escape("a\"b\\c\n"), "a\\\"b\\\\c\\u000a");
    }
}
