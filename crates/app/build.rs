//! Windows 실행 파일에 아이콘 리소스를 넣는다 — 그림은 `tools/gen_icon.py` 가 만든 `assets/coworkAgent.ico`.
fn main() {
    println!("cargo:rerun-if-changed=assets/app.rc");
    println!("cargo:rerun-if-changed=assets/coworkAgent.ico");
    embed_resource::compile("assets/app.rc", embed_resource::NONE)
        .manifest_optional()
        .expect("아이콘 리소스 컴파일");
}
