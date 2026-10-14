//! 감시 프로세스(#1e) — 인자 없는 실행은 앱(트레이 + 수신 · 실행)을 자식으로 띄우고 지켜본다. 자식이 0 으로 끝나면(트레이 "종료" · "열기"
//! 넘기고 끝) 함께 끝나고, 비정상 종료면 다시 띄운다. 짧은 시간에 거듭 죽으면 멈춘다(무한 재시작 없음). 감시는 로거 · DB · 잠금을 쓰지 않는다.
use std::collections::VecDeque;
use std::io;
use std::time::{Duration, Instant};

/// 감시가 자식에 붙이는 환경 변수 — 값 = 다시 띄운 횟수(처음 `0`). 없으면 감시 프로세스, 있으면 앱.
pub const ENV: &str = "COWORKAGENT_SUPERVISED";
/// 이 횟수만큼 [`CRASH_WINDOW`] 안에 비정상 종료되면 멈춘다.
pub const MAX_CRASHES: usize = 3;
pub const CRASH_WINDOW: Duration = Duration::from_secs(60);
/// 비정상 종료 뒤 다시 띄우기까지.
pub const RESTART_DELAY: Duration = Duration::from_secs(1);

#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    /// 자식이 정상 종료 — 감시도 끝.
    Done,
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

/// `spawn(다시 띄운 횟수)` = 자식을 띄워 끝날 때까지 기다리고 정상 종료 여부. 끝날 때까지 돈다.
pub fn supervise(mut spawn: impl FnMut(u32) -> io::Result<bool>, delay: Duration) -> Outcome {
    let mut crashes = Crashes::default();
    let mut n = 0;
    loop {
        match spawn(n) {
            Ok(true) => return Outcome::Done,
            Ok(false) => {
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

/// 실제 자식 — 이 실행 파일을 인자 없이, [`ENV`] = `restarts` 로.
pub fn spawn_app(restarts: u32) -> io::Result<bool> {
    let status = std::process::Command::new(std::env::current_exe()?)
        .env(ENV, restarts.to_string())
        .status()?;
    Ok(status.success())
}

/// 이 프로세스가 감시가 띄운 앱이면 다시 띄운 횟수, 아니면(감시 프로세스가 될 차례) `None`.
pub fn restarts() -> Option<u32> {
    std::env::var(ENV).ok().map(|v| v.parse().unwrap_or(0))
}
