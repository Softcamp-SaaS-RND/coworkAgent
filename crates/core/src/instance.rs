use std::fs::{File, OpenOptions, TryLockError};
use std::io;
use std::path::Path;
use std::time::Duration;

/// 남이 잠깐 들여다보는 중([`InstanceLock::is_held`])일 수 있어 보조로 단정하기 전에 다시 잡아 본다.
const ACQUIRE_TRIES: u32 = 4;
const ACQUIRE_GAP: Duration = Duration::from_millis(50);

pub enum Acquired {
    Primary(InstanceLock),
    Secondary,
}

/// 단일 인스턴스 잠금. 보유 중에만 유효하고, 프로세스가 죽으면 OS 가 푼다.
pub struct InstanceLock {
    _file: File,
}

impl InstanceLock {
    /// 누가 이 잠금을 쥐고 있나 — 잠깐 잡아 보고 바로 놓는다(핸들 닫기에 맡기지 않고 직접 푼다). 파일이 없거나 열 수 없으면 false.
    pub fn is_held(path: &Path) -> bool {
        let Ok(file) = OpenOptions::new().write(true).open(path) else {
            return false;
        };
        match file.try_lock() {
            Ok(()) => {
                let _ = file.unlock();
                false
            }
            Err(TryLockError::WouldBlock) => true,
            Err(TryLockError::Error(_)) => false,
        }
    }

    pub fn acquire(path: &Path) -> io::Result<Acquired> {
        let file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(path)?;
        for n in 1..=ACQUIRE_TRIES {
            match file.try_lock() {
                Ok(()) => return Ok(Acquired::Primary(InstanceLock { _file: file })),
                Err(TryLockError::WouldBlock) if n < ACQUIRE_TRIES => {
                    std::thread::sleep(ACQUIRE_GAP)
                }
                Err(TryLockError::WouldBlock) => break,
                Err(TryLockError::Error(e)) => return Err(e),
            }
        }
        Ok(Acquired::Secondary)
    }
}

/// 이 프로세스의 PID 를 적는다(잠금을 잡은 뒤).
pub fn write_pid(path: &Path) -> io::Result<()> {
    std::fs::write(path, std::process::id().to_string())
}

pub fn read_pid(path: &Path) -> Option<u32> {
    std::fs::read_to_string(path).ok()?.trim().parse().ok()
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

    /// 토큰이 든 파일 — Unix 는 0600 으로 만든다(Windows 는 데이터 루트의 소유자 전용 DACL 을 물려받는다).
    pub fn write(&self, path: &Path) -> io::Result<()> {
        #[cfg(unix)]
        {
            use std::io::Write;
            use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
            let mut f = std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .mode(0o600)
                .open(path)?;
            // 이미 있던 파일은 mode 가 그대로라 다시 맞춘다.
            f.set_permissions(std::fs::Permissions::from_mode(0o600))?;
            f.write_all(self.to_json().as_bytes())
        }
        #[cfg(not(unix))]
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

    /// 잡고 있는 동안만 held — 들여다본 뒤에도 잠금은 그대로다.
    #[test]
    fn is_held_tracks_holder_and_pid_round_trip() {
        use super::{Acquired, InstanceLock, read_pid, write_pid};
        let dir = std::env::temp_dir().join(format!("an-core-held-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let lock = dir.join("x.lock");
        let _ = std::fs::remove_file(&lock);
        assert!(!InstanceLock::is_held(&lock), "파일 없음");
        let Acquired::Primary(held) = InstanceLock::acquire(&lock).unwrap() else {
            panic!("처음은 주")
        };
        assert!(InstanceLock::is_held(&lock));
        assert!(InstanceLock::is_held(&lock), "들여다봐도 그대로");
        drop(held);
        assert!(!InstanceLock::is_held(&lock));
        let pid = dir.join("x.pid");
        write_pid(&pid).unwrap();
        assert_eq!(read_pid(&pid), Some(std::process::id()));
        assert_eq!(read_pid(&dir.join("없음.pid")), None);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn info_rejects_non_hex_token() {
        assert_eq!(InstanceInfo::from_json(r#"{"port":1,"token":"<x>"}"#), None);
        assert_eq!(InstanceInfo::from_json(r#"{"port":1,"token":""}"#), None);
    }
}
