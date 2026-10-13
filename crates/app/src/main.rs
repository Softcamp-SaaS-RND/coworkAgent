// Windows 릴리스는 콘솔 창 없이(트레이 앱). 디버그 빌드는 콘솔을 남긴다.
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use an_agent::chat::Chat;
use an_agent::runner::{find_claude, run_claude};
use an_app::notify::{self, Notifier};
use an_app::server::{self, Status};
use an_app::tray::{Attention, Tray, write_start_error};
use an_app::{browser, chat_api, cli, handoff, peers_api};
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
    // Unix: 이 프로세스가 만드는 파일 · 폴더는 소유자만(0600 · 0700) — DB · 사본 · 회전 로그가 만든 뒤 교정을 기다리지 않게(1a A1).
    #[cfg(unix)]
    // SAFETY: 인자 하나짜리 프로세스 설정 호출, 포인터 없음.
    unsafe {
        libc::umask(0o077);
    }
    // 이름만으로 부르는 `LoadLibrary`(tao · muda 의 uxtheme.dll 등)도 System32 에서만 — 실행 파일 폴더에 심은 DLL 을 올리지 않는다
    // (정적 import 는 `.cargo/config.toml` 의 DependentLoadFlags, 1a A13). 어떤 초기화보다 먼저.
    #[cfg(windows)]
    // SAFETY: 인자 하나짜리 프로세스 설정 호출, 포인터 없음.
    unsafe {
        use windows_sys::Win32::System::LibraryLoader::{
            LOAD_LIBRARY_SEARCH_SYSTEM32, SetDefaultDllDirectories,
        };
        SetDefaultDllDirectories(LOAD_LIBRARY_SEARCH_SYSTEM32);
    }
    // AI 에이전트용 CLI — 트레이 · 잠금 · 로거를 거치지 않는다(주 인스턴스가 되지 않는다). 떠 있는 인스턴스에 요청만 한다.
    // `args()` 는 UTF-8 이 아닌 인자에서 패닉한다 — 트레이 시작까지 막지 않게 손실 변환.
    let args: Vec<String> = std::env::args_os()
        .skip(1)
        .map(|a| a.to_string_lossy().into_owned())
        .collect();
    // 인자가 있으면 CLI 다 — 트레이는 인자 없이만 뜬다. 모르는 명령도 트레이로 새지 않고 사용법 오류(2)로 끝난다.
    if !args.is_empty() {
        std::process::exit(cli::main(&args));
    }
    let (tray, attention) = Tray::new();
    match start(attention) {
        Ok(Some(app)) => tray.run(app.url, app.note, app.on_seen, app.on_quit),
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
    /// 트레이 툴팁 덧붙임 — 피어 포트를 남이 잡았을 때.
    note: Option<String>,
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
    // 주 인스턴스만 — 핸드오프로 뜨는 두 번째 실행이 매번 트리 전체에 권한을 다시 퍼뜨리지 않게.
    if let Err(e) = paths.restrict() {
        log::warn!("데이터 폴더 권한을 사용자 전용으로 맞추지 못했다: {e}");
    }

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
    let (peer_listener, peer_holder) = runtime.block_on(node::bind_peer_port())?;
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
    if let Some(h) = peer_holder {
        node.set_peer_holder(h);
    }
    let note = node.peer_port_note();
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
    // 시작 알림(#1c) — 페어링된 상대마다 Ping 한 번(바뀐 주소 알림 · 집중). 주기 확인은 없다 — 접속 시각은 통신할 때만.
    runtime.spawn(node.clone().announce());

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
        note,
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
