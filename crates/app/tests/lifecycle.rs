//! 실행 파일 수명(#1n) — `start` · `open` · `stop` · 트레이 없는 모드 · 부모와 함께 끝남 · 감시 단일화 · 시작 실패.
//! 디버그 실행 파일 + 시험마다 임시 데이터 루트(`COWORKAGENT_DATA_DIR`) — 이 PC 의 실제 인스턴스 · 데이터를 건드리지 않는다.
//! 릴리스 빌드는 그 변수를 읽지 않아 실제 인스턴스를 건드리므로 이 파일은 디버그 빌드에서만 돈다.
#![cfg(debug_assertions)]
mod common;
use an_app::cli::{EXIT_API, EXIT_OK, EXIT_START_FAILED, EXIT_USAGE};
use an_app::control;
use an_app::watchdog::{ENV as SUPERVISED_ENV, PARENT_ENV};
use an_core::instance::{InstanceInfo, InstanceLock, read_pid};
use an_core::paths::{DATA_DIR_ENV, Paths};
use std::path::PathBuf;
use std::process::{Child, Command, Output, Stdio};
use std::time::{Duration, Instant};

const EXE: &str = env!("CARGO_BIN_EXE_coworkAgent");

/// 시험 하나의 데이터 루트 + 대역 부모 프로세스. 끝나면 `stop` · 부모 종료 · 폴더 정리.
struct Env {
    dir: PathBuf,
    paths: Paths,
    parent: Child,
}

impl Env {
    fn new(tag: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("an-life-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        Self {
            paths: Paths::under(&dir),
            dir,
            parent: sleeper(),
        }
    }

    fn command(&self) -> Command {
        let mut c = Command::new(EXE);
        c.env(DATA_DIR_ENV, &self.dir)
            .env_remove(SUPERVISED_ENV)
            .env_remove(PARENT_ENV);
        c
    }

    fn cli(&self, args: &[&str]) -> Output {
        self.command().args(args).output().unwrap()
    }

    fn start(&self) -> Output {
        self.cli(&["start", "--parent", &self.parent.id().to_string()])
    }

    /// 인자 없는 실행(트레이 모드 감시) — 시험이 직접 띄운다.
    fn tray(&self) -> Child {
        self.command()
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap()
    }

    fn app_pid(&self) -> Option<u32> {
        read_pid(&self.paths.instance_pid_file())
    }

    fn running(&self) -> bool {
        InstanceLock::is_held(&self.paths.supervisor_lock_file())
            || InstanceLock::is_held(&self.paths.lock_file())
    }

    fn mode(&self) -> Option<String> {
        control::probe(&self.paths)
    }
}

impl Drop for Env {
    fn drop(&mut self) {
        let _ = self.cli(&["stop"]);
        // stop 이 실패해도(단정 실패 중) 아이콘 보이는 프로세스를 남기지 않는다 — 잡힌 잠금의 PID 만.
        for (lock, pid) in [
            (
                self.paths.supervisor_lock_file(),
                self.paths.supervisor_pid_file(),
            ),
            (self.paths.lock_file(), self.paths.instance_pid_file()),
        ] {
            if InstanceLock::is_held(&lock)
                && let Some(pid) = read_pid(&pid)
            {
                let _ = an_app::proc::kill(pid);
            }
        }
        let _ = self.parent.kill();
        let _ = self.parent.wait();
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

/// 오래 사는 대역 부모 — 죽이면 끝난다.
fn sleeper() -> Child {
    #[cfg(windows)]
    let mut c = {
        let mut c = Command::new("ping");
        c.args(["-n", "600", "127.0.0.1"]);
        c
    };
    #[cfg(unix)]
    let mut c = {
        let mut c = Command::new("sleep");
        c.arg("600");
        c
    };
    c.stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap()
}

fn text(o: &Output) -> (String, String) {
    (
        String::from_utf8_lossy(&o.stdout).trim().to_string(),
        String::from_utf8_lossy(&o.stderr).trim().to_string(),
    )
}

/// `cond` 가 참이 될 때까지(최대 `limit`).
fn until(limit: Duration, mut cond: impl FnMut() -> bool) -> bool {
    let end = Instant::now() + limit;
    while Instant::now() < end {
        if cond() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    cond()
}

/// A1 · A2 · A5 — 꺼진 상태 start → 트레이 없는 모드 준비, 다시 start 는 같은 앱, stop → 모두 끝, 꺼진 상태 stop → 바로 0.
#[test]
fn start_again_stop_and_stop_when_off() {
    let e = Env::new("basic");
    let o = e.start();
    assert_eq!(o.status.code(), Some(EXIT_OK), "{:?}", text(&o));
    assert_eq!(text(&o).0, "시작함");
    assert_eq!(e.mode().as_deref(), Some("headless"));
    let pid = e.app_pid().expect("instance.pid");

    let o = e.start();
    assert_eq!(o.status.code(), Some(EXIT_OK), "{:?}", text(&o));
    assert_eq!(text(&o).0, "이미 실행 중");
    assert_eq!(e.app_pid(), Some(pid), "새로 띄우지 않는다");

    let o = e.cli(&["stop"]);
    assert_eq!(o.status.code(), Some(EXIT_OK), "{:?}", text(&o));
    assert_eq!(text(&o).0, "종료함");
    assert!(!e.running(), "stop 이 돌아오면 감시 · 앱 모두 끝");

    let o = e.cli(&["stop"]);
    assert_eq!(o.status.code(), Some(EXIT_OK), "{:?}", text(&o));
    assert_eq!(text(&o).0, "실행 중이 아님");
}

/// A3 — 부모가 끝나면 트레이 없는 모드의 앱 · 감시가 끝난다.
#[test]
fn parent_exit_ends_headless_app() {
    let mut e = Env::new("parent");
    let o = e.start();
    assert_eq!(o.status.code(), Some(EXIT_OK), "{:?}", text(&o));
    assert!(e.running());
    e.parent.kill().unwrap();
    // Unix 는 거둬야 끝난 것으로 보인다(좀비는 살아 있음으로 보인다).
    e.parent.wait().unwrap();
    assert!(
        until(Duration::from_secs(15), || !e.running()),
        "부모 끝 뒤 앱 · 감시가 남았다"
    );
}

/// 이미 끝난 PID 를 부모로 주면 사용법 오류(2) — 띄우지 않는다.
#[test]
fn dead_parent_is_usage_error() {
    let e = Env::new("dead");
    let mut gone = sleeper();
    let pid = gone.id();
    gone.kill().unwrap();
    gone.wait().unwrap();
    let o = e.cli(&["start", "--parent", &pid.to_string()]);
    assert_eq!(o.status.code(), Some(EXIT_USAGE), "{:?}", text(&o));
    assert!(text(&o).1.contains("가 없다"), "{:?}", text(&o));
    assert!(!e.running());
}

/// A6 — 감시를 동시에 4개 띄우면 하나만 남는다(나머지는 바로 0). CLI start 4개를 동시에 불러도 앱은 하나.
#[test]
fn concurrent_supervisors_and_starts_leave_one() {
    let e = Env::new("concurrent");
    let mut sups: Vec<Child> = (0..4)
        .map(|_| {
            e.command()
                .env(PARENT_ENV, e.parent.id().to_string())
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .unwrap()
        })
        .collect();
    assert!(
        until(Duration::from_secs(30), || e.mode().as_deref()
            == Some("headless")),
        "준비되지 않았다"
    );
    assert!(
        until(Duration::from_secs(10), || {
            sups.iter_mut()
                .map(|c| c.try_wait().unwrap())
                .filter(Option::is_none)
                .count()
                == 1
        }),
        "감시가 하나만 남아야 한다"
    );
    for c in sups.iter_mut() {
        if let Some(status) = c.try_wait().unwrap() {
            assert_eq!(status.code(), Some(0), "보조 감시는 조용히 0");
        }
    }
    let pid = e.app_pid().expect("instance.pid");

    let parent = e.parent.id().to_string();
    let outs: Vec<Output> = std::thread::scope(|s| {
        let hs: Vec<_> = (0..4)
            .map(|_| s.spawn(|| e.cli(&["start", "--parent", &parent])))
            .collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    });
    for o in &outs {
        assert_eq!(o.status.code(), Some(EXIT_OK), "{:?}", text(o));
        assert_eq!(text(o).0, "이미 실행 중");
    }
    assert_eq!(e.app_pid(), Some(pid));

    // 꺼진 상태에서 동시 start 4개 — 앱 하나로 모인다.
    let o = e.cli(&["stop"]);
    assert_eq!(o.status.code(), Some(EXIT_OK), "{:?}", text(&o));
    for c in sups.iter_mut() {
        let _ = c.wait();
    }
    let outs: Vec<Output> = std::thread::scope(|s| {
        let hs: Vec<_> = (0..4)
            .map(|_| s.spawn(|| e.cli(&["start", "--parent", &parent])))
            .collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    });
    for o in &outs {
        assert_eq!(o.status.code(), Some(EXIT_OK), "{:?}", text(o));
    }
    let pid = e.app_pid().expect("instance.pid");
    assert!(until(Duration::from_secs(5), || e.app_pid() == Some(pid)));
    assert_eq!(e.mode().as_deref(), Some("headless"));
}

/// A7 — 트레이 없는 모드의 시작 실패(DB 자리를 폴더로 막음) → start 가 8 과 사유, 남는 프로세스 없음.
#[test]
fn start_failure_exits_8_with_reason() {
    let e = Env::new("fail");
    std::fs::create_dir_all(e.paths.db_file()).unwrap();
    let o = e.start();
    assert_eq!(o.status.code(), Some(EXIT_START_FAILED), "{:?}", text(&o));
    // 사유는 앱이 시험 데이터 루트 안 사유 파일에 남긴 것(실제 인스턴스의 임시 폴더 파일이 아니다).
    let file = e.dir.join("coworkAgent-start-error.txt");
    let reason = std::fs::read_to_string(&file).expect("사유 파일");
    let why = reason
        .lines()
        .find_map(|l| l.strip_prefix("사유: "))
        .expect("사유 줄");
    assert_eq!(text(&o).1, format!("시작 실패 — {why}"));
    assert!(until(Duration::from_secs(5), || !e.running()));
}

/// A2 — 트레이 모드(인자 없는 실행)로 떠 있으면 start 가 끄고 트레이 없는 모드로 다시 띄운다.
#[test]
fn start_switches_tray_to_headless() {
    let e = Env::new("switch");
    let mut tray = e.tray();
    assert!(
        until(Duration::from_secs(30), || e.mode().as_deref()
            == Some("tray")),
        "트레이 모드가 뜨지 않았다"
    );
    let o = e.start();
    assert_eq!(o.status.code(), Some(EXIT_OK), "{:?}", text(&o));
    assert_eq!(text(&o).0, "시작함");
    assert_eq!(e.mode().as_deref(), Some("headless"));
    assert_eq!(
        tray.wait().unwrap().code(),
        Some(0),
        "트레이 모드 감시는 정상 종료"
    );
}

/// A5 · A6 — 트레이 모드 시작 실패 = 감시의 오류 트레이(감시 잠금을 쥔 채 — 하나). 로컬 API 가 없어도 stop 이 끈다.
/// (거듭 실행의 화면 넘기기 실패는 사유 파일을 OS 기본 앱으로 열므로 여기서 부르지 않는다 — 감시 단일은 동시 실행 시험이 본다.)
#[test]
fn error_tray_is_single_and_stoppable() {
    let e = Env::new("errtray");
    std::fs::create_dir_all(e.paths.db_file()).unwrap();
    let mut tray = e.tray();
    let sup = e.paths.supervisor_lock_file();
    assert!(
        until(Duration::from_secs(30), || {
            InstanceLock::is_held(&sup)
                && e.paths.instance_pid_file().exists()
                && !InstanceLock::is_held(&e.paths.lock_file())
        }),
        "오류 트레이 상태가 되지 않았다"
    );
    let o = e.cli(&["stop"]);
    assert_eq!(o.status.code(), Some(EXIT_OK), "{:?}", text(&o));
    assert!(!e.running());
    assert!(tray.wait().is_ok());
}

/// A4 — 떠 있으면 open 은 화면 열기 한 번(가짜 열기 — 실제 브라우저를 열지 않는다).
#[tokio::test]
async fn open_when_running_opens_once() {
    let r = common::start().await;
    let dir = std::env::temp_dir().join(format!("an-life-open-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let paths = Paths::under(&dir);
    InstanceInfo {
        port: r.port,
        token: r.token.clone(),
    }
    .write(&paths.instance_file())
    .unwrap();
    let me = std::process::id().to_string();
    // 떠 있지 않다고 잘못 보더라도 실제 데이터 루트로 띄우지 않게 — 없는 실행 파일.
    let fake_exe = dir.join("없음.exe");
    let (code, out) = tokio::task::spawn_blocking(move || {
        let (mut out, mut err) = (Vec::new(), Vec::new());
        let code = control::run(
            &["open".to_string(), "--parent".into(), me],
            &paths,
            &fake_exe,
            &mut out,
            &mut err,
        );
        (code, String::from_utf8_lossy(&out).trim().to_string())
    })
    .await
    .unwrap();
    assert_eq!((code, out.as_str()), (EXIT_OK, "열었음"));
    assert_eq!(r.opened(), 1);
    let _ = std::fs::remove_dir_all(&dir);
}

/// 종료 API 가 없는 이전 버전(`router_with` — `/api/quit` 없음)이 떠 있으면 stop · start 는 기다리지 않고 바로 4.
#[tokio::test]
async fn old_version_fails_fast() {
    let r = common::start().await;
    let dir = std::env::temp_dir().join(format!("an-life-old-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let paths = Paths::under(&dir);
    InstanceInfo {
        port: r.port,
        token: r.token.clone(),
    }
    .write(&paths.instance_file())
    .unwrap();
    let me = std::process::id().to_string();
    let fake_exe = dir.join("없음.exe");
    let results = tokio::task::spawn_blocking(move || {
        [
            vec!["stop".to_string()],
            vec!["start".into(), "--parent".into(), me],
        ]
        .map(|args| {
            let at = Instant::now();
            let (mut out, mut err) = (Vec::new(), Vec::new());
            let code = control::run(&args, &paths, &fake_exe, &mut out, &mut err);
            (
                code,
                at.elapsed(),
                String::from_utf8_lossy(&err).to_string(),
            )
        })
    })
    .await
    .unwrap();
    for (code, took, err) in results {
        assert_eq!(code, EXIT_API, "{err}");
        assert!(err.contains("이전 버전"), "{err}");
        assert!(took < Duration::from_secs(3), "{took:?}");
    }
    let _ = std::fs::remove_dir_all(&dir);
}
