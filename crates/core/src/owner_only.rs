//! 데이터 루트를 이 PC 사용자 전용으로(1a A1). Windows = 보호 DACL(현재 사용자 · SYSTEM, 하위 상속) · Unix = 0700.
//! 기존 하위 파일은 상속 항목만 가졌으면 OS 가 새 DACL 로 다시 계산한다.
use std::io;
use std::path::Path;

/// Unix = 루트 · 하위 폴더 0700 · 파일 0600(한 단계 아래 `logs` 까지 — 루트 배치는 평평하다).
#[cfg(unix)]
pub fn restrict(dir: &Path) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mode = |p: &Path, m: u32| std::fs::set_permissions(p, std::fs::Permissions::from_mode(m));
    mode(dir, 0o700)?;
    for e in std::fs::read_dir(dir)? {
        let e = e?;
        let t = e.file_type()?;
        if t.is_dir() {
            mode(&e.path(), 0o700)?;
            for f in std::fs::read_dir(e.path())? {
                let f = f?;
                if f.file_type()?.is_file() {
                    mode(&f.path(), 0o600)?;
                }
            }
        } else if t.is_file() {
            mode(&e.path(), 0o600)?;
        }
    }
    Ok(())
}

#[cfg(windows)]
pub fn restrict(dir: &Path) -> io::Result<()> {
    use std::os::windows::ffi::OsStrExt;
    use std::ptr::{null, null_mut};
    use windows_sys::Win32::Foundation::{CloseHandle, HANDLE, LocalFree};
    use windows_sys::Win32::Security::Authorization::{
        EXPLICIT_ACCESS_W, NO_MULTIPLE_TRUSTEE, SE_FILE_OBJECT, SET_ACCESS, SetEntriesInAclW,
        SetNamedSecurityInfoW, TRUSTEE_IS_SID, TRUSTEE_IS_UNKNOWN, TRUSTEE_W,
    };
    use windows_sys::Win32::Security::{
        ACL, CreateWellKnownSid, DACL_SECURITY_INFORMATION, GetTokenInformation,
        PROTECTED_DACL_SECURITY_INFORMATION, SECURITY_MAX_SID_SIZE,
        SUB_CONTAINERS_AND_OBJECTS_INHERIT, TOKEN_QUERY, TOKEN_USER, TokenUser, WinLocalSystemSid,
    };
    use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

    /// 파일 전체 권한(`FILE_ALL_ACCESS`) — icacls 의 `(F)`.
    const FILE_ALL_ACCESS: u32 = 0x001F_01FF;
    let win = |rc: u32| {
        if rc == 0 {
            Ok(())
        } else {
            Err(io::Error::from_raw_os_error(rc as i32))
        }
    };

    // SAFETY: 모든 포인터는 이 함수의 지역 버퍼를 가리키고, 버퍼 크기는 API 가 알려 준 값이다. 토큰 · ACL 은 끝에서 닫는다.
    unsafe {
        let mut token: HANDLE = null_mut();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == 0 {
            return Err(io::Error::last_os_error());
        }
        let mut len = 0u32;
        GetTokenInformation(token, TokenUser, null_mut(), 0, &mut len);
        // TOKEN_USER 는 포인터를 담는다 — 8바이트 정렬 버퍼.
        let mut buf = vec![0u64; (len as usize).div_ceil(8)];
        let ok = GetTokenInformation(token, TokenUser, buf.as_mut_ptr().cast(), len, &mut len);
        CloseHandle(token);
        if ok == 0 {
            return Err(io::Error::last_os_error());
        }
        let user = (*(buf.as_ptr() as *const TOKEN_USER)).User.Sid;

        let mut system = [0u64; (SECURITY_MAX_SID_SIZE as usize).div_ceil(8)];
        let mut system_len = SECURITY_MAX_SID_SIZE;
        if CreateWellKnownSid(
            WinLocalSystemSid,
            null_mut(),
            system.as_mut_ptr().cast(),
            &mut system_len,
        ) == 0
        {
            return Err(io::Error::last_os_error());
        }

        let entry = |sid: *mut core::ffi::c_void| EXPLICIT_ACCESS_W {
            grfAccessPermissions: FILE_ALL_ACCESS,
            grfAccessMode: SET_ACCESS,
            grfInheritance: SUB_CONTAINERS_AND_OBJECTS_INHERIT,
            Trustee: TRUSTEE_W {
                pMultipleTrustee: null_mut(),
                MultipleTrusteeOperation: NO_MULTIPLE_TRUSTEE,
                TrusteeForm: TRUSTEE_IS_SID,
                TrusteeType: TRUSTEE_IS_UNKNOWN,
                ptstrName: sid.cast(),
            },
        };
        let entries = [entry(user), entry(system.as_mut_ptr().cast())];
        let mut acl: *mut ACL = null_mut();
        win(SetEntriesInAclW(
            entries.len() as u32,
            entries.as_ptr(),
            null(),
            &mut acl,
        ))?;

        let wide: Vec<u16> = dir.as_os_str().encode_wide().chain(Some(0)).collect();
        let rc = SetNamedSecurityInfoW(
            wide.as_ptr(),
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION | PROTECTED_DACL_SECURITY_INFORMATION,
            null_mut(),
            null_mut(),
            acl,
            null(),
        );
        LocalFree(acl.cast());
        win(rc)
    }
}
