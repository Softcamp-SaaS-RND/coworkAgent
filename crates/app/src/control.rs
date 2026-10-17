//! 부모 툴 하위 메뉴용 CLI(#1n) — `start` · `open` · `stop`. 다른 CLI 와 달리 꺼져 있어도 동작한다(handshake 없음).
//! `start` · `open` 은 실행 파일을 인자 없이 트레이 없는 모드(감시 변수 [`PARENT_ENV`] = 부모 PID)로 띄우고 받을 준비를 기다린다.
//! `stop` 은 로컬 API 종료(`POST /api/quit`) → 감시 · 앱 잠금이 풀릴 때까지 기다린다. 응답이 없으면 잡힌 잠금의 PID 를 강제 종료한다.
use crate::cli::{
    EXIT_API, EXIT_OK, EXIT_START_FAILED, EXIT_TIMEOUT, EXIT_USAGE, USAGE, printable,
};
use crate::handoff::local_request;
use crate::proc;
use crate::tray::read_start_error;
use crate::watchdog::{PARENT_ENV, START_FAILED};
use an_core::instance::{InstanceInfo, InstanceLock, read_pid};
use an_core::paths::Paths;
use std::io::Write;
use std::path::Path;
use std::process::Child;
use std::time::{Duration, Instant};

/// `start` 가 준비를 기다리는 상한.
pub const START_LIMIT: Duration = Duration::from_secs(30);
/// `stop` 이 끝나기를 기다리는 상한.
pub const STOP_LIMIT: Duration = Duration::from_secs(10);
/// 응답 없는데 잠금이 잡혀 있을 때(시작 중 · 오류 트레이) 응답을 기다리는 시간 — 지나면 강제 종료.
pub const UNRESPONSIVE_WAIT: Duration = Duration::from_secs(5);
/// 상태 확인 간격.
const POLL: Duration = Duration::from_millis(200);
/// 강제 종료 뒤 잠금이 풀린 채 머무는지 보는 시간.
const SETTLE: Duration = Duration::from_millis(300);
/// 상태 질의 한 번의 상한.
const PROBE_DEADLINE: Duration = Duration::from_secs(1);

/// 첫 인자가 이 모듈의 명령인가.
pub fn is_command(arg: &str) -> bool {
    matches!(arg, "start" | "open" | "stop")
}

/// 실패 — 종료 코드 + 사유.
struct Fail(i32, String);

/// `args` = 하위 명령부터. `exe` = 띄울 실행 파일(운영은 자기 자신). 결과는 `out`, 사유는 `err`.
pub fn run(
    args: &[String],
    paths: &Paths,
    exe: &Path,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> i32 {
    let result = parse(args).and_then(|(cmd, parent)| match cmd.as_str() {
        "start" => launch(paths, exe, parent),
        "open" => open(paths, exe, parent),
        _ => stop(paths),
    });
    match result {
        Ok(msg) => {
            let _ = writeln!(out, "{msg}");
            EXIT_OK
        }
        Err(Fail(code, msg)) => {
            let _ = writeln!(err, "{}", printable(&msg));
            code
        }
    }
}

/// 명령과 부모 PID(`stop` 은 없음). 부모 = `--parent <PID>` 또는 이 CLI 를 부른 프로세스.
fn parse(args: &[String]) -> Result<(String, u32), Fail> {
    let bad = |msg: &str| Fail(EXIT_USAGE, format!("{msg}\n{USAGE}"));
    let Some(cmd) = args.first().filter(|c| is_command(c)) else {
        return Err(bad("모르는 명령"));
    };
    let mut parent = None;
    let mut it = args[1..].iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--parent" if cmd != "stop" => {
                let v = it.next().ok_or_else(|| bad("--parent 에 PID 가 없다"))?;
                parent = Some(
                    v.parse::<u32>()
                        .ok()
                        .filter(|&p| p > 0)
                        .ok_or_else(|| bad("--parent 는 숫자 PID"))?,
                );
            }
            _ => return Err(bad(&format!("{cmd} 에 쓸 수 없는 인자 {a}"))),
        }
    }
    if cmd == "stop" {
        return Ok((cmd.clone(), 0));
    }
    let parent = match parent {
        Some(pid) => pid,
        None => proc::parent_pid()
            .ok_or_else(|| bad("부모 프로세스를 찾지 못했다 — --parent 로 지정"))?,
    };
    if !proc::alive(parent) {
        return Err(Fail(EXIT_USAGE, format!("부모 프로세스 {parent} 가 없다")));
    }
    Ok((cmd.clone(), parent))
}

/// 떠 있는 인스턴스의 모드(`"tray"` · `"headless"`) — 응답이 없으면 `None`.
pub fn probe(paths: &Paths) -> Option<String> {
    let info = InstanceInfo::read(&paths.instance_file()).ok()??;
    let (code, body) = local_request(&info, "GET", "/api/status", None, PROBE_DEADLINE).ok()?;
    if code != 200 {
        return None;
    }
    let v: serde_json::Value = serde_json::from_str(&body).ok()?;
    Some(v["mode"].as_str().unwrap_or("tray").to_string())
}

/// 감시 · 앱 잠금 중 하나라도 잡혀 있나.
fn held(paths: &Paths) -> bool {
    InstanceLock::is_held(&paths.supervisor_lock_file())
        || InstanceLock::is_held(&paths.lock_file())
}

/// `start` — 트레이 없는 모드로 떠 있게 한다. 트레이 모드면 끄고 다시 띄운다. 겹친 `start` 도 이 되풀이로 풀린다(감시 잠금이 하나만 남긴다).
fn launch(paths: &Paths, exe: &Path, parent: u32) -> Result<String, Fail> {
    let until = Instant::now() + START_LIMIT;
    let mut child: Option<Child> = None;
    let mut spawned = false;
    loop {
        if Instant::now() >= until {
            return Err(if held(paths) {
                Fail(
                    EXIT_TIMEOUT,
                    "실행 중이지만 응답이 없다 — stop 뒤 다시".into(),
                )
            } else {
                Fail(
                    EXIT_START_FAILED,
                    format!(
                        "시작 실패 — {}초 안에 준비되지 않았다",
                        START_LIMIT.as_secs()
                    ),
                )
            });
        }
        match probe(paths).as_deref() {
            Some("headless") => {
                return Ok(if spawned {
                    "시작함"
                } else {
                    "이미 실행 중"
                }
                .into());
            }
            Some(_) => {
                quit(paths, until.min(Instant::now() + STOP_LIMIT))?;
                continue;
            }
            None => {}
        }
        if let Some(c) = &mut child
            && let Ok(Some(status)) = c.try_wait()
        {
            child = None;
            match status.code() {
                // 0 = 이미 다른 감시가 돈다(겹친 start) · 앱이 정상 종료 — 잠금이 풀려 있으면 다시 띄운다.
                Some(0) => {}
                Some(START_FAILED) => return Err(start_failed()),
                code => {
                    return Err(Fail(
                        EXIT_START_FAILED,
                        format!("시작 실패 — 감시 프로세스가 코드 {code:?} 로 끝났다"),
                    ));
                }
            }
        }
        if child.is_none() && !held(paths) {
            let c = proc::spawn_detached(exe, [(PARENT_ENV, parent.to_string())])
                .map_err(|e| Fail(EXIT_START_FAILED, format!("시작 실패 — 띄우지 못했다: {e}")))?;
            child = Some(c);
            spawned = true;
        }
        std::thread::sleep(POLL);
    }
}

/// 앱이 남긴 사유 파일로 시작 실패를 알린다.
fn start_failed() -> Fail {
    let why = read_start_error().unwrap_or_else(|| "사유 파일 없음".into());
    Fail(EXIT_START_FAILED, format!("시작 실패 — {why}"))
}

/// `open` — 떠 있으면(모드 무관) 화면을 열고, 꺼져 있으면 트레이 없이 띄운 뒤 연다.
fn open(paths: &Paths, exe: &Path, parent: u32) -> Result<String, Fail> {
    if probe(paths).is_none() {
        launch(paths, exe, parent)?;
    }
    let info = InstanceInfo::read(&paths.instance_file())
        .ok()
        .flatten()
        .ok_or_else(|| Fail(EXIT_API, "instance.json 을 읽지 못했다".into()))?;
    match local_request(
        &info,
        "POST",
        "/api/open",
        None,
        crate::handoff::HANDOFF_DEADLINE,
    ) {
        Ok((204, _)) => Ok("열었음".into()),
        Ok((code, _)) => Err(Fail(EXIT_API, format!("화면 열기 실패: HTTP {code}"))),
        Err(e) => Err(Fail(EXIT_API, format!("화면 열기 실패: {e}"))),
    }
}

/// `stop` — 두 모드 · 오류 트레이 모두 끈다. 전체 상한 [`STOP_LIMIT`] 하나(응답 기다림 · 강제 종료 · 잠금 풀림 대기가 나눠 쓴다).
fn stop(paths: &Paths) -> Result<String, Fail> {
    let until = Instant::now() + STOP_LIMIT;
    if probe(paths).is_some() {
        quit(paths, until)?;
        return Ok("종료함".into());
    }
    if !held(paths) {
        return Ok("실행 중이 아님".into());
    }
    // 응답 없음 + 잠금 잡힘 — 시작 중이면 곧 답한다. 오류 트레이 · 멈춘 앱은 답하지 않는다.
    let answer_until = Instant::now() + UNRESPONSIVE_WAIT;
    while Instant::now() < answer_until {
        if probe(paths).is_some() {
            quit(paths, until)?;
            return Ok("종료함".into());
        }
        if !held(paths) {
            return Ok("종료함".into());
        }
        std::thread::sleep(POLL);
    }
    loop {
        // 감시를 먼저 — 자식 앱까지 끊어 다시 띄우지 않게. 잠금이 잡혀 있을 때만(남은 PID 파일이 다른 프로세스를 가리킬 수 있다).
        for (lock, pid) in [
            (paths.supervisor_lock_file(), paths.supervisor_pid_file()),
            (paths.lock_file(), paths.instance_pid_file()),
        ] {
            if InstanceLock::is_held(&lock)
                && let Some(pid) = read_pid(&pid)
                && let Err(e) = proc::kill_tree(pid)
            {
                log::warn!("강제 종료 실패(PID {pid}): {e}");
            }
        }
        wait_released(paths, until)?;
        // 풀린 채 머무는지 한 번 더(상한 안에서) — 끊기 직전에 뜬 앱이 잠금을 늦게 잡았으면 다시 끊는다.
        std::thread::sleep(SETTLE.min(until.saturating_duration_since(Instant::now())));
        if !held(paths) {
            return Ok("종료함".into());
        }
    }
}

/// 로컬 API 로 끄고 두 잠금이 풀릴 때까지(`until` 까지) 기다린다.
fn quit(paths: &Paths, until: Instant) -> Result<(), Fail> {
    if let Ok(Some(info)) = InstanceInfo::read(&paths.instance_file()) {
        // 거절 · 끊김이어도 잠금 풀림으로 판정한다(이미 끝나는 중일 수 있다). 404 = 종료 API 가 없는 이전 버전 — 기다려도 끝나지 않는다.
        if let Ok((404, _)) = local_request(&info, "POST", "/api/quit", None, PROBE_DEADLINE) {
            return Err(Fail(
                EXIT_API,
                "떠 있는 앱이 종료 명령이 없는 이전 버전이다 — 트레이 \"종료\" 로 끈 뒤 다시"
                    .into(),
            ));
        }
    }
    wait_released(paths, until)
}

fn wait_released(paths: &Paths, until: Instant) -> Result<(), Fail> {
    while held(paths) {
        if Instant::now() >= until {
            return Err(Fail(
                EXIT_TIMEOUT,
                format!("{}초 안에 끝나지 않았다", STOP_LIMIT.as_secs()),
            ));
        }
        std::thread::sleep(POLL);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{Fail, parse};
    use crate::cli::EXIT_USAGE;

    fn args(a: &[&str]) -> Vec<String> {
        a.iter().map(|s| s.to_string()).collect()
    }

    fn code(r: Result<(String, u32), Fail>) -> Option<i32> {
        r.err().map(|Fail(c, _)| c)
    }

    #[test]
    fn parse_parent_and_options() {
        let me = std::process::id();
        let (cmd, parent) = parse(&args(&["start", "--parent", &me.to_string()]))
            .ok()
            .unwrap();
        assert_eq!((cmd.as_str(), parent), ("start", me));
        // 기본 = 부른 프로세스(시험 하네스의 부모 — 살아 있다).
        assert!(parse(&args(&["open"])).is_ok());
        assert_eq!(parse(&args(&["stop"])).ok().unwrap().0, "stop");
        for bad in [
            vec!["start", "--parent"],
            vec!["start", "--parent", "x"],
            vec!["start", "--json"],
            vec!["stop", "--parent", "1"],
            vec!["stop", "now"],
            vec!["restart"],
        ] {
            assert_eq!(code(parse(&args(&bad))), Some(EXIT_USAGE), "{bad:?}");
        }
    }

    /// 끝난 PID 를 부모로 주면 사용법 오류(2) — 띄우지 않는다.
    #[test]
    fn dead_parent_is_usage_error() {
        let mut child = std::process::Command::new(env!("CARGO"))
            .arg("--version")
            .stdout(std::process::Stdio::null())
            .spawn()
            .unwrap();
        let pid = child.id();
        child.wait().unwrap();
        let r = parse(&args(&["start", "--parent", &pid.to_string()]));
        assert!(
            matches!(&r, Err(Fail(EXIT_USAGE, m)) if m.contains("가 없다")),
            "{:?}",
            r.as_ref().err().map(|f| &f.1)
        );
    }
}
