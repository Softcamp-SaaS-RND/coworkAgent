//! 피어 노드 — 수신(인가 · 페어링 응답)과 발신(주소 추가 · 승인 · 삭제 · ping).
use crate::identity::Identity;
use crate::invite::{Invite, normalize_fingerprint};
use crate::proto::{Request, Response, clean_name, read_frame, write_frame};
use crate::tls::{self, Pin};
use an_core::db::{Connection, db_err};
use an_core::peers::{self, Peer};
use ring::rand::{SecureRandom, SystemRandom};
use rustls::ServerConfig;
use rustls::pki_types::ServerName;
use std::collections::HashMap;
use std::io;
use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Semaphore;
use tokio_rustls::{TlsAcceptor, TlsConnector};

/// 피어 수신 기본 포트.
pub const PEER_PORT: u16 = 47100;
/// 발신 연결 1개(TLS 포함 요청·응답 전체)의 상한.
pub const IO_DEADLINE: Duration = Duration::from_secs(10);
/// 수신 연결 1개의 상한 — 인증 전 느린 연결이 자리를 오래 붙잡지 못하게 발신보다 짧다.
pub const INBOUND_DEADLINE: Duration = Duration::from_secs(3);
/// 동시에 처리하는 수신 연결 수. 넘치면 accept 를 멈추지 않고 바로 끊는다.
pub const MAX_CONNS: usize = 32;
/// 출처 IP 하나가 동시에 쥘 수 있는 수신 연결 수.
pub const MAX_CONNS_PER_IP: usize = 4;
/// 처음 보는 키의 hello 를 받아 줄, 내가 승인하지 않은 피어 수 상한.
pub const MAX_UNAPPROVED: i64 = 32;
/// 웹UI "페어링 받기" 가 여는 창의 길이.
pub const PAIRING_WINDOW: Duration = Duration::from_secs(300);
/// 한 창에서 출처 IP 하나가 틀릴 수 있는 초대 코드 수 — 넘으면 그 창 동안 그 IP 의 처음 보는 키 hello 를 거절한다.
pub const MAX_CODE_MISSES: u32 = 10;

/// 상대에게 알리는 나.
#[derive(Debug, Clone)]
pub struct Me {
    pub name: String,
    pub host: String,
    pub port: u16,
}

/// 인바운드 이벤트 — 알림 같은 바깥 층이 받는다. 이름 · 호스트는 `clean_name` 을 거친 원격 문자열이다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PeerEvent {
    /// 페어링 요청이 받아들여졌다(처음 보는 키 · 다시 추가).
    PairRequest {
        name: String,
        host: String,
        fingerprint: String,
    },
    /// 상대 승인이 같은 세대에 반영됐다.
    Approved { name: String, host: String },
    /// 초대 코드로 들어온 처음 보는 PC 를 승인 없이 등록했다(D8 — 모르는 연결이면 바로 삭제).
    AutoPaired {
        name: String,
        host: String,
        fingerprint: String,
    },
    /// 원격 메시지가 왔다 — 받은 요청(내 에이전트에게) 또는 받은 답(원격 에이전트에게서). `preview` 는 앞부분만
    /// (`RelayEnd` 면 이 PC 가 만든 멈춘 사유).
    Message {
        name: String,
        host: String,
        agent: String,
        preview: String,
        kind: MessageKind,
        /// 이 PC 대화 id — 웹UI 변경 신호.
        conv: i64,
    },
    /// 상대 요청을 처리했다 — 알림이 아닌 웹UI 변경 신호만. 이 PC 대화 id, 못 찾으면 `None`.
    Changed { conv: Option<i64> },
}

/// 원격 메시지 종류 — 알림 거르기(종류별 · 자동 왕복 묶기)에 쓴다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageKind {
    /// 사람이 보낸 요청 · 글.
    Request,
    /// 사람 요청에 온 에이전트 답.
    Reply,
    /// 자동 왕복 시작(받은 쪽 첫 에이전트발 요청 · 재개).
    RelayStart,
    /// 자동 왕복 중간 글.
    RelayStep,
    /// 자동 왕복이 저절로 멈췄다(상한 · 실패 · 거절 · 상대 멈춤).
    RelayEnd,
    /// 받은 요청(실행 없음, 요청 목록).
    Ask,
    /// 내 요청의 상태가 바뀌었다(상대가 수용 · 반려 · 완료).
    AskState,
    /// 내 대화의 관람자 수가 바뀌었다(상대가 그 대화를 공유했다).
    Share,
    /// 상대가 그 대화 기록을 모두 지웠다(이 PC 쪽도 비웠다).
    Clear,
}

type EventSink = Box<dyn Fn(PeerEvent) + Send + Sync>;
/// 페어링된 상대의 요청 처리기(에이전트 목록 · 대화 요청) — 막히면 안 된다(느린 일은 큐로).
type RequestHandler = Box<dyn Fn(&Peer, Request) -> Response + Send + Sync>;
/// 페어링된 상대의 `Ping` 을 받았을 때(상대 id) — 막히면 안 된다.
type SeenHandler = Box<dyn Fn(i64) + Send + Sync>;

/// [`Node::set_peer_fields`] 의 실행 설정 묶음(그룹, model, effort).
pub type PeerMeta<'a> = (Option<&'a str>, Option<&'a str>, Option<&'a str>);
/// [`Node::set_peer_fields`] 의 타일 묶음(표시 이름, 아이콘).
pub type PeerTile<'a> = (Option<&'a str>, Option<&'a str>);

pub struct Node {
    id: Identity,
    server: Arc<ServerConfig>,
    db: Mutex<Connection>,
    me: Me,
    /// 처음 보는 키의 hello 를 1건 받아 줄 (마감 시각, 초대 코드). 코드가 맞는 hello 를 받으면 닫힌다.
    window: Mutex<Option<(Instant, String)>>,
    /// 지금 창에서 출처 IP 별 틀린 초대 코드 수 — [`MAX_CODE_MISSES`] 에 닿으면 그 IP 는 창이 끝날 때까지 거절.
    misses: Mutex<HashMap<IpAddr, u32>>,
    /// 피어 수신 전체 자리([`MAX_CONNS`]) — 쓰는 수 = 열린 연결 수.
    slots: Arc<Semaphore>,
    events: OnceLock<EventSink>,
    requests: OnceLock<RequestHandler>,
    /// 페어링된 상대가 `Ping` 했다(돌아온 증거) — 바깥 층이 그 상대에 남은 글을 지금 보낸다.
    seen: OnceLock<SeenHandler>,
    /// 시작 때 피어 포트를 잡고 있던 프로세스(임의 포트로 열었을 때만).
    holder: OnceLock<Holder>,
}

/// 출처 IP 별 수신 연결 수 — 놓으면 줄어든다.
struct IpSlot {
    counts: Arc<Mutex<HashMap<IpAddr, usize>>>,
    ip: IpAddr,
}

impl IpSlot {
    fn take(counts: &Arc<Mutex<HashMap<IpAddr, usize>>>, ip: IpAddr) -> Option<Self> {
        let mut m = an_core::sync::lock(counts);
        let n = m.entry(ip).or_insert(0);
        if *n >= MAX_CONNS_PER_IP {
            return None;
        }
        *n += 1;
        Some(Self {
            counts: counts.clone(),
            ip,
        })
    }
}

impl Drop for IpSlot {
    fn drop(&mut self) {
        let mut m = an_core::sync::lock(&self.counts);
        if let Some(n) = m.get_mut(&self.ip) {
            *n -= 1;
            if *n == 0 {
                m.remove(&self.ip);
            }
        }
    }
}

/// `0.0.0.0:47100`, 사용 중이면 OS 임의 포트 — 돌려주는 값의 둘째는 47100 을 잡은 프로세스(임의 포트로 열었을 때만).
pub async fn bind_peer_port() -> io::Result<(TcpListener, Option<Holder>)> {
    bind_peer_port_on(PEER_PORT).await
}

/// 포트를 잡은 프로세스. 찾지 못하면 이름 `알 수 없음` · PID 0.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Holder {
    pub name: String,
    pub pid: u32,
}

/// `bind_peer_port` 의 포트 지정판 — 못 열면 임의 포트로 연다. 남이 쓰는 중(`AddrInUse`)일 때만 잡은 쪽을 찾는다(예약 포트 · 권한은 로그만).
pub async fn bind_peer_port_on(port: u16) -> io::Result<(TcpListener, Option<Holder>)> {
    match TcpListener::bind(("0.0.0.0", port)).await {
        Ok(l) => Ok((l, None)),
        Err(e) if e.kind() != io::ErrorKind::AddrInUse => {
            log::warn!("피어 포트 {port} 를 열 수 없다({e}) — 임의 포트로 연다");
            Ok((TcpListener::bind(("0.0.0.0", 0)).await?, None))
        }
        Err(e) => {
            let holder = port_holder(port).unwrap_or(Holder {
                name: "알 수 없음".into(),
                pid: 0,
            });
            log::warn!(
                "피어 포트 {port} 사용 불가({e}) — {}(PID {}) 가 잡고 있다 · 임의 포트로 연다",
                holder.name,
                holder.pid
            );
            Ok((TcpListener::bind(("0.0.0.0", 0)).await?, Some(holder)))
        }
    }
}

/// TCP `port` 를 듣는(LISTEN) 프로세스 — Windows `netstat -ano` + `tasklist`, Unix `lsof`. 못 찾으면 `None`.
pub fn port_holder(port: u16) -> Option<Holder> {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        let run = |prog: &str, args: &[&str]| {
            std::process::Command::new(prog)
                .args(args)
                .creation_flags(CREATE_NO_WINDOW)
                .output()
                .ok()
                .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
        };
        // "  TCP    0.0.0.0:47100    0.0.0.0:0    LISTENING    1234" — 상태 글자는 언어에 따라 다를 수 있어 원격 주소 :0 으로 듣기를 가린다.
        let out = run("netstat", &["-ano", "-p", "TCP"])?;
        let pid: u32 = out.lines().find_map(|l| {
            let f: Vec<&str> = l.split_whitespace().collect();
            let local_port = f.get(1)?.rsplit(':').next()?;
            (f.first() == Some(&"TCP")
                && local_port == port.to_string()
                && f.get(2)?.ends_with(":0"))
            .then(|| f.last()?.parse().ok())
            .flatten()
        })?;
        let csv = run(
            "tasklist",
            &["/FI", &format!("PID eq {pid}"), "/FO", "CSV", "/NH"],
        )?;
        let name = csv
            .lines()
            .find_map(|l| l.strip_prefix('"')?.split('"').next().map(str::to_string))
            .unwrap_or_else(|| "알 수 없음".into());
        Some(Holder { name, pid })
    }
    #[cfg(unix)]
    {
        let out = std::process::Command::new("lsof")
            .args(["-nP", &format!("-iTCP:{port}"), "-sTCP:LISTEN", "-Fpc"])
            .output()
            .ok()?;
        let text = String::from_utf8_lossy(&out.stdout);
        let pid = text
            .lines()
            .find_map(|l| l.strip_prefix('p')?.parse().ok())?;
        let name = text
            .lines()
            .find_map(|l| l.strip_prefix('c').map(str::to_string))
            .unwrap_or_else(|| "알 수 없음".into());
        Some(Holder { name, pid })
    }
}

fn random_bytes<const N: usize>() -> [u8; N] {
    let mut b = [0u8; N];
    SystemRandom::new()
        .fill(&mut b)
        .expect("OS 난수원을 쓸 수 없다");
    b
}

/// 페어링 세대 — 16바이트 난수 hex.
fn new_round() -> String {
    random_bytes::<16>()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// 초대 코드 알파벳 — 헷갈리는 0·O·1·I·L 제외 31자. 8자 ≈ 39.6 비트.
const CODE_ALPHABET: &[u8] = b"ABCDEFGHJKMNPQRSTUVWXYZ23456789";

/// `ABCD-EFGH` 꼴 초대 코드. 248 이상 바이트는 버려 편향을 없앤다(248 = 31 × 8).
fn new_code() -> String {
    let mut out = String::new();
    while out.len() < 8 {
        for b in random_bytes::<16>() {
            if b < 248 && out.len() < 8 {
                out.push(CODE_ALPHABET[b as usize % 31] as char);
            }
        }
    }
    format!("{}-{}", &out[..4], &out[4..])
}

/// 대문자 · 영숫자만 남긴다 — 사람이 `abcd efgh` 로 적어도 같은 코드.
fn normalize_code(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_uppercase())
        .collect()
}

/// 길이가 같으면 내용과 무관하게 같은 시간에 비교한다.
fn ct_eq(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

fn not_found(id: i64) -> io::Error {
    io::Error::new(io::ErrorKind::NotFound, format!("피어 {id} 없음"))
}

/// 거절 응답 → 오류. 코드는 정해진 꼴만(`clean_code`) — 지문을 고정하지 않은 주소 추가에서는 임의 서버의 글자가 화면에 닿는다.
fn refused(resp: Response) -> io::Error {
    match resp {
        Response::Error { code } => io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!("상대가 거절: {}", crate::proto::clean_code(&code)),
        ),
        _ => io::Error::new(io::ErrorKind::InvalidData, "예상하지 못한 응답"),
    }
}

impl Node {
    pub fn new(id: Identity, db: Connection, me: Me) -> Arc<Self> {
        let server = tls::server_config(&id);
        Arc::new(Self {
            id,
            server,
            db: Mutex::new(db),
            me,
            window: Mutex::new(None),
            misses: Mutex::new(HashMap::new()),
            slots: Arc::new(Semaphore::new(MAX_CONNS)),
            events: OnceLock::new(),
            requests: OnceLock::new(),
            seen: OnceLock::new(),
            holder: OnceLock::new(),
        })
    }

    /// 페어링된 상대의 요청 처리기(에이전트 목록 · 대화 요청) — 한 번만 설정된다. 설정 전에는 `unsupported`.
    pub fn on_request(&self, f: impl Fn(&Peer, Request) -> Response + Send + Sync + 'static) {
        if self.requests.set(Box::new(f)).is_err() {
            log::warn!("요청 처리기는 한 번만 설정한다");
        }
    }

    /// 페어링된 상대의 `Ping` 을 받을 때 부를 함수(한 번만 설정).
    pub fn on_seen(&self, f: impl Fn(i64) + Send + Sync + 'static) {
        if self.seen.set(Box::new(f)).is_err() {
            log::warn!("Ping 처리기는 한 번만 설정한다");
        }
    }

    /// 바깥 층(에이전트 조율)이 내는 이벤트를 같은 콜백으로 흘린다.
    pub fn emit(&self, e: PeerEvent) {
        if let Some(sink) = self.events.get() {
            sink(e);
        }
    }

    /// 인바운드 이벤트 콜백 — 한 번만 설정된다(두 번째는 무시하고 로그). DB 잠금 밖, 응답을 쓰기 전에 수신 작업 안에서
    /// 불린다 — **막히면 안 된다**(느린 일은 콜백이 큐로 넘긴다).
    pub fn on_event(&self, f: impl Fn(PeerEvent) + Send + Sync + 'static) {
        if self.events.set(Box::new(f)).is_err() {
            log::warn!("이벤트 콜백은 한 번만 설정한다");
        }
    }

    /// 지금부터 `d` 동안, 돌려준 초대 코드를 담은 처음 보는 키의 hello 1건을 받는다. 다시 부르면 새 코드로 바뀐다.
    pub fn open_window(&self, d: Duration) -> String {
        let code = new_code();
        // 창 잠금 안에서 셈을 비운다 — 진행 중인 `take_window` 가 옛 창의 셈을 새 창으로 넘기지 않게.
        let mut w = an_core::sync::lock(&self.window);
        an_core::sync::lock(&self.misses).clear();
        *w = Some((Instant::now() + d, code.clone()));
        code
    }

    /// 창이 열려 있고 코드가 맞으면 닫고 true. 틀린 코드는 창을 닫지 않는다(선점 불가) — 대신 출처 IP 별로 세어 [`MAX_CODE_MISSES`]
    /// 번 틀린 IP 는 코드를 보지 않고 거절한다(추측 — 응답은 창 닫힘과 같아 상한을 알리지 않는다).
    fn take_window(&self, ip: IpAddr, code: Option<&str>) -> bool {
        // 셈 확인 · 증가는 창 잠금을 쥔 채로 — 같은 IP 의 동시 연결이 상한 경계를 넘어 더 추측하지 못하게(잠금 순서: window → misses).
        let mut w = an_core::sync::lock(&self.window);
        if an_core::sync::lock(&self.misses)
            .get(&ip)
            .is_some_and(|&n| n >= MAX_CODE_MISSES)
        {
            return false;
        }
        let Some((until, want)) = w.as_ref() else {
            return false;
        };
        if Instant::now() >= *until {
            *w = None;
            return false;
        }
        let given = normalize_code(code.unwrap_or(""));
        if !ct_eq(given.as_bytes(), normalize_code(want).as_bytes()) {
            *an_core::sync::lock(&self.misses).entry(ip).or_insert(0) += 1;
            return false;
        }
        *w = None;
        true
    }

    /// `bind_peer_port` 가 알려 준 잡은 프로세스를 기록한다(한 번).
    pub fn set_peer_holder(&self, h: Holder) {
        let _ = self.holder.set(h);
    }

    pub fn peer_holder(&self) -> Option<&Holder> {
        self.holder.get()
    }

    /// 트레이 · 내 정보 문구 — 피어 포트를 남이 잡아 임의 포트로 받을 때만.
    pub fn peer_port_note(&self) -> Option<String> {
        self.holder.get().map(|h| {
            format!(
                "피어 포트 {PEER_PORT} 사용 중 — {}(PID {}) · 임의 포트 {} 로 받는다",
                h.name, h.pid, self.me.port
            )
        })
    }

    /// 지금 열린 피어 수신 연결 수.
    pub fn open_connections(&self) -> usize {
        MAX_CONNS - self.slots.available_permits()
    }

    pub fn identity(&self) -> &Identity {
        &self.id
    }

    pub fn me(&self) -> &Me {
        &self.me
    }

    fn db(&self) -> std::sync::MutexGuard<'_, Connection> {
        an_core::sync::lock(&self.db)
    }

    pub fn peers(&self) -> io::Result<Vec<Peer>> {
        peers::list(&self.db()).map_err(db_err)
    }

    fn peer(&self, id: i64) -> io::Result<Peer> {
        peers::by_id(&self.db(), id)
            .map_err(db_err)?
            .ok_or_else(|| not_found(id))
    }

    /// 수신 루프 — 돌아오지 않는다.
    pub async fn serve(self: Arc<Self>, listener: TcpListener) {
        let slots = self.slots.clone();
        let per_ip = Arc::new(Mutex::new(HashMap::new()));
        loop {
            let (tcp, remote) = match listener.accept().await {
                Ok(x) => x,
                Err(e) => {
                    log::warn!("피어 수신 실패: {e}");
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    continue;
                }
            };
            // 출처별 몫을 먼저 — 한 호스트가 전체 자리를 채우지 못한다. 넘치면 바로 끊는다(accept 는 계속).
            let Some(ip_slot) = IpSlot::take(&per_ip, remote.ip()) else {
                continue;
            };
            let Ok(permit) = slots.clone().try_acquire_owned() else {
                continue;
            };
            let node = self.clone();
            tokio::spawn(async move {
                let _held = (permit, ip_slot);
                match tokio::time::timeout(INBOUND_DEADLINE, node.handle(tcp, remote)).await {
                    Ok(Ok(())) => {}
                    Ok(Err(e)) => log::info!("피어 {remote} 연결 종료: {e}"),
                    Err(_) => log::info!("피어 {remote} 시간 초과"),
                }
            });
        }
    }

    async fn handle(&self, tcp: TcpStream, remote: SocketAddr) -> io::Result<()> {
        let mut s = TlsAcceptor::from(self.server.clone()).accept(tcp).await?;
        let spki = s
            .get_ref()
            .1
            .peer_certificates()
            .and_then(|c| c.first())
            .ok_or_else(|| io::Error::new(io::ErrorKind::PermissionDenied, "클라이언트 키 없음"))?
            .to_vec();
        let req: Request = read_frame(&mut s).await?;
        let (resp, event) = self.answer(&spki, remote.ip(), req)?;
        if let (Some(e), Some(sink)) = (event, self.events.get()) {
            sink(e);
        }
        write_frame(&mut s, &resp).await?;
        s.shutdown().await
    }

    /// 인가 — 상대 키로 피어 표를 본다. 미페어링 상대에게는 hello · approve · remove 만 허용. 처음 보는 키의 hello 는
    /// "페어링 받기" 창의 초대 코드가 맞을 때 1건만. approve 는 같은 세대(`round`)에만.
    fn answer(
        &self,
        spki: &[u8],
        ip: IpAddr,
        req: Request,
    ) -> io::Result<(Response, Option<PeerEvent>)> {
        // 대화 요청은 페어링된 상대만 — DB 잠금을 놓고 처리기에 넘긴다(처리기는 자기 연결을 쓴다).
        if matches!(
            req,
            Request::Agents
                | Request::Message { .. }
                | Request::Reply { .. }
                | Request::Post { .. }
                | Request::Relay { .. }
                | Request::ReqState { .. }
                | Request::Progress { .. }
                | Request::Status { .. }
                | Request::Edit { .. }
                | Request::Delete { .. }
                | Request::Context { .. }
                | Request::Share { .. }
                | Request::SharePost { .. }
                | Request::ShareDelete { .. }
                | Request::ShareEnd { .. }
                | Request::ShareNotice { .. }
                | Request::Leave { .. }
                | Request::Clear { .. }
        ) {
            let peer = peers::by_spki(&self.db(), spki).map_err(db_err)?;
            let resp = match (peer, self.requests.get()) {
                (Some(p), _) if p.paired() && !req.ids_ok() => Response::error("invalid"),
                (Some(p), Some(h)) if p.paired() => {
                    // 받기도 통신 증거(주기 확인 대신). 기록 실패는 받기를 막지 않는다.
                    if let Err(e) = peers::touch(&self.db(), spki) {
                        log::debug!("접속 시각 기록 실패: {e}");
                    }
                    h(&p, req)
                }
                (Some(p), None) if p.paired() => Response::error("unsupported"),
                _ => Response::error("not_paired"),
            };
            return Ok((resp, None));
        }
        let db = self.db();
        let mut event = None;
        let resp = match req {
            Request::Hello {
                name,
                host,
                port,
                round,
                code,
            } => {
                let known = peers::by_spki(&db, spki).map_err(db_err)?.is_some();
                if port == 0 || round.is_empty() || round.len() > 64 || spki == self.id.spki() {
                    Response::error("bad_request")
                } else if !known && peers::count_unapproved(&db).map_err(db_err)? >= MAX_UNAPPROVED
                {
                    Response::error("busy")
                } else if !known && !self.take_window(ip, code.as_deref()) {
                    Response::error("closed")
                } else {
                    let addr = SocketAddr::new(ip, port).to_string();
                    let p = peers::upsert(
                        &db,
                        spki,
                        &clean_name(&name),
                        &clean_name(&host),
                        &addr,
                        &round,
                    )
                    .map_err(db_err)?;
                    // 처음 보는 키가 여기 왔다 = 받기 창의 초대 코드가 맞았다 → 내 승인은 자동(D8).
                    let auto = !known;
                    if auto {
                        peers::set_my_ok(&db, p.id).map_err(db_err)?;
                    }
                    let (name, host, fingerprint) =
                        (p.name, p.host, crate::identity::fingerprint(spki));
                    event = Some(if auto {
                        PeerEvent::AutoPaired {
                            name,
                            host,
                            fingerprint,
                        }
                    } else {
                        PeerEvent::PairRequest {
                            name,
                            host,
                            fingerprint,
                        }
                    });
                    Response::Hello {
                        name: self.me.name.clone(),
                        host: self.me.host.clone(),
                        approved: auto,
                    }
                }
            }
            Request::Approve { round } => {
                if peers::set_their_ok(&db, spki, &round).map_err(db_err)? {
                    if let Some(p) = peers::by_spki(&db, spki).map_err(db_err)? {
                        event = Some(PeerEvent::Approved {
                            name: p.name,
                            host: p.host,
                        });
                    }
                    Response::Ok
                } else {
                    match peers::by_spki(&db, spki).map_err(db_err)? {
                        // 같은 세대의 재통지("다시 알리기") — 응답은 같고 이벤트는 없다.
                        Some(p) if p.round == round && p.their_ok => Response::Ok,
                        Some(_) => Response::error("stale"),
                        None => Response::error("unknown"),
                    }
                }
            }
            Request::Remove => {
                peers::remove_spki(&db, spki).map_err(db_err)?;
                Response::Ok
            }
            Request::Ping { port } => {
                let peer = peers::by_spki(&db, spki)
                    .map_err(db_err)?
                    .filter(|p| p.paired());
                if let Some(p) = peer {
                    // 페어링된 키(TLS 인증) — 출처 IP · 알려 준 포트가 저장 주소와 다르면 따라간다(승인 그대로).
                    let now = SocketAddr::new(ip, port).to_string();
                    if port != 0 && now != p.addr {
                        peers::set_addr(&db, p.id, &now).map_err(db_err)?;
                        log::info!("피어 {} 주소 {} → {now}", p.name, p.addr);
                    }
                    // 받기도 통신 증거 · 돌아온 상대에 남은 글을 지금.
                    if let Err(e) = peers::touch(&db, spki) {
                        log::debug!("접속 시각 기록 실패: {e}");
                    }
                    if let Some(f) = self.seen.get() {
                        f(p.id);
                    }
                    // 못 읽으면 꺼짐으로 답한다 — 연결을 끊으면 상대가 이 PC 를 오프라인으로 본다(받기 판정도 못 읽으면 받는다).
                    let focus = an_core::settings::focus(&db).unwrap_or_else(|e| {
                        log::warn!("집중 설정 읽기 실패 — 꺼짐으로 답한다: {e}");
                        false
                    });
                    Response::Pong {
                        focus,
                        ctx: true,
                        share: true,
                        leave: true,
                        clear: true,
                    }
                } else {
                    Response::error("not_paired")
                }
            }
            // 위에서 처리기로 넘겼다.
            Request::Agents
            | Request::Message { .. }
            | Request::Reply { .. }
            | Request::Post { .. }
            | Request::Relay { .. }
            | Request::ReqState { .. }
            | Request::Progress { .. }
            | Request::Status { .. }
            | Request::Edit { .. }
            | Request::Delete { .. }
            | Request::Context { .. }
            | Request::Share { .. }
            | Request::SharePost { .. }
            | Request::ShareDelete { .. }
            | Request::ShareEnd { .. }
            | Request::ShareNotice { .. }
            | Request::Leave { .. }
            | Request::Clear { .. } => Response::error("bad_request"),
        };
        Ok((resp, event))
    }

    /// 요청 1개를 보내고 응답과 상대 SPKI 를 받는다. `pinned` 가 있으면 그 키가 아닌 상대와는 연결되지 않는다.
    pub async fn send(
        &self,
        addr: SocketAddr,
        pinned: Option<Vec<u8>>,
        req: &Request,
    ) -> io::Result<(Response, Vec<u8>)> {
        let got = self
            .exchange(addr, pinned.map_or(Pin::Any, Pin::Key), req)
            .await?;
        // 응답을 받았다 = 통신 성공(주기 확인 대신). 모르는 키면 아무 일도 없다 · 기록 실패는 보내기를 실패로 만들지 않는다.
        if let Err(e) = peers::touch(&self.db(), &got.1) {
            log::debug!("접속 시각 기록 실패: {e}");
        }
        Ok(got)
    }

    async fn exchange(
        &self,
        addr: SocketAddr,
        pin: Pin,
        req: &Request,
    ) -> io::Result<(Response, Vec<u8>)> {
        let connector = TlsConnector::from(tls::client_config_pin(&self.id, pin));
        let exchange = async {
            let tcp = TcpStream::connect(addr).await?;
            let mut s = connector
                .connect(ServerName::IpAddress(addr.ip().into()), tcp)
                .await?;
            let spki = s
                .get_ref()
                .1
                .peer_certificates()
                .and_then(|c| c.first())
                .ok_or_else(|| io::Error::new(io::ErrorKind::PermissionDenied, "서버 키 없음"))?
                .to_vec();
            write_frame(&mut s, req).await?;
            let resp: Response = read_frame(&mut s).await?;
            Ok((resp, spki))
        };
        tokio::time::timeout(IO_DEADLINE, exchange)
            .await
            .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "피어 응답 시간 초과"))?
    }

    /// 주소 추가(첫 연결 — 키 고정 없이). 처음 보는 사이면 상대 화면의 초대 코드가 필요하다. 양쪽에 같은 세대의 승인 대기
    /// 기록이 생긴다(이미 아는 사이면 양쪽 승인이 지워진다).
    pub async fn add(&self, addr: SocketAddr, code: Option<String>) -> io::Result<Peer> {
        self.add_pinned(addr, code, Pin::Any).await
    }

    /// 초대로 추가 — 초대에 실린 지문과 다른 상대와는 연결하지 않는다(hello 도 보내지 않는다).
    pub async fn add_invite(&self, invite: &Invite) -> io::Result<Peer> {
        self.add_pinned(
            invite.addr,
            Some(invite.code.clone()),
            Pin::Fingerprint(invite.fingerprint.clone()),
        )
        .await
    }

    /// 지금 받기 창의 초대(내 주소 · 코드 · 지문). 창을 새로 연다.
    pub fn open_invite(&self, ip: IpAddr, d: Duration) -> Invite {
        Invite {
            addr: SocketAddr::new(ip, self.me.port),
            code: self.open_window(d),
            fingerprint: normalize_fingerprint(&self.id.fingerprint()).expect("우리 지문 형식"),
        }
    }

    async fn add_pinned(
        &self,
        addr: SocketAddr,
        code: Option<String>,
        pin: Pin,
    ) -> io::Result<Peer> {
        let round = new_round();
        let hello = Request::Hello {
            name: self.me.name.clone(),
            host: self.me.host.clone(),
            port: self.me.port,
            round: round.clone(),
            code,
        };
        let (resp, spki) = self.exchange(addr, pin, &hello).await.map_err(|e| {
            if e.to_string().contains("ApplicationVerificationFailure") {
                io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "상대 지문이 초대와 다르다 — 연결하지 않았다",
                )
            } else {
                e
            }
        })?;
        if spki == self.id.spki() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "자기 자신의 주소",
            ));
        }
        match resp {
            Response::Hello {
                name,
                host,
                approved,
            } => {
                let db = self.db();
                let p = peers::upsert(
                    &db,
                    &spki,
                    &clean_name(&name),
                    &clean_name(&host),
                    &addr.to_string(),
                    &round,
                )
                .map_err(db_err)?;
                if !approved {
                    return Ok(p);
                }
                peers::set_their_ok(&db, &spki, &round).map_err(db_err)?;
                peers::by_id(&db, p.id).map_err(db_err)?.ok_or_else(|| {
                    io::Error::new(io::ErrorKind::NotFound, "방금 등록한 상대가 없다")
                })
            }
            other => Err(refused(other)),
        }
    }

    /// 이 PC 만의 상대 설정을 한 트랜잭션으로 바꾼다 — 실행 설정(그룹 · model · effort) · 타일(표시 이름 · 아이콘) 은 통째로(`None` = 지움),
    /// 타일 색은 값 하나. `None` 인 묶음은 그대로, 하나라도 실패하면 아무것도 바꾸지 않는다. 상대에 알리지 않는다.
    pub fn set_peer_fields(
        &self,
        id: i64,
        meta: Option<PeerMeta<'_>>,
        tile: Option<PeerTile<'_>>,
        color: Option<&str>,
    ) -> io::Result<Peer> {
        {
            let db = self.db();
            let tx = an_core::db::write_tx(&db).map_err(db_err)?;
            let mut found = true;
            if let Some((group, model, effort)) = meta {
                found &= peers::set_meta(&tx, id, group, model, effort).map_err(db_err)?;
            }
            if let Some((alias, icon)) = tile {
                found &= peers::set_tile(&tx, id, alias, icon).map_err(db_err)?;
            }
            if let Some(c) = color {
                found &= peers::set_color(&tx, id, c).map_err(db_err)?;
            }
            if !found {
                return Err(io::Error::new(io::ErrorKind::NotFound, "없는 피어"));
            }
            tx.commit().map_err(db_err)?;
        }
        self.peer(id)
    }

    /// 내 승인 → 상대에 통지. 통지가 실패해도 내 승인은 남는다(다시 부르면 재통지).
    pub async fn approve(&self, id: i64) -> io::Result<Peer> {
        let peer = self.peer(id)?;
        peers::set_my_ok(&self.db(), id).map_err(db_err)?;
        let req = Request::Approve { round: peer.round };
        match self
            .send(parse_addr(&peer.addr)?, Some(peer.spki), &req)
            .await?
        {
            (Response::Ok, _) => self.peer(id),
            (other, _) => Err(refused(other)),
        }
    }

    /// 내 기록 삭제 → 상대에 통지. 통지가 실패해도 내 쪽 삭제는 유지하고 `false` 를 돌려준다 — 상대 기록은 남지만 다시
    /// 추가(hello)하면 양쪽 승인이 지워져 처음부터 페어링한다.
    pub async fn remove(&self, id: i64) -> io::Result<bool> {
        let peer = self.peer(id)?;
        peers::remove(&self.db(), id).map_err(db_err)?;
        let notice = async {
            self.send(parse_addr(&peer.addr)?, Some(peer.spki), &Request::Remove)
                .await
        };
        match notice.await {
            Ok((Response::Ok, _)) => Ok(true),
            Ok((other, _)) => {
                log::info!("피어 {} 삭제 통지 거절: {other:?}", peer.addr);
                Ok(false)
            }
            Err(e) => {
                log::info!("피어 {} 삭제 통지 실패: {e}", peer.addr);
                Ok(false)
            }
        }
    }

    /// 화면이 여는 연결 확인 — 마지막 통신이 `within` 초 안이면 보내지 않고 저장된 집중을, 아니면 [`ping`](Self::ping).
    pub async fn refresh(&self, id: i64, within: i64) -> io::Result<bool> {
        let peer = self.peer(id)?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| i64::try_from(d.as_secs()).unwrap_or(i64::MAX));
        if peer
            .seen_at
            .is_some_and(|t| (0..within).contains(&(now - t)))
        {
            return Ok(peer.focus);
        }
        self.ping(id).await
    }

    /// 연결 확인 — 상대가 집중 중인가를 돌려주고, 받은 `Pong` 을 접속 상태로 남긴다(거절 · 전송 오류는 남기지 않는다).
    pub async fn ping(&self, id: i64) -> io::Result<bool> {
        self.pong(id).await.map(|(focus, ..)| focus)
    }

    /// 상대가 `Request::Context` 를 아는가 — [`ping`](Self::ping) 처럼 접속 상태를 남긴다. 닿지 않으면 오류(보내는 줄은 다시 보낸다).
    pub async fn knows_context(&self, id: i64) -> io::Result<bool> {
        self.pong(id).await.map(|(_, ctx, ..)| ctx)
    }

    /// 상대가 `Request::Share*` 를 아는가 — [`knows_context`](Self::knows_context) 와 같다.
    pub async fn knows_share(&self, id: i64) -> io::Result<bool> {
        self.pong(id).await.map(|(_, _, share, ..)| share)
    }

    /// 상대가 `Request::Leave` 를 아는가 — [`knows_context`](Self::knows_context) 와 같다.
    pub async fn knows_leave(&self, id: i64) -> io::Result<bool> {
        self.pong(id).await.map(|(.., leave, _)| leave)
    }

    /// 상대가 `Request::Clear` 를 아는가 — [`knows_context`](Self::knows_context) 와 같다.
    pub async fn knows_clear(&self, id: i64) -> io::Result<bool> {
        self.pong(id).await.map(|(.., clear)| clear)
    }

    /// (집중, 컨텍스트를 앎, 공유 대화를 앎, 나가기를 앎, 기록 지우기를 앎).
    async fn pong(&self, id: i64) -> io::Result<(bool, bool, bool, bool, bool)> {
        let peer = self.peer(id)?;
        match self
            .send(
                parse_addr(&peer.addr)?,
                Some(peer.spki),
                &Request::Ping { port: self.me.port },
            )
            .await?
        {
            (
                Response::Pong {
                    focus,
                    ctx,
                    share,
                    leave,
                    clear,
                },
                _,
            ) => {
                peers::set_presence(&self.db(), id, focus).map_err(db_err)?;
                Ok((focus, ctx, share, leave, clear))
            }
            (other, _) => Err(refused(other)),
        }
    }

    /// 시작 알림 — 페어링된 상대 모두에 [`ping`](Self::ping) 한 번(상대별 작업, 실패는 넘어간다). 내 수신 포트를 실어 상대가 바뀐 주소를
    /// 따라오게 하고 집중을 받는다. 주기 확인은 없다 — 접속 시각은 통신할 때만 갱신.
    pub async fn announce(self: Arc<Self>) {
        let Ok(peers) = self.peers() else {
            return;
        };
        for p in peers.into_iter().filter(Peer::paired) {
            let me = self.clone();
            tokio::spawn(async move {
                let _ = me.ping(p.id).await;
            });
        }
    }
}

fn parse_addr(s: &str) -> io::Result<SocketAddr> {
    s.parse()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, format!("주소 형식 오류: {s}")))
}

#[cfg(test)]
mod tests {
    use super::{CODE_ALPHABET, new_code, normalize_code};

    #[test]
    fn code_is_8_alphabet_chars_and_fresh() {
        let (a, b) = (new_code(), new_code());
        assert_eq!(a.len(), 9);
        let n = normalize_code(&a);
        assert_eq!(n.len(), 8);
        assert!(n.bytes().all(|c| CODE_ALPHABET.contains(&c)));
        assert_ne!(a, b);
        assert_eq!(normalize_code(" abcd efgh "), "ABCDEFGH");
    }
}
