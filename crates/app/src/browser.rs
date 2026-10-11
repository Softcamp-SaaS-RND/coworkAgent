//! 기본 브라우저로 URL 을 연다 — 셸 인용을 거치지 않는다. OS 가 열기를 거절하면 오류.
use std::io;

#[cfg(windows)]
pub fn open(url: &str) -> io::Result<()> {
    use windows_sys::Win32::UI::Shell::ShellExecuteW;
    use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
    let wide = |s: &str| s.encode_utf16().chain([0]).collect::<Vec<u16>>();
    let (verb, file) = (wide("open"), wide(url));
    // SAFETY: 두 인자는 NUL 로 끝나는 UTF-16 버퍼이고 호출 동안 살아 있다.
    let code = unsafe {
        ShellExecuteW(
            std::ptr::null_mut(),
            verb.as_ptr(),
            file.as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            SW_SHOWNORMAL,
        )
    } as isize;
    // 32 이하 = 실패 코드(SE_ERR_*).
    if code > 32 {
        Ok(())
    } else {
        Err(io::Error::other(format!("ShellExecuteW 실패 코드 {code}")))
    }
}

#[cfg(not(windows))]
pub fn open(url: &str) -> io::Result<()> {
    let program = if cfg!(target_os = "macos") {
        "open"
    } else {
        "xdg-open"
    };
    let status = std::process::Command::new(program).arg(url).status()?;
    if status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!("{program} 종료 상태 {status}")))
    }
}
