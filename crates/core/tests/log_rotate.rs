use an_core::log_file::{LOG_ROTATE_BYTES, RotatingFile, backup_path};
use std::path::PathBuf;

fn log_with(name: &str, prefill: u64) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("an-core-log-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("coworkAgent.log");
    std::fs::write(&path, vec![b'x'; prefill as usize]).unwrap();
    path
}

fn size(p: &PathBuf) -> u64 {
    std::fs::metadata(p).unwrap().len()
}

#[test]
fn limit_minus_1_kept() {
    let path = log_with("m1", LOG_ROTATE_BYTES - 2);
    RotatingFile::open(&path).unwrap().write_all(b"y").unwrap();
    assert_eq!(size(&path), LOG_ROTATE_BYTES - 1);
    assert!(!backup_path(&path).exists());
}

#[test]
fn limit_exact_kept() {
    let path = log_with("eq", LOG_ROTATE_BYTES - 1);
    RotatingFile::open(&path).unwrap().write_all(b"y").unwrap();
    assert_eq!(size(&path), LOG_ROTATE_BYTES);
    assert!(!backup_path(&path).exists());
}

#[test]
fn limit_plus_1_rotated() {
    let path = log_with("p1", LOG_ROTATE_BYTES);
    RotatingFile::open(&path).unwrap().write_all(b"y").unwrap();
    assert_eq!(size(&backup_path(&path)), LOG_ROTATE_BYTES);
    assert_eq!(size(&path), 1);
}

#[test]
fn limit_constant_is_1048576() {
    assert_eq!(LOG_ROTATE_BYTES, 1_048_576);
}
