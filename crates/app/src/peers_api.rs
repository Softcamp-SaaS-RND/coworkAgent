//! 내 정보 · 피어 API(S4 · D8) — `server::router_with` 의 guard 뒤에 붙는다. 원격에서 온 이름은 웹UI 가 textContent 로만 그린다.
use an_core::peers::Peer;
use an_net::identity::fingerprint;
use an_net::invite::Invite;
use an_net::node::{Node, PAIRING_WINDOW};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::io;
use std::net::SocketAddr;
use std::sync::Arc;

#[derive(Serialize)]
struct PeerView {
    id: i64,
    name: String,
    host: String,
    addr: String,
    fingerprint: String,
    my_ok: bool,
    their_ok: bool,
    paired: bool,
}

impl From<Peer> for PeerView {
    fn from(p: Peer) -> Self {
        Self {
            id: p.id,
            fingerprint: fingerprint(&p.spki),
            paired: p.paired(),
            name: p.name,
            host: p.host,
            addr: p.addr,
            my_ok: p.my_ok,
            their_ok: p.their_ok,
        }
    }
}

#[derive(Deserialize)]
struct AddPeer {
    #[serde(default)]
    addr: String,
    /// 상대 화면 "페어링 받기" 의 초대 코드 — 처음 보는 사이에만 필요.
    #[serde(default)]
    code: Option<String>,
    /// 메신저로 받은 초대 텍스트 통째(P10) — 있으면 `addr` · `code` 대신 쓰고, 지문을 대조한 뒤 내 승인까지 한다.
    #[serde(default)]
    invite: Option<String>,
}

pub fn routes(node: Arc<Node>) -> Router {
    Router::new()
        .route("/api/me", get(me))
        .route("/api/peers", get(list).post(add))
        .route("/api/peers/{id}", delete(remove))
        .route("/api/peers/{id}/approve", post(approve))
        .route("/api/peers/{id}/ping", post(ping))
        .route("/api/pairing-window", post(open_window))
        .with_state(node)
}

fn fail(e: io::Error) -> Response {
    let code = match e.kind() {
        io::ErrorKind::NotFound => StatusCode::NOT_FOUND,
        io::ErrorKind::InvalidInput => StatusCode::BAD_REQUEST,
        _ => StatusCode::BAD_GATEWAY,
    };
    (code, Json(json!({ "error": e.to_string() }))).into_response()
}

async fn me(State(node): State<Arc<Node>>) -> Json<serde_json::Value> {
    let me = node.me();
    let addrs: Vec<String> = lan_ip()
        .map(|ip| SocketAddr::new(ip, me.port).to_string())
        .into_iter()
        .collect();
    Json(json!({
        "name": me.name,
        "host": me.host,
        "fingerprint": node.identity().fingerprint(),
        "port": me.port,
        "addrs": addrs,
    }))
}

/// 기본 경로의 내 IPv4 — UDP `connect` 는 패킷을 보내지 않고 경로만 고른다.
fn lan_ip() -> Option<std::net::IpAddr> {
    let s = std::net::UdpSocket::bind(("0.0.0.0", 0)).ok()?;
    s.connect(("192.0.2.1", 9)).ok()?;
    Some(s.local_addr().ok()?.ip()).filter(|ip| !ip.is_unspecified())
}

async fn list(State(node): State<Arc<Node>>) -> Response {
    match node.peers() {
        Ok(ps) => Json(ps.into_iter().map(PeerView::from).collect::<Vec<_>>()).into_response(),
        Err(e) => fail(e),
    }
}

async fn add(State(node): State<Arc<Node>>, Json(req): Json<AddPeer>) -> Response {
    if let Some(text) = req.invite.as_deref().filter(|t| !t.trim().is_empty()) {
        let Some(invite) = Invite::parse(text) else {
            return fail(io::Error::new(
                io::ErrorKind::InvalidInput,
                "초대 텍스트에서 coworkagent-invite 토큰을 찾지 못했다",
            ));
        };
        // 초대의 지문과 맞는 상대에게만 붙고(불일치면 연결 전 실패), 붙여 넣은 사람의 승인으로 본다.
        let added = match node.add_invite(&invite).await {
            Ok(p) => p,
            Err(e) => return fail(e),
        };
        return match node.approve(added.id).await {
            Ok(p) => (StatusCode::CREATED, Json(PeerView::from(p))).into_response(),
            Err(e) => fail(e),
        };
    }
    let Ok(addr) = req.addr.trim().parse::<SocketAddr>() else {
        return fail(io::Error::new(
            io::ErrorKind::InvalidInput,
            "주소는 IP:포트 형식이어야 한다",
        ));
    };
    let code = req.code.filter(|c| !c.trim().is_empty());
    match node.add(addr, code).await {
        Ok(p) => (StatusCode::CREATED, Json(PeerView::from(p))).into_response(),
        Err(e) => fail(e),
    }
}

async fn approve(State(node): State<Arc<Node>>, Path(id): Path<i64>) -> Response {
    match node.approve(id).await {
        Ok(p) => Json(PeerView::from(p)).into_response(),
        Err(e) => fail(e),
    }
}

async fn remove(State(node): State<Arc<Node>>, Path(id): Path<i64>) -> Response {
    match node.remove(id).await {
        Ok(notified) => Json(json!({ "notified": notified })).into_response(),
        Err(e) => fail(e),
    }
}

async fn ping(State(node): State<Arc<Node>>, Path(id): Path<i64>) -> Response {
    match node.ping(id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => fail(e),
    }
}

/// "페어링 받기" — 초대 코드를 담은 처음 보는 PC 의 요청 1건을 `PAIRING_WINDOW` 동안 받는다.
async fn open_window(State(node): State<Arc<Node>>) -> Json<serde_json::Value> {
    let ip = lan_ip().unwrap_or(std::net::Ipv4Addr::LOCALHOST.into());
    let invite = node.open_invite(ip, PAIRING_WINDOW);
    let me = node.me();
    Json(json!({
        "seconds": PAIRING_WINDOW.as_secs(),
        "code": invite.code,
        "invite": invite.to_text(&me.name, &me.host),
    }))
}
