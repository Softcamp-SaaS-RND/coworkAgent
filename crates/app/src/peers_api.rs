//! 내 정보 · 피어 API(S4 · D8) — `server::router_with` 의 guard 뒤에 붙는다. 원격에서 온 이름은 웹UI 가 textContent 로만 그린다.
use an_core::peers::Peer;
use an_core::run_opts;
use an_net::identity::fingerprint;
use an_net::invite::Invite;
use an_net::node::{Node, PAIRING_WINDOW};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::io;
use std::net::SocketAddr;
use std::sync::Arc;

#[derive(Serialize)]
pub(crate) struct PeerView {
    id: i64,
    name: String,
    host: String,
    addr: String,
    fingerprint: String,
    my_ok: bool,
    their_ok: bool,
    paired: bool,
    group: Option<String>,
    /// 이 상대 요청 실행의 덮어쓰기 — `null` 이면 전역 기본값.
    model: Option<String>,
    effort: Option<String>,
    /// 이 PC 에서 부르는 표시 이름 · 타일 아이콘(PNG data URL) — `null` 이면 원래 이름 · 글자 타일.
    alias: Option<String>,
    icon: Option<String>,
    /// 접속 상태 — 마지막 `Pong` 이 [`an_core::peers::ONLINE_WITHIN`] 안이면 온라인, 그때 상대가 집중 중이었나(오프라인이면 거짓). 시각은 내지 않는다(레일이
    /// 30초마다 다시 그려지지 않게).
    online: bool,
    focus: bool,
}

impl From<Peer> for PeerView {
    fn from(p: Peer) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| i64::try_from(d.as_secs()).unwrap_or(i64::MAX));
        let online = p.online(now);
        Self {
            id: p.id,
            fingerprint: fingerprint(&p.spki),
            paired: p.paired(),
            name: p.name,
            host: p.host,
            addr: p.addr,
            my_ok: p.my_ok,
            their_ok: p.their_ok,
            group: p.group,
            model: p.model,
            effort: p.effort,
            alias: p.alias,
            icon: p.icon,
            online,
            focus: online && p.focus,
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

/// `PATCH /api/peers/{id}` — 준 항목만 바꾼다. 빈 문자열 = 지움(그룹 없음 · 전역 기본값 · 원래 이름 · 글자 타일).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PeerPatch {
    #[serde(default)]
    group: Option<String>,
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    effort: Option<String>,
    #[serde(default)]
    alias: Option<String>,
    #[serde(default)]
    icon: Option<String>,
}

pub fn routes(node: Arc<Node>) -> Router {
    Router::new()
        .route("/api/me", get(me))
        .route("/api/peers", get(list).post(add))
        .route(
            "/api/peers/{id}",
            axum::routing::delete(remove).patch(patch),
        )
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
    let with_code = code.is_some();
    let added = match node.add(addr, code).await {
        Ok(p) => p,
        Err(e) => return fail(e),
    };
    // 코드가 맞아 초대 생성 측이 이미 승인했으면 붙인 사람의 승인으로 본다 — 초대 붙여넣기와 같다(D8 추가 수용: 직접 추가 경로 포함).
    // 아는 키의 다시 추가는 상대가 승인하지 않으므로(their_ok 거짓) 승인 대기로 남는다.
    if with_code && added.their_ok && !added.my_ok {
        return match node.approve(added.id).await {
            Ok(p) => (StatusCode::CREATED, Json(PeerView::from(p))).into_response(),
            Err(e) => fail(e),
        };
    }
    (StatusCode::CREATED, Json(PeerView::from(added))).into_response()
}

async fn patch(
    State(node): State<Arc<Node>>,
    Path(id): Path<i64>,
    Json(req): Json<PeerPatch>,
) -> Response {
    let invalid = |m: String| fail(io::Error::new(io::ErrorKind::InvalidInput, m));
    let peer = match node.peers().map(|ps| ps.into_iter().find(|p| p.id == id)) {
        Ok(Some(p)) => p,
        Ok(None) => return fail(io::Error::new(io::ErrorKind::NotFound, "없는 피어")),
        Err(e) => return fail(e),
    };
    // 준 쪽만 쓴다 — 화면은 실행 설정(그룹 · model · effort)과 타일(표시 이름 · 아이콘)을 따로 보낸다. 한쪽만 고치는 요청이
    // 미리 읽은 다른 쪽 값을 다시 써서 그 사이 바뀐 값을 덮지 않게. 두 쪽을 함께 주면 차례로 쓴다(한 트랜잭션은 아니다).
    let meta = req.group.is_some() || req.model.is_some() || req.effort.is_some();
    let tile = req.alias.is_some() || req.icon.is_some();
    if !meta && !tile {
        return Json(PeerView::from(peer)).into_response();
    }
    // 없음 = 그대로, 빈 문자열 = 지움.
    let pick = |given: Option<String>, now: Option<String>| match given {
        None => now,
        Some(v) => Some(v.trim().to_string()).filter(|v| !v.is_empty()),
    };
    let group = match req.group.as_deref().map(run_opts::clean_group) {
        None => peer.group,
        Some(Ok(g)) => g,
        Some(Err(m)) => return invalid(m),
    };
    // 준 값만 검증한다 — 저장된 값(이전 버전이 남긴 틀린 값 포함)은 그대로 둔다(실행은 틀린 값 대신 기본값을 쓴다).
    if let Some(m) = req.model.as_deref().map(str::trim)
        && !m.is_empty()
        && !run_opts::valid_model(m)
    {
        return invalid("model 은 1 ~ 64자 영문 · 숫자 · ._:[]- (- 로 시작하지 않음)".into());
    }
    if let Some(e) = req.effort.as_deref().map(str::trim)
        && !e.is_empty()
        && !run_opts::valid_effort(e)
    {
        return invalid(format!("effort 는 {}", run_opts::EFFORTS.join(" · ")));
    }
    let model = pick(req.model, peer.model);
    let effort = pick(req.effort, peer.effort);
    let alias = match req.alias.as_deref().map(run_opts::clean_alias) {
        None => peer.alias,
        Some(Ok(a)) => a,
        Some(Err(m)) => return invalid(m),
    };
    let icon = match req.icon.as_deref().map(run_opts::clean_icon) {
        None => peer.icon,
        Some(Ok(i)) => i,
        Some(Err(m)) => return invalid(m),
    };
    // 전부 검증한 뒤에 쓴다 — 틀린 값이 하나라도 있으면 아무것도 바꾸지 않는다.
    let saved = if tile {
        if meta
            && let Err(e) =
                node.set_peer_meta(id, group.as_deref(), model.as_deref(), effort.as_deref())
        {
            return fail(e);
        }
        node.set_peer_tile(id, alias.as_deref(), icon.as_deref())
    } else {
        node.set_peer_meta(id, group.as_deref(), model.as_deref(), effort.as_deref())
    };
    match saved {
        Ok(p) => Json(PeerView::from(p)).into_response(),
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

/// 연결 확인 — 받은 `Pong` 은 접속 상태로 남는다(`GET /api/peers` 의 `online` · `focus`).
async fn ping(State(node): State<Arc<Node>>, Path(id): Path<i64>) -> Response {
    match node.ping(id).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
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
