//! 프로세스 다루기(#1n) — 부모 PID · 살아 있음 · 부모 끝 지켜보기 · 강제 종료 · 떼어 띄우기. 두 OS 같은 동작(D30): Windows Toolhelp ·
//! `OpenProcess` ↔ Unix `getppid` · `kill`.
use std::ffi::OsStr;
use std::io;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

/// 부모 끝 폴링 간격(Unix · Windows 대기 핸들을 못 열었을 때).
pub const POLL: Duration = Duration::from_secs(1);

/// 이 프로세스의 부모 PID.
#[cfg(windows)]
pub fn parent_pid() -> Option<u32> {
    let me = std::process::id();
    processes()
        .into_iter()
        .find_map(|(pid, parent)| (pid == me).then_some(parent))
}

/// 지금 프로세스 목록 — (PID, 부모 PID). 부모가 끝나도 부모 PID 는 그대로 남는다.
#[cfg(windows)]
fn processes() -> Vec<(u32, u32)> {
    use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW,
        TH32CS_SNAPPROCESS,
    };
    let mut out = Vec::new();
    // SAFETY: 스냅숏 핸들은 이 함수 안에서만 쓰고 닫는다. 항목 버퍼는 dwSize 를 채운 지역 값.
    unsafe {
        let snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snap == INVALID_HANDLE_VALUE {
            return out;
        }
        let mut entry: PROCESSENTRY32W = std::mem::zeroed();
        entry.dwSize = size_of::<PROCESSENTRY32W>() as u32;
        let mut ok = Process32FirstW(snap, &mut entry) != 0;
        while ok {
            out.push((entry.th32ProcessID, entry.th32ParentProcessID));
            ok = Process32NextW(snap, &mut entry) != 0;
        }
        CloseHandle(snap);
    }
    out
}

#[cfg(unix)]
pub fn parent_pid() -> Option<u32> {
    // SAFETY: 인자 없는 조회.
    Some(unsafe { libc::getppid() } as u32)
}

/// 프로세스가 살아 있나 — 권한이 없어 못 여는 것은 살아 있음으로 본다.
#[cfg(windows)]
pub fn alive(pid: u32) -> bool {
    use windows_sys::Win32::Foundation::{
        CloseHandle, ERROR_ACCESS_DENIED, GetLastError, STILL_ACTIVE,
    };
    use windows_sys::Win32::System::Threading::{
        GetExitCodeProcess, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
    };
    // SAFETY: 연 핸들은 이 함수 안에서 닫는다. 종료 코드는 지역 변수로 받는다.
    unsafe {
        let h = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if h.is_null() {
            return GetLastError() == ERROR_ACCESS_DENIED;
        }
        let mut code = 0u32;
        let ok = GetExitCodeProcess(h, &mut code) != 0;
        CloseHandle(h);
        ok && code == STILL_ACTIVE as u32
    }
}

#[cfg(unix)]
pub fn alive(pid: u32) -> bool {
    let Ok(pid) = libc::pid_t::try_from(pid) else {
        return false;
    };
    // pid 0 · 음수는 프로세스 그룹에 보내는 신호다 — 한 프로세스가 아니다.
    if pid <= 0 {
        return false;
    }
    // SAFETY: 신호 0 = 존재 · 권한 확인만.
    let sent = unsafe { libc::kill(pid, 0) } == 0;
    sent || io::Error::last_os_error().raw_os_error() == Some(libc::EPERM)
}

/// `pid` 가 끝나면 `on_gone` 을 한 번 부른다(별도 스레드).
pub fn watch_parent(pid: u32, on_gone: impl FnOnce() + Send + 'static) {
    std::thread::spawn(move || {
        wait_gone(pid);
        on_gone();
    });
}

/// Windows — 프로세스 핸들 대기. 열지 못하면 폴링.
#[cfg(windows)]
fn wait_gone(pid: u32) {
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::Threading::{
        INFINITE, OpenProcess, PROCESS_SYNCHRONIZE, WaitForSingleObject,
    };
    // SAFETY: 연 핸들은 대기 뒤 닫는다.
    unsafe {
        let h = OpenProcess(PROCESS_SYNCHRONIZE, 0, pid);
        if !h.is_null() {
            WaitForSingleObject(h, INFINITE);
            CloseHandle(h);
            return;
        }
    }
    poll_gone(pid);
}

#[cfg(unix)]
fn wait_gone(pid: u32) {
    poll_gone(pid);
}

fn poll_gone(pid: u32) {
    while alive(pid) {
        std::thread::sleep(POLL);
    }
}

/// 강제 종료(`stop` 의 마지막 수단).
#[cfg(windows)]
pub fn kill(pid: u32) -> io::Result<()> {
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::Threading::{OpenProcess, PROCESS_TERMINATE, TerminateProcess};
    // SAFETY: 연 핸들은 이 함수 안에서 닫는다.
    unsafe {
        let h = OpenProcess(PROCESS_TERMINATE, 0, pid);
        if h.is_null() {
            return Err(io::Error::last_os_error());
        }
        let ok = TerminateProcess(h, 1) != 0;
        let e = io::Error::last_os_error();
        CloseHandle(h);
        if ok { Ok(()) } else { Err(e) }
    }
}

#[cfg(unix)]
pub fn kill(pid: u32) -> io::Result<()> {
    let pid = libc::pid_t::try_from(pid).map_err(io::Error::other)?;
    // SAFETY: 신호 보내기 — 포인터 없음.
    if unsafe { libc::kill(pid, libc::SIGKILL) } == 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}

/// `stop` 의 마지막 수단 — 감시 `pid` 와 그 자식(앱)을 강제 종료한다. 감시를 먼저 끊으면 막 띄운 자식이 아직 잠금을 잡기 전이어도 함께 끊는다.
/// Windows 는 이 실행 파일과 이름이 같은 프로세스만(PID 재사용으로 남의 프로세스를 끊지 않게) — 핸들을 연 채 이름을 보고 끊는다.
#[cfg(windows)]
pub fn kill_tree(pid: u32) -> io::Result<()> {
    let me = std::env::current_exe()?;
    let name = me
        .file_name()
        .ok_or_else(|| io::Error::other("실행 파일 이름 없음"))?;
    terminate_named(pid, name)?;
    // 끊은 뒤에 본다 — 끊긴 감시는 새 자식을 띄우지 못한다. 자식의 부모 PID 는 감시가 끝나도 남는다.
    for (child, parent) in processes() {
        if parent == pid {
            let _ = terminate_named(child, name);
        }
    }
    Ok(())
}

/// 이름이 `name` 인 프로세스 `pid` 를 끊고 끝날 때까지(최대 1초) 기다린다.
#[cfg(windows)]
fn terminate_named(pid: u32, name: &OsStr) -> io::Result<()> {
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::Threading::{
        OpenProcess, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SYNCHRONIZE,
        PROCESS_TERMINATE, QueryFullProcessImageNameW, TerminateProcess, WaitForSingleObject,
    };
    // SAFETY: 연 핸들은 이 함수 안에서 닫는다. 이름 버퍼는 길이를 함께 넘긴 지역 배열.
    unsafe {
        let h = OpenProcess(
            PROCESS_TERMINATE | PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SYNCHRONIZE,
            0,
            pid,
        );
        if h.is_null() {
            return Err(io::Error::last_os_error());
        }
        let mut buf = [0u16; 1024];
        let mut len = buf.len() as u32;
        let same = QueryFullProcessImageNameW(h, PROCESS_NAME_WIN32, buf.as_mut_ptr(), &mut len)
            != 0
            && Path::new(&String::from_utf16_lossy(&buf[..len as usize]))
                .file_name()
                .is_some_and(|n| n.eq_ignore_ascii_case(name));
        let result = if !same {
            Err(io::Error::other(format!(
                "PID {pid} 는 이 프로그램이 아니다"
            )))
        } else if TerminateProcess(h, 1) == 0 {
            Err(io::Error::last_os_error())
        } else {
            WaitForSingleObject(h, 1000);
            Ok(())
        };
        CloseHandle(h);
        result
    }
}

#[cfg(unix)]
pub fn kill_tree(pid: u32) -> io::Result<()> {
    let pid = libc::pid_t::try_from(pid).map_err(io::Error::other)?;
    if pid <= 0 {
        return Err(io::Error::other("PID 0"));
    }
    // ponytail: Unix 는 이름 확인 없음 — 잠금 확인 ~ 신호 사이(ms) PID 재사용 창이 남는다. 막으려면 macOS `proc_pidpath` · Linux `/proc/<pid>/exe`.
    // 감시가 프로세스 그룹 우두머리면(`start` 가 떼어 띄움 · 셸 작업) 그룹째 — 자식 앱까지. 아니면 감시만(남은 앱은 `stop` 이 다시 본다).
    // SAFETY: 조회 · 신호 — 포인터 없음.
    let target = if unsafe { libc::getpgid(pid) } == pid {
        -pid
    } else {
        pid
    };
    if unsafe { libc::kill(target, libc::SIGKILL) } == 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}

/// 부른 쪽과 떨어진 프로세스로 띄운다 — 표준 입출력은 비우고(부모 툴의 파이프가 끝나지 않는 일 방지) 새 프로세스 그룹(부모 콘솔의 Ctrl+C 가
/// 닿지 않는다). 감시 변수(`COWORKAGENT_SUPERVISED`)는 지운다 — 띄운 것은 감시 프로세스가 된다.
pub fn spawn_detached<K: AsRef<OsStr>, V: AsRef<OsStr>>(
    exe: &Path,
    envs: impl IntoIterator<Item = (K, V)>,
) -> io::Result<Child> {
    let mut cmd = Command::new(exe);
    cmd.envs(envs)
        .env_remove(crate::watchdog::ENV)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        use windows_sys::Win32::System::Threading::{CREATE_NEW_PROCESS_GROUP, CREATE_NO_WINDOW};
        no_inherit_std_handles();
        // 숨은 콘솔 — 디버그(콘솔) 빌드에서 감시가 띄우는 앱이 새 콘솔 창을 만들지 않게(plan 개정: DETACHED_PROCESS 대신).
        cmd.creation_flags(CREATE_NO_WINDOW | CREATE_NEW_PROCESS_GROUP);
    }
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        cmd.process_group(0);
    }
    cmd.spawn()
}

/// 이 프로세스의 표준 핸들 3개를 상속 불가로 — `CreateProcess` 는 상속 가능한 핸들을 모두 넘기므로, 부른 쪽 파이프(예: 시험의 `output()`)가
/// 떼어 띄운 프로세스에 남아 그 파이프가 끝나지 않는다.
#[cfg(windows)]
fn no_inherit_std_handles() {
    use windows_sys::Win32::Foundation::{HANDLE_FLAG_INHERIT, SetHandleInformation};
    use windows_sys::Win32::System::Console::{
        GetStdHandle, STD_ERROR_HANDLE, STD_INPUT_HANDLE, STD_OUTPUT_HANDLE,
    };
    for which in [STD_INPUT_HANDLE, STD_OUTPUT_HANDLE, STD_ERROR_HANDLE] {
        // SAFETY: 표준 핸들 조회 · 플래그 변경 — 없거나 잘못된 핸들이면 실패만 한다(무시).
        unsafe {
            let h = GetStdHandle(which);
            if !h.is_null() {
                SetHandleInformation(h, HANDLE_FLAG_INHERIT, 0);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn self_and_parent_alive() {
        assert!(super::alive(std::process::id()));
        let parent = super::parent_pid().expect("부모 PID");
        assert_ne!(parent, std::process::id());
        assert!(super::alive(parent));
    }

    /// 이름이 다른 프로세스(PID 재사용 흉내)는 끊지 않는다.
    #[cfg(windows)]
    #[test]
    fn kill_tree_spares_other_programs() {
        let mut other = std::process::Command::new("ping")
            .args(["-n", "30", "127.0.0.1"])
            .stdout(std::process::Stdio::null())
            .spawn()
            .unwrap();
        assert!(super::kill_tree(other.id()).is_err());
        assert!(super::alive(other.id()));
        other.kill().unwrap();
        other.wait().unwrap();
    }

    #[test]
    fn finished_child_not_alive() {
        let mut child = super::spawn_detached(
            std::path::Path::new(env!("CARGO")),
            [("COWORKAGENT_UNUSED", "1")],
        )
        .expect("띄우기");
        let pid = child.id();
        child.wait().unwrap();
        assert!(!super::alive(pid));
    }
}
