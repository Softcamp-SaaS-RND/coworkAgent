#![allow(dead_code)]
use an_agent::chat::{Chat, Run};
use an_agent::runner::{RunInput, RunOutput, Session};
use an_core::{db, paths::Paths};
use an_net::identity::Identity;
use an_net::node::{Me, Node, PAIRING_WINDOW, PeerEvent};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::net::TcpListener;

pub struct Side {
    pub node: Arc<Node>,
    pub chat: Arc<Chat>,
    pub addr: SocketAddr,
    pub paths: Paths,
    /// 실행기 대역이 받은 입력.
    pub runs: Arc<Mutex<Vec<RunInput>>>,
    /// 조율이 낸 이벤트.
    pub events: Arc<Mutex<Vec<PeerEvent>>>,
}

/// 실행기 대역 — "답: {입력}" 을 돌려주고, 새 세션이면 그 ID 를, 재개면 같은 ID 를 쓴다. `delay` 동안 잔다.
/// 입력에 "실패하라" 가 있으면 실행 실패(`is_error`)로 답한다.
pub fn fake_run(runs: Arc<Mutex<Vec<RunInput>>>, delay: Duration) -> Run {
    Arc::new(move |input: RunInput| {
        runs.lock().unwrap().push(input.clone());
        std::thread::sleep(delay);
        let session = match &input.session {
            Session::New(id) | Session::Resume(id) => id.clone(),
        };
        Ok(RunOutput {
            text: format!("답: {}", body(&input.text)),
            session,
            is_error: input.text.contains("실패하라"),
            turns: Some(2),
            duration_ms: Some(1500),
            ctx_tokens: None,
        })
    })
}

/// 실행 입력의 원격 본문(`<원격 본문>` 구분자 안) — 대역이 본문만 되돌려 기존 단정("답: 안녕")을 지킨다.
pub fn body(prompt: &str) -> &str {
    prompt
        .rsplit_once("<원격 본문>\n")
        .and_then(|(_, r)| r.strip_suffix("\n</원격 본문>"))
        .unwrap_or(prompt)
}

pub async fn side(tag: &str, delay: Duration) -> Side {
    side_with(tag, |runs| fake_run(runs, delay)).await
}

/// `side` + 실행기를 고른다(`runs` 에 입력을 모으는 것은 실행기 몫).
pub async fn side_with(tag: &str, make: impl FnOnce(Arc<Mutex<Vec<RunInput>>>) -> Run) -> Side {
    let root = std::env::temp_dir().join(format!("an-agent-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let paths = Paths::under(&root);
    paths.ensure().unwrap();
    let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
    let addr = listener.local_addr().unwrap();
    let node = Node::new(
        Identity::load_or_create(&paths.identity_file()).unwrap(),
        db::open(&paths.db_file()).unwrap(),
        Me {
            name: format!("user-{tag}"),
            host: format!("PC-{tag}"),
            port: addr.port(),
        },
    );
    let events = Arc::new(Mutex::new(Vec::new()));
    let sink = events.clone();
    node.on_event(move |e| sink.lock().unwrap().push(e));
    let runs = Arc::new(Mutex::new(Vec::new()));
    let chat = Chat::new(
        db::open(&paths.db_file()).unwrap(),
        node.clone(),
        make(runs.clone()),
    );
    chat.attach();
    tokio::spawn(node.clone().serve(listener));
    Side {
        node,
        chat,
        addr,
        paths,
        runs,
        events,
    }
}

/// a 와 b 를 페어링하고 서로의 피어 id 를 돌려준다(a 쪽 b, b 쪽 a).
pub async fn pair(a: &Side, b: &Side) -> (i64, i64) {
    let code = b.node.open_window(PAIRING_WINDOW);
    let pb = a.node.add(b.addr, Some(code)).await.unwrap();
    let pa = b.node.peers().unwrap().remove(0);
    a.node.approve(pb.id).await.unwrap();
    b.node.approve(pa.id).await.unwrap();
    (pb.id, pa.id)
}

pub fn db(s: &Side) -> db::Connection {
    db::open(&s.paths.db_file()).unwrap()
}

/// 조건이 참이 될 때까지(최대 15초 — 부하 걸린 시험 PC 에서도 정상 동작이 시간 밖으로 밀리지 않게, 맞으면 바로 끝난다) 기다린다.
pub async fn eventually(mut f: impl FnMut() -> bool) {
    let until = Instant::now() + Duration::from_secs(15);
    while !f() {
        assert!(Instant::now() < until, "15초 안에 조건이 맞지 않았다");
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}

/// `target` 으로 이어 주는 TCP 중계 — 첫 연결만 `first_delay` 뒤에 잇는다.
pub async fn delaying_relay(target: SocketAddr, first_delay: Duration) -> SocketAddr {
    let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let mut delay = first_delay;
        loop {
            let Ok((mut inbound, _)) = listener.accept().await else {
                return;
            };
            let wait = std::mem::take(&mut delay);
            tokio::spawn(async move {
                tokio::time::sleep(wait).await;
                if let Ok(mut outbound) = tokio::net::TcpStream::connect(target).await {
                    let _ = tokio::io::copy_bidirectional(&mut inbound, &mut outbound).await;
                }
            });
        }
    });
    addr
}
