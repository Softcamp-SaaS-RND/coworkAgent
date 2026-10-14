//! 자동 시작(#1e) — 등록 · 경로 고침 · 해제 · LaunchAgent 내용 · 설정 API(끄면 지운 뒤 저장, 지우기 실패면 저장 안 함).
mod common;
use an_app::autostart;
use an_app::notify::{self, Notifier};
use an_app::server;
use std::io;
use std::path::Path;
use std::sync::{Arc, Mutex};

/// 실제 Run 키를 건드리지 않는 시험 키 — 끝나면 통째로 지운다.
#[cfg(windows)]
#[test]
fn register_fixes_path_and_unregisters() {
    let root = format!(r"Software\coworkAgent-test-{}", std::process::id());
    let key = format!(r"{root}\Run");
    let (a, b) = (
        Path::new(r"C:\앱 폴더\coworkAgent.exe"),
        Path::new(r"D:\옮김\coworkAgent.exe"),
    );
    assert_eq!(autostart::registered(&key).unwrap(), None);
    assert!(autostart::register(&key, a).unwrap(), "처음 — 씀");
    assert_eq!(
        autostart::registered(&key).unwrap().as_deref(),
        Some(r#""C:\앱 폴더\coworkAgent.exe""#)
    );
    assert!(
        !autostart::register(&key, a).unwrap(),
        "같은 경로 — 쓰지 않음"
    );
    assert!(autostart::register(&key, b).unwrap(), "옮긴 경로 — 고침");
    assert_eq!(
        autostart::registered(&key).unwrap().as_deref(),
        Some(r#""D:\옮김\coworkAgent.exe""#)
    );
    autostart::unregister(&key).unwrap();
    autostart::unregister(&key).unwrap(); // 없어도 성공
    assert_eq!(autostart::registered(&key).unwrap(), None);
    let _ = std::process::Command::new("reg")
        .args(["delete", &format!(r"HKCU\{root}"), "/f"])
        .output();
}

#[test]
fn launch_agent_plist_escapes_path() {
    let p = autostart::launch_agent_plist(Path::new("/Users/me/a&b <c>/coworkAgent"));
    assert!(p.contains("<string>com.coworkagent</string>"), "{p}");
    assert!(
        p.contains("<string>/Users/me/a&amp;b &lt;c&gt;/coworkAgent</string>"),
        "{p}"
    );
    assert!(p.contains("<key>RunAtLoad</key>\n  <true/>"), "{p}");
    assert!(
        !p.contains("KeepAlive"),
        "다시 띄우기는 감시 프로세스가: {p}"
    );
}

fn db_path(tag: &str) -> std::path::PathBuf {
    let p = std::env::temp_dir().join(format!("an-app-autostart-{tag}-{}.db", std::process::id()));
    let _ = std::fs::remove_file(&p);
    p
}

async fn web(n: Arc<Notifier>) -> (u16, (&'static str, String)) {
    let listener = server::bind().await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let token = server::new_token();
    let status = server::Status {
        version: "t".into(),
        data_dir: "d".into(),
        db: "ok".into(),
    };
    let app = server::router_with(port, token.clone(), status, || Ok(()), notify::routes(n));
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    (port, ("Authorization", format!("Bearer {token}")))
}

async fn get(port: u16, auth: &(&'static str, String)) -> serde_json::Value {
    let (code, raw) = common::send(
        port,
        "GET",
        "/api/settings",
        &[common::host(port), auth.clone()],
    )
    .await;
    assert_eq!(code, 200, "{raw}");
    serde_json::from_str(raw.split("\r\n\r\n").nth(1).unwrap()).unwrap()
}

async fn put(port: u16, auth: &(&'static str, String), body: &str) -> u16 {
    let headers = [
        common::host(port),
        common::origin(port),
        auth.clone(),
        ("Content-Type", "application/json".to_string()),
    ];
    common::send_body(port, "PUT", "/api/settings", &headers, body)
        .await
        .0
}

#[tokio::test]
async fn setting_default_on_and_off_persists() {
    let path = db_path("toggle");
    let n = Notifier::new(an_core::db::open(&path).unwrap(), |_| {}).unwrap();
    let calls = Arc::new(Mutex::new(Vec::new()));
    let seen = calls.clone();
    n.set_autostart_hook(move |on| {
        seen.lock().unwrap().push(on);
        Ok(())
    });
    let (port, auth) = web(n).await;
    assert_eq!(get(port, &auth).await["autostart"], true, "D13 기본 켜짐");
    assert_eq!(put(port, &auth, r#"{"autostart":false}"#).await, 200);
    assert_eq!(*calls.lock().unwrap(), vec![false]);
    assert_eq!(get(port, &auth).await["autostart"], false);
    // 다시 열어도 꺼짐 — 앱 시작 때 등록하지 않는 근거.
    assert!(!an_core::settings::autostart(&an_core::db::open(&path).unwrap()).unwrap());
}

#[tokio::test]
async fn hook_failure_500_and_not_saved() {
    let n = Notifier::new(an_core::db::open(&db_path("fail")).unwrap(), |_| {}).unwrap();
    n.set_autostart_hook(|_| Err(io::Error::other("레지스트리 거부")));
    let (port, auth) = web(n).await;
    assert_eq!(put(port, &auth, r#"{"autostart":false}"#).await, 500);
    assert_eq!(get(port, &auth).await["autostart"], true);
}

/// 순서 — 켜기는 OS 등록이 저장보다 먼저, 끄기는 저장 뒤에 OS 지우기(라운드 2 high). 훅이 부를 때의 저장 값을 보면 어느 지점에서 중단돼도
/// "저장 켜짐 · OS 꺼짐" 이 남지 않는다: 켜기 훅 때 저장은 아직 꺼짐 · 끄기 훅 때 저장은 이미 꺼짐.
#[tokio::test]
async fn toggle_order_never_leaves_saved_on_os_off() {
    let path = db_path("order");
    let n = Notifier::new(an_core::db::open(&path).unwrap(), |_| {}).unwrap();
    let calls = Arc::new(Mutex::new(Vec::new()));
    let (seen, at) = (calls.clone(), path.clone());
    n.set_autostart_hook(move |on| {
        let saved = an_core::settings::autostart(&an_core::db::open(&at).unwrap()).unwrap();
        seen.lock().unwrap().push((on, saved));
        Ok(())
    });
    let (port, auth) = web(n).await;
    assert_eq!(put(port, &auth, r#"{"autostart":false}"#).await, 200);
    assert_eq!(put(port, &auth, r#"{"autostart":true}"#).await, 200);
    assert_eq!(*calls.lock().unwrap(), vec![(false, false), (true, false)]);
}

/// 켜기 뒤 저장이 실패하면 등록을 저장된 값으로 되돌린다 · 끄기는 저장이 실패하면 OS 를 건드리지 않는다(라운드 1 high).
#[tokio::test]
async fn save_failure_restores_registration() {
    let path = db_path("undo");
    let n = Notifier::new(an_core::db::open(&path).unwrap(), |_| {}).unwrap();
    an_core::settings::set(&an_core::db::open(&path).unwrap(), "autostart", "0").unwrap();
    let calls = Arc::new(Mutex::new(Vec::new()));
    let (seen, at) = (calls.clone(), path.clone());
    n.set_autostart_hook(move |on| {
        let mut c = seen.lock().unwrap();
        if c.is_empty() {
            // 첫 호출(켜기) 뒤 저장이 실패하게 — 설정 표를 없앤다.
            an_core::db::open(&at)
                .unwrap()
                .execute("DROP TABLE settings", [])
                .unwrap();
        }
        c.push(on);
        Ok(())
    });
    let (port, auth) = web(n).await;
    assert_eq!(put(port, &auth, r#"{"autostart":true}"#).await, 500);
    assert_eq!(
        *calls.lock().unwrap(),
        vec![true, false],
        "켜기 → 저장 실패 → 꺼짐으로 되돌림"
    );
    assert_eq!(put(port, &auth, r#"{"autostart":false}"#).await, 500);
    assert_eq!(
        calls.lock().unwrap().len(),
        2,
        "끄기는 저장 실패면 OS 를 건드리지 않는다"
    );
}

/// 동시 요청 — 마지막 등록과 저장된 값이 같다(훅 · 저장이 한 순서).
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_puts_end_consistent() {
    let path = db_path("race");
    let n = Notifier::new(an_core::db::open(&path).unwrap(), |_| {}).unwrap();
    let calls = Arc::new(Mutex::new(Vec::new()));
    let seen = calls.clone();
    n.set_autostart_hook(move |on| {
        std::thread::sleep(std::time::Duration::from_millis(30));
        seen.lock().unwrap().push(on);
        Ok(())
    });
    let (port, auth) = web(n).await;
    let reqs = (0..6).map(|i| {
        let auth = auth.clone();
        async move {
            let body = format!(r#"{{"autostart":{}}}"#, i % 2 == 0);
            put(port, &auth, &body).await
        }
    });
    for code in futures_util::future::join_all(reqs).await {
        assert_eq!(code, 200);
    }
    let last = *calls.lock().unwrap().last().unwrap();
    assert_eq!(
        an_core::settings::autostart(&an_core::db::open(&path).unwrap()).unwrap(),
        last
    );
}

/// LaunchAgent 교체는 통째로 — 쓰다 실패하면 원래 파일 그대로(라운드 1 medium).
#[test]
fn write_atomic_keeps_old_on_failure() {
    let dir = std::env::temp_dir().join(format!("an-app-atomic-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("com.coworkagent.plist");
    autostart::write_atomic(&file, "처음").unwrap();
    autostart::write_atomic(&file, "바꿈").unwrap();
    assert_eq!(std::fs::read_to_string(&file).unwrap(), "바꿈");
    assert!(
        !dir.join("com.coworkagent.plist.tmp").exists(),
        "임시 파일 남지 않음"
    );
    // 임시 파일 자리를 폴더로 막아 쓰기를 실패시킨다.
    std::fs::create_dir(dir.join("com.coworkagent.plist.tmp")).unwrap();
    assert!(autostart::write_atomic(&file, "깨짐").is_err());
    assert_eq!(std::fs::read_to_string(&file).unwrap(), "바꿈");
}
