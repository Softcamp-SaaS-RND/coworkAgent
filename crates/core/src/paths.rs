use std::io;
use std::path::{Path, PathBuf};

pub const APP_DIR: &str = "coworkAgent";
/// 시험용 데이터 루트 환경 변수 — 디버그 빌드만 읽는다(릴리스는 무시).
pub const DATA_DIR_ENV: &str = "COWORKAGENT_DATA_DIR";
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
    /// 디버그 빌드만 [`DATA_DIR_ENV`] 가 있으면 그 경로 — 시험이 이 PC 의 실제 인스턴스 · 데이터를 건드리지 않게(#1n).
    pub fn default_root() -> Option<PathBuf> {
        if let Some(dir) = test_root() {
            return Some(dir);
        }
        dirs::data_local_dir().map(|d| d.join(APP_DIR))
    }
    /// 개명 전 데이터 루트(`default_root` 와 같은 자리의 `agentNetwork`). 시험용 루트를 쓰면 없음 — 실제 옛 데이터를 옮겨 오지 않게.
    pub fn legacy_root() -> Option<PathBuf> {
        if test_root().is_some() {
            return None;
        }
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
    /// 앱(`instance.lock` 보유자)의 PID — `stop` 의 강제 종료용.
    pub fn instance_pid_file(&self) -> PathBuf {
        self.root.join("instance.pid")
    }
    /// 감시 프로세스가 남긴 자동 업데이트 결과 한 줄(#1y — 감시는 로거를 쓰지 않는다, 앱이 시작 때 로그에 옮기고 지운다).
    pub fn update_note_file(&self) -> PathBuf {
        self.root.join("update.txt")
    }
    /// 뜨지 못해 되돌린 업데이트 태그(#1y) — 그 태그는 다시 받지 않는다.
    pub fn update_skip_file(&self) -> PathBuf {
        self.root.join("update-skip.txt")
    }
    /// 감시 프로세스 단일 잠금(#1n) — 동시에 여러 번 실행해도 감시는 하나.
    pub fn supervisor_lock_file(&self) -> PathBuf {
        self.root.join("supervisor.lock")
    }
    pub fn supervisor_pid_file(&self) -> PathBuf {
        self.root.join("supervisor.pid")
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

    /// 루트를 이 PC 사용자 전용 권한으로 맞춘다(매 시작 — 기존 설치도, 1a A1). 로그를 연 뒤 부른다 — 실패는 경고로 남기고 계속한다
    /// (데이터는 이미 상속 권한으로 있었다).
    pub fn restrict(&self) -> io::Result<()> {
        crate::owner_only::restrict(&self.root)
    }
}

/// 디버그 빌드의 [`DATA_DIR_ENV`] 값.
fn test_root() -> Option<PathBuf> {
    if !cfg!(debug_assertions) {
        return None;
    }
    std::env::var_os(DATA_DIR_ENV)
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
}

/// 데이터 루트(지금 · 옛 이름) — 에이전트 폴더가 품으면 안 되는 곳.
pub fn protected_roots() -> Vec<PathBuf> {
    [Paths::default_root(), Paths::legacy_root()]
        .into_iter()
        .flatten()
        .collect()
}

/// 에이전트 폴더로 쓸 수 없는 이유 — 원격 문의의 읽기 범위에 키 · 다른 상대와의 대화 DB 가 들어가거나(데이터 루트를 품음 · 그 안),
/// 네트워크 경로(정규화가 로컬 경로와 맞지 않는다)면. 등록 · 수정 · 실행 직전에 같은 규칙(1a A7).
pub fn agent_folder_problem(folder: &Path) -> Option<&'static str> {
    use std::path::{Component, Prefix};
    let unc = |p: &Path| {
        matches!(
            p.components().next(),
            Some(Component::Prefix(x)) if matches!(x.kind(), Prefix::UNC(..) | Prefix::VerbatimUNC(..))
        )
    };
    if unc(folder) || unc(&norm_path(folder)) {
        return Some("네트워크 경로는 에이전트 폴더로 쓸 수 없다");
    }
    if folder_blocked(folder, &protected_roots()) {
        return Some("데이터 폴더를 포함하는 폴더는 에이전트 폴더로 쓸 수 없다");
    }
    None
}

/// `folder` 가 `protected` 중 하나와 같거나 그 조상이거나 그 안인가. 있는 부분까지 정규화하고(링크 · `..`) 대소문자는 가리지 않는다.
pub fn folder_blocked(folder: &Path, protected: &[PathBuf]) -> bool {
    let f = path_key(folder);
    protected.iter().any(|r| {
        let r = path_key(r);
        f.starts_with(&r) || r.starts_with(&f)
    })
}

fn path_key(p: &Path) -> Vec<String> {
    norm_path(p)
        .components()
        .map(|c| c.as_os_str().to_string_lossy().to_lowercase())
        .collect()
}

/// 있으면 정규화, 없으면 있는 조상까지 정규화하고 나머지를 붙인다(아직 없는 옛 루트).
fn norm_path(p: &Path) -> PathBuf {
    if let Ok(c) = std::fs::canonicalize(p) {
        return c;
    }
    match (p.parent(), p.file_name()) {
        (Some(d), Some(n)) => norm_path(d).join(n),
        _ => p.to_path_buf(),
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
