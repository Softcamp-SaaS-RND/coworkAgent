//! 감시 프로세스(#1e) — 비정상 종료면 다시 띄우고, 거듭 죽으면 멈춘다. 정상 종료면 함께 끝난다.
use an_app::watchdog::{CRASH_WINDOW, Crashes, MAX_CRASHES, Outcome, START_FAILED, supervise};
use std::io;
use std::time::{Duration, Instant};

/// 결과를 차례로 돌려주는 가짜 자식 — 받은 횟수(자식에 넘긴 값)를 모은다.
fn run(results: Vec<io::Result<i32>>) -> (Outcome, Vec<u32>) {
    let mut results = results.into_iter();
    let mut seen = Vec::new();
    let out = supervise(
        |n| {
            seen.push(n);
            results.next().expect("더 띄우지 않아야 한다")
        },
        Duration::ZERO,
    );
    (out, seen)
}

#[test]
fn normal_exit_runs_once() {
    assert_eq!(run(vec![Ok(0)]), (Outcome::Done, vec![0]));
}

#[test]
fn crash_respawns_with_count() {
    assert_eq!(
        run(vec![Ok(1), Ok(-1), Ok(0)]),
        (Outcome::Done, vec![0, 1, 2])
    );
}

#[test]
fn three_crashes_in_window_give_up() {
    let (out, seen) = run((0..MAX_CRASHES).map(|_| Ok(1)).collect());
    assert_eq!(seen.len(), MAX_CRASHES);
    assert!(
        matches!(&out, Outcome::GaveUp(why) if why.contains("3번")),
        "{out:?}"
    );
}

/// 시작 실패(#1n) — 다시 띄우지 않고 바로 멈춘다(비정상 종료 뒤라도).
#[test]
fn start_failed_stops_without_respawn() {
    assert_eq!(
        run(vec![Ok(1), Ok(START_FAILED)]),
        (Outcome::StartFailed, vec![0, 1])
    );
}

#[test]
fn spaced_crashes_continue() {
    let t = Instant::now();
    let at = |s: u64| t + Duration::from_secs(s);
    let mut c = Crashes::default();
    // 60초를 넘겨 떨어진 비정상 종료는 쌓이지 않는다.
    assert!(!c.record(at(0)) && !c.record(at(61)) && !c.record(at(122)));
    // 창 안에 셋째가 들어오면 멈춘다.
    assert!(!c.record(at(150)));
    assert!(c.record(at(122) + CRASH_WINDOW - Duration::from_secs(1)));
}

#[test]
fn spawn_failure_gives_up() {
    let (out, seen) = run(vec![Err(io::Error::other("실행 파일 없음"))]);
    assert_eq!(seen, vec![0]);
    assert!(
        matches!(&out, Outcome::GaveUp(why) if why.contains("실행 파일 없음")),
        "{out:?}"
    );
}

/// 실제 실행 파일 — 인자가 있으면 감시 없이 CLI(사용법 오류 2)로 바로 끝나고, 그 비정상 종료를 감시하면 3번에 멈춘다.
#[test]
fn real_exe_cli_bypasses_and_crashes_stop() {
    let exe = env!("CARGO_BIN_EXE_coworkAgent");
    let started = Instant::now();
    let status = std::process::Command::new(exe)
        .arg("bogus")
        .output()
        .unwrap()
        .status;
    assert_eq!(status.code(), Some(2));
    assert!(
        started.elapsed() < Duration::from_secs(10),
        "감시를 거치지 않는다"
    );
    let mut n = 0;
    let out = supervise(
        |_| {
            n += 1;
            std::process::Command::new(exe)
                .arg("bogus")
                .output()
                .map(|o| o.status.code().unwrap_or(-1))
        },
        Duration::ZERO,
    );
    assert_eq!(n, MAX_CRASHES);
    assert!(matches!(out, Outcome::GaveUp(_)));
}
