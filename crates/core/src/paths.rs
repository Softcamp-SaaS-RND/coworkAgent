use std::io;
use std::path::{Path, PathBuf};

pub const APP_DIR: &str = "coworkAgent";
/// 개명 전 이름(agentNetwork, 2026-10-11)의 데이터 폴더 · 파일 이름 — 첫 실행 때 새 이름으로 옮긴다.
pub const LEGACY_APP_DIR: &str = "agentNetwork";
const LEGACY_DB: &str = "agentNetwork.db";
const LEGACY_LOG: &str = "agentNetwork.log";

/// 데이터 루트 아래 파일 배치(D15). 루트는 인자로 받는다 — 운영은 `default_root`, 테스트는 임시 폴더.
#[derive(Debug, Clone)]
pub struct Paths {
    root: PathBuf,
}

impl Paths {
    /// Windows `%LOCALAPPDATA%\coworkAgent` · macOS `~/Library/Application Support/coworkAgent`.
    pub fn default_root() -> Option<PathBuf> {
        dirs::data_local_dir().map(|d| d.join(APP_DIR))
    }
    /// 개명 전 데이터 루트(`default_root` 와 같은 자리의 `agentNetwork`).
    pub fn legacy_root() -> Option<PathBuf> {
        dirs::data_local_dir().map(|d| d.join(LEGACY_APP_DIR))
    }
    pub fn under(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }
    pub fn root(&self) -> &Path {
        &self.root
    }
    pub fn db_file(&self) -> PathBuf {
        self.root.join("coworkAgent.db")
    }
    pub fn log_dir(&self) -> PathBuf {
        self.root.join("logs")
    }
    pub fn log_file(&self) -> PathBuf {
        self.log_dir().join("coworkAgent.log")
    }
    pub fn lock_file(&self) -> PathBuf {
        self.root.join("instance.lock")
    }
    pub fn instance_file(&self) -> PathBuf {
        self.root.join("instance.json")
    }
    /// 설치별 신원 키(ed25519 PKCS#8).
    pub fn identity_file(&self) -> PathBuf {
        self.root.join("identity.key")
    }
    /// 개명 전 데이터를 이어받는다. 옛 루트의 `instance.lock` 을 먼저 잡고(옛 앱이 실행 중이면 아무것도 바꾸지 않고 오류), 잡은 채로
    /// 옛 루트의 항목을 새 루트로 옮긴 뒤(옛 DB · 로그는 새 이름으로) 옛 루트를 지운다. 두 루트 모두에 데이터(DB · 신원)가 있으면 고르지
    /// 않고 충돌 오류 — 사람이 하나를 정리한다. 이미 있는 새 항목은 덮어쓰지 않아 중간에 끊겨도 다시 부르면 이어서 끝난다.
    /// 돌려주는 값: 옛 데이터를 옮겼나.
    pub fn adopt_legacy(&self, legacy_root: &Path) -> io::Result<bool> {
        let moved = if legacy_root.is_dir() {
            self.move_legacy(legacy_root)?
        } else {
            false
        };
        // 새 루트 안에 남은 옛 이름(이전 판의 폴더째 옮기기 · 중단).
        for (old, new) in [
            (self.root.join(LEGACY_DB), self.db_file()),
            (self.log_dir().join(LEGACY_LOG), self.log_file()),
        ] {
            if old.is_file() && !new.exists() {
                std::fs::rename(old, new)?;
            }
        }
        Ok(moved)
    }

    fn move_legacy(&self, legacy: &Path) -> io::Result<bool> {
        let has_data =
            |dir: &Path, db: &str| dir.join("identity.key").exists() || dir.join(db).exists();
        let legacy_data = has_data(legacy, LEGACY_DB);
        let new_data = has_data(&self.root, "coworkAgent.db") || self.root.join(LEGACY_DB).exists();
        if legacy_data && new_data {
            return Err(io::Error::other(format!(
                "옛 데이터({})와 새 데이터({})가 둘 다 있다 — 쓸 쪽만 남기고 다른 쪽을 옮기거나 지운 뒤 다시 실행",
                legacy.display(),
                self.root.display()
            )));
        }
        // 옛 앱이 실행 중이면 손대지 않는다 — 잡은 잠금은 옮기기가 끝날 때까지 쥔다.
        let lock = match crate::instance::InstanceLock::acquire(&legacy.join("instance.lock"))? {
            crate::instance::Acquired::Primary(lock) => lock,
            crate::instance::Acquired::Secondary => {
                return Err(io::Error::other(format!(
                    "옛 앱(agentNetwork)이 실행 중이다({}) — 끄고 다시 실행",
                    legacy.display()
                )));
            }
        };
        if !legacy_data {
            drop(lock);
            return Ok(false);
        }
        std::fs::create_dir_all(&self.root)?;
        for entry in std::fs::read_dir(legacy)? {
            let entry = entry?;
            let name = entry.file_name();
            let to = match name.to_str() {
                Some("instance.lock" | "instance.json") => continue, // 실행 중 표시 — 옮기지 않는다
                Some(LEGACY_DB) => self.db_file(),
                _ => self.root.join(&name),
            };
            move_entry(&entry.path(), &to)?;
        }
        drop(lock);
        std::fs::remove_dir_all(legacy)?;
        Ok(true)
    }

    /// 루트와 logs 폴더를 만든다.
    pub fn ensure(&self) -> io::Result<()> {
        std::fs::create_dir_all(self.log_dir())
    }
}

/// 항목 하나를 옮긴다 — 폴더끼리면 안의 것을 하나씩, 이미 있는 대상은 덮어쓰지 않는다(로그 · 사본만 남는다).
fn move_entry(from: &Path, to: &Path) -> io::Result<()> {
    if from.is_dir() && to.is_dir() {
        for entry in std::fs::read_dir(from)? {
            let entry = entry?;
            move_entry(&entry.path(), &to.join(entry.file_name()))?;
        }
        Ok(())
    } else if to.exists() {
        Ok(())
    } else {
        std::fs::rename(from, to)
    }
}
