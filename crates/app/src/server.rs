//! 로컬 웹UI 서버(D9) — 127.0.0.1 전용 · 실행마다 바뀌는 토큰 · Host/Origin 검사.
//! 토큰은 `Authorization: Bearer` 로만(탭 메모리 — 쿠키는 포트를 가리지 않아 같은 PC 의 다른 포트로도 실린다).
//! 정적 자산은 공개(헤더를 못 싣는 `<script src>` · `import`), 모든 응답에 보안 헤더.
use axum::Router;
use axum::extract::{Path, Request, State};
use axum::http::{HeaderMap, HeaderValue, Method, StatusCode, header};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::Json;
use serde_json::json;
use std::sync::Arc;
use tokio::net::TcpListener;

/// 모든 응답의 CSP. `style-src 'unsafe-inline'` · `img-src data:` 는 HTML 본문 iframe(`srcdoc` 은 부모 CSP 를 물려받는다) · data: 아이콘 때문.
/// `base-uri` · `form-action` 은 넣지 않는다 — `srcdoc` 이 물려받아 본문의 `<base target="_top">` 을 막는다.
pub const CSP: &str = "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'self'; frame-ancestors 'none'; object-src 'none'";

/// CLI 가 기대하는 로컬 API 판(`/api/status` 의 `cli_api`) — CLI 출처 표시(`via`)를 아는 주 인스턴스만 이 값을 낸다.
pub const CLI_API: u32 = 1;

/// 상태 페이지에 보이는 값.
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
    /// `/api/status` 의 `mode` — `"tray"` · `"headless"`.
    mode: &'static str,
    /// `POST /api/quit` 이 부른다 — 트레이 "종료" 와 같은 정상 종료.
    quit: Option<Quit>,
}

/// 종료 요청 — 트레이 이벤트 루프 · 트레이 없는 모드의 대기를 깨운다.
pub type Quit = Box<dyn Fn() + Send + Sync>;

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

/// `router` + 더 붙일 경로(`extra`) — 같은 guard 뒤에 놓인다. 트레이 모드 · 종료 API 없음.
pub fn router_with(
    port: u16,
    token: String,
    status: Status,
    open_browser: impl Fn() -> std::io::Result<()> + Send + Sync + 'static,
    extra: Router,
) -> Router {
    router_control(port, token, status, open_browser, "tray", None, extra)
}

/// `router_with` + 실행 모드 · 종료 API. `quit` 이 있을 때만 `POST /api/quit` 을 둔다.
pub fn router_control(
    port: u16,
    token: String,
    status: Status,
    open_browser: impl Fn() -> std::io::Result<()> + Send + Sync + 'static,
    mode: &'static str,
    quit: Option<Quit>,
    extra: Router,
) -> Router {
    let has_quit = quit.is_some();
    let state = Arc::new(AppState {
        token,
        host: format!("127.0.0.1:{port}"),
        origin: format!("http://127.0.0.1:{port}"),
        status,
        open_browser: Box::new(open_browser),
        mode,
        quit,
    });
    let mut base = Router::new();
    if has_quit {
        base = base.route("/api/quit", post(api_quit));
    }
    base.route("/", get(index))
        .route("/js/{*file}", get(script))
        .route("/app.css", get(app_css))
        .route("/favicon.ico", get(favicon))
        .route("/fonts/{file}", get(font))
        .route("/api/status", get(api_status))
        .route("/api/debug", get(api_debug))
        .route("/api/debug/log", get(api_debug_log))
        .route("/api/open", post(api_open))
        .with_state(state.clone())
        .merge(extra)
        .layer(middleware::from_fn_with_state(state, guard))
}

/// 토큰 없이 주는 정적 자산(실행 파일에 박힌 상수 — 비밀 없음). `/fonts/` 는 `FONTS`, `/js/` 는 `SCRIPTS` 의 이름과 정확히 같을 때만.
const PUBLIC: [&str; 3] = ["/", "/app.css", "/favicon.ico"];

/// 화면 모듈 — `/js/<이름>`(진입 `app.js` · `core/` · 도메인 · `settings/` · DOM 없는 `logic/`). `web/js/` 의 파일과 같아야 한다(시험 `web_assets`).
pub const SCRIPTS: &[(&str, &str)] = &[
    ("app.js", include_str!("../web/js/app.js")),
    ("core/token.js", include_str!("../web/js/core/token.js")),
    ("core/api.js", include_str!("../web/js/core/api.js")),
    ("core/dom.js", include_str!("../web/js/core/dom.js")),
    ("core/widgets.js", include_str!("../web/js/core/widgets.js")),
    ("core/state.js", include_str!("../web/js/core/state.js")),
    ("core/float.js", include_str!("../web/js/core/float.js")),
    ("shell.js", include_str!("../web/js/shell.js")),
    ("events.js", include_str!("../web/js/events.js")),
    ("rail.js", include_str!("../web/js/rail.js")),
    ("tiles.js", include_str!("../web/js/tiles.js")),
    ("sidebar.js", include_str!("../web/js/sidebar.js")),
    ("home.js", include_str!("../web/js/home.js")),
    ("conv.js", include_str!("../web/js/conv.js")),
    ("conv-menu.js", include_str!("../web/js/conv-menu.js")),
    ("composer.js", include_str!("../web/js/composer.js")),
    ("detail.js", include_str!("../web/js/detail.js")),
    ("activity.js", include_str!("../web/js/activity.js")),
    ("requests.js", include_str!("../web/js/requests.js")),
    ("agents.js", include_str!("../web/js/agents.js")),
    ("peers.js", include_str!("../web/js/peers.js")),
    ("compose.js", include_str!("../web/js/compose.js")),
    ("drag.js", include_str!("../web/js/drag.js")),
    ("search.js", include_str!("../web/js/search.js")),
    ("settings/index.js", include_str!("../web/js/settings/index.js")),
    ("settings/conn.js", include_str!("../web/js/settings/conn.js")),
    ("settings/groups.js", include_str!("../web/js/settings/groups.js")),
    ("settings/agents.js", include_str!("../web/js/settings/agents.js")),
    ("settings/prefs.js", include_str!("../web/js/settings/prefs.js")),
    ("settings/debug.js", include_str!("../web/js/settings/debug.js")),
    ("logic/format.js", include_str!("../web/js/logic/format.js")),
    ("logic/conv.js", include_str!("../web/js/logic/conv.js")),
    ("logic/message.js", include_str!("../web/js/logic/message.js")),
    ("logic/context.js", include_str!("../web/js/logic/context.js")),
    ("logic/markdown.js", include_str!("../web/js/logic/markdown.js")),
    ("logic/peer.js", include_str!("../web/js/logic/peer.js")),
    ("logic/inbox.js", include_str!("../web/js/logic/inbox.js")),
    ("logic/author.js", include_str!("../web/js/logic/author.js")),
    ("logic/order.js", include_str!("../web/js/logic/order.js")),
    ("logic/rail.js", include_str!("../web/js/logic/rail.js")),
    ("logic/home.js", include_str!("../web/js/logic/home.js")),
    ("logic/request.js", include_str!("../web/js/logic/request.js")),
    ("logic/activity.js", include_str!("../web/js/logic/activity.js")),
    ("logic/settings.js", include_str!("../web/js/logic/settings.js")),
    ("logic/agent.js", include_str!("../web/js/logic/agent.js")),
    ("logic/shell.js", include_str!("../web/js/logic/shell.js")),
    ("logic/sync.js", include_str!("../web/js/logic/sync.js")),
    ("logic/search.js", include_str!("../web/js/logic/search.js")),
];

/// 화면 글꼴 — IBM Plex Sans KR(SIL OFL 1.1, IBM 배포 woff2 그대로 · 완성형 한글)과 그 라이선스. 파일을 바꾸면 이름도 바꾼다(1년 캐시).
const FONTS: [(&str, &str, &[u8]); 4] = [
    (
        "IBMPlexSansKR-Regular.woff2",
        "font/woff2",
        include_bytes!("../web/fonts/IBMPlexSansKR-Regular.woff2"),
    ),
    (
        "IBMPlexSansKR-Medium.woff2",
        "font/woff2",
        include_bytes!("../web/fonts/IBMPlexSansKR-Medium.woff2"),
    ),
    (
        "IBMPlexSansKR-SemiBold.woff2",
        "font/woff2",
        include_bytes!("../web/fonts/IBMPlexSansKR-SemiBold.woff2"),
    ),
    (
        "LICENSE.txt",
        "text/plain; charset=utf-8",
        include_bytes!("../web/fonts/LICENSE.txt"),
    ),
];

fn is_public(path: &str) -> bool {
    PUBLIC.contains(&path)
        || path
            .strip_prefix("/fonts/")
            .is_some_and(|f| FONTS.iter().any(|(name, ..)| *name == f))
        || path
            .strip_prefix("/js/")
            .is_some_and(|f| SCRIPTS.iter().any(|(name, _)| *name == f))
}

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
    if req.method() == Method::GET && is_public(path) {
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

async fn script(Path(file): Path<String>) -> Response {
    match SCRIPTS.iter().find(|(name, _)| *name == file) {
        Some((_, body)) => (
            [(header::CONTENT_TYPE, "text/javascript; charset=utf-8")],
            *body,
        )
            .into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn app_css() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/css; charset=utf-8")],
        include_str!("../web/app.css"),
    )
}

async fn font(Path(file): Path<String>) -> Response {
    match FONTS.iter().find(|(name, ..)| *name == file) {
        Some((_, mime, body)) => (
            [
                (header::CONTENT_TYPE, *mime),
                (header::CACHE_CONTROL, "public, max-age=31536000, immutable"),
            ],
            *body,
        )
            .into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
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
        r#"{{"version":"{}","data_dir":"{}","db":"{}","cli_api":{CLI_API},"mode":"{}"}}"#,
        json_escape(&st.version),
        json_escape(&st.data_dir),
        json_escape(&st.db),
        s.mode
    );
    ([(header::CONTENT_TYPE, "application/json")], body)
}

/// 디버깅 화면 로그 끝 — 줄 수 · 파일 끝에서 읽는 바이트 상한.
pub const LOG_TAIL_LINES: usize = 200;
pub const LOG_TAIL_BYTES: u64 = 64 * 1024;

/// `GET /api/debug`(디버깅 화면) — 버전 · OS · 데이터 폴더 · DB · 로그 파일의 경로와 크기(없으면 null).
/// 신원 키 · 토큰(`instance.json`) · 지문은 내지 않는다.
async fn api_debug(State(s): State<Arc<AppState>>) -> Response {
    let paths = an_core::paths::Paths::under(&s.status.data_dir);
    let file = |key: &str, p: std::path::PathBuf| {
        json!({ "key": key, "path": p.display().to_string(), "size": std::fs::metadata(&p).ok().map(|m| m.len()) })
    };
    Json(json!({
        "version": s.status.version,
        "os": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "data_dir": s.status.data_dir,
        "files": [file("db", paths.db_file()), file("log", paths.log_file())],
    }))
    .into_response()
}

/// `GET /api/debug/log` — 로그 파일 끝 [`LOG_TAIL_BYTES`] 안의 마지막 [`LOG_TAIL_LINES`] 줄. 파일이 없으면 빈 목록.
async fn api_debug_log(State(s): State<Arc<AppState>>) -> Response {
    let path = an_core::paths::Paths::under(&s.status.data_dir).log_file();
    match tail_lines(&path, LOG_TAIL_BYTES, LOG_TAIL_LINES) {
        Ok(lines) => Json(json!({ "path": path.display().to_string(), "lines": lines })).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("로그를 읽지 못했습니다: {e}") })),
        )
            .into_response(),
    }
}

/// 파일 끝 `max_bytes` 안의 마지막 `max_lines` 줄 — 중간에서 시작했으면 잘린 첫 줄은 버린다(UTF-8 이 아닌 바이트는 대체 문자).
pub fn tail_lines(path: &std::path::Path, max_bytes: u64, max_lines: usize) -> std::io::Result<Vec<String>> {
    use std::io::{Read, Seek, SeekFrom};
    let mut f = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(e),
    };
    let start = f.metadata()?.len().saturating_sub(max_bytes);
    f.seek(SeekFrom::Start(start))?;
    let mut buf = Vec::new();
    f.take(max_bytes).read_to_end(&mut buf)?;
    let text = String::from_utf8_lossy(&buf);
    let mut lines: Vec<&str> = text.lines().collect();
    if start > 0 && !lines.is_empty() {
        lines.remove(0);
    }
    let skip = lines.len().saturating_sub(max_lines);
    Ok(lines[skip..].iter().map(|l| l.to_string()).collect())
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

/// 204 를 먼저 돌려주고 잠깐 뒤 종료를 부른다 — 응답이 나가기 전에 런타임이 멈추지 않게.
async fn api_quit(State(s): State<Arc<AppState>>) -> StatusCode {
    log::info!("종료 요청(로컬 API)");
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        if let Some(quit) = &s.quit {
            quit();
        }
    });
    StatusCode::NO_CONTENT
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
