//! 대화 왕복 조율(S1 · D17 · D19) — 보내기(사람 → 원격 에이전트), 받기(대기열 → `claude -p` → 답 되돌리기), 답 받기.
//! 원격에서 온 것은 신뢰하지 않는 입력: 페어링 확인은 `Node`, 에이전트 이름 · 본문 크기 · 대화 소유는 여기서 본다.
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

/// 에이전트 실행 — 운영은 `runner::run_claude`, 테스트는 대역. `spawn_blocking` 안에서 불린다.
pub type Run = Arc<dyn Fn(RunInput) -> io::Result<RunOutput> + Send + Sync>;

/// 자동 왕복 상한의 최댓값(왕복, D24).
pub const MAX_RELAY_CAP: i64 = 50;

/// 자동 왕복 상한 기본값(왕복, D24) — 설정(`RELAY_CAP_KEY`)이 없을 때.
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

/// PC 전체 `claude -p` 동시 실행 상한(1b, D24 개정) — 설정 키 · 기본값 · 범위.
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

/// 상대가 알려 준 내 실행 요청의 차례(1b) — `queued`(앞 사람 수) | `running`(+ 단계 요약, #1c).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Progress {
    pub state: String,
    pub ahead: u32,
    pub detail: Option<StepDetail>,
}

/// outbox 행 종류(#1c) — ack 뒤 처리([`Chat::settle`]) · 포기 처리([`Chat::give_up`])를 가른다.
const MESSAGE: &str = "message";
const RELAY_OPEN: &str = "relay_open";
const RELAY_STEP: &str = "relay_step";
/// 실행 없는 사람 글의 미러(상대가 받으면 내 글 `done`).
const NOTE: &str = "note";
/// 받은 쪽 실행 요청 사람 글의 미러(내 실행과 무관).
const MIRROR: &str = "mirror";
const REPLY: &str = "reply";
const REPLY_POST: &str = "reply_post";
const RELAY_LAST: &str = "relay_last";
/// 요청 상태(#1d) — 받은 쪽 사람이 바꾼 상태를 쓴 쪽에.
const REQ_STATE: &str = "req_state";
const RELAY: &str = "relay";
const PROGRESS: &str = "progress";
/// 컨텍스트(#1m) — 압축 · 비우기 요청(내 구분선 글) · 처리 결과(상대가 요청한 구분선 글). 보내기 전에 상대가 이 종류를 아는지 본다.
const CTX: &str = "context";
const CTX_DONE: &str = "context_done";
/// 공유 대화(#1o) — 관람자 앞 `Share*` · 원 상대 앞 관람자 수. 보내기 전에 상대가 이 종류를 아는지 본다 · 포기하지 않는다.
const SHARE: &str = outbox::SHARE;
const SHARE_NOTICE: &str = outbox::SHARE_NOTICE;

/// 공유 사본(#1o)에 쓰려 했다.
const READ_ONLY: &str = "공유 대화는 읽기 전용";

/// 컨텍스트 요청 거절 코드(#1m) — 상대가 이 기능을 모른다(0.4.0 이하) · 요약 실행 실패.
pub const CTX_UNSUPPORTED: &str = "unsupported";
const CTX_FAILED: &str = "compact_failed";
/// 이 PC 도 처리한 구분선 글에 남기는 상대 쪽 실패 코드(#1m — 부분 처리).
fn peer_code(code: &str) -> String {
    format!("peer_{code}")
}

/// 압축 요약 상한(바이트, #1m) — 통신 본문 한도 안.
const MAX_SUMMARY: usize = 8 * 1024;
/// 받은 창 사용량의 상한(원격 값 검증).
const MAX_CTX_TOKENS: i64 = 100_000_000;

/// 다시 보내기를 포기한 내 글의 거절 코드 — 화면 "보내지 못함".
pub const UNREACHABLE: &str = "unreachable";

/// 받은 쪽이 자동 왕복을 멈췄다 — 에이전트발 실행 요청 거절 코드.
const STOPPED: &str = "stopped";

/// 받는 쪽이 잠시 처리하지 못했다(거절 아님 — 보낸 쪽이 다시 보낸다, #1c).
pub const RETRY: &str = "retry";

/// 보내기 결과 — 상대 응답 또는 보관(전송 오류 · 앞선 행이 있어 줄에 섬 — 사유는 outbox 행에).
enum Shipped {
    Answered(Response),
    Kept,
}

/// CLI 트랜잭션 보기(#1c) — `state` = `sending`(이 PC 에 보관 — 아직 상대에 닿지 않음) · `sent` · `queued` · `running` · `answered` · `failed` ·
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

/// CLI 트랜잭션 감시(초) — 상태를 묻는 간격 · 닿지 않음이 이만큼 이어지면 멈춤. 운영 60초 · 10분(#1c 결정 6), 시험은 [`Chat::set_watch`].
#[derive(Clone, Copy)]
struct Watch {
    every: i64,
    stall_after: i64,
}

/// 다시 보내기 간격 · 포기 한도(초) — 운영 5초 · 5분 · 24시간(#1c 결정 7), 시험은 [`Chat::set_retry`].
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
/// 실행 입력에 붙이는 메모 상한 — 개수 · 본문 바이트 합(1a A8).
pub const MAX_NOTES: usize = 20;
pub const MAX_NOTES_BYTES: usize = 32 * 1024;
/// 실행 입력의 구분자 — 원격 글 안에서는 지운다.
const FRAMES: [&str; 6] = [
    "</원격 본문>",
    "<원격 본문>",
    "</대화 메모>",
    "<대화 메모>",
    "</대화 요약>",
    "<대화 요약>",
];

/// 원격 글에서 구분자 글자를 지운다(지운 자리에 새 구분자가 생기지 않을 때까지).
fn unframe(text: &str) -> String {
    let mut t = text.to_string();
    while let Some(f) = FRAMES.iter().find(|f| t.contains(**f)) {
        t = t.replace(f, "");
    }
    t
}

pub struct Chat {
    db: Mutex<Connection>,
    node: Arc<Node>,
    run: Run,
    /// 대화별 직렬 작업 깨우기(D19 — 같은 세션 동시 재개 금지). 대기열 자체는 DB(`queued` 메시지, id 순).
    // ponytail: 대화마다 작업 하나를 계속 둔다 — 대화가 아주 많아지면 유휴 작업 정리가 필요하다.
    queues: Mutex<HashMap<i64, mpsc::UnboundedSender<()>>>,
    /// 대화별 보내기 잠금 — 저장 순서 = 상대 도착 순서(동시 보내기가 상대 세션에서 뒤바뀌지 않게).
    // ponytail: 대화마다 잠금 하나를 계속 둔다 — `queues` 와 같은 상한.
    outbound: Mutex<HashMap<i64, Arc<tokio::sync::Mutex<()>>>>,
    /// 실행 문 — PC 전체 동시 실행 상한(1b).
    gate: Arc<crate::gate::Gate>,
    /// 상대가 알려 준 내 실행 요청의 차례(메시지 id → 진행). 진행 표시일 뿐 — 다시 켜면 사라진다.
    // ponytail: 답이 온 글의 항목은 읽을 때 지운다 — 읽지 않는 글이 많아지면 답 받을 때 지우기로.
    progress: Mutex<HashMap<i64, Progress>>,
    /// 상대별 보내기 줄 깨우기(#1c) — 상대마다 tokio 작업 하나가 outbox 를 id 순으로 보낸다. 응답 없는 상대가 다른 상대를 막지 않는다.
    // ponytail: 상대마다 작업 하나를 계속 둔다 — 상대 수(사람 수)만큼이라 정리하지 않는다.
    lanes: Mutex<HashMap<i64, Lane>>,
    retry: Mutex<Retry>,
    watch: Mutex<Watch>,
    /// 실행 중 단계 요약을 보낸 쪽에 알리는 간격(#1c — 기본 10초, 그 사이 바뀐 것은 마지막 값 하나로 합친다).
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

/// `MAX_TEXT` 바이트 안으로(글자 경계에서) 자른다.
fn fit(text: String) -> String {
    fit_to(text, MAX_TEXT)
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

/// 받은 답의 model · effort — `run_opts` 글자 규칙에 맞는 것만 남긴다(원격 값은 신뢰하지 않는 입력).
fn run_meta(model: Option<String>, effort: Option<String>) -> (Option<String>, Option<String>) {
    (
        model.filter(|m| an_core::run_opts::valid_model(m)),
        effort.filter(|e| an_core::run_opts::valid_effort(e)),
    )
}

/// 새 사람 글의 선택 — 출처(`via` — `cli` 면 사용자 AI 에이전트가 CLI 로 쓴 글) · 새 대화 제목(`title` — 이 PC 에만, 이어 쓰기면 쓰지 않는다) ·
/// 종류(`request` — 참 = 요청(건의), 거짓 = 질의(기본), 양쪽 이력에 남는다) · 멘션(`mention` — 보낸 대화 이어 쓰기에서 부르는 상대 PC
/// 에이전트, 대화 에이전트 이름이면 멘션이 아니다).
#[derive(Debug, Clone, Copy, Default)]
pub struct Compose<'a> {
    pub via: Option<&'a str>,
    /// CLI 트랜잭션 키(`ask --id`, #1c) — `via` 가 있으면 글과 같은 DB 트랜잭션에서 `cli_tx` 를 만든다(키가 겹치면 글도 남지 않는다).
    pub tx_key: Option<&'a str>,
    pub title: Option<&'a str>,
    pub request: bool,
    pub mention: Option<&'a str>,
    /// 새 대화를 넣을 레일 그룹(#1i — 이 PC 에만, 대화와 같은 트랜잭션). 이어 쓰기면 쓰지 않는다.
    pub group: Option<i64>,
}

/// 받은 `message` 의 나머지 — 순서(`lc`) · 자동 왕복(`by_agent` · `relay`) · 출처(`via`) · 종류(`request`) · 멘션(`mention`).
type MessageExtra = (
    Option<i64>,
    Option<String>,
    Option<String>,
    Option<String>,
    bool,
    Option<String>,
);

/// 새 대화를 넣을 레일 그룹이 없다(#1i) — 404 로 간다.
fn no_group() -> io::Error {
    io::Error::new(io::ErrorKind::NotFound, "없는 그룹")
}

/// 멘션을 받지 않는 자리(새 대화 · 받은 대화 · 자동 왕복 대화 · 빈 이름)의 오류 문구.
const MENTION_ONLY: &str =
    "멘션은 내가 연 대화(자동 왕복 아님)에 이어 쓸 때만 — 상대 PC 에이전트 이름";
/// 요청(#1d)은 실행이 없다 — 부를 에이전트(멘션)도, 답을 기다리는 CLI 도 없다.
const REQUEST_NO_MENTION: &str = "요청은 멘션 없이 — 실행 없이 상대 요청 목록에 남는다";
const REQUEST_NO_CLI: &str = "CLI 는 질의만 — 요청은 답이 없다";
const REQUEST_NO_RELAY: &str = "요청은 자동 왕복을 열지 않는다 — 실행이 없다";

/// 요청 규칙(#1d) — 멘션(대화 에이전트가 아닌 이름) · CLI 출처와 함께 쓰지 않는다.
fn request_ok(opts: &Compose<'_>, conv_agent: &str) -> io::Result<()> {
    if !opts.request {
        return Ok(());
    }
    if opts.via.is_some() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, REQUEST_NO_CLI));
    }
    if opts.mention.is_some_and(|m| m != conv_agent) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            REQUEST_NO_MENTION,
        ));
    }
    Ok(())
}

/// 압축 요약 하나(#1m) — `key` = `ctx_seeds.agent`(빈 글자 = 대화 세션 주인), `agent` · `uid` = 요약을 만든 에이전트와 그 세대.
struct Seed {
    key: String,
    agent: String,
    uid: String,
    text: String,
}

/// 구분선에 보일 요약 — 세션 하나면 그 글, 여럿이면 에이전트별 머리.
fn summary_text(conv: &Conversation, seeds: &[Seed]) -> String {
    let text = match seeds {
        [one] => one.text.clone(),
        many => many
            .iter()
            .map(|s| {
                format!(
                    "## {}\n{}",
                    if s.key.is_empty() {
                        &conv.agent
                    } else {
                        &s.key
                    },
                    s.text
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n"),
    };
    fit_to(text, MAX_SUMMARY)
}

/// 끝난 실행 — 결과 · 실행한 에이전트 · 정한 실행 옵션.
struct Ran {
    out: RunOutput,
    agent: agents::Agent,
    model: String,
    effort: String,
}

/// 실행 실패 — 내 쪽 행 · 로그에 남길 상세와 상대에게 보내는 정해진 문구(stderr · 경로 · DB 오류를 싣지 않는다, 1a A4).
struct Failure {
    detail: String,
    public: &'static str,
}

impl Failure {
    /// 실행기 오류 — 종류로 문구를 고른다.
    fn run(e: &std::io::Error) -> Self {
        let public = match e.kind() {
            std::io::ErrorKind::TimedOut => "실행 실패: 받는 PC 의 시간 제한(30분)을 넘겼다",
            std::io::ErrorKind::Unsupported => "실행 실패: 받는 PC 의 Claude Code 버전이 낮다",
            std::io::ErrorKind::NotFound => "실행 실패: 받는 PC 에서 Claude Code 를 찾지 못했다",
            _ => RUN_FAILED,
        };
        Failure {
            detail: e.to_string(),
            public,
        }
    }

    /// 받는 PC 의 기록 · 조회 오류.
    fn record(e: impl std::fmt::Display) -> Self {
        Failure {
            detail: format!("받는 PC 기록 오류 — {e}"),
            public: RECORD_FAILED,
        }
    }
}

const RUN_FAILED: &str = "실행 실패: 받는 PC 에서 실행하지 못했다 — 원인은 받는 PC 화면에 있다";
const RECORD_FAILED: &str = "실행 실패: 받는 PC 기록 오류";
const RESULT_ERROR: &str =
    "실행 실패: 받는 PC 의 에이전트가 오류로 끝났다 — 원인은 받는 PC 화면에 있다";
const EMPTY_ANSWER: &str = "실행 실패: 받는 PC 의 에이전트가 빈 답을 냈다";
const FOLDER_BLOCKED: &str =
    "실행 실패: 받는 PC 의 에이전트 폴더를 쓸 수 없다 — 받는 PC 에서 폴더를 바꿔야 한다";

/// 집중 거절 코드(D17 개정 — 사용자가 켠 집중 중에는 새 실행 요청을 거부).
const FOCUS: &str = "focus";

fn refused(resp: Response) -> io::Error {
    match resp {
        Response::Error { code } if code == FOCUS => io::Error::new(
            io::ErrorKind::PermissionDenied,
            "상대가 집중 중 — 요청을 받지 않습니다. 나중에 다시 보내세요",
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

    /// 실행 문 — (상한, 실행 중 메시지 id, 기다리는 메시지 id). 기다림 = 문의 줄(닿은 순서) 뒤에, 같은 대화의 앞 실행을 기다려 아직 문에
    /// 닿지 않은 실행 요청(DB `queued`, id 순).
    pub fn run_queue(&self) -> (usize, Vec<i64>, Vec<i64>) {
        let (cap, running, mut waiting) = self.gate.snapshot();
        let db = self.db();
        let blocked = chat::pending_conversations(&db).and_then(|cs| {
            let mut ids = Vec::new();
            for c in cs {
                ids.extend(
                    chat::pending(&db, c)?
                        .into_iter()
                        .filter(|m| m.state == State::Queued)
                        .map(|m| m.id),
                );
            }
            Ok(ids)
        });
        match blocked {
            Ok(mut ids) => {
                ids.retain(|id| !running.contains(id) && !waiting.contains(id));
                ids.sort_unstable();
                waiting.extend(ids);
            }
            Err(e) => log::warn!("실행 목록 — 대화 대기열 읽기 실패: {e}"),
        }
        (cap, running, waiting)
    }

    /// 같은 대화의 앞 실행 때문에 기다리는 상대 실행 요청 — 닿자마자 `queued` 를 알린다(대화 작업이 문에 데려가기 전에는 문이 모른다).
    /// 앞 사람 수 = 같은 대화의 앞 요청 + 지금 문 줄.
    fn tell_waiting(&self, conversation_id: i64, uuid: &str) {
        let found = {
            let db = self.db();
            chat::by_id(&db, conversation_id)
                .and_then(|c| Ok((c, chat::pending(&db, conversation_id)?)))
        };
        let (Some(conv), pending) = (match found {
            Ok(x) => x,
            Err(e) => return log::info!("대화 {conversation_id} 대기 알림 — 읽기 실패: {e}"),
        }) else {
            return;
        };
        let Some(i) = pending.iter().position(|m| m.uuid == uuid) else {
            return;
        };
        if i == 0 || conv.dir != Dir::In || pending[i].mine {
            return;
        }
        let ahead = i + self.gate.snapshot().2.len();
        self.send_progress(&conv, &pending[i], "queued", ahead);
    }

    /// 동시 실행 상한을 바꾸고 이 PC 에 저장한다(1 ~ `MAX_RUNS`).
    pub fn set_max_runs(&self, n: usize) -> io::Result<()> {
        if !(1..=MAX_RUNS).contains(&n) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("동시 실행 상한은 1 ~ {MAX_RUNS}"),
            ));
        }
        an_core::settings::set(&self.db(), MAX_RUNS_KEY, &n.to_string()).map_err(db_err)?;
        self.gate.set_cap(n);
        Ok(())
    }

    /// 상대가 알려 준 차례 — 아직 답을 기다리는(`sent`) 글만. 답이 온 글의 항목은 여기서 지운다.
    pub fn progress_of(&self, m: &Message) -> Option<Progress> {
        let mut p = an_core::sync::lock(&self.progress);
        if m.state != State::Sent {
            p.remove(&m.id);
            return None;
        }
        p.get(&m.id).cloned()
    }

    /// 실행 문을 지난다(PC 전체 동시 실행 상한, 1b) — 상대가 보낸 실행 요청이면 차례를 알린다(기다리면 `queued` → `running`, 바로면 `running`).
    async fn admit(&self, conversation_id: i64, asked: &Message) -> crate::gate::Ticket {
        let conv = chat::by_id(&self.db(), conversation_id).ok().flatten();
        let remote = conv.filter(|c| c.dir == Dir::In && !asked.mine);
        let (mut ticket, ahead) = self.gate.enter(asked.id);
        if let Some(k) = ahead {
            if let Some(c) = &remote {
                self.send_progress(c, asked, "queued", k);
            }
            ticket.ready().await;
        }
        if let Some(c) = &remote {
            self.send_progress(c, asked, "running", 0);
        }
        ticket
    }

    /// 상대 실행 요청의 차례를 보낸 쪽에 알린다 — 그 상대 보내기 줄에(같은 글의 기다리는 진행은 마지막 값으로 바꾼다, #1c). 답과 같은 줄이라
    /// 순서가 지켜지고, 닿지 않으면 다시 보낸다.
    fn send_progress(&self, conv: &Conversation, asked: &Message, state: &str, ahead: usize) {
        self.send_step(conv, asked, state, ahead, None);
    }

    fn send_step(
        &self,
        conv: &Conversation,
        asked: &Message,
        state: &str,
        ahead: usize,
        detail: Option<StepDetail>,
    ) {
        let req = Request::Progress {
            conv: conv.uuid.clone(),
            msg: asked.uuid.clone(),
            state: state.into(),
            ahead: u32::try_from(ahead).unwrap_or(u32::MAX),
            detail,
        };
        let key = format!("progress:{}", asked.uuid);
        self.enqueue(conv, None, PROGRESS, &req, Some(&key));
    }

    /// CLI 트랜잭션 감시 설정 — 시험용(상태를 묻는 간격 · 멈춤까지, 초).
    pub fn set_watch(&self, every: i64, stall_after: i64) {
        *an_core::sync::lock(&self.watch) = Watch { every, stall_after };
    }

    pub fn tx_by_key(&self, key: &str) -> io::Result<Option<cli_tx::Tx>> {
        cli_tx::by_key(&self.db(), key).map_err(db_err)
    }

    /// 글의 CLI 트랜잭션.
    pub fn tx_of_message(&self, message_id: i64) -> io::Result<Option<cli_tx::Tx>> {
        cli_tx::by_message(&self.db(), message_id).map_err(db_err)
    }

    /// 키 있는 CLI 보내기 잠금 — 같은 키 확인부터 보내기까지 잡는다.
    pub async fn keyed(&self) -> tokio::sync::MutexGuard<'_, ()> {
        self.keyed.lock().await
    }

    /// 이어 받기(`wait`) — 멈춤을 풀고 다시 감시한다.
    pub fn tx_resume(&self, id: i64) -> io::Result<()> {
        cli_tx::resume(&self.db(), id).map_err(db_err)
    }

    /// 트랜잭션의 지금 상태 — 끝나지 않았고 물을 때면 상대에 상태를 묻는 작업을 띄운다(돌아오기를 기다리지 않음 — 다음 조회가 결과를 본다).
    pub fn tx_view(&self, id: i64) -> io::Result<Option<TxView>> {
        let found = {
            let db = self.db();
            let Some(tx) = cli_tx::by_id(&db, id).map_err(db_err)? else {
                return Ok(None);
            };
            let Some(message) = chat::message_by_id(&db, tx.message_id).map_err(db_err)? else {
                return Ok(None);
            };
            let reply = chat::messages(&db, message.conversation_id)
                .map_err(db_err)?
                .into_iter()
                .find(|r| r.author == Author::Agent && r.reply_to == Some(message.id));
            (tx, message, reply)
        };
        let (tx, message, reply) = found;
        let outbox = self.outbox_of(&message);
        let progress = self.progress_of(&message);
        let state = match (&reply, &outbox) {
            (Some(r), _) if r.state == State::Done => "answered",
            (Some(r), _) if r.state == State::Error => "failed",
            _ if message.state == State::Error => "failed",
            _ if tx.stalled_at.is_some() => "stalled",
            (_, Some(o)) if o.state == "failed" => "failed",
            (_, Some(_)) => "sending",
            _ => match progress.as_ref().map(|p| p.state.as_str()) {
                Some("queued") => "queued",
                Some("running") => "running",
                _ => "sent",
            },
        };
        if matches!(state, "sending" | "sent" | "queued" | "running") {
            self.watch_tx(&tx, &message, state == "sending");
        }
        Ok(Some(TxView {
            tx,
            state,
            message,
            reply,
            progress,
            outbox,
        }))
    }

    /// 상대에 닿는지 본다 — 보관 중이면 그 상대 줄이 닿지 않고 있는지, 아니면 상태 질의. 닿으면 닿지 않음을 지우고, 아니면 시작 시각부터
    /// 멈춤까지 센다.
    fn watch_tx(&self, tx: &cli_tx::Tx, message: &Message, sending: bool) {
        let w = *an_core::sync::lock(&self.watch);
        let due = cli_tx::due(&self.db(), tx.id, w.every);
        if !matches!(due, Ok(true)) || !an_core::sync::lock(&self.watching).insert(tx.id) {
            return;
        }
        let Some(me) = self.me.upgrade() else {
            return;
        };
        let _ = cli_tx::asked(&self.db(), tx.id);
        let (id, message) = (tx.id, message.clone());
        tokio::spawn(async move {
            let reached = if sending {
                let peer = chat::by_id(&me.db(), message.conversation_id)
                    .ok()
                    .flatten()
                    .map(|c| c.peer_id);
                peer.is_some_and(|p| matches!(outbox::failing(&me.db(), p), Ok(false)))
            } else {
                me.ask_status(&message).await.is_ok()
            };
            let marked = if reached {
                cli_tx::reached(&me.db(), id).map(|_| false)
            } else {
                cli_tx::unreachable(&me.db(), id, w.stall_after)
            };
            match marked {
                Ok(true) => log::info!(
                    "CLI 트랜잭션 {id} — 상대에 {}초 닿지 않아 멈춤",
                    w.stall_after
                ),
                Ok(false) => {}
                Err(e) => log::warn!("CLI 트랜잭션 {id} 감시 기록 실패: {e}"),
            }
            an_core::sync::lock(&me.watching).remove(&id);
        });
    }

    /// 단계 요약 간격 — 시험용.
    pub fn set_step_every(&self, every: std::time::Duration) {
        *an_core::sync::lock(&self.step_every) = every;
    }

    /// 보내기 설정 — 시험용(재시도 첫 간격 · 최대 간격 · 포기 한도, 초).
    pub fn set_retry(&self, base: i64, max: i64, give_up: i64) {
        *an_core::sync::lock(&self.retry) = Retry { base, max, give_up };
    }

    /// 보내기 행을 적는다(#1c, `inflight` — 곧 [`Chat::dispatch`] 가 보낸다. 그 전에 꺼지면 다시 켤 때 줄이 보낸다). `conn` 은 글을 적는 트랜잭션일
    /// 수 있다 — 글과 행이 함께 남거나 함께 없다.
    fn stage(
        conn: &Connection,
        conv: &Conversation,
        msg_id: Option<i64>,
        kind: &'static str,
        req: &Request,
    ) -> io::Result<i64> {
        let frame = serde_json::to_string(req).map_err(io::Error::other)?;
        outbox::add(
            conn,
            conv.peer_id,
            Some(conv.id),
            msg_id,
            kind,
            &frame,
            None,
            "inflight",
        )
        .map_err(db_err)
    }

    /// 상대에게 대화 프레임을 보낸다(#1c) — outbox 에 먼저 적고, 그 상대 줄에 앞선 행이 없으면 바로 한 번. 응답이면 행을 지우고 응답을, 전송
    /// 오류 · 앞선 행이면 줄에 남기고 `Kept`(상대 줄이 ack 까지 다시 보내고 [`settle`](Self::settle) 이 뒷일을 한다).
    async fn ship(
        &self,
        conv: &Conversation,
        msg_id: Option<i64>,
        kind: &'static str,
        req: &Request,
    ) -> io::Result<Shipped> {
        let id = Self::stage(&self.db(), conv, msg_id, kind, req)?;
        self.dispatch(conv, id, req).await
    }

    /// [`stage`](Self::stage) 로 이미 적은 행을 보낸다 — 글과 행을 한 DB 트랜잭션으로 적은 곳(사람 요청 · 답)이 커밋 뒤 부른다.
    async fn dispatch(&self, conv: &Conversation, id: i64, req: &Request) -> io::Result<Shipped> {
        // 방금 남긴 글을 관람자에게(#1o — 글을 남기는 길목).
        self.fan_share(conv.id);
        let peer_id = conv.peer_id;
        let behind = {
            let db = self.db();
            // 같은 대화 순서만 지킨다. 진행 · 멈춤 알림은 앞질러도 된다 — 답보다 늦게 닿은 진행은 받는 쪽이 버린다(`unknown`).
            // 공유 프레임(#1o)은 다른 상대(관람자)의 줄이다 — 꺼진 관람자의 행이 원 상대로 가는 보내기를 밀어내지 않게.
            // 닿지 않고 있는 상대면 바로 보내지 않는다 — 화면이 연결 시간 초과(최대 10초)를 기다리지 않게, 상대가 돌아오면(Ping · 요청) 줄이 보낸다.
            outbox::has_before(&db, conv.id, id, &[PROGRESS, RELAY, SHARE, SHARE_NOTICE])
                .map_err(db_err)?
                || outbox::failing(&db, peer_id).map_err(db_err)?
        };
        let me = self.me.upgrade().filter(|_| !behind);
        let Some(me) = me else {
            outbox::set_state(&self.db(), id, "pending").map_err(db_err)?;
            self.kick(peer_id);
            return Ok(Shipped::Kept);
        };
        // 보내기 · 뒷정리는 따로 띄운 작업에서 — 부른 쪽(HTTP 처리기 등)이 기다리다 취소돼도 행이 `inflight` 로 남지 않는다.
        let req = req.clone();
        let sent = tokio::spawn(async move {
            let got = me.send_to(peer_id, &req).await;
            match &got {
                Ok(resp) => {
                    me.acked(id, resp);
                }
                Err(e) => me.backoff(id, e),
            }
            // 그새 뒤에 선 행은 줄이 이어 보낸다.
            me.kick(peer_id);
            got
        })
        .await;
        Ok(match sent {
            Ok(Ok(resp)) => Shipped::Answered(resp),
            _ => Shipped::Kept,
        })
    }

    /// 줄에만 넣는다(바로 보내지 않음) — `key` 가 같은 기다리는 행이 있으면 그 행을 바꾼다(진행 — 마지막 값만).
    fn enqueue(
        &self,
        conv: &Conversation,
        msg_id: Option<i64>,
        kind: &'static str,
        req: &Request,
        key: Option<&str>,
    ) {
        let added = serde_json::to_string(req)
            .map_err(io::Error::other)
            .and_then(|frame| {
                let db = self.db();
                outbox::add(
                    &db,
                    conv.peer_id,
                    Some(conv.id),
                    msg_id,
                    kind,
                    &frame,
                    key,
                    "pending",
                )
                .map_err(db_err)
            });
        match added {
            Ok(_) => self.kick(conv.peer_id),
            Err(e) => log::warn!("보낼 글 보관 실패({kind}): {e}"),
        }
        self.fan_share(conv.id);
    }

    /// 페어링된 상대에 요청 하나(키 고정) — 응답 내용은 부르는 쪽이 본다.
    async fn send_to(&self, peer_id: i64, req: &Request) -> io::Result<Response> {
        let peer = self.paired_peer(peer_id)?;
        match self.node.send(addr(&peer)?, Some(peer.spki), req).await?.0 {
            // 상대가 잠시 처리하지 못했다(DB 잠금 등) — 거절이 아니라 다시 보낼 일.
            Response::Error { code } if code == RETRY => Err(io::Error::other(
                "상대가 잠시 처리하지 못했다 — 다시 보낸다",
            )),
            resp => Ok(resp),
        }
    }

    /// 전송 오류 — 간격을 늘려 줄로 되돌리거나, 기다림 합이 한도를 넘으면 실패 보관 + [`give_up`](Self::give_up).
    fn backoff(&self, id: i64, e: &io::Error) {
        let r = *an_core::sync::lock(&self.retry);
        let item = match outbox::get(&self.db(), id) {
            Ok(Some(it)) => it,
            Ok(None) => return,
            Err(err) => return log::warn!("보낼 글 {id} 읽기 실패: {err}"),
        };
        let delay = r
            .base
            .saturating_mul(1i64 << item.tries.clamp(0, 20))
            .min(r.max);
        // 직전 간격까지만 센다(앞당긴 재시도 · 앱을 꺼 둔 시간은 한도에 넣지 않는다).
        let prev = r
            .base
            .saturating_mul(1i64 << (item.tries - 1).clamp(0, 20))
            .min(r.max);
        let why = e.to_string();
        let waited = match outbox::retry(&self.db(), id, delay, prev, &why) {
            Ok(w) => w,
            Err(err) => return log::warn!("보낼 글 {id} 재시도 기록 실패: {err}"),
        };
        if waited < r.give_up {
            return;
        }
        // 그 상대에 한도만큼 닿지 않았다 — 줄에 기다리는 글도 함께 포기한다(뒤 글이 하나씩 24시간을 다시 기다리지 않게). 지금 보내는 중
        // (`inflight`)인 글은 그 결과가 정한다 — 닿은 글을 "보내지 못함" 으로 덮지 않게(적대적 평가 라운드 2).
        let queued = outbox::queued_for(&self.db(), item.peer_id).unwrap_or_default();
        log::warn!(
            "상대 {} 에 {waited}초 동안 닿지 않아 보내기 포기 — {}건: {why}",
            item.peer_id,
            queued.len()
        );
        let mut unshared = Vec::new();
        for it in queued {
            // 컨텍스트 처리 결과(#1m)는 포기하지 않는다 — 요청자 구분선이 `sent` 에 영영 묶이지 않게, 돌아오면 줄이 보낸다(라운드 2).
            if it.kind == CTX_DONE {
                continue;
            }
            // 공유 프레임(#1o) — 관람자에 한도만큼 닿지 않으면 그 관람을 끝낸다(행을 지우고 관람자에서 뺀다 — 꺼진 관람자 몫이 끝없이 쌓이지 않게,
            // 적대적 평가 라운드 1). 원 상대 알림은 버린다. 사본은 관람자 PC 에 남고, 빼기 표시 한 행이 돌아오면 간다(다시 넣으면 처음부터).
            if matches!(it.kind.as_str(), SHARE | SHARE_NOTICE) {
                // 빼기 표시(`ShareEnd`)는 남긴다 — 돌아온 관람자 사본이 "공유 해제됨" 이 되게(대화마다 한 행, 라운드 2).
                if matches!(
                    serde_json::from_str::<Request>(&it.frame),
                    Ok(Request::ShareEnd { .. })
                ) {
                    continue;
                }
                if let Err(err) = outbox::remove(&self.db(), it.id) {
                    log::warn!("보낼 공유 글 {} 지우기 실패: {err}", it.id);
                }
                if it.kind == SHARE
                    && let Some(conv) = it.conv_id
                    && !unshared.contains(&conv)
                {
                    unshared.push(conv);
                }
                continue;
            }
            // 편집 · 삭제 반영(#1f)은 실패 보관과 '상대 반영 실패' 기록을 한 트랜잭션으로 — 하나만 남지 않게(실패하면 행은 줄에 남아 다시 포기한다).
            if matches!(it.kind.as_str(), outbox::EDIT | outbox::DELETE) {
                let failed = (|| {
                    let db = self.db();
                    let tx = an_core::db::write_tx(&db).map_err(db_err)?;
                    outbox::fail(&tx, it.id, &why).map_err(db_err)?;
                    Self::note_sync_failed(&tx, &it)?;
                    tx.commit().map_err(db_err)
                })();
                if let Err(err) = failed {
                    log::warn!("보낼 글 {} 실패 보관 기록 실패: {err}", it.id);
                }
                continue;
            }
            if let Err(err) = outbox::fail(&self.db(), it.id, &why) {
                log::warn!("보낼 글 {} 실패 보관 기록 실패: {err}", it.id);
            }
            self.give_up(&it);
        }
        for conv in unshared {
            let found = chat::by_id(&self.db(), conv);
            match found {
                Ok(Some(c)) => {
                    if let Err(err) = self.unshare(&c, item.peer_id, true) {
                        log::warn!("대화 {conv} 관람자 {} 빼기 실패: {err}", item.peer_id);
                    }
                }
                Ok(None) => {}
                Err(err) => log::warn!("대화 {conv} 읽기 실패: {err}"),
            }
        }
    }

    /// 상대 줄을 깨운다(없으면 띄운다 — 상대마다 tokio 작업 하나).
    fn kick(&self, peer_id: i64) {
        let Some(me) = self.me.upgrade() else {
            return;
        };
        let mut lanes = an_core::sync::lock(&self.lanes);
        // 줄 작업이 끝났으면(panic) 다시 띄운다 — 죽은 작업을 깨우며 그 상대가 멈추지 않게.
        if lanes.get(&peer_id).is_none_or(|(_, h)| h.is_finished()) {
            let wake = Arc::new(tokio::sync::Notify::new());
            let handle = tokio::spawn(me.lane(peer_id, wake.clone()));
            lanes.insert(peer_id, (wake, handle));
        }
        if let Some((wake, _)) = lanes.get(&peer_id) {
            wake.notify_one();
        }
    }

    /// 상대 줄 — 맨 앞 행부터 하나씩(앞 행이 못 가면 뒤도 기다린다 — 대화 순서 보존). 바로 보내는 중(`inflight`)이면 끝나고 깨울 때까지.
    async fn lane(self: Arc<Self>, peer_id: i64, wake: Arc<tokio::sync::Notify>) {
        loop {
            let head = outbox::head(&self.db(), peer_id);
            let item = match head {
                Ok(Some(it)) if it.state == "pending" => it,
                Ok(_) => {
                    wake.notified().await;
                    continue;
                }
                Err(e) => {
                    log::warn!("상대 {peer_id} 보낼 글 읽기 실패: {e}");
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    continue;
                }
            };
            if item.due_in > 0 {
                let wait = Duration::from_secs(item.due_in.unsigned_abs());
                tokio::select! {
                    _ = tokio::time::sleep(wait) => {}
                    _ = wake.notified() => {}
                }
                continue;
            }
            self.deliver(item).await;
        }
    }

    async fn deliver(&self, item: outbox::Item) {
        let req: Request = match serde_json::from_str(&item.frame) {
            Ok(r) => r,
            Err(e) => {
                log::warn!("보낼 글 {} 프레임 형식 오류 — 보관: {e}", item.id);
                let _ = outbox::fail(&self.db(), item.id, "프레임 형식 오류");
                return;
            }
        };
        // 결과를 먼저 받는다 — `if let` 조건의 DB 잠금은 블록 끝까지 살아 있다(await 를 넘기면 안 된다).
        let marked = outbox::set_state(&self.db(), item.id, "inflight");
        if let Err(e) = marked {
            log::warn!("보낼 글 {} 상태 기록 실패: {e}", item.id);
            tokio::time::sleep(Duration::from_secs(5)).await;
            return;
        }
        // 컨텍스트 프레임(#1m) — 이 종류를 모르는 상대(0.4.0)는 응답 없이 연결을 끊어 24시간 다시 보내게 된다. 먼저 Ping 으로 묻는다.
        if matches!(item.kind.as_str(), CTX | CTX_DONE) {
            match self.node.knows_context(item.peer_id).await {
                Ok(true) => {}
                Ok(false) => {
                    let resp = Response::error(CTX_UNSUPPORTED);
                    if self.acked(item.id, &resp) {
                        self.settle(&item, &resp);
                    }
                    return;
                }
                Err(e) => return self.backoff(item.id, &e),
            }
        }
        // 공유 프레임(#1o)도 같다 — 모르는 상대면 관람자를 빼거나(`settle`) 원 상대 알림을 버린다.
        // ponytail: 공유 글마다 Ping 한 번 더 — 이력이 길어 느리면 상대별로 아는지 기억한다.
        if matches!(item.kind.as_str(), SHARE | SHARE_NOTICE) {
            let known = match self.node.knows_share(item.peer_id).await {
                Ok(k) => k,
                // Ping 을 거절했다 = 상대가 이 PC 를 페어링된 상대로 보지 않는다(`not_paired`).
                Err(e) if e.kind() == io::ErrorKind::PermissionDenied => {
                    let resp = Response::error("not_paired");
                    if self.acked(item.id, &resp) {
                        self.settle(&item, &resp);
                    }
                    return;
                }
                Err(e) => return self.backoff(item.id, &e),
            };
            if !known {
                let resp = Response::error("unsupported");
                if self.acked(item.id, &resp) {
                    self.settle(&item, &resp);
                }
                return;
            }
        }
        // 빠진 관람자(#1o) — 빼기 · 나가기 · 포기 뒤 남은(보내던 중 · 다시 보낼) 공유 글은 보내지 않고 지운다. 빼기 표시(`ShareEnd`)는 보낸다
        // (적대적 평가 라운드 1) — 능력 확인(Ping) 뒤, 보내기 바로 앞에서(라운드 2). ponytail: 확인 ~ 보내기 사이(ms)에 빠지면 그 한 글은 간다 — 막으려면
        // 관람자별 잠금으로 빼기와 줄 세우거나 프레임에 공유 세대를 싣는다.
        if item.kind == SHARE && !matches!(req, Request::ShareEnd { .. }) && !self.watching(&item) {
            let _ = self.acked(item.id, &Response::Ok);
            return;
        }
        match self.send_to(item.peer_id, &req).await {
            Ok(resp) => {
                if self.acked(item.id, &resp) {
                    self.settle(&item, &resp);
                }
            }
            Err(e) => self.backoff(item.id, &e),
        }
    }

    /// 응답을 받은 행을 지운다 — 편집 · 삭제 반영(#1f)이 거절됐으면 그 대화의 '상대 반영 실패' 기록과 행 지우기를 한 트랜잭션으로(그 사이에
    /// 꺼져도 실패가 사라지지 않게). 그 트랜잭션이 실패하면(어느 종류의 행이든) 행을 줄로 되돌려 다시 보낸다 — 거짓, 뒷일(`settle`)은 다시 받은
    /// 응답이 한다(프레임은 uuid 로 멱등이라 다시 보내도 상대에서 한 번만 적용된다). 그 전에는 지우기 실패를 로그만 남기고 바로 정산했다.
    fn acked(&self, id: i64, resp: &Response) -> bool {
        let noted = (|| {
            let db = self.db();
            let tx = an_core::db::write_tx(&db).map_err(db_err)?;
            let item = outbox::get(&tx, id).map_err(db_err)?;
            let sync = item
                .as_ref()
                .is_some_and(|it| matches!(it.kind.as_str(), outbox::EDIT | outbox::DELETE));
            if sync && !matches!(resp, Response::Ok) {
                Self::note_sync_failed(&tx, item.as_ref().expect("위에서 확인"))?;
            }
            outbox::remove(&tx, id).map_err(db_err)?;
            tx.commit().map_err(db_err)?;
            Ok::<_, io::Error>(sync)
        })();
        match noted {
            Ok(_) => true,
            Err(e) => {
                log::warn!("보낸 글 {id} 정산 실패 — 다시 보낸다: {e}");
                self.backoff(id, &e);
                false
            }
        }
    }

    /// 글이 속한 대화.
    fn conv_of(&self, msg_id: i64) -> Option<Conversation> {
        let db = self.db();
        let m = chat::message_by_id(&db, msg_id).ok().flatten()?;
        chat::by_id(&db, m.conversation_id).ok().flatten()
    }

    /// ack 뒤 처리 — 줄이 나중에 보낸 글의 응답을 그 자리의 처리와 같게(거절이면 글 실패 · 자동 왕복 끔, 미러 `Ok` 면 완료).
    fn settle(&self, item: &outbox::Item, resp: &Response) {
        let ok = matches!(resp, Response::Ok);
        // 편집 · 삭제 반영(#1f) — 거절 기록은 응답 정산(`acked`)이 행 지우기와 함께 했다.
        if matches!(item.kind.as_str(), outbox::EDIT | outbox::DELETE) {
            return;
        }
        // 공유 프레임(#1o) — 실어 나르는 글이 없다.
        if matches!(item.kind.as_str(), SHARE | SHARE_NOTICE) {
            if !ok {
                self.share_refused(item, resp);
            }
            return;
        }
        let Some(msg_id) = item.msg_id else {
            if !ok {
                log::info!(
                    "상대 {} 가 {} 를 받지 않았다: {resp:?}",
                    item.peer_id,
                    item.kind
                );
            }
            return;
        };
        match (item.kind.as_str(), ok) {
            // 상대가 멈춘 뒤 닿은 넘김 — 멈춤 알림이 먼저 온 것과 같게(내 답은 실패가 아니다).
            (RELAY_STEP, false) if matches!(resp, Response::Error { code } if code == STOPPED) => {
                if let Some(conv) = self.conv_of(msg_id) {
                    self.relay_stopped(&conv, msg_id);
                }
            }
            (MESSAGE | RELAY_OPEN | RELAY_STEP, false) => {
                self.refused_by_peer(msg_id, item.peer_id, resp);
                if item.kind == MESSAGE {
                    return;
                }
                if let Some(conv) = self.conv_of(msg_id) {
                    let _ = chat::relay_on(&self.db(), conv.id, false, false);
                    if item.kind == RELAY_STEP {
                        let focus = matches!(resp, Response::Error { code } if code == FOCUS);
                        self.relay_end(
                            &conv,
                            if focus {
                                "상대가 집중 중"
                            } else {
                                "상대가 거절"
                            },
                        );
                    }
                }
            }
            (NOTE, true) => {
                let _ = chat::set_state(&self.db(), msg_id, State::Done);
            }
            (NOTE, false) => {
                let _ = chat::set_state(&self.db(), msg_id, State::Error);
            }
            (REPLY | REPLY_POST | RELAY_LAST, false) => {
                let _ = chat::set_state(&self.db(), msg_id, State::Undelivered);
            }
            (REQ_STATE, false) => self.req_state_refused(msg_id, resp),
            (CTX, false) => {
                let code = match resp {
                    Response::Error { code } => an_net::proto::clean_code(code).to_string(),
                    _ => "unknown".into(),
                };
                self.context_refused(msg_id, &code);
            }
            (_, false) => log::info!(
                "상대 {} 가 {} 를 받지 않았다: {resp:?}",
                item.peer_id,
                item.kind
            ),
            _ => {}
        }
    }

    /// 다시 보내기 포기(한도) — 내 글은 "보내지 못함"(실패 + `unreachable`), 답은 "답 미전달", 자동 왕복은 멈춘다.
    fn give_up(&self, item: &outbox::Item) {
        let Some(msg_id) = item.msg_id else {
            return;
        };
        match item.kind.as_str() {
            MESSAGE | RELAY_OPEN | RELAY_STEP | NOTE => {
                {
                    let db = self.db();
                    let _ = chat::set_state(&db, msg_id, State::Error);
                    let _ = chat::set_refused(&db, msg_id, Some(UNREACHABLE));
                }
                let relay = matches!(item.kind.as_str(), RELAY_OPEN | RELAY_STEP);
                if let Some(conv) = self.conv_of(msg_id).filter(|_| relay) {
                    let _ = chat::relay_on(&self.db(), conv.id, false, false);
                    if item.kind == RELAY_STEP {
                        self.relay_end(&conv, "상대에 닿지 않음");
                    }
                }
            }
            REPLY | REPLY_POST | RELAY_LAST => {
                let _ = chat::set_state(&self.db(), msg_id, State::Undelivered);
            }
            CTX => self.context_refused(msg_id, UNREACHABLE),
            // 받은 요청의 상태를 요청자에 알리지 못했다 — 그 글에 표시, 받은 사람이 "다시 알리기"(#1d R1).
            REQ_STATE => {
                let _ = chat::set_refused(&self.db(), msg_id, Some(UNREACHABLE));
            }
            _ => {}
        }
    }

    /// 상대에서 무엇이든 받았다 — 그 상대 줄에 남은 글을 지금 보낸다(돌아온 증거).
    fn peer_back(&self, peer_id: i64) {
        let waiting = outbox::head(&self.db(), peer_id);
        match waiting {
            Ok(Some(_)) => {
                let hurried = outbox::hurry(&self.db(), peer_id);
                if let Err(e) = hurried {
                    log::warn!("상대 {peer_id} 보낼 글 앞당기기 실패: {e}");
                }
                self.kick(peer_id);
            }
            Ok(None) => {}
            Err(e) => log::warn!("상대 {peer_id} 보낼 글 읽기 실패: {e}"),
        }
    }

    /// 내 글의 보관 상태(웹UI — 보내는 중 · 보내지 못함). 상대에 닿은 글은 `None`.
    pub fn outbox_of(&self, m: &Message) -> Option<outbox::Item> {
        if !m.mine {
            return None;
        }
        outbox::for_message(&self.db(), m.id).ok().flatten()
    }

    /// 상태 질의(#1c) — 보낸 쪽이 자기 실행 요청의 지금 상태를 상대에 묻는다(바로 — 보관하지 않음). `queued` · `running` 이면 진행을 갱신한다.
    pub async fn ask_status(&self, m: &Message) -> io::Result<Response> {
        let conv = chat::by_id(&self.db(), m.conversation_id)
            .map_err(db_err)?
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "없는 대화"))?;
        let req = Request::Status {
            conv: conv.uuid.clone(),
            msg: m.uuid.clone(),
        };
        let resp = self.send_to(conv.peer_id, &req).await?;
        self.peer_back(conv.peer_id);
        if let Response::Status { state, ahead } = &resp
            && matches!(state.as_str(), "queued" | "running")
            && m.state == State::Sent
        {
            let mut p = an_core::sync::lock(&self.progress);
            let detail = p.get(&m.id).and_then(|x| x.detail.clone());
            p.insert(
                m.id,
                Progress {
                    state: state.clone(),
                    ahead: if state == "running" { 0 } else { *ahead },
                    detail: if state == "running" { detail } else { None },
                },
            );
        }
        Ok(resp)
    }

    /// 상대가 묻는 자기 실행 요청의 상태 — 그 상대 대화의 상대 실행 요청 글만. 끝났으면 `done`(그 상대에 남은 답이 있으면 지금 보낸다).
    fn inbound_status(&self, peer: &Peer, conv: String, msg: String) -> io::Result<Response> {
        let found = {
            let db = self.db();
            match chat::find_own(&db, &conv, peer.id).map_err(db_err)? {
                Some(c) => chat::message_by_uuid(&db, &msg)
                    .map_err(db_err)?
                    .filter(|m| m.conversation_id == c.id && !m.mine && m.exec),
                None => None,
            }
        };
        let Some(m) = found else {
            return Ok(Response::Status {
                state: "unknown".into(),
                ahead: 0,
            });
        };
        let (state, ahead) = match m.state {
            State::Queued => {
                let (_, _, waiting) = self.run_queue();
                let ahead = waiting.iter().position(|&id| id == m.id).unwrap_or(0);
                ("queued", u32::try_from(ahead).unwrap_or(u32::MAX))
            }
            State::Running => ("running", 0),
            _ => {
                self.peer_back(peer.id);
                ("done", 0)
            }
        };
        Ok(Response::Status {
            state: state.into(),
            ahead,
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
        // 지난 실행에서 보내지 못한 글 — 상대 줄을 다시 띄운다(#1c). 7일 지난 실패 보관은 지운다.
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

    fn paired_peer(&self, peer_id: i64) -> io::Result<Peer> {
        let peer = self
            .node
            .peers()?
            .into_iter()
            .find(|p| p.id == peer_id)
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::NotFound, format!("피어 {peer_id} 없음"))
            })?;
        if !peer.paired() {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "페어링되지 않은 상대",
            ));
        }
        Ok(peer)
    }

    /// 상대 PC 의 에이전트 이름.
    pub async fn remote_agents(&self, peer_id: i64) -> io::Result<Vec<String>> {
        let peer = self.paired_peer(peer_id)?;
        match self
            .node
            .send(addr(&peer)?, Some(peer.spki.clone()), &Request::Agents)
            .await?
        {
            (Response::Agents { names }, _) => Ok(names),
            (other, _) => Err(refused(other)),
        }
    }

    /// 사람 → 원격 에이전트. `conversation` 이 있으면 그 대화에 이어 쓴다. 상대가 받아 대기열에 넣으면 끝(답은 나중에 `reply`).
    pub async fn send(
        &self,
        peer_id: i64,
        agent: &str,
        conversation: Option<i64>,
        text: &str,
    ) -> io::Result<(Conversation, Message)> {
        self.send_via(peer_id, agent, conversation, text, None, None)
            .await
    }

    /// [`send`](Self::send) + 글의 출처(`via` — `cli` 면 사용자 AI 에이전트가 CLI 로 쓴 글, 양쪽 이력에 남는다) · 새 대화의 제목
    /// (`title` — 이 PC 에만, 이어 쓰기면 쓰지 않는다).
    pub async fn send_via(
        &self,
        peer_id: i64,
        agent: &str,
        conversation: Option<i64>,
        text: &str,
        via: Option<&str>,
        title: Option<&str>,
    ) -> io::Result<(Conversation, Message)> {
        let opts = Compose {
            via,
            title,
            ..Compose::default()
        };
        self.send_with(peer_id, agent, conversation, text, opts)
            .await
    }

    /// [`send`](Self::send) + 새 글의 선택([`Compose`]).
    pub async fn send_with(
        &self,
        peer_id: i64,
        agent: &str,
        conversation: Option<i64>,
        text: &str,
        opts: Compose<'_>,
    ) -> io::Result<(Conversation, Message)> {
        if text.trim().is_empty() || text.len() > MAX_TEXT {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("본문은 1 ~ {MAX_TEXT} 바이트"),
            ));
        }
        request_ok(&opts, agent)?;
        // 새 대화의 에이전트는 `agent` — 멘션은 이어 쓰기에서만(대화를 만들기 전에 거른다).
        if conversation.is_none() && opts.mention.is_some_and(|m| m != agent) {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, MENTION_ONLY));
        }
        let peer = self.paired_peer(peer_id)?;
        let conv = {
            let db = self.db();
            match conversation {
                Some(id) => {
                    let c = chat::by_id(&db, id).map_err(db_err)?;
                    if let Some(c) = &c {
                        Self::writable(c)?;
                    }
                    c.filter(|c| c.dir == Dir::Out && c.peer_id == peer_id)
                        .ok_or_else(|| {
                            io::Error::new(io::ErrorKind::NotFound, format!("대화 {id} 없음"))
                        })?
                }
                None => {
                    // 대화와 제목은 한 트랜잭션, 보내기 전에 — 전송이 실패해도 대화는 남으므로 제목도 남는다.
                    let tx = an_core::db::write_tx(&db).map_err(db_err)?;
                    let c = chat::open_conversation(&tx, &new_uuid(), peer_id, agent, Dir::Out)
                        .map_err(db_err)?;
                    chat::set_title(&tx, c.id, opts.title).map_err(db_err)?;
                    // 그룹(#1i)이 그 사이 지워졌으면 대화도 만들지 않고 보내지 않는다(트랜잭션을 버린다 — 되돌림).
                    if let Some(g) = opts.group
                        && !an_core::groups::join(&tx, g, c.id).map_err(db_err)?
                    {
                        return Err(no_group());
                    }
                    tx.commit().map_err(db_err)?;
                    c
                }
            }
        };
        // 멘션 — 자동 왕복 대화는 아니고(상대 에이전트끼리의 차례에 끼지 않는다), 대화 에이전트 이름이면 멘션이 아니다.
        let mention = opts.mention.filter(|m| *m != conv.agent);
        if mention.is_some_and(|m| conv.relay.is_some() || m.trim().is_empty()) {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, MENTION_ONLY));
        }
        let opts = Compose { mention, ..opts };
        // 사람이 쓰면 자동 왕복은 멈춘다(D24 — 사람 입력 대기).
        if conversation.is_some() {
            let _ = chat::relay_on(&self.db(), conv.id, false, false);
        }
        self.send_human(conv, &peer, text, None, opts).await
    }

    /// 두 에이전트 자동 왕복 대화를 연다(D24) — `my_agent` 는 이 PC 프로필, 상한 1 ~ 50 왕복. `title` = 이 PC 에서 붙이는 제목.
    pub async fn start_relay(
        &self,
        peer_id: i64,
        agent: &str,
        my_agent: &str,
        cap: i64,
        text: &str,
        title: Option<&str>,
    ) -> io::Result<(Conversation, Message)> {
        let opts = Compose {
            title,
            ..Compose::default()
        };
        self.start_relay_with(peer_id, agent, my_agent, cap, text, opts)
            .await
    }

    /// [`start_relay`](Self::start_relay) + 첫 글의 선택([`Compose`] — 출처는 쓰지 않는다).
    pub async fn start_relay_with(
        &self,
        peer_id: i64,
        agent: &str,
        my_agent: &str,
        cap: i64,
        text: &str,
        opts: Compose<'_>,
    ) -> io::Result<(Conversation, Message)> {
        if text.trim().is_empty() || text.len() > MAX_TEXT {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("본문은 1 ~ {MAX_TEXT} 바이트"),
            ));
        }
        if !(1..=MAX_RELAY_CAP).contains(&cap) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("자동 왕복 상한은 1 ~ {MAX_RELAY_CAP}"),
            ));
        }
        // 요청(#1d)은 실행이 없어 넘길 답이 없다 — 대화를 만들기 전에 거른다.
        if opts.request {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                REQUEST_NO_RELAY,
            ));
        }
        let peer = self.paired_peer(peer_id)?;
        let conv = {
            let db = self.db();
            if agents::by_name(&db, my_agent).map_err(db_err)?.is_none() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("내 에이전트 {my_agent} 가 없다"),
                ));
            }
            let tx = an_core::db::write_tx(&db).map_err(db_err)?;
            let c = chat::open_conversation(&tx, &new_uuid(), peer_id, agent, Dir::Out)
                .map_err(db_err)?;
            chat::set_relay(&tx, c.id, my_agent, cap).map_err(db_err)?;
            chat::set_title(&tx, c.id, opts.title).map_err(db_err)?;
            if let Some(g) = opts.group
                && !an_core::groups::join(&tx, g, c.id).map_err(db_err)?
            {
                return Err(no_group());
            }
            let c = chat::by_id(&tx, c.id).map_err(db_err)?.unwrap_or(c);
            tx.commit().map_err(db_err)?;
            c
        };
        let opts = Compose {
            via: None,
            mention: None,
            ..opts
        };
        self.send_human(conv, &peer, text, Some(my_agent.to_string()), opts)
            .await
    }

    /// 사람 글을 상대 에이전트에게 실행 요청으로. `relay` = 자동 왕복을 여는 첫 요청(내 에이전트 이름).
    async fn send_human(
        &self,
        conv: Conversation,
        peer: &Peer,
        text: &str,
        relay: Option<String>,
        opts: Compose<'_>,
    ) -> io::Result<(Conversation, Message)> {
        let via = opts.via;
        // 저장부터 상대 응답까지 대화 단위로 한 번에 하나.
        let lock = self.turn(conv.id);
        let _turn = lock.lock().await;
        let msg = {
            let db = self.db();
            // 글과 출처는 한 트랜잭션 — 출처 없이 글만 남지 않게.
            let tx = an_core::db::write_tx(&db).map_err(db_err)?;
            let mut msg =
                chat::add_message(&tx, conv.id, &new_uuid(), Author::Human, text, State::Sent)
                    .map_err(db_err)?;
            if via.is_some() {
                chat::set_via(&tx, msg.id, via).map_err(db_err)?;
                msg.via = via.map(str::to_string);
            }
            chat::set_request(&tx, msg.id, opts.request).map_err(db_err)?;
            msg.request = opts.request;
            if opts.request {
                // 요청(#1d) — 상대 에이전트를 실행하지 않는다: 답을 기다리지 않는 사람 글(닿으면 `done`).
                chat::set_flags(&tx, msg.id, true, false, State::Sent).map_err(db_err)?;
                msg.exec = false;
            }
            if opts.mention.is_some() {
                chat::set_agent(&tx, msg.id, opts.mention).map_err(db_err)?;
                msg.agent = opts.mention.map(str::to_string);
            }
            if via.is_some() {
                cli_tx::create(&tx, opts.tx_key, msg.id).map_err(db_err)?;
            }
            let req = Request::Message {
                conv: conv.uuid.clone(),
                msg: msg.uuid.clone(),
                agent: conv.agent.clone(),
                text: text.to_string(),
                lc: Some(msg.lc),
                by_agent: None,
                relay: relay.clone(),
                via: msg.via.clone(),
                request: msg.request,
                mention: msg.agent.clone(),
            };
            let kind = match (relay.is_some(), msg.request) {
                (true, _) => RELAY_OPEN,
                (false, true) => NOTE,
                (false, false) => MESSAGE,
            };
            // 보내기 행도 같은 트랜잭션 — 글만 남고 보낼 행이 없는 채로 꺼지지 않게(#1c).
            let staged = Self::stage(&tx, &conv, Some(msg.id), kind, &req)?;
            tx.commit().map_err(db_err)?;
            (msg, req, staged)
        };
        let (msg, req, staged) = msg;
        let starts_relay = relay.is_some();
        let _ = peer;
        match self.dispatch(&conv, staged, &req).await? {
            // 상대가 받았거나, 닿지 않아 이 PC 에 보관했다(#1c — 줄이 다시 보낸다, 같은 uuid 라 상대에서 한 번만 실행).
            Shipped::Answered(Response::Ok) if msg.request => {
                let _ = chat::set_state(&self.db(), msg.id, State::Done);
                let msg = chat::message_by_id(&self.db(), msg.id)
                    .map_err(db_err)?
                    .unwrap_or(msg);
                Ok((conv, msg))
            }
            Shipped::Answered(Response::Ok) | Shipped::Kept => Ok((conv, msg)),
            // 명시적 거절만 실패로 확정한다. 자동 왕복을 여는 글이면 켜짐도 거둔다(상대에 그 대화가 없다 — 재개가 갈 곳이 없다).
            Shipped::Answered(other) => {
                self.refused_by_peer(msg.id, conv.peer_id, &other);
                if starts_relay {
                    let _ = chat::relay_on(&self.db(), conv.id, false, false);
                }
                Err(refused(other))
            }
        }
    }

    /// 대화에 사람 글(D18 · D23) — `to_agent` 면 대화 에이전트에게 실행 요청, 아니면 실행 없이 양쪽 이력에만.
    /// 보낸 대화(`out`)의 실행 요청은 [`send`](Self::send), 그 밖은 내 이력에 남기고 상대에 `post` 로 미러한다.
    /// 세 번째 값은 경고 — 내 에이전트 실행은 받아들였지만 상대 이력 미러가 실패한 경우(실패로 돌려주면 다시 보내 두 번 실행된다).
    pub async fn post(
        self: &Arc<Self>,
        conversation_id: i64,
        text: &str,
        to_agent: bool,
    ) -> io::Result<(Conversation, Message, Option<String>)> {
        self.post_via(conversation_id, text, to_agent, None).await
    }

    /// [`post`](Self::post) + 글의 출처(`via`).
    pub async fn post_via(
        self: &Arc<Self>,
        conversation_id: i64,
        text: &str,
        to_agent: bool,
        via: Option<&str>,
    ) -> io::Result<(Conversation, Message, Option<String>)> {
        let opts = Compose {
            via,
            ..Compose::default()
        };
        self.post_with(conversation_id, text, to_agent, opts).await
    }

    /// [`post`](Self::post) + 새 글의 선택([`Compose`] — 제목은 쓰지 않는다).
    pub async fn post_with(
        self: &Arc<Self>,
        conversation_id: i64,
        text: &str,
        to_agent: bool,
        opts: Compose<'_>,
    ) -> io::Result<(Conversation, Message, Option<String>)> {
        let via = opts.via;
        if text.trim().is_empty() || text.len() > MAX_TEXT {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("본문은 1 ~ {MAX_TEXT} 바이트"),
            ));
        }
        let conv = chat::by_id(&self.db(), conversation_id)
            .map_err(db_err)?
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("대화 {conversation_id} 없음"),
                )
            })?;
        Self::writable(&conv)?;
        request_ok(&opts, &conv.agent)?;
        // 멘션은 보낸 대화의 실행 요청에만 — 받은 대화의 에이전트는 내 것이다(해석 1).
        if opts.mention.is_some_and(|m| m != conv.agent) && !(conv.dir == Dir::Out && to_agent) {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, MENTION_ONLY));
        }
        // 요청(#1d) — 보낸 대화는 `Message`(상대에 대화가 아직 없을 수 있다), 받은 대화는 실행 없는 미러. 어느 쪽도 에이전트를 실행하지 않는다.
        let to_agent = to_agent && !opts.request;
        if conv.dir == Dir::Out && (to_agent || opts.request) {
            let opts = Compose {
                title: None,
                ..opts
            };
            let (c, m) = self
                .send_with(conv.peer_id, &conv.agent, Some(conv.id), text, opts)
                .await?;
            return Ok((c, m, None));
        }
        let peer = self.paired_peer(conv.peer_id)?;
        // 사람이 쓰면 자동 왕복은 멈춘다(D24) — 연 쪽은 끄고, 받은 쪽은 에이전트발 요청을 거절하고 연 쪽에 알린다.
        if conv.dir == Dir::Out {
            let _ = chat::relay_on(&self.db(), conv.id, false, false);
        } else {
            // 거절은 바로, 연 쪽 알림은 기다리지 않는다(상대가 꺼져 있어도 글쓰기가 늦어지지 않게).
            let _ = chat::set_blocked(&self.db(), conv.id, true);
            let (me, c, p) = (self.clone(), conv.clone(), peer.clone());
            tokio::spawn(async move { me.block_relay(&c, &p).await });
        }
        let lock = self.turn(conv.id);
        let _turn = lock.lock().await;
        // 실행 요청도 먼저 `sent` 로 — 상대 이력에 사람 글이 먼저 닿은 뒤에 실행해야 답 미러가 붙는다.
        let msg = {
            let db = self.db();
            // 글과 출처는 한 트랜잭션 — 출처 없이 글만 남지 않게.
            let tx = an_core::db::write_tx(&db).map_err(db_err)?;
            let mut msg =
                chat::add_human(&tx, conv.id, &new_uuid(), text, State::Sent, true, to_agent)
                    .map_err(db_err)?;
            if via.is_some() {
                chat::set_via(&tx, msg.id, via).map_err(db_err)?;
                msg.via = via.map(str::to_string);
            }
            chat::set_request(&tx, msg.id, opts.request).map_err(db_err)?;
            msg.request = opts.request;
            if via.is_some() {
                cli_tx::create(&tx, opts.tx_key, msg.id).map_err(db_err)?;
            }
            let req = Request::Post {
                conv: conv.uuid.clone(),
                msg: msg.uuid.clone(),
                author: "human".into(),
                text: text.to_string(),
                exec: to_agent,
                reply_to: None,
                ok: None,
                turns: None,
                duration_ms: None,
                lc: Some(msg.lc),
                via: msg.via.clone(),
                model: None,
                effort: None,
                request: msg.request,
                ctx: None,
            };
            let kind = if to_agent { MIRROR } else { NOTE };
            // 보내기 행도 같은 트랜잭션(#1c).
            let staged = Self::stage(&tx, &conv, Some(msg.id), kind, &req)?;
            tx.commit().map_err(db_err)?;
            (msg, req, staged)
        };
        let (msg, req, staged) = msg;
        let sent = self.dispatch(&conv, staged, &req).await;
        if to_agent {
            // 내 에이전트는 미러 결과와 무관하게 실행한다.
            let _ = chat::set_state(&self.db(), msg.id, State::Queued);
            self.wake(conv.id);
        }
        let msg = chat::message_by_uuid(&self.db(), &msg.uuid)
            .map_err(db_err)?
            .unwrap_or(msg);
        let mirror_failed = match sent {
            Ok(Shipped::Answered(Response::Ok)) => {
                if !to_agent {
                    let _ = chat::set_state(&self.db(), msg.id, State::Done);
                }
                let msg = chat::message_by_id(&self.db(), msg.id)
                    .map_err(db_err)?
                    .unwrap_or(msg);
                return Ok((conv, msg, None));
            }
            // 닿지 않아 보관 — 줄이 다시 보낸다(#1c). 실행 없는 글은 상대에 닿으면 `done`.
            Ok(Shipped::Kept) => return Ok((conv, msg, None)),
            Ok(Shipped::Answered(other)) => {
                if !to_agent {
                    let _ = chat::set_state(&self.db(), msg.id, State::Error);
                    return Err(refused(other));
                }
                refused(other).to_string()
            }
            Err(e) => {
                if !to_agent {
                    return Err(e);
                }
                e.to_string()
            }
        };
        log::warn!("대화 {} 미러 실패: {mirror_failed}", conv.uuid);
        Ok((
            conv,
            msg,
            Some(format!(
                "내 에이전트는 실행한다 — 상대 이력에는 남기지 못했다({mirror_failed}). 다시 보내지 마세요"
            )),
        ))
    }

    /// 페어링된 상대의 요청 — 막히지 않는다(실행은 대기열로).
    fn handle(self: &Arc<Self>, peer: &Peer, req: Request) -> Response {
        self.peer_back(peer.id);
        // 글을 남기는 요청 — 끝나면 그 대화의 관람자에게(#1o).
        let fan = match &req {
            Request::Message { conv, .. }
            | Request::Reply { conv, .. }
            | Request::Post { conv, .. } => Some(conv.clone()),
            _ => None,
        };
        let result = match req {
            Request::Agents => self.agents(),
            Request::Message {
                conv,
                msg,
                agent,
                text,
                lc,
                by_agent,
                relay,
                via,
                request,
                mention,
            } => self.inbound_message(
                peer,
                conv,
                msg,
                agent,
                text,
                (lc, by_agent, relay, via, request, mention),
            ),
            Request::Relay { conv, on } => self.inbound_relay(peer, conv, on),
            Request::ReqState {
                conv,
                msg,
                state,
                note,
            } => self.inbound_req_state(peer, conv, msg, state, note),
            Request::Progress {
                conv,
                msg,
                state,
                ahead,
                detail,
            } => self.inbound_progress(peer, conv, msg, state, (ahead, detail)),
            Request::Status { conv, msg } => self.inbound_status(peer, conv, msg),
            Request::Edit { conv, msg, text } => self.inbound_edit(peer, conv, msg, text),
            Request::Delete { conv, msg } => self.inbound_delete(peer, conv, msg),
            Request::Context {
                conv,
                msg,
                op,
                text,
            } => self.inbound_context(peer, conv, msg, op, text),
            Request::Share {
                conv,
                title,
                agent,
                owner_dir,
                other,
            } => self.inbound_share(peer, conv, title, agent, (owner_dir, other)),
            Request::SharePost { conv, msg } => self.inbound_share_post(peer, conv, msg),
            Request::ShareDelete { conv, msg } => self.inbound_share_delete(peer, conv, msg),
            Request::ShareEnd { conv } => self.inbound_share_end(peer, conv),
            Request::ShareNotice { conv, n } => self.inbound_share_notice(peer, conv, n),
            reply @ Request::Reply { .. } => self.inbound_reply(peer, reply),
            post @ Request::Post { .. } => self.inbound_post(peer, post),
            _ => Ok(Response::error("bad_request")),
        };
        if let Some(conv) = fan {
            let found = chat::find_own(&self.db(), &conv, peer.id);
            if let Ok(Some(c)) = found {
                self.fan_share(c.id);
            }
        }
        // 처리 중 오류(DB 잠금 등)는 거절이 아니다 — 보낸 쪽이 다시 보내게 `retry`(#1c).
        result.unwrap_or_else(|e| {
            log::warn!("대화 요청 처리 실패: {e}");
            Response::error(RETRY)
        })
    }

    fn agents(&self) -> io::Result<Response> {
        let names = agents::list(&self.db())
            .map_err(db_err)?
            .into_iter()
            .map(|a| a.name)
            .collect();
        Ok(Response::Agents { names })
    }

    fn inbound_message(
        self: &Arc<Self>,
        peer: &Peer,
        conv: String,
        msg: String,
        agent: String,
        text: String,
        (lc, by_agent, relay, via, request, mention): MessageExtra,
    ) -> io::Result<Response> {
        if text.trim().is_empty() || text.len() > MAX_TEXT || conv.is_empty() || msg.is_empty() {
            return Ok(Response::error("bad_request"));
        }
        // 요청(#1d) — 실행이 없어 집중과 무관하게 받는다. 자동 왕복 · 멘션과 함께 오지 않는다.
        if request {
            if relay.is_some()
                || by_agent.is_some()
                || mention.as_ref().is_some_and(|m| *m != agent)
            {
                return Ok(Response::error("bad_request"));
            }
            return self.inbound_ask(peer, &conv, &msg, agent, &text, (lc, via));
        }
        // 집중(이 PC) — 새 실행 요청(자동 왕복 차례 포함)은 실행 · 기록 · 알림 없이 거부. 판정을 먼저 받는다(조건식의 DB 잠금은 블록 끝까지
        // 산다 — edition 2024).
        // 이미 받은 글의 재전송(같은 uuid)은 새 요청이 아니다 — 아래 기록 단계가 `Ok`(한 번만 실행). 자동 왕복 재개의 승격은 새 실행이라 거부.
        let focused = self.focused(&self.db());
        if focused && !self.resent(peer.id, &conv, &msg, by_agent.is_some())? {
            return Ok(Response::error(FOCUS));
        }
        // 자동 왕복을 여는 사람 요청은 "시작" 하나로, 그 뒤 에이전트발 요청은 중간 글(알림 묶기).
        let kind = match (relay.is_some(), by_agent.is_some()) {
            (_, true) => MessageKind::RelayStep,
            (true, false) => MessageKind::RelayStart,
            (false, false) => MessageKind::Request,
        };
        // 멘션 — 대화 에이전트가 아닌 내 에이전트(해석 1). 자동 왕복 글에는 받지 않는다(대화를 만들기 전에 거른다).
        let mention = mention.filter(|m| *m != agent);
        if mention.is_some() && (relay.is_some() || by_agent.is_some()) {
            return Ok(Response::error("bad_request"));
        }
        let conversation_id = {
            let db = self.db();
            for name in std::iter::once(&agent).chain(mention.as_ref()) {
                if agents::by_name(&db, name).map_err(db_err)?.is_none() {
                    return Ok(Response::error("unknown_agent"));
                }
            }
            let Ok(c) = chat::open_conversation(&db, &conv, peer.id, &agent, Dir::In) else {
                return Ok(Response::error("bad_request")); // 다른 상대의 대화 uuid
            };
            // 자동 왕복 대화에는 멘션을 받지 않는다.
            if c.dir != Dir::In || c.agent != agent || (mention.is_some() && c.relay.is_some()) {
                return Ok(Response::error("bad_request"));
            }
            // 자동 왕복 대화로 기록(받은 쪽) — 첫 실행 실패의 끝 알림 · 상대 에이전트 이름 표시에 쓴다.
            if let Some(name) = relay.as_deref().or(by_agent.as_deref()) {
                chat::mark_relayed(&db, c.id, &an_net::proto::clean_name(name)).map_err(db_err)?;
            }
            if by_agent.is_some() && c.relay_blocked {
                return Ok(Response::error(STOPPED));
            }
            if let Some(old) = chat::message_by_uuid(&db, &msg).map_err(db_err)? {
                // 자동 왕복 재개 — 상한에서 기록만 받은 상대 에이전트 글을 같은 uuid 로 실행 요청으로. 그 밖은 재전송(한 번만 실행).
                let promote = by_agent.is_some()
                    && old.conversation_id == c.id
                    && old.author == Author::Agent
                    && !old.mine
                    && !old.exec;
                if !promote {
                    return Ok(Response::Ok);
                }
                chat::set_flags(&db, old.id, false, true, State::Queued).map_err(db_err)?;
            } else {
                // 저장이 곧 대기열 등록 — 실행 순서는 id 순(저장 순)이다. 글 · 출처 · 순서는 한 트랜잭션.
                let tx = an_core::db::write_tx(&db).map_err(db_err)?;
                let m = if by_agent.is_some() {
                    chat::add_agent_request(&tx, c.id, &msg, &text, State::Queued)
                        .map_err(db_err)?
                } else {
                    let m = chat::add_message(&tx, c.id, &msg, Author::Human, &text, State::Queued)
                        .map_err(db_err)?;
                    chat::set_via(&tx, m.id, via.as_deref()).map_err(db_err)?;
                    chat::set_request(&tx, m.id, request).map_err(db_err)?;
                    if mention.is_some() {
                        chat::set_agent(&tx, m.id, mention.as_deref()).map_err(db_err)?;
                    }
                    m
                };
                chat::observe(&tx, m.id, lc).map_err(db_err)?;
                tx.commit().map_err(db_err)?;
            }
            c.id
        };
        self.wake(conversation_id);
        self.tell_waiting(conversation_id, &msg);
        // 알림의 에이전트 = 실행할 에이전트(멘션이면 부른 이름).
        self.notify(
            conversation_id,
            peer,
            mention.unwrap_or(agent),
            kind,
            preview(&text),
        );
        Ok(Response::Ok)
    }

    /// 상대 사람 글 · 상대 에이전트 답의 미러 — 그 상대 소유 대화에만. 에이전트 답 · 실행 요청 글은 에이전트가 상대 쪽에 있는
    /// 대화(내 `out`)에만 받는다.
    fn inbound_post(&self, peer: &Peer, post: Request) -> io::Result<Response> {
        let Request::Post {
            conv,
            msg,
            author,
            text,
            exec,
            reply_to,
            ok,
            turns,
            duration_ms,
            lc,
            via,
            model,
            effort,
            request,
            ctx,
        } = post
        else {
            return Ok(Response::error("bad_request"));
        };
        let ctx = ctx.filter(|v| (0..=MAX_CTX_TOKENS).contains(v));
        let (model, effort) = run_meta(model, effort);
        // 에이전트 답은 비어 있으면 받지 않는다 — 조용한 빈 성공 · 자동 왕복의 빈 입력(1a R1).
        if text.len() > MAX_TEXT || msg.is_empty() || (author == "agent" && text.trim().is_empty())
        {
            return Ok(Response::error("bad_request"));
        }
        let mut kind = MessageKind::Request;
        let (cid, agent) = {
            let db = self.db();
            let Some(c) = chat::find_own(&db, &conv, peer.id).map_err(db_err)? else {
                return Ok(Response::error("unknown"));
            };
            if chat::message_by_uuid(&db, &msg).map_err(db_err)?.is_some() {
                return Ok(Response::Ok); // 같은 글 재전송
            }
            match author.as_str() {
                "human" => {
                    if text.trim().is_empty() || (exec && (c.dir != Dir::Out || request)) {
                        return Ok(Response::error("bad_request"));
                    }
                    let tx = an_core::db::write_tx(&db).map_err(db_err)?;
                    let m = chat::add_human(
                        &tx,
                        c.id,
                        &msg,
                        &text,
                        if exec { State::Sent } else { State::Done },
                        false,
                        exec,
                    )
                    .map_err(db_err)?;
                    chat::set_via(&tx, m.id, via.as_deref()).map_err(db_err)?;
                    chat::set_request(&tx, m.id, request).map_err(db_err)?;
                    chat::observe(&tx, m.id, lc).map_err(db_err)?;
                    tx.commit().map_err(db_err)?;
                    if request {
                        kind = MessageKind::Ask;
                    }
                }
                "agent" if c.dir == Dir::In => {
                    // 자동 왕복 상한 — 상대 에이전트가 내 에이전트 답에 단 마지막 답. 실행 없이 기록만.
                    let answered = match reply_to.as_deref() {
                        Some(r) => chat::message_by_uuid(&db, r).map_err(db_err)?,
                        None => None,
                    }
                    .filter(|m| m.conversation_id == c.id && m.author == Author::Agent && m.mine);
                    if exec || text.trim().is_empty() || answered.is_none() {
                        return Ok(Response::error("bad_request"));
                    }
                    let tx = an_core::db::write_tx(&db).map_err(db_err)?;
                    let m = chat::add_agent_request(&tx, c.id, &msg, &text, State::Done)
                        .map_err(db_err)?;
                    chat::set_flags(&tx, m.id, false, false, State::Done).map_err(db_err)?;
                    chat::set_run(
                        &tx,
                        m.id,
                        turns,
                        duration_ms,
                        model.as_deref(),
                        effort.as_deref(),
                    )
                    .map_err(db_err)?;
                    chat::observe(&tx, m.id, lc).map_err(db_err)?;
                    chat::set_ctx_tokens(&tx, m.id, ctx).map_err(db_err)?;
                    tx.commit().map_err(db_err)?;
                    kind = MessageKind::RelayEnd;
                }
                "agent" => {
                    kind = MessageKind::Reply;
                    let asked = match reply_to.as_deref() {
                        Some(r) => chat::message_by_uuid(&db, r).map_err(db_err)?,
                        None => None,
                    }
                    .filter(|m| {
                        m.conversation_id == c.id
                            && m.author == Author::Human
                            && !m.mine
                            && m.exec
                            && m.state == State::Sent
                    });
                    let Some(asked) = asked else {
                        return Ok(Response::error("unknown"));
                    };
                    let state = if ok.unwrap_or(false) {
                        State::Done
                    } else {
                        State::Error
                    };
                    let tx = an_core::db::write_tx(&db).map_err(db_err)?;
                    let r = chat::add_reply(
                        &tx,
                        c.id,
                        &msg,
                        &text,
                        state,
                        asked.id,
                        turns,
                        duration_ms,
                        model.as_deref(),
                        effort.as_deref(),
                    )
                    .map_err(db_err)?;
                    chat::observe(&tx, r.id, lc).map_err(db_err)?;
                    chat::set_ctx_tokens(&tx, r.id, ctx).map_err(db_err)?;
                    chat::set_state(&tx, asked.id, state).map_err(db_err)?;
                    tx.commit().map_err(db_err)?;
                }
                _ => return Ok(Response::error("bad_request")),
            }
            (c.id, c.agent)
        };
        let shown = if kind == MessageKind::RelayEnd {
            RELAY_END_CAP.to_string()
        } else {
            preview(&text)
        };
        self.notify(cid, peer, agent, kind, shown);
        Ok(Response::Ok)
    }

    /// 알림 이벤트(S9) — 상대 이름(이 PC 의 표시 이름이 있으면 그것) · PC 와 종류(거르기는 알림 쪽이 한다). 자동 왕복 시작 · 끝은
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
            | MessageKind::Share => None,
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
        });
    }

    /// 자동 왕복이 저절로 멈췄다 — 사유와 함께 알린다(사람이 멈춘 것은 부르지 않는다).
    fn relay_end(&self, conv: &Conversation, reason: &str) {
        if let Ok(peer) = self.paired_peer(conv.peer_id) {
            self.notify(
                conv.id,
                &peer,
                conv.agent.clone(),
                MessageKind::RelayEnd,
                reason.to_string(),
            );
        }
    }

    /// 대화 단위 차례 — 저장부터 상대 응답까지 한 번에 하나(사람 글 · 완료 표시).
    fn turn(&self, conversation_id: i64) -> Arc<tokio::sync::Mutex<()>> {
        an_core::sync::lock(&self.outbound)
            .entry(conversation_id)
            .or_default()
            .clone()
    }

    /// 받은 요청의 상태 · 메모를 바꾼다(#1d) — 받은 쪽 사람만, 대기 → 수용 · 반려 · 수용 → 완료만. 저장(`saved`)도 같은 트랜잭션.
    /// 상태와 보내기 행(`ReqState`)은 한 트랜잭션(1c) — 쓴 쪽이 꺼져 있으면 줄이 나중에 보낸다. 바뀐 글과 경고(상대가 거절했을 때만).
    pub async fn set_req_state(
        &self,
        id: i64,
        state: ReqState,
        note: &str,
        saved: Option<bool>,
    ) -> io::Result<(Message, Option<String>)> {
        let note = chat::clean_req_note(note)
            .map_err(|m| io::Error::new(io::ErrorKind::InvalidInput, m))?;
        let conv = {
            let db = self.db();
            let m = chat::message_by_id(&db, id)
                .map_err(db_err)?
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "없는 메시지"))?;
            if m.author != Author::Human || m.mine || !m.request {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "상태는 받은 요청만 바꾼다",
                ));
            }
            chat::by_id(&db, m.conversation_id)
                .map_err(db_err)?
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "없는 대화"))?
        };
        Self::writable(&conv)?;
        // 확인 · 쓰기는 대화 차례 안에서 — 두 창이 함께 바꿔도 옮김 규칙이 지켜지고 상대에 같은 순서로 간다.
        let lock = self.turn(conv.id);
        let _turn = lock.lock().await;
        let (m, req, staged) = {
            let db = self.db();
            let tx = an_core::db::write_tx(&db).map_err(db_err)?;
            let now = chat::message_by_id(&tx, id)
                .map_err(db_err)?
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "없는 메시지"))?;
            // 다시 알리기 — 요청자에 알리지 못한 상태(오래 닿지 않아 포기 · 요청자가 거절)는 같은 상태로 다시 보낼 수 있다(#1d R1 · R2).
            let again = now.req_state == Some(state) && now.refused.is_some();
            if !again && !now.req_state.unwrap_or(ReqState::Wait).can_become(state) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "대기 → 수용 · 반려, 수용 → 완료만",
                ));
            }
            chat::set_req_state(&tx, id, state, note.as_deref()).map_err(db_err)?;
            chat::set_refused(&tx, id, None).map_err(db_err)?;
            if let Some(on) = saved {
                chat::set_saved(&tx, id, on).map_err(db_err)?;
            }
            let req = Request::ReqState {
                conv: conv.uuid.clone(),
                msg: now.uuid.clone(),
                state: state.as_str().into(),
                note: note.clone(),
            };
            // 보내기 행은 그 요청 글에 잇는다 — 포기하면 그 글에 "요청자에게 알리지 못함"(`give_up`).
            let staged = Self::stage(&tx, &conv, Some(id), REQ_STATE, &req)?;
            tx.commit().map_err(db_err)?;
            let m = chat::message_by_id(&db, id).map_err(db_err)?.unwrap_or(now);
            (m, req, staged)
        };
        let warning = match self.dispatch(&conv, staged, &req).await {
            Ok(Shipped::Answered(Response::Ok) | Shipped::Kept) => None,
            Ok(Shipped::Answered(other)) => {
                // 거절은 그 글에 남긴다 — 새로 고친 뒤에도 "다시 알리기" 로 복구한다(#1d R2).
                self.req_state_refused(id, &other);
                Some(refused(other).to_string())
            }
            Err(e) => Some(e.to_string()),
        }
        .map(|e| format!("이 PC 에는 바꿨다 — 상대에 알리지 못했다({e})"));
        let m = chat::message_by_id(&self.db(), id)
            .map_err(db_err)?
            .unwrap_or(m);
        Ok((m, warning))
    }

    /// 요청자가 상태 알림을 거절했다 — 그 받은 요청 글에 거절 코드(화면 "다시 알리기").
    fn req_state_refused(&self, msg_id: i64, resp: &Response) {
        let code = match resp {
            Response::Error { code } => an_net::proto::clean_code(code),
            _ => "unknown",
        };
        let _ = chat::set_refused(&self.db(), msg_id, Some(code));
    }

    /// 상대가 내 요청에 단 상태(#1d) — 그 상대 소유 대화의 내 요청 사람 글에만(받은 쪽이 정한다 — 옮김 검사 없이 받은 대로, 줄이 순서를
    /// 지킨다).
    fn inbound_req_state(
        &self,
        peer: &Peer,
        conv: String,
        msg: String,
        state: String,
        note: Option<String>,
    ) -> io::Result<Response> {
        let (Some(state), Ok(note)) = (
            ReqState::parse(&state),
            chat::clean_req_note(note.as_deref().unwrap_or("")),
        ) else {
            return Ok(Response::error("bad_request"));
        };
        let (cid, agent, text) = {
            let db = self.db();
            let Some(c) = chat::find_own(&db, &conv, peer.id).map_err(db_err)? else {
                return Ok(Response::error("unknown"));
            };
            let mine = chat::message_by_uuid(&db, &msg)
                .map_err(db_err)?
                .filter(|m| {
                    m.conversation_id == c.id && m.author == Author::Human && m.mine && m.request
                });
            let Some(m) = mine else {
                return Ok(Response::error("unknown"));
            };
            // 같은 상태 · 메모의 재전송(응답 유실 · 다시 알리기)은 시각 · 알림을 다시 만들지 않는다(#1d R2).
            if m.req_state == Some(state) && m.req_note == note {
                return Ok(Response::Ok);
            }
            // 상태와 내 활동 기록(#1q — 안 본 것)을 한 트랜잭션으로 — 기록만 빠지면 재전송이 "같은 상태" 로 일찍 돌아가 영영 남지 않는다.
            let tx = an_core::db::write_tx(&db).map_err(db_err)?;
            chat::set_req_state(&tx, m.id, state, note.as_deref()).map_err(db_err)?;
            chat::note_req_state(&tx, m.id).map_err(db_err)?;
            tx.commit().map_err(db_err)?;
            (c.id, c.agent, note)
        };
        let label = match state {
            ReqState::Wait => "요청 대기",
            ReqState::Accepted => "요청 수용",
            ReqState::Rejected => "요청 반려",
            ReqState::Done => "요청 완료",
        };
        let shown = text.map_or(label.to_string(), |n| format!("{label} — {n}"));
        self.notify(cid, peer, agent, MessageKind::AskState, preview(&shown));
        Ok(Response::Ok)
    }

    /// 편집 · 삭제할 수 있는 글인가(#1f) — 내 사람 글 ∧ 끝남(`done` · `error` · `undelivered`) ∧ 그 글을 실어 나르는 보관 행이 기다리지 않음.
    /// `conn` 은 부르는 쪽 연결(트랜잭션일 수 있다).
    pub fn tidy_ok(conn: &Connection, m: &Message) -> io::Result<bool> {
        if m.author != Author::Human
            || !m.mine
            || !matches!(m.state, State::Done | State::Error | State::Undelivered)
        {
            return Ok(false);
        }
        let carrying = outbox::for_message(conn, m.id).map_err(db_err)?;
        Ok(carrying.is_none_or(|o| o.state == "failed"))
    }

    /// 고를 글들 — 모두 있고 한 대화이며 편집 · 삭제할 수 있어야 한다(아니면 `InvalidInput` · `NotFound`). 그 대화.
    fn tidy_targets(conn: &Connection, ids: &[i64]) -> io::Result<(Conversation, Vec<Message>)> {
        let mut out = Vec::with_capacity(ids.len());
        for &id in ids {
            let m = chat::message_by_id(conn, id)
                .map_err(db_err)?
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "없는 메시지"))?;
            if out
                .first()
                .is_some_and(|f: &Message| f.conversation_id != m.conversation_id)
            {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "한 대화의 글만 고른다",
                ));
            }
            if !Self::tidy_ok(conn, &m)? {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "편집 · 삭제는 끝난 내 사람 글만",
                ));
            }
            out.push(m);
        }
        let first = out
            .first()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "고른 글이 없다"))?;
        let conv = chat::by_id(conn, first.conversation_id)
            .map_err(db_err)?
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "없는 대화"))?;
        Ok((conv, out))
    }

    /// 내 사람 글 본문 편집(#1f — 이력 정리, 실행 · 알림 없음). `mirror` 면 같은 트랜잭션에 상대 반영 행(`Edit`)을 적고 커밋 뒤 보낸다.
    /// 바뀐 글과 경고(상대 창에 반영하지 못했을 때).
    pub async fn edit_message(
        &self,
        id: i64,
        text: &str,
        mirror: bool,
    ) -> io::Result<(Message, Option<String>)> {
        if text.trim().is_empty() || text.len() > MAX_TEXT {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("본문은 1 ~ {MAX_TEXT} 바이트"),
            ));
        }
        let conv_id = chat::message_by_id(&self.db(), id)
            .map_err(db_err)?
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "없는 메시지"))?
            .conversation_id;
        // 확인 · 쓰기 · 반영은 대화 차례 안에서 — 상대에 쓴 순서대로 간다.
        let lock = self.turn(conv_id);
        let _turn = lock.lock().await;
        let (conv, m, staged, viewers) = {
            let db = self.db();
            let tx = an_core::db::write_tx(&db).map_err(db_err)?;
            let (conv, mut ms) = Self::tidy_targets(&tx, &[id])?;
            let m = ms.remove(0);
            chat::edit_text(&tx, id, text).map_err(db_err)?;
            // 이미 관람자에게 간 글이면 고친 글을 함께(#1o).
            let edited = chat::message_by_id(&tx, id)
                .map_err(db_err)?
                .unwrap_or_else(|| m.clone());
            let viewers = Self::share_change(&tx, &conv, &edited, false)?;
            let staged = if mirror {
                let req = Request::Edit {
                    conv: conv.uuid.clone(),
                    msg: m.uuid.clone(),
                    text: text.to_string(),
                };
                vec![(Self::stage(&tx, &conv, Some(id), outbox::EDIT, &req)?, req)]
            } else {
                Vec::new()
            };
            tx.commit().map_err(db_err)?;
            let m = chat::message_by_id(&db, id).map_err(db_err)?.unwrap_or(m);
            (conv, m, staged, viewers)
        };
        for p in viewers {
            self.kick(p);
        }
        let warning = self.mirror_tidy(&conv, staged).await;
        Ok((m, warning))
    }

    /// 내 사람 글 여러 개를 흔적 없이 지운다(#1f — 이력 정리). 한 대화의 글만, 하나라도 지울 수 없으면 아무것도 지우지 않는다. `mirror` 면
    /// 글마다 상대 반영 행(`Delete`)을 같은 트랜잭션에 적고 커밋 뒤 보낸다. 지운 수와 경고.
    pub async fn delete_messages(
        &self,
        ids: &[i64],
        mirror: bool,
    ) -> io::Result<(usize, Option<String>)> {
        let mut ids = ids.to_vec();
        ids.sort_unstable();
        ids.dedup();
        let first = *ids
            .first()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "고른 글이 없다"))?;
        let conv_id = chat::message_by_id(&self.db(), first)
            .map_err(db_err)?
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "없는 메시지"))?
            .conversation_id;
        let lock = self.turn(conv_id);
        let _turn = lock.lock().await;
        let (conv, staged, viewers) = {
            let db = self.db();
            let tx = an_core::db::write_tx(&db).map_err(db_err)?;
            let (conv, ms) = Self::tidy_targets(&tx, &ids)?;
            let mut staged = Vec::new();
            let mut viewers = Vec::new();
            for m in &ms {
                viewers.extend(Self::share_change(&tx, &conv, m, true)?);
                if mirror {
                    let req = Request::Delete {
                        conv: conv.uuid.clone(),
                        msg: m.uuid.clone(),
                    };
                    staged.push((
                        Self::stage(&tx, &conv, Some(m.id), outbox::DELETE, &req)?,
                        req,
                    ));
                }
                chat::delete_message(&tx, m.id).map_err(db_err)?;
            }
            tx.commit().map_err(db_err)?;
            (conv, staged, viewers)
        };
        for p in viewers {
            self.kick(p);
        }
        let warning = self.mirror_tidy(&conv, staged).await;
        Ok((ids.len(), warning))
    }

    /// 적어 둔 편집 · 삭제 반영 행을 차례로 보낸다 — 상대가 거절하면 내 활동 '상대 반영 실패' + 경고. 닿지 않으면 줄이 다시 보낸다(경고 없음).
    async fn mirror_tidy(
        &self,
        conv: &Conversation,
        staged: Vec<(i64, Request)>,
    ) -> Option<String> {
        let mut warning = None;
        for (id, req) in staged {
            let got = match self.dispatch(conv, id, &req).await {
                Ok(Shipped::Answered(Response::Ok) | Shipped::Kept) => None,
                // '상대 반영 실패' 기록은 응답 정산(`acked`)이 행 지우기와 함께 했다.
                Ok(Shipped::Answered(other)) => Some(refused(other).to_string()),
                Err(e) => Some(e.to_string()),
            };
            warning = warning.or(got);
        }
        warning.map(|e| format!("이 PC 에는 반영했다 — 상대 창에 반영하지 못했다({e})"))
    }

    /// 상대 반영 실패(#1f) — 편집 · 삭제 반영 행의 대화에 내 활동 기록(`sync_failed`, 사유 "편집 — {미리보기}" · "삭제"). `conn` 은 행을 정리하는
    /// 트랜잭션 — 기록과 행 정리가 함께 남거나 함께 없다.
    fn note_sync_failed(conn: &Connection, item: &outbox::Item) -> io::Result<()> {
        let conv = item
            .conv_id
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "반영 행에 대화가 없다"))?;
        // 편집 실패는 그 글에 잇는다 — 나중에 글을 지우면 본문 미리보기도 함께 사라진다(완전 삭제, 적대적 평가 라운드 2). 삭제 실패는 본문이 없고
        // 글 id 도 이미 없다.
        let (what, message) = match serde_json::from_str::<Request>(&item.frame) {
            Ok(Request::Edit { text, .. }) => (format!("편집 — {}", preview(&text)), item.msg_id),
            _ => ("삭제".to_string(), None),
        };
        chat::add_relay_event(conn, conv, chat::RelayMark::SyncFailed(message), &what)
            .map_err(db_err)
    }

    /// 상대가 쓴 사람 글(#1f — 그 상대 소유 대화의 `!mine` 사람 글). 아니면 `None`.
    fn theirs(
        conn: &Connection,
        peer: &Peer,
        conv: &str,
        msg: &str,
    ) -> io::Result<Option<Message>> {
        let Some(c) = chat::find_own(conn, conv, peer.id).map_err(db_err)? else {
            return Ok(None);
        };
        Ok(chat::message_by_uuid(conn, msg)
            .map_err(db_err)?
            .filter(|m| m.conversation_id == c.id && m.author == Author::Human && !m.mine))
    }

    /// 상대가 자기 사람 글을 편집했다(#1f) — 그 글에만, 실행 · 알림 없이. 실행 대기 · 실행 중이면 `busy`. 같은 본문 재전송은 `Ok`.
    fn inbound_edit(
        &self,
        peer: &Peer,
        conv: String,
        msg: String,
        text: String,
    ) -> io::Result<Response> {
        if text.trim().is_empty() || text.len() > MAX_TEXT {
            return Ok(Response::error("bad_request"));
        }
        let viewers = {
            let db = self.db();
            let Some(m) = Self::theirs(&db, peer, &conv, &msg)? else {
                return Ok(Response::error("unknown"));
            };
            if matches!(m.state, State::Queued | State::Running) {
                return Ok(Response::error("busy"));
            }
            if m.text == text && m.edited_at.is_some() {
                return Ok(Response::Ok);
            }
            // 고친 글과 관람자 행(#1o)은 한 트랜잭션.
            let tx = an_core::db::write_tx(&db).map_err(db_err)?;
            chat::edit_text(&tx, m.id, &text).map_err(db_err)?;
            let edited = chat::message_by_id(&tx, m.id).map_err(db_err)?.unwrap_or(m);
            let viewers = match chat::by_id(&tx, edited.conversation_id).map_err(db_err)? {
                Some(c) => Self::share_change(&tx, &c, &edited, false)?,
                None => Vec::new(),
            };
            tx.commit().map_err(db_err)?;
            viewers
        };
        for p in viewers {
            self.kick(p);
        }
        Ok(Response::Ok)
    }

    /// 상대가 자기 사람 글을 지웠다(#1f) — 그 글만 흔적 없이, 실행 · 알림 없이. 이미 없으면 `Ok`(재전송), 실행 대기 · 실행 중이면 `busy`.
    fn inbound_delete(&self, peer: &Peer, conv: String, msg: String) -> io::Result<Response> {
        let viewers = {
            let db = self.db();
            let Some(c) = chat::find_own(&db, &conv, peer.id).map_err(db_err)? else {
                return Ok(Response::error("unknown"));
            };
            if chat::message_by_uuid(&db, &msg).map_err(db_err)?.is_none() {
                return Ok(Response::Ok);
            }
            let Some(m) = Self::theirs(&db, peer, &conv, &msg)? else {
                return Ok(Response::error("unknown"));
            };
            if matches!(m.state, State::Queued | State::Running) {
                return Ok(Response::error("busy"));
            }
            let tx = an_core::db::write_tx(&db).map_err(db_err)?;
            let viewers = Self::share_change(&tx, &c, &m, true)?;
            chat::delete_message(&tx, m.id).map_err(db_err)?;
            tx.commit().map_err(db_err)?;
            viewers
        };
        for p in viewers {
            self.kick(p);
        }
        Ok(Response::Ok)
    }

    /// 받은 요청(#1d) — 실행 없이 기록만(대기열 · 실행 기록 없음), 상태 '대기'. 같은 uuid 재전송은 `Ok`(한 번만).
    fn inbound_ask(
        &self,
        peer: &Peer,
        conv: &str,
        msg: &str,
        agent: String,
        text: &str,
        (lc, via): (Option<i64>, Option<String>),
    ) -> io::Result<Response> {
        let cid = {
            let db = self.db();
            if agents::by_name(&db, &agent).map_err(db_err)?.is_none() {
                return Ok(Response::error("unknown_agent"));
            }
            let Ok(c) = chat::open_conversation(&db, conv, peer.id, &agent, Dir::In) else {
                return Ok(Response::error("bad_request")); // 다른 상대의 대화 uuid
            };
            if c.dir != Dir::In || c.agent != agent {
                return Ok(Response::error("bad_request"));
            }
            if chat::message_by_uuid(&db, msg).map_err(db_err)?.is_some() {
                return Ok(Response::Ok);
            }
            let tx = an_core::db::write_tx(&db).map_err(db_err)?;
            let m =
                chat::add_human(&tx, c.id, msg, text, State::Done, false, false).map_err(db_err)?;
            chat::set_via(&tx, m.id, via.as_deref()).map_err(db_err)?;
            chat::set_request(&tx, m.id, true).map_err(db_err)?;
            chat::observe(&tx, m.id, lc).map_err(db_err)?;
            tx.commit().map_err(db_err)?;
            c.id
        };
        self.notify(cid, peer, agent, MessageKind::Ask, preview(text));
        Ok(Response::Ok)
    }

    /// 상대가 알려 준 내 실행 요청의 차례 — 그 상대 소유 대화의 내 실행 요청 글 · 아직 답을 기다리는 것만(1b).
    fn inbound_progress(
        &self,
        peer: &Peer,
        conv: String,
        msg: String,
        state: String,
        (ahead, detail): (u32, Option<StepDetail>),
    ) -> io::Result<Response> {
        if !matches!(state.as_str(), "queued" | "running") {
            return Ok(Response::error("bad_request"));
        }
        let db = self.db();
        let Some(c) = chat::find_own(&db, &conv, peer.id).map_err(db_err)? else {
            return Ok(Response::error("unknown"));
        };
        let mine = chat::message_by_uuid(&db, &msg)
            .map_err(db_err)?
            .filter(|m| m.conversation_id == c.id && m.mine && m.exec && m.state == State::Sent);
        let Some(m) = mine else {
            return Ok(Response::error("unknown"));
        };
        let running = state == "running";
        let progress = Progress {
            ahead: if running { 0 } else { ahead },
            detail: detail.filter(|_| running).map(StepDetail::cleaned),
            state,
        };
        an_core::sync::lock(&self.progress).insert(m.id, progress);
        Ok(Response::Ok)
    }

    /// 자동 왕복 알림 — 그 상대 소유 대화만. 연 쪽은 `on = false` 만(받은 쪽 멈춤), 받은 쪽은 거절 켜기 · 끄기.
    fn inbound_relay(&self, peer: &Peer, conv: String, on: bool) -> io::Result<Response> {
        let db = self.db();
        let Some(c) = chat::find_own(&db, &conv, peer.id).map_err(db_err)? else {
            return Ok(Response::error("unknown"));
        };
        let event = match (c.dir, on) {
            (Dir::Shared, _) => return Ok(Response::error("unknown")),
            (Dir::Out, false) => {
                chat::relay_on(&db, c.id, false, false).map_err(db_err)?;
                // 켜져 있던 자동 왕복을 상대가 멈췄다.
                c.relay
                    .as_ref()
                    .filter(|r| r.on)
                    .map(|_| (MessageKind::RelayEnd, "상대가 멈춤"))
            }
            (Dir::Out, true) => return Ok(Response::error("bad_request")),
            (Dir::In, on) => {
                chat::set_blocked(&db, c.id, !on).map_err(db_err)?;
                on.then_some((MessageKind::RelayStart, "상대가 자동 왕복을 재개"))
            }
        };
        drop(db);
        if let Some((kind, text)) = event {
            self.notify(c.id, peer, c.agent, kind, text.to_string());
        }
        Ok(Response::Ok)
    }

    /// 받은 쪽 멈춤 — 에이전트발 요청을 거절하고 연 쪽에 알린다(알림 실패는 거절로 충분).
    async fn block_relay(&self, conv: &Conversation, peer: &Peer) {
        let _ = chat::set_blocked(&self.db(), conv.id, true);
        let req = Request::Relay {
            conv: conv.uuid.clone(),
            on: false,
        };
        let _ = peer;
        let _ = self.ship(conv, None, RELAY, &req).await;
    }

    /// 자동 왕복 멈춤 · 재개(D24). 연 쪽: 끄기 / 켜기(카운트 0, 받은 쪽 거절 해제, 아직 넘기지 않은 상대 답이 있으면 바로 넘김).
    /// 받은 쪽: 끄기만(거절 + 알림).
    pub async fn relay(
        self: &Arc<Self>,
        conversation_id: i64,
        on: bool,
    ) -> io::Result<Conversation> {
        let conv = chat::by_id(&self.db(), conversation_id)
            .map_err(db_err)?
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "없는 대화"))?;
        let peer = self.paired_peer(conv.peer_id)?;
        match (conv.dir, on) {
            (Dir::Shared, _) => {
                return Err(io::Error::new(io::ErrorKind::InvalidInput, READ_ONLY));
            }
            (Dir::In, false) => self.block_relay(&conv, &peer).await,
            (Dir::In, true) => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "재개는 대화를 연 쪽이 한다",
                ));
            }
            (Dir::Out, _) if conv.relay.is_none() => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "자동 왕복 대화가 아니다",
                ));
            }
            (Dir::Out, false) => {
                chat::relay_on(&self.db(), conv.id, false, false).map_err(db_err)?;
            }
            (Dir::Out, true) => {
                let req = Request::Relay {
                    conv: conv.uuid.clone(),
                    on: true,
                };
                match self
                    .node
                    .send(addr(&peer)?, Some(peer.spki.clone()), &req)
                    .await
                {
                    Ok((Response::Ok, _)) => {}
                    Ok((other, _)) => return Err(refused(other)),
                    Err(e) => return Err(e),
                }
                let (queued, resend) = {
                    let db = self.db();
                    let tx = an_core::db::write_tx(&db).map_err(db_err)?;
                    chat::relay_on(&tx, conv.id, true, true).map_err(db_err)?;
                    let last = chat::messages(&tx, conv.id)
                        .map_err(db_err)?
                        .pop()
                        .filter(|m| m.author == Author::Agent);
                    // 아직 내 에이전트에 넘기지 않은 상대 답.
                    let queued = last
                        .as_ref()
                        .filter(|m| !m.mine && !m.exec && m.state == State::Done);
                    if let Some(m) = queued {
                        chat::set_flags(&tx, m.id, false, true, State::Queued).map_err(db_err)?;
                    }
                    tx.commit().map_err(db_err)?;
                    // 상대에 넘기지 못한 내 답 — 멈춤 · 상한으로 기록만(`done` · `undelivered`) 또는 거절 · 전송 오류(`error` · `sent`).
                    let resend = last.clone().filter(|m| {
                        m.mine
                            && if m.exec {
                                matches!(m.state, State::Error | State::Sent)
                            } else {
                                matches!(m.state, State::Done | State::Undelivered)
                            }
                    });
                    (queued.is_some(), resend)
                };
                if queued {
                    self.wake(conv.id);
                }
                if let Some(m) = resend {
                    self.relay_forward(&conv, Some(m.id), true, &m.text, Some(m.lc), None)
                        .await;
                }
            }
        }
        chat::by_id(&self.db(), conversation_id)
            .map_err(db_err)?
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "없는 대화"))
    }

    fn inbound_reply(self: &Arc<Self>, peer: &Peer, reply: Request) -> io::Result<Response> {
        let mut relay_next = None;
        let mut kind = MessageKind::Reply;
        let mut ended = None;
        let Request::Reply {
            conv,
            msg,
            reply_to,
            text,
            ok,
            turns,
            duration_ms,
            lc,
            model,
            effort,
            ctx,
        } = reply
        else {
            return Ok(Response::error("bad_request"));
        };
        let ctx = ctx.filter(|v| (0..=MAX_CTX_TOKENS).contains(v));
        let (model, effort) = run_meta(model, effort);
        if text.len() > MAX_TEXT || msg.is_empty() || text.trim().is_empty() {
            return Ok(Response::error("bad_request"));
        }
        let (cid, agent) = {
            let db = self.db();
            let Some(c) = chat::find_own(&db, &conv, peer.id)
                .map_err(db_err)?
                .filter(|c| c.dir == Dir::Out)
            else {
                return Ok(Response::error("unknown"));
            };
            if chat::message_by_uuid(&db, &msg).map_err(db_err)?.is_some() {
                return Ok(Response::Ok); // 같은 답 재전송
            }
            // 요청 없이 온 답 · 이미 답이 온 메시지에 또 온 답은 받지 않는다.
            let asked = chat::message_by_uuid(&db, &reply_to)
                .map_err(db_err)?
                .filter(|m| m.conversation_id == c.id && m.exec && m.state == State::Sent);
            let Some(asked) = asked else {
                return Ok(Response::error("unknown"));
            };
            if asked.author == Author::Agent {
                kind = MessageKind::RelayStep; // 자동 왕복으로 넘긴 내 에이전트 답에 온 답
            } else if c.relay.as_ref().is_some_and(|r| r.on) {
                kind = MessageKind::RelayStart; // 자동 왕복을 연 사람 요청에 온 첫 답
            }
            let state = if ok { State::Done } else { State::Error };
            let tx = an_core::db::write_tx(&db).map_err(db_err)?;
            let reply = chat::add_reply(
                &tx,
                c.id,
                &msg,
                &text,
                state,
                asked.id,
                turns,
                duration_ms,
                model.as_deref(),
                effort.as_deref(),
            )
            .map_err(db_err)?;
            chat::observe(&tx, reply.id, lc).map_err(db_err)?;
            chat::set_ctx_tokens(&tx, reply.id, ctx).map_err(db_err)?;
            chat::set_state(&tx, asked.id, state).map_err(db_err)?;
            // 멘션한 요청의 답 — 실행한 에이전트는 요청이 부른 이름 그대로(해석 4).
            if asked.agent.is_some() {
                chat::set_agent(&tx, reply.id, asked.agent.as_deref()).map_err(db_err)?;
            }
            // 자동 왕복 중이고 성공한 답이며 상한 전 · 집중 아님이면 이 답을 내 에이전트 입력으로(연 쪽 구동, D24). 답 기록과 한 트랜잭션 —
            // 사이에 끊겨도 "켜짐인데 대기열 없음" 이 남지 않는다.
            let focused = self.focused(&tx);
            match c.relay.as_ref().filter(|r| r.on) {
                Some(r) if ok && r.count < r.cap && !focused => {
                    chat::set_flags(&tx, reply.id, false, true, State::Queued).map_err(db_err)?;
                    relay_next = Some(c.id);
                }
                // 상대 실행 실패 · 상한 · 집중 — 멈추고 사람 입력을 기다린다.
                Some(r) => {
                    chat::relay_on(&tx, c.id, false, false).map_err(db_err)?;
                    ended = Some(if !ok {
                        "상대 에이전트 실행 실패"
                    } else if r.count >= r.cap {
                        RELAY_END_CAP
                    } else {
                        "집중 중"
                    });
                }
                None => {}
            }
            tx.commit().map_err(db_err)?;
            (c.id, asked.agent.clone().unwrap_or(c.agent))
        };
        if let Some(id) = relay_next {
            self.wake(id);
        }
        self.notify(cid, peer, agent.clone(), kind, preview(&text));
        if let Some(reason) = ended {
            self.notify(cid, peer, agent, MessageKind::RelayEnd, reason.to_string());
        }
        Ok(Response::Ok)
    }

    /// 대화 작업을 깨운다 — 없으면 만든다(대화 안에서는 DB 순서대로 하나씩).
    fn wake(self: &Arc<Self>, conversation_id: i64) {
        let mut queues = an_core::sync::lock(&self.queues);
        let tx = queues.entry(conversation_id).or_insert_with(|| {
            let (tx, mut rx) = mpsc::unbounded_channel::<()>();
            let me = self.clone();
            tokio::spawn(async move {
                // 한 번 꺼낸 메시지는 다시 꺼내지 않는다 — 상태 저장이 실패해도 같은 메시지를 되풀이하지 않게. 커서(가장 큰 id)가
                // 아니라 꺼낸 것의 집합 — 낮은 id 가 늦게 `queued` 가 돼도 빠지지 않는다.
                let mut taken = std::collections::HashSet::new();
                while rx.recv().await.is_some() {
                    loop {
                        let next = chat::pending(&me.db(), conversation_id)
                            .map(|ms| ms.into_iter().find(|m| !taken.contains(&m.id)));
                        match next {
                            Ok(Some(m)) => {
                                taken.insert(m.id);
                                me.process(conversation_id, m).await;
                            }
                            Ok(None) => break,
                            Err(e) => {
                                log::error!("대화 {conversation_id} 대기열 읽기 실패: {e}");
                                break;
                            }
                        }
                    }
                }
            });
            tx
        });
        if tx.send(()).is_err() {
            log::error!("대화 {conversation_id} 작업이 멈췄다");
        }
    }

    /// 사람 메시지 하나 — 실행 → 기록(한 트랜잭션) → 답 보내기. 기록이 실패하면 성공 답을 보내지 않는다.
    async fn process(self: &Arc<Self>, conversation_id: i64, asked: Message) {
        if asked.author == Author::Context {
            return self.run_context(conversation_id, asked).await;
        }
        // 실행 문 — 먼저 줄을 서고, 차례가 온 뒤의 대화 · 에이전트 · 멈춤 상태로 판단한다(기다리는 동안 바뀐 것을 따른다).
        let ticket = self.admit(conversation_id, &asked).await;
        let loaded = {
            let db = self.db();
            chat::by_id(&db, conversation_id).and_then(|c| {
                let agent = match &c {
                    // 멘션이면 부른 에이전트(해석 3).
                    Some(c) => agents::by_name(&db, asked.agent.as_deref().unwrap_or(&c.agent))?,
                    None => None,
                };
                Ok(c.map(|c| (c, agent)))
            })
        };
        let (conv, agent) = match loaded {
            Ok(Some(found)) => found,
            Ok(None) => return log::error!("대화 {conversation_id} 를 찾지 못했다"),
            Err(e) => return log::error!("대화 {conversation_id} 읽기 실패: {e}"),
        };
        // 연 쪽에서 실행 요청이 된 상대 에이전트 답 = 자동 왕복 — 실행 에이전트는 내 에이전트.
        let relaying = conv.dir == Dir::Out && asked.author == Author::Agent;
        // 그사이 멈췄으면(사람이 씀 · 멈춤) 넘기지 않은 상대 답으로 되돌린다 — 재개 때 다시 넘긴다.
        if relaying && !conv.relay.as_ref().is_some_and(|r| r.on) {
            let _ = chat::set_flags(&self.db(), asked.id, false, false, State::Done);
            return;
        }
        let agent = if relaying {
            let _ = chat::relay_step(&self.db(), conv.id);
            conv.relay
                .as_ref()
                .and_then(|r| agents::by_name(&self.db(), &r.agent).ok().flatten())
        } else {
            agent
        };
        // 실패는 (내 쪽 글, 상대에게 보낼 글) 둘 — 중단 · 에이전트 삭제는 상대도 알아도 되는 사실이라 같은 글.
        let result = if asked.state == State::Running {
            let t = format!("실행 실패: {INTERRUPTED}");
            Err((t.clone(), t))
        } else if let Some(agent) = agent {
            self.execute(&conv, agent, &asked)
                .await
                .map_err(|f| (format!("실행 실패: {}", f.detail), f.public.to_string()))
        } else {
            let t = format!(
                "실행 실패: 에이전트 {} 가 삭제됐다",
                asked.agent.as_deref().unwrap_or(&conv.agent)
            );
            Err((t.clone(), t))
        };
        drop(ticket);
        let (text, public, ok, run) = match result {
            // 빈 답은 실패로 — 받는 쪽이 받지 않는다.
            Ok(ran) if ran.out.text.trim().is_empty() => (
                EMPTY_ANSWER.to_string(),
                Some(EMPTY_ANSWER.to_string()),
                false,
                Some(ran),
            ),
            // claude 가 오류로 끝난 결과(`is_error`)는 원문을 내 쪽에만 — 인증 · 사용량 · 경로가 실릴 수 있다(1a R1).
            Ok(ran) if ran.out.is_error => (
                ran.out.text.clone(),
                Some(RESULT_ERROR.to_string()),
                false,
                Some(ran),
            ),
            Ok(ran) => (ran.out.text.clone(), None, true, Some(ran)),
            Err((mine, theirs)) => (mine, Some(theirs), false, None),
        };
        let mut text = fit(text);
        let mut public = public;
        let mut ok = ok;
        let reply_uuid = new_uuid();
        let state = if ok { State::Done } else { State::Error };
        let (turns, duration_ms) = (
            run.as_ref().and_then(|r| r.out.turns),
            run.as_ref().and_then(|r| r.out.duration_ms),
        );
        let (model, effort) = (
            run.as_ref().map(|r| r.model.clone()),
            run.as_ref().map(|r| r.effort.clone()),
        );
        let ctx = run.as_ref().and_then(|r| r.out.ctx_tokens);
        // 상대의 실행 요청이면 답(`reply`), 이 PC 사람의 실행 요청이면 상대 이력에 미러(`post`).
        let reply_req = |sent: String, lc: Option<i64>, ok: bool| {
            if asked.mine {
                Request::Post {
                    conv: conv.uuid.clone(),
                    msg: reply_uuid.clone(),
                    author: "agent".into(),
                    text: sent,
                    exec: false,
                    reply_to: Some(asked.uuid.clone()),
                    ok: Some(ok),
                    turns,
                    duration_ms,
                    lc,
                    via: None,
                    model: model.clone(),
                    effort: effort.clone(),
                    request: false,
                    ctx,
                }
            } else {
                Request::Reply {
                    conv: conv.uuid.clone(),
                    msg: reply_uuid.clone(),
                    reply_to: asked.uuid.clone(),
                    text: sent,
                    ok,
                    turns,
                    duration_ms,
                    lc,
                    model: model.clone(),
                    effort: effort.clone(),
                    ctx,
                }
            }
        };
        let kind = if asked.mine { REPLY_POST } else { REPLY };
        // 답과 보내기 행을 한 트랜잭션으로(#1c — 기록 뒤 보내기 전에 꺼져도 줄이 보낸다). 자동 왕복 넘김은 `relay_forward` 가 보낸다.
        let outgoing = public.clone().unwrap_or_else(|| text.clone());
        let staging =
            (!relaying).then_some((kind, |lc: i64| reply_req(outgoing.clone(), Some(lc), ok)));
        let recorded = self.record(
            &conv,
            &asked,
            run.as_ref(),
            &reply_uuid,
            (&text, state),
            staging
                .as_ref()
                .map(|(k, f)| (*k, f as &dyn Fn(i64) -> Request)),
        );
        let (recorded, staged) = match recorded {
            Ok((id, staged)) => (Ok(id), staged),
            Err(e) => (Err(e), None),
        };
        // 자동 압축(#1m) — 이 PC 에서 돈 세션이 문턱을 넘었다. 구분선 글은 대화 줄 맨 앞에 서서 다음 글보다 먼저 처리된다.
        if recorded.is_ok() && ok && ctx.is_some_and(|n| n >= an_core::settings::AUTO_COMPACT_AT) {
            let on = an_core::settings::auto_compact(&self.db()).unwrap_or_else(|e| {
                log::warn!("자동 압축 설정 읽기 실패 — 켜짐으로 본다: {e}");
                true
            });
            if on && let Err(e) = self.request_context(&conv, "auto_compact") {
                log::warn!("대화 {} 자동 압축 요청 실패: {e}", conv.uuid);
            }
        }
        if let Err(e) = &recorded {
            log::error!("대화 {} 기록 실패: {e}", conv.uuid);
            let _ = chat::set_state(&self.db(), asked.id, State::Error);
            (text, ok) = (format!("실행 실패: 받는 PC 기록 오류 — {e}"), false);
            public = Some(RECORD_FAILED.to_string());
        }
        // 상대에게 가는 글 — 실패면 정해진 문구.
        let sent = public.unwrap_or_else(|| text.clone());
        let lc = recorded
            .as_ref()
            .ok()
            .and_then(|id| {
                chat::messages(&self.db(), conv.id)
                    .ok()?
                    .into_iter()
                    .find(|m| m.id == *id)
            })
            .map(|m| m.lc);
        if relaying {
            // 내 에이전트 답 — 이 PC 글로.
            if let Ok(id) = recorded {
                let _ = chat::set_flags(&self.db(), id, true, false, state);
            }
            return self
                .relay_forward(&conv, recorded.ok(), ok, &sent, lc, Some(&asked.uuid))
                .await;
        }
        // 받은 쪽: 자동 왕복 요청(에이전트발 · 자동 왕복을 연 첫 사람 요청)을 내 에이전트가 실패하면 자동 왕복이 끝난다.
        let relay_request = conv.dir == Dir::In
            && (asked.author == Author::Agent
                || (conv.relay.is_some()
                    && !asked.mine
                    && matches!(
                        chat::last_exec_before(&self.db(), conv.id, &asked),
                        Ok(None)
                    )));
        if !ok && relay_request {
            self.relay_end(&conv, "내 에이전트 실행 실패");
        }
        // 닿지 않으면 보관 — 줄이 다시 보낸다(#1c). 상대가 거절했을 때만 지금 "답 미전달". 기록이 실패했으면(행 없음) 정해진 문구로 지금 적고 보낸다.
        let req = reply_req(sent, lc, ok);
        let shipped = match staged {
            Some(id) => self.dispatch(&conv, id, &req).await,
            None => {
                self.ship(&conv, recorded.as_ref().ok().copied(), kind, &req)
                    .await
            }
        };
        let delivered = matches!(shipped, Ok(Shipped::Answered(Response::Ok) | Shipped::Kept));
        if !delivered {
            log::warn!("대화 {} 답을 전달하지 못했다", conv.uuid);
            if let Ok(id) = recorded {
                let _ = chat::set_state(&self.db(), id, State::Undelivered);
            }
        }
    }

    /// 자동 왕복 — 내 에이전트 답을 상대 에이전트 실행 요청으로 보낸다. 실패 · 멈춤 · 거절이면 자동 왕복을 끈다.
    async fn relay_forward(
        &self,
        conv: &Conversation,
        reply_id: Option<i64>,
        ok: bool,
        text: &str,
        lc: Option<i64>,
        reply_to: Option<&str>,
    ) {
        let relay = chat::by_id(&self.db(), conv.id)
            .ok()
            .flatten()
            .and_then(|c| c.relay);
        let still_on = relay.as_ref().is_some_and(|r| r.on);
        let (Some(reply_id), true, true) = (reply_id, ok, still_on) else {
            let _ = chat::relay_on(&self.db(), conv.id, false, false);
            // 사람이 멈춘 것(`still_on` 거짓)은 알리지 않는다.
            if still_on {
                self.relay_end(
                    conv,
                    if ok {
                        "기록 실패"
                    } else {
                        "내 에이전트 실행 실패"
                    },
                );
            }
            return;
        };
        let Some(msg) = chat::messages(&self.db(), conv.id)
            .ok()
            .and_then(|ms| ms.into_iter().find(|m| m.id == reply_id))
        else {
            return;
        };
        // 상한 — 이 답으로 왕복을 채웠다. 상대 에이전트를 더 돌리지 않고 상대 이력에 기록만 남긴 뒤 멈춘다.
        if relay.as_ref().is_some_and(|r| r.count >= r.cap) {
            let _ = chat::relay_on(&self.db(), conv.id, false, false);
            self.relay_end(conv, RELAY_END_CAP);
            let req = Request::Post {
                conv: conv.uuid.clone(),
                msg: msg.uuid.clone(),
                author: "agent".into(),
                text: text.to_string(),
                exec: false,
                reply_to: reply_to.map(str::to_string),
                ok: Some(true),
                turns: msg.turns,
                duration_ms: msg.duration_ms,
                lc,
                via: None,
                model: msg.model.clone(),
                effort: msg.effort.clone(),
                request: false,
                ctx: msg.ctx_tokens,
            };
            let sent = self.ship(conv, Some(msg.id), RELAY_LAST, &req).await;
            if !matches!(sent, Ok(Shipped::Answered(Response::Ok) | Shipped::Kept)) {
                log::warn!("대화 {} 자동 왕복 마지막 답 기록 실패", conv.uuid);
                let _ = chat::set_state(&self.db(), msg.id, State::Undelivered);
            }
            return;
        }
        let req = Request::Message {
            conv: conv.uuid.clone(),
            msg: msg.uuid.clone(),
            agent: conv.agent.clone(),
            text: text.to_string(),
            lc,
            by_agent: conv.relay.as_ref().map(|r| r.agent.clone()),
            relay: None,
            via: None,
            request: false,
            mention: None,
        };
        // 보내기 전에 `sent` 실행 요청으로 — 저장 못 하면 보내지 않는다(늦게 온 답을 받을 근거가 없다). 보내기 행과 한 트랜잭션(#1c — `sent`
        // 만 남고 보낼 행이 없는 채로 꺼져 자동 왕복이 멈추지 않게).
        // 결과를 먼저 받는다 — `if let` 조건의 DB 잠금은 블록 끝까지 살아 있어(edition 2024) 안에서 다시 잠그면 멈춘다.
        let flagged = {
            let db = self.db();
            an_core::db::write_tx(&db).map_err(db_err).and_then(|tx| {
                chat::set_flags(&tx, msg.id, true, true, State::Sent)
                    .and_then(|_| chat::set_refused(&tx, msg.id, None))
                    .map_err(db_err)?;
                let staged = Self::stage(&tx, conv, Some(msg.id), RELAY_STEP, &req)?;
                tx.commit().map_err(db_err)?;
                Ok(staged)
            })
        };
        let staged = match flagged {
            Ok(id) => id,
            Err(e) => {
                log::error!("대화 {} 자동 왕복 상태 저장 실패 — 멈춤: {e}", conv.uuid);
                let _ = chat::relay_on(&self.db(), conv.id, false, false);
                self.relay_end(conv, "상태 저장 실패");
                return;
            }
        };
        // 닿지 않으면 보관 — 줄이 다시 보내고 닿으면 이어 간다(#1c). 명시적 거절 · 보관 실패만 멈춘다.
        let sent = self.dispatch(conv, staged, &req).await;
        if !matches!(sent, Ok(Shipped::Answered(Response::Ok) | Shipped::Kept)) {
            log::warn!("대화 {} 자동 왕복 전달 실패 — 멈춤", conv.uuid);
            let _ = chat::relay_on(&self.db(), conv.id, false, false);
            match sent {
                Ok(Shipped::Answered(Response::Error { code })) if code == STOPPED => {
                    self.relay_stopped(conv, msg.id);
                }
                Ok(Shipped::Answered(other)) => {
                    let focus = matches!(&other, Response::Error { code } if code == FOCUS);
                    self.refused_by_peer(msg.id, conv.peer_id, &other);
                    self.relay_end(
                        conv,
                        if focus {
                            "상대가 집중 중"
                        } else {
                            "상대가 거절"
                        },
                    );
                }
                _ => self.relay_end(conv, "상태 저장 실패"),
            }
        }
    }

    /// 받은 쪽이 멈춘 뒤 닿은 넘김(`stopped`) — 멈춤 알림(`Relay{on: false}`)이 먼저 닿은 것과 같은 결과: 내 답은 넘기지 않은 답으로
    /// 되돌리고(실패 아님) 자동 왕복을 끄고 "상대가 멈춤". 알림이 뒤늦게 와도 이미 꺼져 있어 다시 알리지 않는다.
    fn relay_stopped(&self, conv: &Conversation, msg_id: i64) {
        {
            let db = self.db();
            let _ = chat::set_flags(&db, msg_id, true, false, State::Done);
            let _ = chat::relay_on(&db, conv.id, false, false);
        }
        self.relay_end(conv, "상대가 멈춤");
    }

    /// 집중(이 PC 설정) 중인가 — 못 읽으면 받는다: 실행이 같은 설정 오류로 실패 답을 낸다(15 R2 결정 — 기본값으로 감추지 않는다).
    fn focused(&self, conn: &Connection) -> bool {
        an_core::settings::focus(conn).unwrap_or_else(|e| {
            log::warn!("집중 설정 읽기 실패 — 받는다: {e}");
            false
        })
    }

    /// 이미 받은 글의 재전송인가 — 같은 uuid 가 있고 자동 왕복 재개의 승격(상한에서 기록만 받은 상대 에이전트 글 → 실행 요청)이 아니다.
    fn resent(&self, peer_id: i64, conv: &str, msg: &str, by_agent: bool) -> io::Result<bool> {
        let db = self.db();
        let Some(old) = chat::message_by_uuid(&db, msg).map_err(db_err)? else {
            return Ok(false);
        };
        let same = chat::find_own(&db, conv, peer_id)
            .map_err(db_err)?
            .is_some_and(|c| c.id == old.conversation_id);
        Ok(!(by_agent && same && old.author == Author::Agent && !old.mine && !old.exec))
    }

    /// 상대가 거절한 내 글 — 실패 + 거절 코드. 집중(`focus`)이면 그 상대를 접속 · 집중으로 남긴다(거절도 접속 증거 — 다음 Ping 을 기다리지
    /// 않게).
    fn refused_by_peer(&self, msg_id: i64, peer_id: i64, resp: &Response) {
        let db = self.db();
        let _ = chat::set_state(&db, msg_id, State::Error);
        if let Response::Error { code } = resp {
            let _ = chat::set_refused(&db, msg_id, Some(an_net::proto::clean_code(code)));
            if code == FOCUS {
                let _ = an_core::peers::set_presence(&db, peer_id, true);
            }
        }
    }

    /// `running` 표시 후 실행. 표시를 못 하면 실행하지 않는다(재시작 뒤 중복 실행 판별이 안 되므로).
    async fn execute(
        &self,
        conv: &Conversation,
        agent: agents::Agent,
        asked: &Message,
    ) -> Result<Ran, Failure> {
        // 등록 · 수정 때 거른 규칙을 실행 직전에 다시 — 1a 이전에 등록된 폴더 · 등록 뒤 링크가 바뀐 폴더.
        if let Some(why) = an_core::paths::agent_folder_problem(std::path::Path::new(&agent.folder))
        {
            return Err(Failure {
                detail: format!("에이전트 {} 폴더 — {why}", agent.name),
                public: FOLDER_BLOCKED,
            });
        }
        chat::set_state(&self.db(), asked.id, State::Running).map_err(Failure::record)?;
        // 세션 — 대화 에이전트는 대화의 것, 멘션한 에이전트는 대화 × 에이전트(해석 3).
        let saved = match asked.agent.as_deref() {
            Some(name) => {
                chat::agent_session(&self.db(), conv.id, name).map_err(Failure::record)?
            }
            None => conv.session.clone(),
        };
        let session = saved
            .map(Session::Resume)
            .unwrap_or_else(|| Session::New(new_uuid()));
        let resumed = matches!(session, Session::Resume(_));
        // 그 상대의 덮어쓰기 → 전역 설정 → 기본값.
        let (model, effort) = {
            let db = self.db();
            let peer = an_core::peers::by_id(&db, conv.peer_id).map_err(Failure::record)?;
            an_core::run_opts::resolve(&db, peer.as_ref()).map_err(Failure::record)?
        };
        // 상대가 보낸 실행 요청이면 단계 요약을 `step_every` 에 한 번 이하로 보낸 쪽에 알린다(#1c) — 실행기는 바뀔 때마다 마지막 값을 두고,
        // 이 작업이 그 사이 바뀐 것만 보낸다.
        let latest: Arc<Mutex<Option<StepDetail>>> = Arc::default();
        let remote = conv.dir == Dir::In && !asked.mine;
        let steps = if remote {
            let l = latest.clone();
            StepSink::new(move |d| *an_core::sync::lock(&l) = Some(d.clone()))
        } else {
            StepSink::default()
        };
        let ticker = self.me.upgrade().filter(|_| remote).map(|me| {
            let (conv, asked, every) = (
                conv.clone(),
                asked.clone(),
                *an_core::sync::lock(&self.step_every),
            );
            let latest = Arc::downgrade(&latest);
            tokio::spawn(async move {
                loop {
                    tokio::time::sleep(every).await;
                    let Some(latest) = latest.upgrade() else {
                        return;
                    };
                    let step = an_core::sync::lock(&latest).take();
                    if let Some(d) = step {
                        me.send_step(&conv, &asked, "running", 0, Some(d));
                    }
                }
            })
        });
        let mut text = self.with_notes(conv, asked).map_err(Failure::record)?;
        // 압축 뒤 첫 실행(#1m) — 새 세션 앞에 요약을 한 번 붙인다(세션을 남기면 `record` 가 지운다).
        if !resumed {
            let seed = chat::ctx_seed(&self.db(), conv.id, asked.agent.as_deref().unwrap_or(""))
                .map_err(Failure::record)?;
            if let Some(seed) = seed {
                text = format!(
                    "[이전 대화 요약 — 컨텍스트 압축, 실행 요청 아님]\n<대화 요약>\n{}\n</대화 요약>\n\n{text}",
                    unframe(&seed)
                );
            }
        }
        let input = RunInput {
            agent: agent.clone(),
            session,
            text,
            model: model.clone(),
            effort: effort.clone(),
            steps,
        };
        let run = self.run.clone();
        let first = input.clone();
        let ran = match tokio::task::spawn_blocking(move || run(first)).await {
            // 이어 쓸 세션이 사라졌다(#1h) — 새 세션으로 한 번 더. 답과 새 세션은 아래 기록에서 그대로 남아 다음 요청은 새 세션을 잇는다.
            Ok(Err(e)) if resumed && crate::runner::session_missing(&e) => {
                log::warn!("대화 {} — {e} · 새 세션으로 다시 실행", conv.id);
                let run = self.run.clone();
                let again = RunInput {
                    session: Session::New(new_uuid()),
                    ..input
                };
                tokio::task::spawn_blocking(move || run(again)).await
            }
            other => other,
        };
        if let Some(t) = ticker {
            t.abort();
        }
        match ran {
            Ok(Ok(out)) => Ok(Ran {
                out,
                agent,
                model,
                effort,
            }),
            Ok(Err(e)) => Err(Failure::run(&e)),
            Err(e) => Err(Failure {
                detail: format!("실행 작업 실패: {e}"),
                public: RUN_FAILED,
            }),
        }
    }

    /// 실행 입력 — 지난 실행 요청 뒤 그 사이 사람 글(메모)을 발화자와 함께 앞에, 요청 본문은 늘 머리말 뒤 구분자 안에(1a A10).
    /// 메모는 최근 [`MAX_NOTES`] 개 · 본문 합 [`MAX_NOTES_BYTES`] 까지(오래된 것부터 버린다, 1a A8). 본문 · 메모 안의 구분자 글자는
    /// 지운다 — 원격 글이 틀을 일찍 닫고 이 PC 사용자 지시처럼 꾸미지 못하게.
    fn with_notes(
        &self,
        conv: &Conversation,
        asked: &Message,
    ) -> Result<String, an_core::db::Error> {
        let mut notes = {
            let db = self.db();
            let after = chat::last_exec_before(&db, conv.id, asked)?;
            chat::notes_between(&db, conv.id, after.as_ref(), asked)?
        };
        let mut bytes = 0;
        let keep = notes
            .iter()
            .rev()
            .take_while(|n| {
                bytes += n.text.len();
                bytes <= MAX_NOTES_BYTES
            })
            .take(MAX_NOTES)
            .count();
        let dropped = notes.len() - keep;
        notes.drain(..dropped);
        let peer_name = self
            .node
            .peers()
            .ok()
            .and_then(|ps| ps.into_iter().find(|p| p.id == conv.peer_id))
            .map(|p| unframe(&format!("{} · {}", p.name, p.host)))
            .unwrap_or_default();
        // 자동 왕복의 에이전트 글 = 상대 에이전트의 요청(D24) — 사람 요청과 구분해 표시한다. 상대 이름은 상대가 정한다 — 늘 `상대(…)` 안에
        // 넣어 `이 PC 사용자` 를 흉내 내지 못하게 하고 구분자는 지운다.
        let who = |m: &Message| {
            if m.author == Author::Agent {
                format!("상대 에이전트({peer_name})")
            } else if m.mine {
                "이 PC 사용자".to_string()
            } else {
                format!("상대({peer_name})")
            }
        };
        let mut text = String::new();
        if !notes.is_empty() {
            text.push_str("[대화 메모 — 실행 요청 아님]\n<대화 메모>\n");
            if dropped > 0 {
                text.push_str(&format!("(앞의 메모 {dropped}개 생략)\n"));
            }
            for n in &notes {
                // 메모 안 줄바꿈 뒤는 들여 쓴다 — 본문에 `- 이 PC 사용자: …` 줄을 꾸며 넣어도 발화자 줄과 구별된다.
                let body = unframe(&n.text).replace('\n', "\n  ");
                text.push_str(&format!("- {}: {body}\n", who(n)));
            }
            text.push_str("</대화 메모>\n\n");
        }
        text.push_str(&format!(
            "[요청 — {}]\n<원격 본문>\n{}\n</원격 본문>",
            who(asked),
            unframe(&asked.text)
        ));
        Ok(text)
    }

    /// 컨텍스트 압축 · 비우기(#1m, 화면) — `op` = `compact` · `clear`. 이 PC 에 그 대화 세션이 있으면(받은 대화 · 자동 왕복) 대화 줄에서 처리하고,
    /// 상대 PC 에 있을 수 있으면(보낸 대화 · 자동 왕복) 상대에 요청한다(꺼져 있으면 보관했다 보낸다). 진행 중인 압축 · 비우기가 있으면 거절.
    pub fn context(self: &Arc<Self>, conversation_id: i64, op: &str) -> io::Result<Message> {
        if !matches!(op, "compact" | "clear") {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "op 는 compact · clear",
            ));
        }
        let conv = chat::by_id(&self.db(), conversation_id)
            .map_err(db_err)?
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "없는 대화"))?;
        self.request_context(&conv, op)
    }

    /// 구분선 글 + (이 PC 처리면) 대화 줄 깨우기 + (상대 쪽이면) 요청 줄 — 한 트랜잭션. 자동 압축도 여기로.
    fn request_context(self: &Arc<Self>, conv: &Conversation, op: &str) -> io::Result<Message> {
        Self::writable(conv)?;
        let local = conv.dir == Dir::In || conv.relay.is_some();
        let remote = conv.dir == Dir::Out || conv.relay.is_some();
        let marker = {
            let db = self.db();
            let tx = an_core::db::write_tx(&db).map_err(db_err)?;
            // 수동은 실행 대기 · 실행 중 글이 있으면 거절 — 앞 요청이 기대한 맥락을 먼저 비우지 않게(라운드 2). 자동 압축은 방금 끝난 실행 뒤라 줄 앞으로.
            let manual = op != "auto_compact";
            let busy = chat::messages(&tx, conv.id)
                .map_err(db_err)?
                .into_iter()
                .any(|m| {
                    let ctx = m.author == Author::Context
                        && matches!(m.state, State::Queued | State::Running | State::Sent);
                    let run = manual && m.exec && matches!(m.state, State::Queued | State::Running);
                    ctx || run
                });
            if busy {
                return Err(io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    "압축 · 비우기 · 실행이 진행 중 — 끝난 뒤에",
                ));
            }
            let state = if local { State::Queued } else { State::Sent };
            let marker = chat::add_context(&tx, conv.id, &new_uuid(), op, state, true, local)
                .map_err(db_err)?;
            if remote {
                let req = Request::Context {
                    conv: conv.uuid.clone(),
                    msg: marker.uuid.clone(),
                    op: op.into(),
                    text: String::new(),
                };
                let frame = serde_json::to_string(&req).map_err(io::Error::other)?;
                outbox::add(
                    &tx,
                    conv.peer_id,
                    Some(conv.id),
                    Some(marker.id),
                    CTX,
                    &frame,
                    None,
                    "pending",
                )
                .map_err(db_err)?;
            }
            tx.commit().map_err(db_err)?;
            marker
        };
        if local {
            self.wake(conv.id);
        }
        if remote {
            self.kick(conv.peer_id);
        }
        Ok(marker)
    }

    /// 상대가 보낸 컨텍스트 프레임(#1m) — 요청(`compact` · `auto_compact` · `clear`)은 내 세션을 대화 줄에서 처리하고 다시 넘기지 않는다. 결과
    /// (`done` · `failed`)는 내가 상대에 요청한 구분선 글에 적는다.
    fn inbound_context(
        self: &Arc<Self>,
        peer: &Peer,
        conv: String,
        msg: String,
        op: String,
        text: String,
    ) -> io::Result<Response> {
        if !an_net::proto::CTX_OPS.contains(&op.as_str()) || text.len() > MAX_TEXT {
            return Ok(Response::error("bad_request"));
        }
        let db = self.db();
        let Some(c) = chat::find_own(&db, &conv, peer.id).map_err(db_err)? else {
            return Ok(Response::error("unknown"));
        };
        let found = chat::message_by_uuid(&db, &msg).map_err(db_err)?;
        match op.as_str() {
            "done" | "failed" => {
                let Some(m) =
                    found.filter(|m| m.conversation_id == c.id && m.author == Author::Context)
                else {
                    return Ok(Response::error("unknown"));
                };
                // 이 PC 도 처리한 글(자동 왕복) — 상대 실패는 숨기지 않고 `peer_{코드}` 로(내 처리 결과 · 상태는 그대로, 라운드 1).
                if m.exec {
                    if op == "failed" {
                        chat::set_refused(
                            &db,
                            m.id,
                            Some(&peer_code(an_net::proto::clean_code(&text))),
                        )
                        .map_err(db_err)?;
                    }
                    return Ok(Response::Ok);
                }
                if m.state != State::Sent {
                    return Ok(Response::Ok);
                }
                if op == "done" {
                    chat::finish_context(&db, m.id, State::Done, &text).map_err(db_err)?;
                } else {
                    let tx = an_core::db::write_tx(&db).map_err(db_err)?;
                    chat::finish_context(&tx, m.id, State::Error, "").map_err(db_err)?;
                    chat::set_refused(&tx, m.id, Some(an_net::proto::clean_code(&text)))
                        .map_err(db_err)?;
                    tx.commit().map_err(db_err)?;
                }
                Ok(Response::Ok)
            }
            _ => {
                if found.is_some() {
                    return Ok(Response::Ok); // 같은 요청 재전송
                }
                // 내게 그 대화 세션이 없는 대화(내가 보낸 대화 · 자동 왕복 아님) — 처리할 것이 없다.
                if c.dir == Dir::Out && c.relay.is_none() {
                    return Ok(Response::error("bad_request"));
                }
                chat::add_context(&db, c.id, &msg, &op, State::Queued, false, true)
                    .map_err(db_err)?;
                drop(db);
                self.wake(c.id);
                Ok(Response::Ok)
            }
        }
    }

    /// 상대에 보낸 컨텍스트 요청이 거절 · 포기됐다(#1m) — 이 PC 가 처리하지 않는 글만 실패로(자동 왕복이면 내 처리 결과가 그대로).
    fn context_refused(&self, msg_id: i64, code: &str) {
        let db = self.db();
        match chat::message_by_id(&db, msg_id) {
            Ok(Some(m)) if !m.exec && m.state == State::Sent => {
                let _ = chat::finish_context(&db, m.id, State::Error, "");
                let _ = chat::set_refused(&db, m.id, Some(code));
            }
            Ok(Some(m)) if m.exec => {
                let _ = chat::set_refused(&db, m.id, Some(&peer_code(code)));
            }
            Ok(_) => log::info!("컨텍스트 요청 {msg_id} 상대 거절({code}) — 이미 끝난 글"),
            Err(e) => log::warn!("컨텍스트 요청 {msg_id} 읽기 실패: {e}"),
        }
    }

    /// 대화 줄의 구분선 글 처리(#1m) — 비우기는 세션만, 압축은 세션마다 요약 실행 뒤 세션을 비우고 요약을 다음 첫 실행용으로 남긴다. 세션 비우기 ·
    /// 요약 남기기 · 구분선 끝 · (상대가 요청한 글이면) 결과 회신 줄은 한 트랜잭션 — 사이에 꺼져도 요청자가 결과를 영영 못 받는 일이 없다(라운드 1).
    async fn run_context(&self, conversation_id: i64, marker: Message) {
        let conv = match chat::by_id(&self.db(), conversation_id) {
            Ok(Some(c)) => c,
            Ok(None) => return log::error!("대화 {conversation_id} 를 찾지 못했다"),
            Err(e) => return log::error!("대화 {conversation_id} 읽기 실패: {e}"),
        };
        let op = marker.ctx_op.clone().unwrap_or_default();
        let result: Result<Vec<Seed>, &'static str> = if marker.state == State::Running {
            Err(CTX_FAILED) // 앱이 도중에 끝났다 — 세션은 그대로(비우기 · 요약 기록은 끝 트랜잭션에서만)
        } else {
            let _ = chat::set_state(&self.db(), marker.id, State::Running);
            if op == "clear" {
                Ok(Vec::new())
            } else {
                self.compact(&conv, &marker).await
            }
        };
        let summary = match &result {
            Ok(seeds) => summary_text(&conv, seeds),
            Err(_) => String::new(),
        };
        let reply = (!marker.mine).then(|| Request::Context {
            conv: conv.uuid.clone(),
            msg: marker.uuid.clone(),
            op: if result.is_ok() { "done" } else { "failed" }.into(),
            text: match result {
                Ok(_) => summary.clone(),
                Err(code) => code.to_string(),
            },
        });
        let saved = (|| -> io::Result<()> {
            let db = self.db();
            let tx = an_core::db::write_tx(&db).map_err(db_err)?;
            match &result {
                Ok(seeds) => {
                    chat::clear_sessions(&tx, conv.id, false).map_err(db_err)?;
                    for s in seeds {
                        // 요약 실행 뒤 그 에이전트 폴더가 바뀌었거나 지웠다 다시 등록됐으면(세대가 다르다) 옛 요약을 남기지 않는다(라운드 1).
                        let same = agents::by_name(&tx, &s.agent)
                            .map_err(db_err)?
                            .is_some_and(|a| a.uid == s.uid);
                        if same {
                            chat::set_ctx_seed(&tx, conv.id, &s.key, &s.text).map_err(db_err)?;
                        }
                    }
                    chat::finish_context(&tx, marker.id, State::Done, &summary).map_err(db_err)?;
                }
                Err(code) => {
                    chat::finish_context(&tx, marker.id, State::Error, "").map_err(db_err)?;
                    chat::set_refused(&tx, marker.id, Some(code)).map_err(db_err)?;
                }
            }
            if let Some(req) = &reply {
                let frame = serde_json::to_string(req).map_err(io::Error::other)?;
                outbox::add(
                    &tx,
                    conv.peer_id,
                    Some(conv.id),
                    Some(marker.id),
                    CTX_DONE,
                    &frame,
                    None,
                    "pending",
                )
                .map_err(db_err)?;
            }
            tx.commit().map_err(db_err)
        })();
        if let Err(e) = saved {
            log::error!("대화 {} 컨텍스트 기록 실패: {e}", conv.uuid);
            let _ = chat::set_state(&self.db(), marker.id, State::Error);
            return;
        }
        if reply.is_some() {
            self.kick(conv.peer_id);
        }
    }

    /// 압축 — 대화 세션마다 그 세션을 이어 요약 실행(실행 문을 지난다). 하나라도 실패하면 아무것도 바꾸지 않는다(세션 그대로). 사라진 세션은 건너뛴다.
    /// 기록(세션 비우기 · 요약)은 부른 쪽 끝 트랜잭션이 한다.
    async fn compact(
        &self,
        conv: &Conversation,
        marker: &Message,
    ) -> Result<Vec<Seed>, &'static str> {
        let sessions = chat::sessions(&self.db(), conv).map_err(|e| {
            log::warn!("대화 {} 세션 읽기 실패: {e}", conv.uuid);
            CTX_FAILED
        })?;
        if sessions.is_empty() {
            return Ok(Vec::new());
        }
        let (mut ticket, ahead) = self.gate.enter(marker.id);
        if ahead.is_some() {
            ticket.ready().await;
        }
        let mut seeds = Vec::new();
        for (name, session) in sessions {
            // 대화 에이전트 세션 — 받은 대화는 대화 에이전트, 자동 왕복을 연 쪽은 내 왕복 에이전트.
            let who = name
                .clone()
                .unwrap_or_else(|| match (&conv.dir, &conv.relay) {
                    (Dir::Out, Some(r)) => r.agent.clone(),
                    _ => conv.agent.clone(),
                });
            let (agent, opts) = {
                let db = self.db();
                let agent = agents::by_name(&db, &who).ok().flatten();
                let peer = an_core::peers::by_id(&db, conv.peer_id).ok().flatten();
                (agent, an_core::run_opts::resolve(&db, peer.as_ref()).ok())
            };
            let (Some(agent), Some((model, effort))) = (agent, opts) else {
                log::warn!(
                    "대화 {} 압축 — 에이전트 {who} 없음 · 실행 옵션 읽기 실패",
                    conv.uuid
                );
                return Err(CTX_FAILED);
            };
            if an_core::paths::agent_folder_problem(std::path::Path::new(&agent.folder)).is_some() {
                return Err(CTX_FAILED);
            }
            let uid = agent.uid.clone();
            let input = RunInput {
                agent,
                session: Session::Resume(session),
                text: SUMMARY_PROMPT.to_string(),
                model,
                effort,
                steps: StepSink::default(),
            };
            let run = self.run.clone();
            match tokio::task::spawn_blocking(move || run(input)).await {
                Ok(Ok(out)) if !out.is_error && !out.text.trim().is_empty() => seeds.push(Seed {
                    key: name.unwrap_or_default(),
                    agent: who,
                    uid,
                    text: fit_to(out.text, MAX_SUMMARY),
                }),
                Ok(Err(e)) if crate::runner::session_missing(&e) => {
                    log::info!("대화 {} 압축 — 사라진 세션 건너뜀: {e}", conv.uuid);
                }
                other => {
                    log::warn!("대화 {} 압축 요약 실패: {other:?}", conv.uuid);
                    return Err(CTX_FAILED);
                }
            }
        }
        Ok(seeds)
    }

    /// 세션 · 사람 메시지 상태 · 에이전트 답(원 요청 · 실행 정보)을 한 트랜잭션으로. `run` = 끝난 실행(실패면 없음).
    /// 돌려주는 값: 답 메시지 id.
    fn record(
        &self,
        conv: &Conversation,
        asked: &Message,
        run: Option<&Ran>,
        reply_uuid: &str,
        (text, state): (&str, State),
        staging: Option<(&'static str, &dyn Fn(i64) -> Request)>,
    ) -> Result<(i64, Option<i64>), an_core::db::Error> {
        let db = self.db();
        let tx = an_core::db::write_tx(&db)?;
        if let Some(ran) = run {
            // 실행 도중 폴더가 바뀌었거나 지웠다 다시 등록했으면(세대가 다르다 — 세션을 지웠다) 옛 세션을 되살리지 않는다.
            if agents::by_name(&tx, &ran.agent.name)?.is_some_and(|now| now.uid == ran.agent.uid) {
                match asked.agent.as_deref() {
                    Some(name) => chat::set_agent_session(&tx, conv.id, name, &ran.out.session)?,
                    None => chat::set_session(&tx, conv.id, &ran.out.session)?,
                }
                chat::drop_ctx_seed(&tx, conv.id, asked.agent.as_deref().unwrap_or(""))?;
            }
        }
        chat::set_state(&tx, asked.id, state)?;
        let reply = chat::add_reply(
            &tx,
            conv.id,
            reply_uuid,
            text,
            state,
            asked.id,
            run.and_then(|r| r.out.turns),
            run.and_then(|r| r.out.duration_ms),
            run.map(|r| r.model.as_str()),
            run.map(|r| r.effort.as_str()),
        )?;
        if asked.agent.is_some() {
            chat::set_agent(&tx, reply.id, asked.agent.as_deref())?;
        }
        chat::set_ctx_tokens(&tx, reply.id, run.and_then(|r| r.out.ctx_tokens))?;
        // 답을 보낼 행(#1c) — 답과 같은 트랜잭션.
        let staged = match staging {
            Some((kind, frame)) => {
                let frame = serde_json::to_string(&frame(reply.lc))
                    .map_err(|e| an_core::db::Error::ToSqlConversionFailure(Box::new(e)))?;
                Some(outbox::add(
                    &tx,
                    conv.peer_id,
                    Some(conv.id),
                    Some(reply.id),
                    kind,
                    &frame,
                    None,
                    "inflight",
                )?)
            }
            None => None,
        };
        tx.commit()?;
        Ok((reply.id, staged))
    }

    /// 공유 사본(#1o)은 읽기 전용 — 글쓰기 · 컨텍스트 · 자동 왕복 · 요청 상태 · CLI 보내기를 막는다.
    fn writable(conv: &Conversation) -> io::Result<()> {
        if conv.dir == Dir::Shared {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, READ_ONLY));
        }
        Ok(())
    }

    fn conversation(&self, id: i64) -> io::Result<Conversation> {
        chat::by_id(&self.db(), id)
            .map_err(db_err)?
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, format!("대화 {id} 없음")))
    }

    /// 대화에 관람자를 넣는다(#1o, A — 대화를 가진 PC). 거절: 공유 사본(`InvalidInput`) · 원 상대(`InvalidInput`) · 페어링 안 됨 · 이미 관람자
    /// (`AlreadyExists`) · 상한 [`share::MAX_WATCHERS`](`InvalidInput`) · 이 기능을 모르는 상대(`Unsupported`, Ping 으로 묻는다 — 닿지 않으면 그 오류).
    /// 관람자 행 · 머리(`Share`) · 이력(`SharePost` — 글 하나 = 프레임 하나)을 한 트랜잭션으로 그 관람자 줄에 넣고, 원 상대에게 관람자 수를 알린다.
    pub async fn share_add(&self, conversation_id: i64, peer_id: i64) -> io::Result<()> {
        let conv = self.conversation(conversation_id)?;
        if conv.dir == Dir::Shared {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "공유 대화는 다시 공유하지 않는다",
            ));
        }
        if peer_id == conv.peer_id {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "대화 상대는 관람자로 넣지 않는다",
            ));
        }
        self.paired_peer(peer_id)?;
        Self::share_room(&self.db(), conv.id, peer_id)?;
        if !self.node.knows_share(peer_id).await? {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "상대 앱 업데이트 필요 — 공유 대화를 모르는 버전",
            ));
        }
        {
            let db = self.db();
            let title = chat::summaries(&db)
                .map_err(db_err)?
                .into_iter()
                .find(|s| s.conversation.id == conv.id)
                .map(|s| s.title)
                .unwrap_or_default();
            let other = an_core::peers::by_id(&db, conv.peer_id)
                .map_err(db_err)?
                .map(|p| p.name)
                .unwrap_or_default();
            let tx = an_core::db::write_tx(&db).map_err(db_err)?;
            // Ping 하는 사이 다른 창이 넣었을 수 있다 — 트랜잭션 안에서 다시.
            Self::share_room(&tx, conv.id, peer_id)?;
            share::add(&tx, conv.id, peer_id).map_err(db_err)?;
            let head = Request::Share {
                conv: conv.uuid.clone(),
                title,
                agent: conv.agent.clone(),
                owner_dir: conv.dir.as_str().into(),
                other,
            };
            Self::stage_share(&tx, &conv, peer_id, &head)?;
            // 이력도 같은 트랜잭션 — 머리만 가고 글이 빠진 채로 꺼지지 않게.
            Self::fan_share_in(&tx, conv.id)?;
            tx.commit().map_err(db_err)?;
        }
        self.kick(peer_id);
        self.notice_watchers(&conv);
        Ok(())
    }

    /// 관람자 자리 — 이미 관람자 · 상한이면 거절.
    fn share_room(conn: &Connection, conversation_id: i64, peer_id: i64) -> io::Result<()> {
        let ws = share::watchers(conn, conversation_id).map_err(db_err)?;
        if ws.iter().any(|w| w.peer_id == peer_id) {
            return Err(io::Error::new(io::ErrorKind::AlreadyExists, "이미 관람자"));
        }
        if ws.len() >= share::MAX_WATCHERS {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("관람자는 {}명까지", share::MAX_WATCHERS),
            ));
        }
        Ok(())
    }

    /// 관람자를 뺀다(#1o, A) — 이후 글은 가지 않고 관람자 사본은 읽기 전용으로 남는다(`ShareEnd`). 원 상대에게 관람자 수. 관람자가 아니면
    /// `NotFound`.
    pub fn share_remove(&self, conversation_id: i64, peer_id: i64) -> io::Result<()> {
        let conv = self.conversation(conversation_id)?;
        if !self.unshare(&conv, peer_id, true)? {
            return Err(io::Error::new(io::ErrorKind::NotFound, "관람자가 아니다"));
        }
        Ok(())
    }

    /// 공유 대화에서 나간다(#1o, C — 관람자 PC) — 사본은 읽기 전용으로 남고(빠짐 표시) 대화를 가진 PC 에 `ShareEnd`. 이미 빠졌으면 그대로.
    pub fn share_leave(&self, conversation_id: i64) -> io::Result<()> {
        let conv = self.conversation(conversation_id)?;
        if conv.dir != Dir::Shared {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "나가기는 공유 대화만",
            ));
        }
        let ended = {
            let db = self.db();
            let tx = an_core::db::write_tx(&db).map_err(db_err)?;
            let ended = share::end(&tx, conv.id).map_err(db_err)?;
            if ended {
                let end = Request::ShareEnd {
                    conv: conv.uuid.clone(),
                };
                Self::stage_share(&tx, &conv, conv.peer_id, &end)?;
            }
            tx.commit().map_err(db_err)?;
            ended
        };
        if ended {
            self.kick(conv.peer_id);
        }
        Ok(())
    }

    /// 관람자 행 · 그 관람자 앞 기다리는 공유 프레임을 지우고, `end` 면 관람자에게 `ShareEnd`, 원 상대에게 바뀐 관람자 수. 없던 관람자면 거짓.
    /// 공유 줄 행의 관람자가 아직 그 대화의 관람자인가(#1o). 읽지 못하면 참(보내던 대로 — 다음 시도에 다시 본다).
    fn watching(&self, item: &outbox::Item) -> bool {
        let Some(conv) = item.conv_id else {
            return true;
        };
        share::watchers(&self.db(), conv)
            .map_or(true, |ws| ws.iter().any(|w| w.peer_id == item.peer_id))
    }

    fn unshare(&self, conv: &Conversation, peer_id: i64, end: bool) -> io::Result<bool> {
        let gone = {
            let db = self.db();
            let tx = an_core::db::write_tx(&db).map_err(db_err)?;
            let gone = share::remove(&tx, conv.id, peer_id).map_err(db_err)?;
            if gone && end {
                let req = Request::ShareEnd {
                    conv: conv.uuid.clone(),
                };
                Self::stage_share(&tx, conv, peer_id, &req)?;
            }
            tx.commit().map_err(db_err)?;
            gone
        };
        if gone {
            if end {
                self.kick(peer_id);
            }
            self.notice_watchers(conv);
        }
        Ok(gone)
    }

    /// 원 상대(B)에게 그 대화의 관람자 수(#1o — 이름 없음). 기다리는 알림은 마지막 값으로 바꾼다.
    fn notice_watchers(&self, conv: &Conversation) {
        let found = share::watchers(&self.db(), conv.id);
        let n = match found {
            Ok(ws) => ws.len(),
            Err(e) => return log::warn!("대화 {} 관람자 읽기 실패: {e}", conv.uuid),
        };
        let req = Request::ShareNotice {
            conv: conv.uuid.clone(),
            n: u32::try_from(n).unwrap_or(u32::MAX),
        };
        let key = format!("share_notice:{}", conv.uuid);
        self.enqueue(conv, None, SHARE_NOTICE, &req, Some(&key));
    }

    /// 공유 대화(#1o) — 관람자마다 보낸 커서 뒤 글(구분선 제외)을 `SharePost` 행으로 넣고 커서를 올린다(한 트랜잭션 — 몇 번 불러도 같은 글을
    /// 두 번 넣지 않는다). 글을 남기는 길목(`dispatch` · `enqueue` · `handle` 끝)이 부른다. 실패는 로그만 — 다음 부름이 이어 보낸다.
    // ponytail: 글 하나 = 프레임 하나 = 연결 하나 — 이력이 길면 느리다, 느리면 크기를 재서 묶는다.
    fn fan_share(&self, conversation_id: i64) {
        let any = share::watchers(&self.db(), conversation_id);
        if any.as_ref().is_ok_and(Vec::is_empty) {
            return;
        }
        let fanned = (|| -> io::Result<Vec<i64>> {
            let db = self.db();
            let tx = an_core::db::write_tx(&db).map_err(db_err)?;
            let peers = Self::fan_share_in(&tx, conversation_id)?;
            tx.commit().map_err(db_err)?;
            Ok(peers)
        })();
        match fanned {
            Ok(peers) => {
                for p in peers {
                    self.kick(p);
                }
            }
            Err(e) => log::warn!("대화 {conversation_id} 관람자에게 보낼 글 보관 실패: {e}"),
        }
    }

    /// [`fan_share`](Self::fan_share) 의 몸 — `conn` 은 부르는 쪽 트랜잭션. 돌려주는 값: 깨울 관람자.
    fn fan_share_in(conn: &Connection, conversation_id: i64) -> io::Result<Vec<i64>> {
        let Some(conv) = chat::by_id(conn, conversation_id).map_err(db_err)? else {
            return Ok(Vec::new());
        };
        let mut ms = chat::messages(conn, conversation_id).map_err(db_err)?;
        ms.sort_by_key(|m| m.id);
        let top = ms.last().map_or(0, |m| m.id);
        let mut peers = Vec::new();
        for w in share::watchers(conn, conversation_id).map_err(db_err)? {
            if w.sent_upto >= top {
                continue;
            }
            for m in ms
                .iter()
                .filter(|m| m.id > w.sent_upto && m.author != Author::Context)
            {
                Self::stage_share(conn, &conv, w.peer_id, &Self::share_post(&conv, m))?;
            }
            share::set_sent(conn, conversation_id, w.peer_id, top).map_err(db_err)?;
            peers.push(w.peer_id);
        }
        Ok(peers)
    }

    /// 이미 관람자에게 간 글(커서 안)을 고쳤거나(`SharePost` 덮어쓰기) 지웠다(`ShareDelete`) — 그 관람자 줄에. 돌려주는 값: 깨울 관람자.
    /// 커서 뒤 글은 [`fan_share`](Self::fan_share) 가 지금 모습으로 보낸다.
    fn share_change(
        conn: &Connection,
        conv: &Conversation,
        m: &Message,
        deleted: bool,
    ) -> io::Result<Vec<i64>> {
        let mut peers = Vec::new();
        for w in share::watchers(conn, conv.id).map_err(db_err)? {
            if w.sent_upto < m.id || m.author == Author::Context {
                continue;
            }
            let req = if deleted {
                Request::ShareDelete {
                    conv: conv.uuid.clone(),
                    msg: m.uuid.clone(),
                }
            } else {
                Self::share_post(conv, m)
            };
            Self::stage_share(conn, conv, w.peer_id, &req)?;
            peers.push(w.peer_id);
        }
        Ok(peers)
    }

    /// 공유 프레임을 그 상대 줄에(`msg_id` 없음 — 글을 나르는 행이 아니라 편집 판정 · 보관 표시에 끼지 않는다).
    fn stage_share(
        conn: &Connection,
        conv: &Conversation,
        peer_id: i64,
        req: &Request,
    ) -> io::Result<()> {
        let frame = serde_json::to_string(req).map_err(io::Error::other)?;
        outbox::add(
            conn,
            peer_id,
            Some(conv.id),
            None,
            SHARE,
            &frame,
            None,
            "pending",
        )
        .map(drop)
        .map_err(db_err)
    }

    /// 글 하나의 `SharePost` — `owner` = 이 PC(대화를 가진 쪽) 글.
    fn share_post(conv: &Conversation, m: &Message) -> Request {
        let mut msg = ShareMsg {
            uuid: m.uuid.clone(),
            author: m.author.as_str().into(),
            text: m.text.clone(),
            lc: Some(m.lc),
            at: m.created_at,
            owner: m.mine,
            via: m.via.clone(),
            agent: m.agent.clone(),
            ok: (m.state == State::Error).then_some(false),
            edited: m.edited_at.is_some(),
        };
        // 제어 문자는 JSON 에서 6배가 된다 — 프레임 상한을 넘으면 본문을 줄인다(공유 프레임은 포기하지 않아 넘는 프레임이 관람자 줄을 영영 막는다).
        let size = serde_json::to_vec(&msg).map_or(usize::MAX, |b| b.len());
        if size > MAX_FRAME - 256 {
            msg.text = fit_to(msg.text, MAX_TEXT / 4);
        }
        Request::SharePost {
            conv: conv.uuid.clone(),
            msg,
        }
    }

    /// 공유 프레임이 거절됐다(#1o) — 관람자가 이 기능을 모르거나(`unsupported`) 이 PC 를 페어링된 상대로 보지 않으면(`not_paired`) 그 관람자를
    /// 뺀다(관람자 목록에서 빠진다). 원 상대 알림 · 관람자의 나가기 거절은 로그만.
    fn share_refused(&self, item: &outbox::Item, resp: &Response) {
        let code = match resp {
            Response::Error { code } => an_net::proto::clean_code(code),
            _ => "unknown",
        };
        log::info!(
            "상대 {} 가 공유 프레임({})을 받지 않았다: {code}",
            item.peer_id,
            item.kind
        );
        if item.kind != SHARE || !matches!(code, "not_paired" | "unsupported") {
            return;
        }
        let conv = item
            .conv_id
            .and_then(|id| chat::by_id(&self.db(), id).ok().flatten());
        let Some(conv) = conv.filter(|c| c.dir != Dir::Shared) else {
            return;
        };
        match self.unshare(&conv, item.peer_id, false) {
            Ok(true) => log::warn!(
                "대화 {} 관람자 {} 를 뺐다 — 상대가 받지 않음({code})",
                conv.uuid,
                item.peer_id
            ),
            Ok(false) => {}
            Err(e) => log::warn!("대화 {} 관람자 빼기 실패: {e}", conv.uuid),
        }
    }

    /// 공유 머리(#1o, C) — 대화를 가진 상대의 사본을 만들거나 다시 받는다(글을 비운다). 같은 uuid 가 내 대화 · 다른 상대의 것이면 `exists`.
    fn inbound_share(
        &self,
        peer: &Peer,
        conv: String,
        title: String,
        agent: String,
        (owner_dir, other): (String, String),
    ) -> io::Result<Response> {
        if !matches!(owner_dir.as_str(), "out" | "in") {
            return Ok(Response::error("bad_request"));
        }
        // 제목은 이 PC 제목 규칙에 맞을 때만(아니면 첫 글 앞부분).
        let title = an_core::run_opts::clean_title(&title).ok().flatten();
        let (agent, other) = (
            an_net::proto::clean_name(&agent),
            an_net::proto::clean_name(&other),
        );
        let db = self.db();
        let tx = an_core::db::write_tx(&db).map_err(db_err)?;
        let copy = share::open_copy(
            &tx,
            &conv,
            peer.id,
            &agent,
            title.as_deref(),
            (&other, &owner_dir),
        )
        .map_err(db_err)?;
        if copy.is_none() {
            return Ok(Response::error("exists"));
        }
        tx.commit().map_err(db_err)?;
        Ok(Response::Ok)
    }

    /// 그 상대가 보낸 살아 있는 사본(#1o, C — 빠지지 않은 것).
    fn live_copy(conn: &Connection, peer: &Peer, conv: &str) -> io::Result<Option<Conversation>> {
        let Some(c) = chat::find(conn, conv, peer.id)
            .map_err(db_err)?
            .filter(|c| c.dir == Dir::Shared)
        else {
            return Ok(None);
        };
        let ended = share::copy(conn, c.id)
            .map_err(db_err)?
            .is_some_and(|x| x.ended);
        Ok((!ended).then_some(c))
    }

    /// 공유 글(#1o, C) — 그 상대 사본에만, 실행 · 알림 없이. 같은 uuid 가 다른 대화의 글이면 `exists`.
    fn inbound_share_post(&self, peer: &Peer, conv: String, msg: ShareMsg) -> io::Result<Response> {
        let author = match msg.author.as_str() {
            "human" => Author::Human,
            "agent" => Author::Agent,
            _ => return Ok(Response::error("bad_request")),
        };
        if msg.text.trim().is_empty() || msg.text.len() > MAX_TEXT {
            return Ok(Response::error("bad_request"));
        }
        let agent = msg.agent.as_deref().map(an_net::proto::clean_name);
        let db = self.db();
        let Some(c) = Self::live_copy(&db, peer, &conv)? else {
            return Ok(Response::error("unknown"));
        };
        let post = share::Post {
            uuid: &msg.uuid,
            author,
            text: &msg.text,
            lc: msg.lc,
            at: msg.at,
            owner: msg.owner,
            via: msg.via.as_deref(),
            agent: agent.as_deref(),
            ok: msg.ok.unwrap_or(true),
            edited: msg.edited,
        };
        Ok(if share::upsert(&db, c.id, &post).map_err(db_err)? {
            Response::Ok
        } else {
            Response::error("exists")
        })
    }

    /// 공유 글 지움(#1o, C) — 그 상대 사본의 글만.
    fn inbound_share_delete(&self, peer: &Peer, conv: String, msg: String) -> io::Result<Response> {
        let db = self.db();
        let Some(c) = Self::live_copy(&db, peer, &conv)? else {
            return Ok(Response::error("unknown"));
        };
        let tx = an_core::db::write_tx(&db).map_err(db_err)?;
        share::delete(&tx, c.id, &msg).map_err(db_err)?;
        tx.commit().map_err(db_err)?;
        Ok(Response::Ok)
    }

    /// 공유 끝(#1o) — C: 대화를 가진 PC 가 뺐다(사본은 읽기 전용으로 남는다). A: 관람자가 나갔다(관람자 행을 지우고 원 상대에게 관람자 수).
    fn inbound_share_end(&self, peer: &Peer, conv: String) -> io::Result<Response> {
        let own = {
            let db = self.db();
            if let Some(c) = chat::find(&db, &conv, peer.id)
                .map_err(db_err)?
                .filter(|c| c.dir == Dir::Shared)
            {
                share::end(&db, c.id).map_err(db_err)?;
                return Ok(Response::Ok);
            }
            share::own_by_uuid(&db, &conv).map_err(db_err)?
        };
        if let Some(c) = own {
            self.unshare(&c, peer.id, false)?;
        }
        Ok(Response::Ok)
    }

    /// 원 상대(B)가 받은 관람자 수(#1o) — 그 상대 소유 대화(사본 아님)에만. 바뀌었을 때만 알린다(재전송은 알림 없음).
    fn inbound_share_notice(&self, peer: &Peer, conv: String, n: u32) -> io::Result<Response> {
        if usize::try_from(n).map_or(true, |n| n > share::MAX_WATCHERS) {
            return Ok(Response::error("bad_request"));
        }
        let (c, changed) = {
            let db = self.db();
            let Some(c) = chat::find_own(&db, &conv, peer.id).map_err(db_err)? else {
                return Ok(Response::error("unknown"));
            };
            let changed = share::set_watched(&db, c.id, i64::from(n)).map_err(db_err)?;
            (c, changed)
        };
        if changed {
            self.notify(
                c.id,
                peer,
                c.agent.clone(),
                MessageKind::Share,
                format!("관람자 {n}"),
            );
        }
        Ok(Response::Ok)
    }
}
