use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

/// 이번 기록 뒤 크기가 이 값을 넘게 되면 먼저 회전한다(정확히 이 값이면 유지).
pub const LOG_ROTATE_BYTES: u64 = 1_048_576;

/// 크기 회전 로그 파일. ponytail: 백업은 `.1` 하나만 — 더 오래 남겨야 하면 개수 노브를 추가.
pub struct RotatingFile {
    path: PathBuf,
    file: File,
    len: u64,
}

impl RotatingFile {
    pub fn open(path: &Path) -> io::Result<Self> {
        let file = append(path)?;
        let len = file.metadata()?.len();
        Ok(Self {
            path: path.to_path_buf(),
            file,
            len,
        })
    }

    pub fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        if self.len > 0 && self.len + buf.len() as u64 > LOG_ROTATE_BYTES {
            fs::rename(&self.path, backup_path(&self.path))?;
            self.file = append(&self.path)?;
            self.len = 0;
        }
        self.file.write_all(buf)?;
        self.len += buf.len() as u64;
        Ok(())
    }
}

fn append(path: &Path) -> io::Result<File> {
    OpenOptions::new().create(true).append(true).open(path)
}

pub fn backup_path(path: &Path) -> PathBuf {
    let mut s = path.as_os_str().to_owned();
    s.push(".1");
    PathBuf::from(s)
}

/// `log` 파사드를 파일로 받는다 — Windows 릴리스는 콘솔이 없어 이것이 유일한 진단 수단.
pub struct FileLogger {
    file: Mutex<RotatingFile>,
}

impl FileLogger {
    pub fn install(path: &Path, level: log::LevelFilter) -> io::Result<()> {
        let logger = FileLogger {
            file: Mutex::new(RotatingFile::open(path)?),
        };
        log::set_boxed_logger(Box::new(logger)).map_err(io::Error::other)?;
        log::set_max_level(level);
        // 패닉도 로그에 — 콘솔 없는 릴리스에서 비정상 종료 근거가 남는 곳은 여기뿐. 기본 훅(stderr)도 그대로 부른다.
        let default = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            let at = info
                .location()
                .map(|l| format!("{}:{}", l.file(), l.line()))
                .unwrap_or_else(|| "?".into());
            log::error!("패닉: {} — {at}", info.payload_as_str().unwrap_or("?"));
            default(info);
        }));
        Ok(())
    }
}

impl log::Log for FileLogger {
    fn enabled(&self, _: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let line = format!(
            "{secs} {} {}: {}\n",
            record.level(),
            record.target(),
            record.args()
        );
        if let Ok(mut f) = self.file.lock() {
            // 로그 실패를 다시 로그할 곳이 없다 — 버린다.
            let _ = f.write_all(line.as_bytes());
        }
    }

    fn flush(&self) {}
}
