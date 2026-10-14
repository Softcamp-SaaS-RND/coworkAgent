//! 자동 시작(S7 · D13, #1e) — 로그인 때 이 실행 파일을 인자 없이(= 감시 프로세스) 띄운다. 사용자 권한 자리만 쓴다: Windows
//! `HKCU\…\Run` 값 · macOS `~/Library/LaunchAgents` plist. 다시 띄우기는 감시 프로세스가 맡아 두 OS 같은 동작(D30 — `KeepAlive` 없음).
use std::io;
use std::path::Path;

/// Windows 로그인 실행 키(HKCU 아래).
pub const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
/// Run 키의 값 이름.
pub const VALUE_NAME: &str = "coworkAgent";
/// macOS LaunchAgent 이름 — 파일은 `{LABEL}.plist`.
pub const LABEL: &str = "com.coworkagent";

/// 이 OS 의 실제 자리에 켜기(없거나 경로가 다르면 등록) · 끄기(지움).
pub fn sync(on: bool) -> io::Result<()> {
    let exe = std::env::current_exe()?;
    #[cfg(windows)]
    {
        if on {
            register(RUN_KEY, &exe).map(|_| ())
        } else {
            unregister(RUN_KEY)
        }
    }
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var_os("HOME").ok_or_else(|| io::Error::other("HOME 없음"))?;
        let dir = Path::new(&home).join("Library/LaunchAgents");
        let file = dir.join(format!("{LABEL}.plist"));
        if on {
            let want = launch_agent_plist(&exe);
            if std::fs::read_to_string(&file).ok().as_deref() != Some(want.as_str()) {
                std::fs::create_dir_all(&dir)?;
                write_atomic(&file, &want)?;
            }
            Ok(())
        } else {
            match std::fs::remove_file(&file) {
                Err(e) if e.kind() != io::ErrorKind::NotFound => Err(e),
                _ => Ok(()),
            }
        }
    }
    #[cfg(not(any(windows, target_os = "macos")))]
    {
        let _ = (on, exe);
        Ok(())
    }
}

/// `path` 를 통째로 바꾼다 — 같은 폴더의 `{이름}.tmp` 에 다 쓰고 디스크에 내린 뒤 이름 바꾸기. 도중에 실패하면 원래 파일은 그대로다
/// (덮어쓰기로 빈 · 반쪽 파일이 남아 다음 로그인 실행이 사라지지 않게).
pub fn write_atomic(path: &Path, content: &str) -> io::Result<()> {
    use std::io::Write;
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(".tmp");
    let tmp = std::path::PathBuf::from(tmp);
    let written = (|| {
        let mut f = std::fs::File::create(&tmp)?;
        f.write_all(content.as_bytes())?;
        f.sync_all()?;
        drop(f);
        std::fs::rename(&tmp, path)
    })();
    if written.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    written
}

/// macOS LaunchAgent — 로그인 때 한 번(`RunAtLoad`). 경로는 XML 이스케이프.
pub fn launch_agent_plist(exe: &Path) -> String {
    let path = exe
        .display()
        .to_string()
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;");
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>{LABEL}</string>
  <key>ProgramArguments</key>
  <array>
    <string>{path}</string>
  </array>
  <key>RunAtLoad</key>
  <true/>
</dict>
</plist>
"#
    )
}

#[cfg(windows)]
fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain([0]).collect()
}

/// HKCU `key` 의 [`VALUE_NAME`] 값(문자열) — 없으면 `None`.
#[cfg(windows)]
pub fn registered(key: &str) -> io::Result<Option<String>> {
    use windows_sys::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_SUCCESS};
    use windows_sys::Win32::System::Registry::{HKEY_CURRENT_USER, RRF_RT_REG_SZ, RegGetValueW};
    let (key, name) = (wide(key), wide(VALUE_NAME));
    // ponytail: 1024 칸 고정 — 더 긴 값은 오류(경로 길이 제한을 푼 PC 의 아주 긴 경로), 필요하면 크기 묻기 → 다시 읽기.
    let mut buf = vec![0u16; 1024];
    let mut bytes = (buf.len() * 2) as u32;
    // SAFETY: 이름은 NUL 로 끝나는 UTF-16, `buf` · `bytes` 는 호출 동안 살아 있고 `bytes` 가 버퍼 크기(바이트)다.
    let code = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            key.as_ptr(),
            name.as_ptr(),
            RRF_RT_REG_SZ,
            std::ptr::null_mut(),
            buf.as_mut_ptr().cast(),
            &mut bytes,
        )
    };
    match code {
        ERROR_SUCCESS => {
            // 돌려준 크기에는 끝 NUL 이 들어 있다.
            let n = (bytes as usize / 2).saturating_sub(1);
            Ok(Some(String::from_utf16_lossy(&buf[..n])))
        }
        ERROR_FILE_NOT_FOUND => Ok(None),
        e => Err(io::Error::from_raw_os_error(e as i32)),
    }
}

/// HKCU `key` 에 `"exe"` 를 적는다(키가 없으면 만든다). 같은 값이면 쓰지 않고 false.
#[cfg(windows)]
pub fn register(key: &str, exe: &Path) -> io::Result<bool> {
    use windows_sys::Win32::Foundation::ERROR_SUCCESS;
    use windows_sys::Win32::System::Registry::{HKEY_CURRENT_USER, REG_SZ, RegSetKeyValueW};
    let value = format!("\"{}\"", exe.display());
    if registered(key)?.as_deref() == Some(value.as_str()) {
        return Ok(false);
    }
    let (key, name, data) = (wide(key), wide(VALUE_NAME), wide(&value));
    // SAFETY: 세 버퍼 모두 NUL 로 끝나는 UTF-16 이고 호출 동안 살아 있다. 크기는 끝 NUL 포함 바이트.
    let code = unsafe {
        RegSetKeyValueW(
            HKEY_CURRENT_USER,
            key.as_ptr(),
            name.as_ptr(),
            REG_SZ,
            data.as_ptr().cast(),
            (data.len() * 2) as u32,
        )
    };
    if code == ERROR_SUCCESS {
        Ok(true)
    } else {
        Err(io::Error::from_raw_os_error(code as i32))
    }
}

/// HKCU `key` 의 [`VALUE_NAME`] 값을 지운다 — 없어도 성공.
#[cfg(windows)]
pub fn unregister(key: &str) -> io::Result<()> {
    use windows_sys::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_SUCCESS};
    use windows_sys::Win32::System::Registry::{HKEY_CURRENT_USER, RegDeleteKeyValueW};
    let (key, name) = (wide(key), wide(VALUE_NAME));
    // SAFETY: 두 이름 모두 NUL 로 끝나는 UTF-16 이고 호출 동안 살아 있다.
    match unsafe { RegDeleteKeyValueW(HKEY_CURRENT_USER, key.as_ptr(), name.as_ptr()) } {
        ERROR_SUCCESS | ERROR_FILE_NOT_FOUND => Ok(()),
        e => Err(io::Error::from_raw_os_error(e as i32)),
    }
}
