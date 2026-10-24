//! 대화 왕복 조율 — 보내기(사람 → 원격 에이전트), 받기(대기열 → 에이전트 실행 → 답 되돌리기), 답 받기.
//! 원격에서 온 것은 신뢰하지 않는 입력: 페어링 확인은 `Node`, 에이전트 이름 · 본문 크기 · 대화 소유는 여기서 본다.

mod queue;
mod tx;
mod delivery;
mod send;
mod inbound;
mod request;
mod tidy;
mod relay;
mod execute;
mod context;
mod sharing;
mod lifecycle;

use crate::runner::{RunInput, RunOutput, SUMMARY_PROMPT, Session, StepSink};
use an_core::chat::{self, Author, Conversation, Dir, Message, ReqState, State};
use an_core::db::{Connection, db_err};
use an_core::peers::Peer;
use an_core::{agents, cli_tx, outbox, share};
use an_net::node::{MessageKind, Node, PeerEvent};
use an_net::proto::{MAX_FRAME, MAX_TEXT, Request, Response, ShareMsg, StepDetail};
use std::collections::HashMap;
use std::io;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex, Weak};
use std::time::Duration;
use tokio::sync::mpsc;

/// 에이전트 실행 — 운영은 `runner::run_agent`, 테스트는 대역. `spawn_blocking` 안에서 불린다.
pub type Run = Arc<dyn Fn(RunInput) -> io::Result<RunOutput> + Send + Sync>;

/// 자동 왕복 상한의 최댓값(왕복).
pub const MAX_RELAY_CAP: i64 = 50;

/// 자동 왕복 상한 기본값(왕복) — 설정(`RELAY_CAP_KEY`)이 없을 때.
pub const DEFAULT_RELAY_CAP: i64 = 10;

/// 설정 키 — 새 자동 왕복 대화의 상한 기본값(1 ~ `MAX_RELAY_CAP`).
pub const RELAY_CAP_KEY: &str = "relay_cap_default";

/// 자동 왕복 끝 알림 사유 — 상한.
const RELAY_END_CAP: &str = "상한 도달";

/// 새 자동 왕복 대화의 상한 기본값 — 설정이 없거나 범위 밖이면 `DEFAULT_RELAY_CAP`. DB 오류는 기본값으로 감추지 않는다.
pub fn default_relay_cap(conn: &Connection) -> Result<i64, an_core::db::Error> {
    Ok(an_core::settings::get(conn, RELAY_CAP_KEY)?
        .and_then(|v| v.parse().ok())
        .filter(|v| (1..=MAX_RELAY_CAP).contains(v))
        .unwrap_or(DEFAULT_RELAY_CAP))
}

/// PC 전체 에이전트 실행 동시 실행 상한 — 설정 키 · 기본값 · 범위.
pub const MAX_RUNS_KEY: &str = "max_runs";
pub const DEFAULT_MAX_RUNS: usize = 2;
pub const MAX_RUNS: usize = 8;

/// 동시 실행 상한 — 설정이 없거나 범위 밖이면 `DEFAULT_MAX_RUNS`. DB 오류는 기본값으로 감추지 않는다.
pub fn max_runs(conn: &Connection) -> Result<usize, an_core::db::Error> {
    Ok(an_core::settings::get(conn, MAX_RUNS_KEY)?
        .and_then(|v| v.parse().ok())
        .filter(|v| (1..=MAX_RUNS).contains(v))
        .unwrap_or(DEFAULT_MAX_RUNS))
}

/// 상대가 알려 준 내 실행 요청의 차례 — `queued`(앞 사람 수) | `running`(+ 단계 요약).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Progress {
    pub state: String,
    pub ahead: u32,
    pub detail: Option<StepDetail>,
}

/// outbox 행 종류 — ack 뒤 처리([`Chat::settle`]) · 포기 처리([`Chat::give_up`])를 가른다.
const MESSAGE: &str = "message";
const RELAY_OPEN: &str = "relay_open";
const RELAY_STEP: &str = "relay_step";
/// 실행 없는 사람 글의 미러(상대가 받으면 내 글 `done`).
const NOTE: &str = "note";
const REPLY: &str = "reply";
const REPLY_POST: &str = "reply_post";
const RELAY_LAST: &str = "relay_last";
/// 요청 상태 — 받은 쪽 사람이 바꾼 상태를 쓴 쪽에.
const REQ_STATE: &str = "req_state";
const RELAY: &str = "relay";
const PROGRESS: &str = "progress";
/// 컨텍스트 — 압축 · 비우기 요청(내 구분선 글) · 처리 결과(상대가 요청한 구분선 글). 보내기 전에 상대가 이 종류를 아는지 본다.
const CTX: &str = "context";
const CTX_DONE: &str = "context_done";
/// 공유 대화 — 관람자 앞 `Share*` · 원 상대 앞 관람자 수. 보내기 전에 상대가 이 종류를 아는지 본다.
const SHARE: &str = outbox::SHARE;
const SHARE_NOTICE: &str = outbox::SHARE_NOTICE;
/// 대화 나가기 — 상대 앞 `Leave`. 보내기 전에 상대가 이 종류를 아는지 본다 · 포기하지 않는다.
const LEAVE: &str = outbox::LEAVE;
/// 모든 사용자 글 삭제 — 상대 앞 `Clear`. 보내기 전에 상대가 아는지 본다 · 포기하지 않는다.
const CLEAR: &str = outbox::CLEAR;

/// 공유 사본에 쓰려 했다.
const READ_ONLY: &str = "공유 대화는 읽기 전용";
/// 나간 대화(내가 · 상대가 나감)에 쓰려 했다.
const CLOSED: &str = "나간 대화는 읽기 전용";
/// 모든 사용자 글 삭제 — 상대 앱이 `Clear` 를 모른다(이전 버전).
pub const CLEAR_UNKNOWN: &str = "상대 앱이 이 기능을 모릅니다";

/// 컨텍스트 요청 거절 코드 — 상대가 이 기능을 모른다(0.4.0 이하).
pub const CTX_UNSUPPORTED: &str = "unsupported";
/// 다시 보내기를 포기한 내 글의 거절 코드 — 화면 "보내지 못함".
pub const UNREACHABLE: &str = "unreachable";

/// 받은 쪽이 자동 왕복을 멈췄다 — 에이전트발 실행 요청 거절 코드.
const STOPPED: &str = "stopped";

/// 받는 쪽이 잠시 처리하지 못했다(거절 아님 — 보낸 쪽이 다시 보낸다).
pub const RETRY: &str = "retry";

/// 보내기 결과 — 상대 응답 또는 보관(전송 오류 · 앞선 행이 있어 줄에 섬 — 사유는 outbox 행에).
enum Shipped {
    Answered(Response),
    Kept,
}

/// CLI 트랜잭션 보기 — `state` = `sending`(이 PC 에 보관 — 아직 상대에 닿지 않음) · `sent` · `queued` · `running` · `answered` · `failed` ·
/// `stalled`(상대에 정해진 시간 닿지 않음 — `wait` 으로 이어 받는다).
#[derive(Debug, Clone)]
pub struct TxView {
    pub tx: cli_tx::Tx,
    pub state: &'static str,
    pub message: Message,
    pub reply: Option<Message>,
    pub progress: Option<Progress>,
    pub outbox: Option<outbox::Item>,
}

/// 상대 보내기 줄 — 깨우기 · 작업(끝났으면 다시 띄운다).
type Lane = (Arc<tokio::sync::Notify>, tokio::task::JoinHandle<()>);

/// CLI 트랜잭션 감시(초) — 상태를 묻는 간격 · 닿지 않음이 이만큼 이어지면 멈춤. 운영 60초 · 10분, 시험은 [`Chat::set_watch`].
#[derive(Clone, Copy)]
struct Watch {
    every: i64,
    stall_after: i64,
}

/// 다시 보내기 간격 · 포기 한도(초) — 운영 5초 · 5분 · 24시간, 시험은 [`Chat::set_retry`].
#[derive(Clone, Copy)]
struct Retry {
    base: i64,
    max: i64,
    give_up: i64,
}

/// 알림 미리 보기 길이(글자).
const PREVIEW_CHARS: usize = 80;

/// 앱이 실행 도중 끝나 결과가 없는 메시지에 돌려줄 답.
const INTERRUPTED: &str = "실행 도중 상대 앱이 종료돼 결과가 없다 — 다시 보내 주세요";
/// 실행 입력에 붙이는 메모 상한 — 개수 · 본문 바이트 합.
pub const MAX_NOTES: usize = 20;
pub const MAX_NOTES_BYTES: usize = 32 * 1024;
pub struct Chat {
    db: Mutex<Connection>,
    node: Arc<Node>,
    run: Run,
    /// 대화별 직렬 작업 깨우기(같은 세션 동시 재개 금지). 대기열 자체는 DB(`queued` 메시지, id 순).
    // ponytail: 대화마다 작업 하나를 계속 둔다 — 대화가 아주 많아지면 유휴 작업 정리가 필요하다.
    queues: Mutex<HashMap<i64, mpsc::UnboundedSender<()>>>,
    /// 대화별 보내기 잠금 — 저장 순서 = 상대 도착 순서(동시 보내기가 상대 세션에서 뒤바뀌지 않게).
    // ponytail: 대화마다 잠금 하나를 계속 둔다 — `queues` 와 같은 상한.
    outbound: Mutex<HashMap<i64, Arc<tokio::sync::Mutex<()>>>>,
    /// 실행 문 — PC 전체 동시 실행 상한.
    gate: Arc<crate::gate::Gate>,
    /// 상대가 알려 준 내 실행 요청의 차례(메시지 id → 진행). 진행 표시일 뿐 — 다시 켜면 사라진다.
    // ponytail: 답이 온 글의 항목은 읽을 때 지운다 — 읽지 않는 글이 많아지면 답 받을 때 지우기로.
    progress: Mutex<HashMap<i64, Progress>>,
    /// 상대별 보내기 줄 깨우기 — 상대마다 tokio 작업 하나가 outbox 를 id 순으로 보낸다. 응답 없는 상대가 다른 상대를 막지 않는다.
    // ponytail: 상대마다 작업 하나를 계속 둔다 — 상대 수(사람 수)만큼이라 정리하지 않는다.
    lanes: Mutex<HashMap<i64, Lane>>,
    retry: Mutex<Retry>,
    watch: Mutex<Watch>,
    /// 실행 중 단계 요약을 보낸 쪽에 알리는 간격(기본 10초, 그 사이 바뀐 것은 마지막 값 하나로 합친다).
    step_every: Mutex<std::time::Duration>,
    /// 키 있는 CLI 보내기 줄 세우기 — 같은 키를 동시에 두 번 보내도 확인 · 보내기 사이에 끼어들지 못하게.
    // ponytail: 키 있는 CLI 보내기 전체를 한 줄로 — 동시 CLI 가 많아지면 키별 잠금으로.
    keyed: tokio::sync::Mutex<()>,
    /// 지금 상대에 상태를 묻고 있는 트랜잭션(같은 것을 겹쳐 묻지 않게).
    watching: Mutex<std::collections::HashSet<i64>>,
    /// 줄 작업을 띄울 때 쓰는 자기 참조.
    me: Weak<Chat>,
}

/// UUID v4 문자열 — `claude --session-id` 는 유효한 UUID 만 받는다.
pub fn new_uuid() -> String {
    let mut b = [0u8; 16];
    getrandom::fill(&mut b).expect("OS 난수원을 쓸 수 없다");
    b[6] = (b[6] & 0x0f) | 0x40;
    b[8] = (b[8] & 0x3f) | 0x80;
    let h: String = b.iter().map(|x| format!("{x:02x}")).collect();
    format!(
        "{}-{}-{}-{}-{}",
        &h[..8],
        &h[8..12],
        &h[12..16],
        &h[16..20],
        &h[20..]
    )
}

fn preview(text: &str) -> String {
    text.chars().take(PREVIEW_CHARS).collect()
}

fn fit_to(mut text: String, max: usize) -> String {
    if text.len() <= max {
        return text;
    }
    let mut cut = max - "\n…(잘림)".len();
    while !text.is_char_boundary(cut) {
        cut -= 1;
    }
    text.truncate(cut);
    text.push_str("\n…(잘림)");
    text
}

/// 새 사람 글의 선택 — 출처(`via` — `cli` 면 사용자 AI 에이전트가 CLI 로 쓴 글) · 새 대화 제목(`title` — 이 PC 에만, 이어 쓰기면 쓰지 않는다) ·
/// 종류(`request` — 참 = 요청(건의), 거짓 = 질의(기본), 양쪽 이력에 남는다) · 멘션(`mention` — 보낸 대화 이어 쓰기에서 부르는 상대 PC
/// 에이전트, 대화 에이전트 이름이면 멘션이 아니다).
#[derive(Debug, Clone, Copy, Default)]
pub struct Compose<'a> {
    pub via: Option<&'a str>,
    /// CLI 트랜잭션 키(`ask --id`) — `via` 가 있으면 글과 같은 DB 트랜잭션에서 `cli_tx` 를 만든다(키가 겹치면 글도 남지 않는다).
    pub tx_key: Option<&'a str>,
    pub title: Option<&'a str>,
    pub request: bool,
    pub mention: Option<&'a str>,
    /// 새 대화를 넣을 레일 그룹(이 PC 에만, 대화와 같은 트랜잭션). 이어 쓰기면 쓰지 않는다.
    pub group: Option<i64>,
}

/// 새 대화를 넣을 레일 그룹이 없다 — 404 로 간다.
fn no_group() -> io::Error {
    io::Error::new(io::ErrorKind::NotFound, "없는 그룹")
}

const REQUEST_NO_RELAY: &str = "요청은 자동 왕복을 열지 않는다 — 실행이 없다";

/// 집중 거절 코드 — 사용자가 켠 집중 중에는 새 실행 요청을 거부.
const FOCUS: &str = "focus";

fn refused(resp: Response) -> io::Error {
    match resp {
        Response::Error { code } if code == FOCUS => io::Error::new(
            io::ErrorKind::PermissionDenied,
            "상대가 집중 중 — 요청을 받지 않습니다. 나중에 다시 보내세요",
        ),
        Response::Error { code } if code == chat::LEFT => io::Error::new(
            io::ErrorKind::PermissionDenied,
            "상대가 대화를 나갔습니다",
        ),
        Response::Error { code } => io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!("상대가 거절: {}", an_net::proto::clean_code(&code)),
        ),
        // 내용은 싣지 않는다 — 상대가 고른 글자(최대 64 KiB)가 화면 · CLI 오류로 간다.
        _ => io::Error::new(io::ErrorKind::InvalidData, "예상하지 못한 응답"),
    }
}

fn addr(peer: &Peer) -> io::Result<SocketAddr> {
    peer.addr.parse().map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("주소 형식 오류: {}", peer.addr),
        )
    })
}

impl Chat {
    /// `conn` 은 피어 노드와 같은 DB 파일의 다른 연결.
    pub fn new(conn: Connection, node: Arc<Node>, run: Run) -> Arc<Self> {
        // 못 읽으면 기본값 — 시작을 막지 않는다(설정 화면이 같은 오류를 보인다).
        let cap = max_runs(&conn).unwrap_or_else(|e| {
            log::warn!("동시 실행 상한 읽기 실패 — 기본 {DEFAULT_MAX_RUNS}: {e}");
            DEFAULT_MAX_RUNS
        });
        Arc::new_cyclic(|me| Self {
            db: Mutex::new(conn),
            node,
            run,
            queues: Mutex::new(HashMap::new()),
            outbound: Mutex::new(HashMap::new()),
            gate: crate::gate::Gate::new(cap),
            progress: Mutex::new(HashMap::new()),
            lanes: Mutex::new(HashMap::new()),
            retry: Mutex::new(Retry {
                base: 5,
                max: 300,
                give_up: 24 * 3600,
            }),
            watch: Mutex::new(Watch {
                every: 60,
                stall_after: 600,
            }),
            step_every: Mutex::new(std::time::Duration::from_secs(10)),
            keyed: tokio::sync::Mutex::new(()),
            watching: Mutex::new(std::collections::HashSet::new()),
            me: me.clone(),
        })
    }

    /// 노드의 대화 요청 처리기로 붙는다.
    pub fn attach(self: &Arc<Self>) {
        let me = self.clone();
        self.node.on_request(move |peer, req| me.handle(peer, req));
        let me = self.clone();
        self.node.on_seen(move |peer_id| me.peer_back(peer_id));
    }

    /// 시작 시 — 지난 실행에서 남은 `queued` 는 실행하고, `running`(실행 도중 종료)은 오류 답으로 끝낸다. tokio 런타임 안에서 부른다.
    pub fn recover(self: &Arc<Self>) -> io::Result<usize> {
        let ids = chat::pending_conversations(&self.db()).map_err(db_err)?;
        for &id in &ids {
            self.wake(id);
        }
        // 지난 실행에서 보내지 못한 글 — 상대 줄을 다시 띄운다. 7일 지난 실패 보관은 지운다.
        if let Err(e) = outbox::prune_failed(&self.db(), 7) {
            log::warn!("오래된 실패 보관 지우기 실패: {e}");
        }
        let peers = outbox::pending_peers(&self.db()).map_err(db_err)?;
        for p in peers {
            // 다시 켰다 — 기다리던 간격과 무관하게 지금 한 번.
            outbox::hurry(&self.db(), p).map_err(db_err)?;
            self.kick(p);
        }
        Ok(ids.len())
    }

    /// 대화 DB — await 를 넘겨 쥐지 않는다.
    pub fn db(&self) -> std::sync::MutexGuard<'_, Connection> {
        an_core::sync::lock(&self.db)
    }

    /// 알림 이벤트 — 상대 이름(이 PC 의 표시 이름이 있으면 그것) · PC 와 종류(거르기는 알림 쪽이 한다). 자동 왕복 시작 · 끝은
    /// 알림 설정과 무관하게 여기서 내 활동 기록으로 남긴다(사유 · 미리보기 = `preview`). 부르는 쪽은 DB 잠금을 쥐고 있지 않다.
    fn notify(
        &self,
        conversation_id: i64,
        peer: &Peer,
        agent: String,
        kind: MessageKind,
        preview: String,
    ) {
        let mark = match kind {
            MessageKind::RelayStart => Some(chat::RelayMark::Start),
            MessageKind::RelayEnd => Some(chat::RelayMark::End),
            MessageKind::Request
            | MessageKind::Reply
            | MessageKind::RelayStep
            | MessageKind::Ask
            | MessageKind::AskState
            | MessageKind::Share
            | MessageKind::Clear => None,
        };
        if let Some(mark) = mark
            && let Err(e) = chat::add_relay_event(&self.db(), conversation_id, mark, &preview)
        {
            log::warn!("대화 {conversation_id} 자동 왕복 기록 실패: {e}");
        }
        self.node.emit(PeerEvent::Message {
            name: peer.alias.clone().unwrap_or_else(|| peer.name.clone()),
            host: peer.host.clone(),
            agent,
            preview,
            kind,
            conv: conversation_id,
        });
    }

    /// 대화 단위 차례 — 저장부터 상대 응답까지 한 번에 하나(사람 글 · 완료 표시).
    fn turn(&self, conversation_id: i64) -> Arc<tokio::sync::Mutex<()>> {
        an_core::sync::lock(&self.outbound)
            .entry(conversation_id)
            .or_default()
            .clone()
    }

    /// 공유 사본 · 나간 대화는 읽기 전용 — 글쓰기 · 컨텍스트 · 자동 왕복 · 요청 상태 · CLI 보내기 · 이력 정리 · 공유를 막는다.
    fn writable(conv: &Conversation) -> io::Result<()> {
        if conv.dir == Dir::Shared {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, READ_ONLY));
        }
        if conv.closed() {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, CLOSED));
        }
        Ok(())
    }

    fn conversation(&self, id: i64) -> io::Result<Conversation> {
        chat::by_id(&self.db(), id)
            .map_err(db_err)?
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, format!("대화 {id} 없음")))
    }
}
