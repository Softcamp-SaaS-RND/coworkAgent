// Windows 릴리스는 콘솔 창 없이(트레이 앱). 디버그 빌드는 콘솔을 남긴다.
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use an_agent::chat::Chat;
use an_agent::runner::{find_claude, run_claude};
use an_app::notify::{self, Notifier};
use an_app::server::{self, Status};
use an_app::tray::{Attention, Tray, write_start_error};
use an_app::{browser, chat_api, handoff, peers_api};
use an_core::db;
use an_core::instance::{Acquired, InstanceInfo, InstanceLock};
use an_core::log_file::FileLogger;
use an_core::paths::Paths;
use an_net::identity::Identity;
use an_net::node::{self, Me, Node};
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;

fn main() {
    let (tray, attention) = Tray::new();
    match start(attention) {
        Ok(Some(app)) => tray.run(app.url, app.on_seen, app.on_quit),
        Ok(None) => {}
        Err(e) => {
            log::error!("시작 실패: {e}");
            eprintln!("coworkAgent: {e}");
            // 두 OS 같은 모습 — 트레이 오류 모드(D30). 사유 파일도 못 쓰면 표시만.
            let file = write_start_error(&e.to_string()).unwrap_or_default();
            tray.run_error(&e.to_string(), &file)
        }
    }
}

/// 시작이 끝난 주 인스턴스 — 트레이에 넘길 것.
struct App {
    url: String,
    on_seen: Box<dyn Fn()>,
    on_quit: Box<dyn FnOnce()>,
}

/// `Ok(None)` = 이미 실행 중이라 핸드오프하고 끝.
fn start(attention: Attention) -> Result<Option<App>, Box<dyn Error>> {
    let root = Paths::default_root().ok_or("사용자 데이터 경로를 찾지 못했다")?;
    let paths = Paths::under(root);
    // 개명 전 설치의 신원 · 피어 · 대화 · 설정을 이어받는다.
    if let Some(legacy) = Paths::legacy_root() {
        paths.adopt_legacy(&legacy).map_err(|e| {
            format!(
                "옛 데이터({}) 옮기기 실패 — 옛 앱을 끄고 다시 실행: {e}",
                legacy.display()
            )
        })?;
    }
    paths.ensure()?;
    FileLogger::install(&paths.log_file(), log::LevelFilter::Info)?;

    let lock = match InstanceLock::acquire(&paths.lock_file())? {
        Acquired::Primary(lock) => lock,
        Acquired::Secondary => return hand_off(&paths).map(|()| None),
    };

    let conn = db::open(&paths.db_file())?;
    let db_status = format!("ok (스키마 {})", db::user_version(&conn)?);
    let identity = Identity::load_or_create(&paths.identity_file())?;

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    let listener = runtime.block_on(server::bind())?;
    let port = listener.local_addr()?.port();
    let token = server::new_token();
    let url = format!("http://127.0.0.1:{port}/?t={token}");
    let status = Status {
        version: env!("CARGO_PKG_VERSION").to_string(),
        data_dir: paths.root().display().to_string(),
        db: db_status,
    };
    let peer_listener = runtime.block_on(node::bind_peer_port())?;
    let me = Me {
        name: user_name(),
        host: host_name(),
        port: peer_listener.local_addr()?.port(),
    };
    log::info!(
        "피어 수신 0.0.0.0:{} · 지문 {}",
        me.port,
        identity.fingerprint()
    );
    let node = Node::new(identity, conn, me);
    let notifier = Notifier::new(db::open(&paths.db_file())?, move |n| attention.set(n))?;
    let on_event = notifier.clone();
    node.on_event(move |e| on_event.on_event(&e));
    // 원격 메시지 → 프로필 폴더에서 `claude -p`(D20 · D22). 실행 파일은 실행마다 찾는다 — 앱을 띄운 뒤 설치해도 된다.
    let chat = Chat::new(
        db::open(&paths.db_file())?,
        node.clone(),
        Arc::new(|input| run_claude(&find_claude(), &input)),
    );
    chat.attach();
    {
        // 지난 실행에서 남은 대기열 — 대화 작업은 tokio 작업이라 런타임 안에서.
        let _rt = runtime.enter();
        let n = chat.recover()?;
        if n > 0 {
            log::info!("남은 대화 대기열 {n}건 복구");
        }
    }
    runtime.spawn(node.clone().serve(peer_listener));

    let open_url = url.clone();
    let app = server::router_with(
        port,
        token.clone(),
        status,
        move || browser::open(&open_url),
        peers_api::routes(node)
            .merge(notify::routes(notifier.clone()))
            .merge(chat_api::routes(chat)),
    );
    runtime.spawn(async move {
        if let Err(e) = axum::serve(listener, app).await {
            log::error!("웹UI 서버 중단: {e}");
        }
    });
    InstanceInfo { port, token }.write(&paths.instance_file())?;
    log::info!("시작 — 127.0.0.1:{port}");

    let info_file = paths.instance_file();
    Ok(Some(App {
        url,
        on_seen: Box::new(move || notifier.clear()),
        on_quit: Box::new(move || {
            log::info!("종료");
            // 서버를 먼저 멈추고 잠금은 마지막에 푼다 — 그 전에 뜬 새 프로세스는 계속 보조 인스턴스다.
            runtime.shutdown_timeout(Duration::from_secs(1));
            let _ = std::fs::remove_file(info_file);
            drop(lock);
        }),
    }))
}

/// 이미 실행 중 — 주 인스턴스에 "열기" 를 넘기고 끝낸다. 주 인스턴스가 막 떠서 `instance.json` 을
/// 아직 안 썼을 수 있으므로 짧게 다시 시도한다.
fn hand_off(paths: &Paths) -> Result<(), Box<dyn Error>> {
    let mut last = String::from("instance.json 없음");
    for _ in 0..10 {
        match InstanceInfo::read(&paths.instance_file()) {
            Ok(Some(info)) => match handoff::request_open(&info) {
                Ok(204) => {
                    log::info!("이미 실행 중 — 기존 인스턴스에 열기 전달");
                    return Ok(());
                }
                // 5xx = 주 인스턴스가 받았지만 열기에 실패 — 다시 보내도 같다.
                Ok(code) if code >= 500 => {
                    return Err(format!("기존 인스턴스의 열기 실패: HTTP {code}").into());
                }
                Ok(code) => last = format!("HTTP {code}"),
                Err(e) => last = e.to_string(),
            },
            Ok(None) => last = "instance.json 형식 오류".into(),
            Err(e) => last = e.to_string(),
        }
        std::thread::sleep(Duration::from_millis(300));
    }
    Err(format!("기존 인스턴스에 전달 실패: {last}").into())
}

/// OS 사용자 이름(P7).
fn user_name() -> String {
    std::env::var("USERNAME")
        .or_else(|_| std::env::var("USER"))
        .unwrap_or_else(|_| "?".into())
}

/// PC 이름(P7) — Windows `COMPUTERNAME`, 그 밖은 `hostname` 명령.
fn host_name() -> String {
    if let Ok(name) = std::env::var("COMPUTERNAME") {
        return name;
    }
    std::process::Command::new("hostname")
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "?".into())
}
