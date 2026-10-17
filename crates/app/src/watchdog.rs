//! 감시 프로세스(#1e) — 인자 없는 실행은 앱(트레이 + 수신 · 실행)을 자식으로 띄우고 지켜본다. 자식이 0 으로 끝나면(트레이 "종료" · "열기"
//! 넘기고 끝) 함께 끝나고, 비정상 종료면 다시 띄운다. 짧은 시간에 거듭 죽으면 멈춘다(무한 재시작 없음). 앱이 시작에 실패하면([`START_FAILED`])
//! 다시 띄우지 않는다. 감시는 로거 · DB 를 쓰지 않고, 감시 잠금(`supervisor.lock`, #1n) 하나로 감시가 하나만 돈다.
use an_core::instance::{Acquired, InstanceLock, write_pid};
use an_core::paths::Paths;
use std::collections::VecDeque;
use std::io;
use std::time::{Duration, Instant};

/// 감시가 자식에 붙이는 환경 변수 — 값 = 다시 띄운 횟수(처음 `0`). 없으면 감시 프로세스, 있으면 앱.
pub const ENV: &str = "COWORKAGENT_SUPERVISED";
/// 트레이 없는 모드(#1n) — 값 = 지켜볼 부모 PID. `start` · `open` 이 감시에 붙이고, 감시가 앱에 물려준다(사용자 literal 아님).
pub const PARENT_ENV: &str = "COWORKAGENT_PARENT";
/// 앱의 시작 실패 종료 코드 — 사유 파일을 쓴 뒤. 감시는 다시 띄우지 않는다.
pub const START_FAILED: i32 = 3;
/// 이 횟수만큼 [`CRASH_WINDOW`] 안에 비정상 종료되면 멈춘다.
pub const MAX_CRASHES: usize = 3;
pub const CRASH_WINDOW: Duration = Duration::from_secs(60);
/// 비정상 종료 뒤 다시 띄우기까지.
pub const RESTART_DELAY: Duration = Duration::from_secs(1);

#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    /// 자식이 정상 종료 — 감시도 끝.
    Done,
    /// 자식이 시작에 실패([`START_FAILED`]) — 사유는 앱이 쓴 사유 파일에.
    StartFailed,
    /// 다시 띄우기를 멈췄다 — 사유.
    GaveUp(String),
}

/// 최근 비정상 종료 시각.
#[derive(Default)]
pub struct Crashes(VecDeque<Instant>);

impl Crashes {
    /// `at` 의 비정상 종료를 적고, [`CRASH_WINDOW`] 안에 [`MAX_CRASHES`] 번이 찼으면 true(멈춤).
    pub fn record(&mut self, at: Instant) -> bool {
        while self
            .0
            .front()
            .is_some_and(|&t| at.saturating_duration_since(t) >= CRASH_WINDOW)
        {
            self.0.pop_front();
        }
        self.0.push_back(at);
        self.0.len() >= MAX_CRASHES
    }
}

/// `spawn(다시 띄운 횟수)` = 자식을 띄워 끝날 때까지 기다리고 종료 코드. 끝날 때까지 돈다.
pub fn supervise(mut spawn: impl FnMut(u32) -> io::Result<i32>, delay: Duration) -> Outcome {
    let mut crashes = Crashes::default();
    let mut n = 0;
    loop {
        match spawn(n) {
            Ok(0) => return Outcome::Done,
            Ok(START_FAILED) => return Outcome::StartFailed,
            Ok(_) => {
                if crashes.record(Instant::now()) {
                    return Outcome::GaveUp(format!(
                        "앱이 {}초 안에 {MAX_CRASHES}번 비정상 종료 — 다시 띄우지 않습니다",
                        CRASH_WINDOW.as_secs()
                    ));
                }
            }
            Err(e) => return Outcome::GaveUp(format!("앱을 띄우지 못했다: {e}")),
        }
        std::thread::sleep(delay);
        n += 1;
    }
}

/// 실제 자식 — 이 실행 파일을 인자 없이, [`ENV`] = `restarts` 로([`PARENT_ENV`] 는 물려받는다). 신호로 끝나 코드가 없으면 -1.
/// [`START_FAILED`] 는 MSVC `abort()` 의 코드와 같다 — 이 자식(PID)이 사유 파일을 남겼을 때만 시작 실패로 넘기고, 아니면 비정상 종료(-1)로 다시 띄운다.
/// 자식 핸들을 쥔 동안 확인한다 — 그 사이 PID 는 재사용되지 않는다.
pub fn spawn_app(restarts: u32) -> io::Result<i32> {
    let mut child = std::process::Command::new(std::env::current_exe()?)
        .env(ENV, restarts.to_string())
        .spawn()?;
    let status = child.wait()?;
    Ok(match status.code() {
        Some(START_FAILED) if !crate::tray::start_error_by(child.id()) => -1,
        code => code.unwrap_or(-1),
    })
}

/// 감시 잠금을 잡고 PID 를 적는다 — `None` = 이미 다른 감시가 돈다. 잠금은 프로세스가 끝나면 OS 가 푼다.
pub fn acquire(paths: &Paths) -> io::Result<Option<InstanceLock>> {
    std::fs::create_dir_all(paths.root())?;
    match InstanceLock::acquire(&paths.supervisor_lock_file())? {
        Acquired::Primary(lock) => {
            write_pid(&paths.supervisor_pid_file())?;
            Ok(Some(lock))
        }
        Acquired::Secondary => Ok(None),
    }
}

/// 트레이 없는 모드면 지켜볼 부모 PID(숫자가 아니면 트레이 모드로 본다).
pub fn parent() -> Option<u32> {
    std::env::var(PARENT_ENV).ok()?.parse().ok()
}

/// 이 프로세스가 감시가 띄운 앱이면 다시 띄운 횟수, 아니면(감시 프로세스가 될 차례) `None`.
pub fn restarts() -> Option<u32> {
    std::env::var(ENV).ok().map(|v| v.parse().unwrap_or(0))
}
