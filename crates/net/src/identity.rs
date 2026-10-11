//! 설치별 신원 — ed25519 키(PKCS#8) 파일과 지문.
use ring::rand::SystemRandom;
use ring::signature::Ed25519KeyPair;
use rustls::pki_types::{CertificateDer, PrivatePkcs8KeyDer};
use rustls::sign::CertifiedKey;
use std::io;
use std::path::Path;
use std::sync::Arc;

pub struct Identity {
    key: Arc<CertifiedKey>,
    spki: Vec<u8>,
}

impl Identity {
    /// 파일이 없으면 만들고, 있으면 연다. 열 수 없는 파일은 오류 — 새로 만들면 기존 페어링이 조용히 깨진다.
    pub fn load_or_create(path: &Path) -> io::Result<Self> {
        if !path.exists() {
            let pkcs8 = Ed25519KeyPair::generate_pkcs8(&SystemRandom::new())
                .map_err(|_| io::Error::other("ed25519 키 생성 실패"))?;
            let tmp = path.with_extension("key.tmp");
            write_private(&tmp, pkcs8.as_ref())?;
            std::fs::rename(&tmp, path)?;
        }
        Self::from_pkcs8(std::fs::read(path)?).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("신원 키 {} 를 열 수 없다: {e}", path.display()),
            )
        })
    }

    /// ed25519 PKCS#8 만 받는다.
    pub fn from_pkcs8(der: Vec<u8>) -> Result<Self, rustls::Error> {
        let key = rustls::crypto::ring::sign::any_eddsa_type(&PrivatePkcs8KeyDer::from(der))?;
        let spki = key
            .public_key()
            .ok_or_else(|| rustls::Error::General("공개키 없음".into()))?
            .to_vec();
        let key = Arc::new(CertifiedKey::new(
            vec![CertificateDer::from(spki.clone())],
            key,
        ));
        Ok(Self { key, spki })
    }

    /// 공개키 SPKI DER — 피어 표의 식별자.
    pub fn spki(&self) -> &[u8] {
        &self.spki
    }

    pub fn fingerprint(&self) -> String {
        fingerprint(&self.spki)
    }

    pub(crate) fn certified_key(&self) -> Arc<CertifiedKey> {
        self.key.clone()
    }
}

/// SHA-256(SPKI) 앞 16바이트 → `7F3A 19C2 …`(대문자 hex 4자리 8묶음).
pub fn fingerprint(spki: &[u8]) -> String {
    let d = ring::digest::digest(&ring::digest::SHA256, spki);
    d.as_ref()[..16]
        .chunks(2)
        .map(|c| format!("{:02X}{:02X}", c[0], c[1]))
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(unix)]
fn write_private(path: &Path, bytes: &[u8]) -> io::Result<()> {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(path)?;
    f.write_all(bytes)?;
    f.sync_all()
}

/// Windows 는 `%LOCALAPPDATA%` 의 사용자 전용 ACL 을 물려받는다.
#[cfg(not(unix))]
fn write_private(path: &Path, bytes: &[u8]) -> io::Result<()> {
    use std::io::Write;
    let mut f = std::fs::File::create(path)?;
    f.write_all(bytes)?;
    f.sync_all()
}
