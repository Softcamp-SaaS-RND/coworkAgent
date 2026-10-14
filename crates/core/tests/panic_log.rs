//! 패닉 기록(#1e) — 로거를 설치하면 패닉 메시지 · 파일 · 줄이 로그 파일에 남는다(전역 로거라 이 시험 하나만 둔다).
use an_core::log_file::FileLogger;

#[test]
fn panic_goes_to_log() {
    let dir = std::env::temp_dir().join(format!("an-core-panic-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("coworkAgent.log");
    FileLogger::install(&path, log::LevelFilter::Info).unwrap();
    let line = line!() + 1;
    let r = std::thread::spawn(|| panic!("시험 패닉")).join();
    assert!(r.is_err());
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(
        text.contains("패닉: 시험 패닉") && text.contains(&format!("panic_log.rs:{line}")),
        "{text}"
    );
}
