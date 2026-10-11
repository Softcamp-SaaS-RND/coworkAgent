use an_app::tray::write_start_error;

#[test]
fn start_error_file_has_reason() {
    let path = write_start_error("데이터 경로를 만들 수 없다: 액세스가 거부되었습니다").unwrap();
    assert!(path.starts_with(std::env::temp_dir()), "{path:?}");
    assert_eq!(path.file_name().unwrap(), "coworkAgent-start-error.txt");
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(
        text.contains("데이터 경로를 만들 수 없다: 액세스가 거부되었습니다"),
        "{text}"
    );
    assert!(text.contains("coworkAgent 시작 실패"), "{text}");
    // 다시 쓰면 덮어쓴다(지난 실패가 섞이지 않는다).
    write_start_error("두 번째").unwrap();
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(
        text.contains("두 번째") && !text.contains("액세스가 거부"),
        "{text}"
    );
}
