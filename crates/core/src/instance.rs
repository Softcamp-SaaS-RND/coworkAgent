use std::fs::{File, OpenOptions, TryLockError};
use std::io;
use std::path::Path;

pub enum Acquired {
    Primary(InstanceLock),
    Secondary,
}

/// 단일 인스턴스 잠금(D14). 보유 중에만 유효하고, 프로세스가 죽으면 OS 가 푼다.
pub struct InstanceLock {
    _file: File,
}

impl InstanceLock {
    pub fn acquire(path: &Path) -> io::Result<Acquired> {
        let file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(path)?;
        match file.try_lock() {
            Ok(()) => Ok(Acquired::Primary(InstanceLock { _file: file })),
            Err(TryLockError::WouldBlock) => Ok(Acquired::Secondary),
            Err(TryLockError::Error(e)) => Err(e),
        }
    }
}

/// 주 인스턴스가 남기는 핸드오프 정보 — 두 번째 실행이 읽어 `POST /api/open` 을 보낸다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstanceInfo {
    pub port: u16,
    pub token: String,
}

impl InstanceInfo {
    pub fn to_json(&self) -> String {
        format!("{{\"port\":{},\"token\":\"{}\"}}", self.port, self.token)
    }

    /// 우리가 쓴 형식만 읽는다. 토큰은 hex 만 허용.
    pub fn from_json(s: &str) -> Option<Self> {
        let port = s.split("\"port\":").nth(1)?;
        let port: u16 = port
            .split(|c: char| !c.is_ascii_digit())
            .next()?
            .parse()
            .ok()?;
        let token = s.split("\"token\":\"").nth(1)?.split('"').next()?;
        let hex = !token.is_empty() && token.bytes().all(|b| b.is_ascii_hexdigit());
        hex.then(|| Self {
            port,
            token: token.to_string(),
        })
    }

    pub fn write(&self, path: &Path) -> io::Result<()> {
        std::fs::write(path, self.to_json())
    }

    pub fn read(path: &Path) -> io::Result<Option<Self>> {
        Ok(Self::from_json(&std::fs::read_to_string(path)?))
    }
}

#[cfg(test)]
mod tests {
    use super::InstanceInfo;

    #[test]
    fn info_round_trip() {
        let info = InstanceInfo {
            port: 51234,
            token: "ab12".repeat(16),
        };
        assert_eq!(InstanceInfo::from_json(&info.to_json()), Some(info));
    }

    #[test]
    fn info_rejects_non_hex_token() {
        assert_eq!(InstanceInfo::from_json(r#"{"port":1,"token":"<x>"}"#), None);
        assert_eq!(InstanceInfo::from_json(r#"{"port":1,"token":""}"#), None);
    }
}
