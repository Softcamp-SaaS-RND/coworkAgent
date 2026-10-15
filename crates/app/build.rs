//! Windows 실행 파일 리소스 — 아이콘(`tools/gen_icon.py` 가 만든 `assets/coworkAgent.ico`)과 파일 버전(탐색기 속성 → 자세히).
//! 버전은 `Cargo.toml` 의 `version` 하나에서 온다 — rc 파일을 빌드 때 `OUT_DIR` 에 만든다.
fn main() {
    println!("cargo:rerun-if-changed=assets/coworkAgent.ico");
    let ver = env!("CARGO_PKG_VERSION");
    let nums: Vec<u16> = ver
        .split(['.', '-', '+'])
        .take(3)
        .map(|n| n.parse().expect("버전 숫자"))
        .collect();
    let [major, minor, patch] = nums[..] else {
        panic!("버전은 major.minor.patch: {ver}");
    };
    let manifest = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
    let icon = format!("{manifest}/assets/coworkAgent.ico").replace('\\', "/");
    let rc = format!(
        r#"1 ICON "{icon}"
1 VERSIONINFO
FILEVERSION {major},{minor},{patch},0
PRODUCTVERSION {major},{minor},{patch},0
FILEOS 0x40004
FILETYPE 0x1
BEGIN
  BLOCK "StringFileInfo"
  BEGIN
    BLOCK "041204B0"
    BEGIN
      VALUE "FileDescription", "coworkAgent"
      VALUE "FileVersion", "{ver}"
      VALUE "InternalName", "coworkAgent"
      VALUE "OriginalFilename", "coworkAgent.exe"
      VALUE "ProductName", "coworkAgent"
      VALUE "ProductVersion", "{ver}"
    END
  END
  BLOCK "VarFileInfo"
  BEGIN
    VALUE "Translation", 0x412, 1200
  END
END
"#
    );
    let path = std::path::Path::new(&std::env::var("OUT_DIR").expect("OUT_DIR")).join("app.rc");
    std::fs::write(&path, rc).expect("rc 파일 쓰기");
    embed_resource::compile(&path, embed_resource::NONE)
        .manifest_optional()
        .expect("리소스 컴파일");
}
