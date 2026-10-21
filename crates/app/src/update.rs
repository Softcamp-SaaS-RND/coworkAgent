//! 자동 업데이트(S6 · D12 · D36 개정, #1y) — 트레이 모드 감시 프로세스가 앱을 띄우기 전에 GitHub 정식 릴리스를 한 번 본다.
//! 더 새 버전이면 실행 파일을 받아 묻지 않고 바꾼다(지금 exe → `.old`, 새 파일 → 제자리). 받은 파일은 회사 인증서 Authenticode 서명이 유효하고 서명자가
//! [`SIGNER`] 일 때만 바꾼다(#30 — 저장소를 빼앗겨도 서명 없는 exe 는 깔리지 않는다). 어떤 실패든 바꾸지 않고 지금 버전으로 뜬다. 버전 확인은 API 가 아닌
//! 고정 주소(`releases/latest` 넘겨주기 — 사내 공용 IP 의 API 횟수 제한, D36 ⑤).

use std::io;
#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// 배포처(#30 — 사용자 2026-10-21).
pub const RELEASES: &str = "https://github.com/Softcamp-SaaS-RND/coworkAgent/releases";
/// 받은 exe 의 Authenticode 서명 인증서 이름(CN) — 회사 코드 서명 인증서(#30).
pub const SIGNER: &str = "SOFTCAMP CO.,LTD.";
/// 받은 exe 의 버전 리소스 `ProductName`(`build.rs`) — 같은 회사가 서명한 다른 제품을 막는다.
pub const PRODUCT: &str = "coworkAgent";
/// 이 OS 의 릴리스 자산 — 없으면 확인하지 않는다(macOS 는 Phase 4, D25).
#[cfg(windows)]
pub const ASSET: Option<&str> = Some("coworkAgent.exe");
#[cfg(not(windows))]
pub const ASSET: Option<&str> = None;
/// 받는 자산 크기 상한.
pub const MAX_ASSET: u64 = 64 << 20;
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const LATEST_TIMEOUT: Duration = Duration::from_secs(10);
const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(120);

/// [`check`] 결과.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// 바꿨다 — 새 태그.
    Updated(String),
    /// 지금이 최신(또는 정식 릴리스 없음).
    UpToDate,
    /// 바꾸지 않았다 — 사유.
    Skipped(String),
}

/// 확인 · 받기 · 교체를 한 번에. `base` = 릴리스 주소(`…/releases`), `current` = 지금 버전(`X.Y.Z`), `skip` = 전에 뜨지 못해 되돌린 태그(다시 받지 않는다),
/// `trusted(.new, tag)` = 받아 쓴 `{exe}.new` 를 바꾸기 전에 확인(실행 = [`verify`]).
pub fn check(base: &str, asset: &str, current: &str, skip: Option<&str>, exe: &Path, trusted: impl Fn(&Path, &str) -> io::Result<()>) -> Outcome {
    let tag = match latest_tag(base) {
        Ok(Some(tag)) => tag,
        Ok(None) => return Outcome::UpToDate,
        Err(e) => return Outcome::Skipped(format!("최신 버전 확인 실패: {e}")),
    };
    if !newer(&tag, current) {
        return Outcome::UpToDate;
    }
    if skip == Some(tag.as_str()) {
        return Outcome::Skipped(format!("{tag} 는 전에 뜨지 못해 되돌린 버전 — 다음 버전을 기다린다"));
    }
    let url = format!("{base}/download/{tag}/{asset}");
    let bytes = match download(&url, MAX_ASSET) {
        Ok(b) => b,
        Err(e) => return Outcome::Skipped(format!("{tag} 받기 실패: {e}")),
    };
    // 이 PC 의 실행 파일 꼴(PE · 같은 아키텍처)이 아니면 바꾸지 않는다 — 200 으로 온 오류 · 로그인 페이지 · 잘린 파일이 exe 자리에 들어가 다음 실행부터
    // 뜨지 않는 일을 막는다(서명 확인은 아님). 그래도 뜨지 못하면 감시가 되돌린다(`rollback`).
    if !is_native_pe(&bytes) {
        return Outcome::Skipped(format!("{tag} 받은 파일이 이 PC 의 실행 파일이 아니다 — 바꾸지 않음"));
    }
    let new = sibling(exe, ".new");
    if let Err(e) = std::fs::write(&new, &bytes) {
        return Outcome::Skipped(format!("{tag} 교체 실패: {e}"));
    }
    if let Err(e) = trusted(&new, &tag) {
        let _ = std::fs::remove_file(&new);
        return Outcome::Skipped(format!("{tag} 서명 확인 실패 — 바꾸지 않음: {e}"));
    }
    match swap(exe) {
        Ok(()) => Outcome::Updated(tag),
        Err(e) => Outcome::Skipped(format!("{tag} 교체 실패: {e}")),
    }
}

fn agent(redirects: u32, timeout: Duration) -> ureq::Agent {
    let tls = ureq::tls::TlsConfig::builder()
        .root_certs(ureq::tls::RootCerts::PlatformVerifier)
        .build();
    ureq::Agent::config_builder()
        .tls_config(tls)
        .timeout_connect(Some(CONNECT_TIMEOUT))
        .timeout_global(Some(timeout))
        .max_redirects(redirects)
        .http_status_as_error(false)
        .build()
        .into()
}

/// 정식 릴리스의 최신 태그 — `{base}/latest` 의 302 `Location` 이 `…/tag/v{X.Y.Z}` 로 끝날 때만(사전 배포만 있으면 `…/releases` 로 넘겨 `None`).
pub fn latest_tag(base: &str) -> io::Result<Option<String>> {
    let resp = agent(0, LATEST_TIMEOUT)
        .get(&format!("{base}/latest"))
        .call()
        .map_err(io::Error::other)?;
    if !resp.status().is_redirection() {
        return Ok(None);
    }
    Ok(resp
        .headers()
        .get("location")
        .and_then(|v| v.to_str().ok())
        .and_then(|loc| loc.rsplit_once("/tag/"))
        .map(|(_, tag)| tag.to_string())
        .filter(|tag| version(tag).is_some()))
}

/// `v` 뗀 `X.Y.Z` 정수 셋.
fn version(tag: &str) -> Option<(u64, u64, u64)> {
    let mut it = tag.strip_prefix('v').unwrap_or(tag).split('.').map(|n| n.parse::<u64>().ok());
    let v = (it.next()??, it.next()??, it.next()??);
    it.next().is_none().then_some(v)
}

/// `tag` 가 `current` 보다 엄격히 새 버전인가(꼴이 틀리면 거짓 — 같은 · 낮은 버전으로 바꾸지 않는다).
pub fn newer(tag: &str, current: &str) -> bool {
    matches!((version(tag), version(current)), (Some(t), Some(c)) if t > c)
}

fn download(url: &str, limit: u64) -> io::Result<Vec<u8>> {
    let resp = agent(10, DOWNLOAD_TIMEOUT).get(url).call().map_err(io::Error::other)?;
    if !resp.status().is_success() {
        return Err(io::Error::other(format!("HTTP {}", resp.status())));
    }
    resp.into_body()
        .with_config()
        .limit(limit)
        .read_to_vec()
        .map_err(io::Error::other)
}

/// 이 실행 파일과 같은 아키텍처의 PE 인가 — `MZ` · `e_lfanew` 의 `PE\0\0` · COFF `Machine`(x86-64 `0x8664` · ARM64 `0xAA64`).
pub fn is_native_pe(bytes: &[u8]) -> bool {
    let machine: u16 = if cfg!(target_arch = "aarch64") { 0xAA64 } else { 0x8664 };
    let Some(e) = bytes.get(0x3C..0x40).map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]) as usize) else {
        return false;
    };
    bytes.starts_with(b"MZ")
        && bytes.get(e..e + 4) == Some(b"PE\0\0".as_slice())
        && bytes.get(e + 4..e + 6) == Some(machine.to_le_bytes().as_slice())
}

fn sibling(exe: &Path, ext: &str) -> PathBuf {
    let mut name = exe.file_name().unwrap_or_default().to_os_string();
    name.push(ext);
    exe.with_file_name(name)
}

/// 실행 파일 바꾸기 — 받아 둔 `{exe}.new` 를, 지금 exe 를 `{exe}.old` 로 이름 바꾼 뒤(Windows 는 실행 중인 파일도 이름은 바꿀 수 있다 · 덮어쓰지는 못한다) 새 파일을
/// 제자리로. 마지막 단계가 실패하면 `.old` 를 되돌리고, 되돌리기도 실패하면 둘 다 알린다.
/// ponytail: 두 이름 바꾸기 사이(마이크로초)에 전원이 꺼지면 exe 자리가 빈다 — 실행 중 이미지는 한 번에 바꿀 수 없다(`.old` 를 손으로 되돌린다, 개발자 가이드 2.4).
pub fn swap(exe: &Path) -> io::Result<()> {
    let (new, old) = (sibling(exe, ".new"), sibling(exe, ".old"));
    if old.exists() {
        std::fs::remove_file(&old)?;
    }
    std::fs::rename(exe, &old)?;
    if let Err(e) = std::fs::rename(&new, exe) {
        return match std::fs::rename(&old, exe) {
            Ok(()) => Err(e),
            Err(back) => Err(io::Error::other(format!("{e} — 되돌리기도 실패: {back} ({} 를 {} 로 바꿔야 한다)", old.display(), exe.display()))),
        };
    }
    Ok(())
}

/// 받은 exe 를 바꿔도 되나 — 회사 인증서 서명([`authenticode`] · [`SIGNER`]) 다음, 서명이 덮는 버전 리소스가 이 제품 · 그 태그 버전인가([`identity`] · [`PRODUCT`]).
/// 태그만 믿으면 저장소를 빼앗은 쪽이 회사 서명된 옛 버전(업데이터 없는 1.1.1 등)을 높은 태그에 붙여 내려 깔 수 있다(적대적 평가 R1).
pub fn verify(path: &Path, tag: &str) -> io::Result<()> {
    authenticode(path, SIGNER)?;
    identity(path, PRODUCT, tag)
}

#[cfg(windows)]
fn wide(path: &Path) -> Vec<u16> {
    path.as_os_str().encode_wide().chain([0]).collect()
}

/// `path` 의 Authenticode 서명이 유효하고(신뢰 루트까지 · 코드 서명 용도 · 인증서 폐기 확인) 서명 인증서 이름(CN)이 `signer` 인가.
#[cfg(windows)]
pub fn authenticode(path: &Path, signer: &str) -> io::Result<()> {
    use windows_sys::Win32::Security::Cryptography::{CERT_NAME_SIMPLE_DISPLAY_TYPE, CertGetNameStringW};
    use windows_sys::Win32::Security::WinTrust::{
        WINTRUST_ACTION_GENERIC_VERIFY_V2, WINTRUST_DATA, WINTRUST_DATA_0, WINTRUST_FILE_INFO, WTD_CHOICE_FILE, WTD_REVOCATION_CHECK_CHAIN_EXCLUDE_ROOT,
        WTD_REVOKE_WHOLECHAIN, WTD_STATEACTION_CLOSE, WTD_STATEACTION_VERIFY, WTD_UI_NONE, WTHelperGetProvSignerFromChain, WTHelperProvDataFromStateData,
        WinVerifyTrust,
    };
    let wide = wide(path);
    let mut file = WINTRUST_FILE_INFO { cbStruct: size_of::<WINTRUST_FILE_INFO>() as u32, pcwszFilePath: wide.as_ptr(), ..Default::default() };
    let mut data = WINTRUST_DATA {
        cbStruct: size_of::<WINTRUST_DATA>() as u32,
        dwUIChoice: WTD_UI_NONE,
        fdwRevocationChecks: WTD_REVOKE_WHOLECHAIN,
        dwUnionChoice: WTD_CHOICE_FILE,
        Anonymous: WINTRUST_DATA_0 { pFile: &mut file },
        dwStateAction: WTD_STATEACTION_VERIFY,
        dwProvFlags: WTD_REVOCATION_CHECK_CHAIN_EXCLUDE_ROOT,
        ..Default::default()
    };
    let mut action = WINTRUST_ACTION_GENERIC_VERIFY_V2;
    // SAFETY: `data` · `file` · `wide` 는 두 호출 동안 살아 있다. 서명자 인증서는 VERIFY 가 만든 상태(`hWVTStateData`) 안의 것이라 CLOSE 전에만 읽는다.
    let (status, name) = unsafe {
        let status = WinVerifyTrust(std::ptr::null_mut(), &mut action, (&raw mut data).cast());
        let prov = WTHelperProvDataFromStateData(data.hWVTStateData);
        let sgnr = if status == 0 && !prov.is_null() { WTHelperGetProvSignerFromChain(prov, 0, 0, 0) } else { std::ptr::null_mut() };
        let mut buf = [0u16; 256];
        let n = if sgnr.is_null() || (*sgnr).csCertChain == 0 {
            0
        } else {
            CertGetNameStringW((*(*sgnr).pasCertChain).pCert, CERT_NAME_SIMPLE_DISPLAY_TYPE, 0, std::ptr::null(), buf.as_mut_ptr(), buf.len() as u32)
        };
        data.dwStateAction = WTD_STATEACTION_CLOSE;
        WinVerifyTrust(std::ptr::null_mut(), &mut action, (&raw mut data).cast());
        (status, (n > 1).then(|| String::from_utf16_lossy(&buf[..n as usize - 1])))
    };
    match name {
        _ if status != 0 => Err(io::Error::other(format!("Authenticode 확인 실패 0x{:08X}", status as u32))),
        Some(n) if n == signer => Ok(()),
        n => Err(io::Error::other(format!("서명자 {n:?} — {signer:?} 아님"))),
    }
}

/// `path` 의 버전 리소스 `ProductName`(`build.rs` 의 `041204B0` 묶음)이 `product` 이고 파일 버전 앞 셋이 `tag`(`vX.Y.Z`)와 같은가.
#[cfg(windows)]
pub fn identity(path: &Path, product: &str, tag: &str) -> io::Result<()> {
    use windows_sys::Win32::Storage::FileSystem::{GetFileVersionInfoSizeW, GetFileVersionInfoW, VS_FIXEDFILEINFO, VerQueryValueW};
    let want = version(tag).ok_or_else(|| io::Error::other(format!("태그 꼴이 아니다: {tag}")))?;
    let (path, root, key) = (wide(path), wide(Path::new("\\")), wide(Path::new("\\StringFileInfo\\041204B0\\ProductName")));
    let mut buf = vec![0u8; unsafe { GetFileVersionInfoSizeW(path.as_ptr(), std::ptr::null_mut()) } as usize];
    // SAFETY: `buf` 는 GetFileVersionInfoW 가 채운 블록 — VerQueryValueW 가 돌려주는 포인터 · 길이는 그 안이고 `buf` 가 사는 동안만 읽는다.
    let (fixed, name) = unsafe {
        if buf.is_empty() || GetFileVersionInfoW(path.as_ptr(), 0, buf.len() as u32, buf.as_mut_ptr().cast()) == 0 {
            return Err(io::Error::other("버전 리소스 없음"));
        }
        let (mut p, mut len) = (std::ptr::null_mut(), 0u32);
        let fixed = (VerQueryValueW(buf.as_ptr().cast(), root.as_ptr(), &mut p, &mut len) != 0 && len as usize >= size_of::<VS_FIXEDFILEINFO>())
            .then(|| p.cast::<VS_FIXEDFILEINFO>().read_unaligned());
        let name = (VerQueryValueW(buf.as_ptr().cast(), key.as_ptr(), &mut p, &mut len) != 0 && len > 0)
            .then(|| String::from_utf16_lossy(std::slice::from_raw_parts(p.cast::<u16>(), len as usize)).trim_end_matches('\0').to_string());
        (fixed, name)
    };
    let got = fixed.map(|f| (u64::from(f.dwFileVersionMS >> 16), u64::from(f.dwFileVersionMS & 0xFFFF), u64::from(f.dwFileVersionLS >> 16)));
    match (name, got) {
        (Some(n), Some(v)) if n == product && v == want => Ok(()),
        (n, v) => Err(io::Error::other(format!("제품 {n:?} · 버전 {v:?} — {product:?} {tag} 아님"))),
    }
}

/// Windows 밖은 아직 확인하지 않는다(자산 없음 — [`ASSET`]).
#[cfg(not(windows))]
pub fn authenticode(_: &Path, _: &str) -> io::Result<()> {
    Err(io::ErrorKind::Unsupported.into())
}

#[cfg(not(windows))]
pub fn identity(_: &Path, _: &str, _: &str) -> io::Result<()> {
    Err(io::ErrorKind::Unsupported.into())
}

/// 새 버전이 뜨지 못했을 때 — 지금 exe 를 `{exe}.bad` 로 빼고 `{exe}.old` 를 제자리로. `.old` 가 없으면 거짓(되돌릴 것 없음).
pub fn rollback(exe: &Path) -> io::Result<bool> {
    let (old, bad) = (sibling(exe, ".old"), sibling(exe, ".bad"));
    if !old.exists() {
        return Ok(false);
    }
    if bad.exists() {
        std::fs::remove_file(&bad)?;
    }
    if exe.exists() {
        std::fs::rename(exe, &bad)?;
    }
    std::fs::rename(&old, exe)?;
    Ok(true)
}

/// 교체 뒤 아직 한 번도 제대로 뜨지 않았나(`.old` 가 남아 있다 — 앱이 처음 준비되면 [`settled`] 가 지운다).
pub fn pending(exe: &Path) -> bool {
    sibling(exe, ".old").exists()
}

/// 새 버전 앱이 준비됨 — 되돌릴 `.old` 를 지운다(앱 시작 끝). 바꾼 지난 감시가 아직 `.old` 이미지로 떠 있으면(새 감시를 최대 15초 지켜본다) Windows 는
/// 실행 중 이미지를 지우지 못하므로, 그때는 1초마다 다시 지운다(최대 60번 — 남아 있으면 이 버전이 다음 시작 실패 때 되돌려진다, 적대적 평가 R2).
pub fn settled(exe: &Path) {
    let old = sibling(exe, ".old");
    if std::fs::remove_file(&old).is_ok() || !old.exists() {
        return;
    }
    std::thread::spawn(move || {
        for _ in 0..60 {
            std::thread::sleep(Duration::from_secs(1));
            if std::fs::remove_file(&old).is_ok() || !old.exists() {
                return;
            }
        }
        log::warn!("자동 업데이트: 지난 버전 {} 를 지우지 못했다 — 다음 시작에 다시", old.display());
    });
}

/// 지난 교체 · 되돌리기가 남긴 `{exe}.new` · `{exe}.bad` 를 지운다(`.old` 는 [`settled`] 가 — 새 버전이 뜨지 못하면 되돌릴 것).
pub fn clean(exe: &Path) {
    for ext in [".new", ".bad"] {
        let _ = std::fs::remove_file(sibling(exe, ext));
    }
}
