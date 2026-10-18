//! 데이터 루트 소유자 전용 권한 · 오염된 잠금(1a A1 · A14).
use an_core::{owner_only, sync};
#[cfg(windows)]
use std::path::Path;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

fn temp_root(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("an-core-hard-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// `icacls` 출력의 ACE 줄 — (주체, 권한 표기). 첫 줄은 경로가 앞에 붙는다.
#[cfg(windows)]
fn aces(path: &Path) -> Vec<(String, String)> {
    let out = std::process::Command::new("icacls")
        .arg(path)
        .output()
        .unwrap();
    assert!(out.status.success(), "icacls 실패");
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    let path_text = path.display().to_string();
    text.lines()
        .filter_map(|l| {
            let l = l.strip_prefix(path_text.as_str()).unwrap_or(l).trim();
            let (who, rights) = l.split_once(":(")?;
            Some((who.trim().to_string(), format!("({rights}")))
        })
        .collect()
}

#[cfg(windows)]
#[test]
fn root_dacl_is_protected_owner_and_system_only() {
    let root = temp_root("dacl");
    owner_only::restrict(&root).unwrap();
    let a = aces(&root);
    assert_eq!(a.len(), 2, "{a:?}");
    // 보호 DACL — 상속(I) 항목이 없고, 둘 다 하위로 물려준다.
    assert!(
        a.iter()
            .all(|(_, r)| !r.contains("(I)") && r.contains("(OI)(CI)")),
        "{a:?}"
    );
    assert!(a.iter().any(|(w, _)| w.ends_with("SYSTEM")), "{a:?}");
    let user = std::env::var("USERNAME").unwrap().to_lowercase();
    assert!(
        a.iter().any(|(w, _)| w.to_lowercase().ends_with(&user)),
        "{a:?}"
    );
}

#[cfg(windows)]
#[test]
fn existing_and_new_children_follow_root() {
    let root = temp_root("child");
    let before = root.join("identity.key");
    std::fs::write(&before, b"k").unwrap();
    // 시작 전 파일은 %TEMP% 의 상속 권한(관리자 등)을 갖고 있다.
    assert!(aces(&before).len() > 2, "{:?}", aces(&before));
    owner_only::restrict(&root).unwrap();
    let after = root.join("instance.json");
    std::fs::write(&after, b"{}").unwrap();
    for f in [&before, &after] {
        let a = aces(f);
        assert_eq!(a.len(), 2, "{f:?} {a:?}");
        assert!(a.iter().all(|(_, r)| r.contains("(I)")), "{f:?} {a:?}");
    }
}

#[cfg(unix)]
#[test]
fn root_dacl_is_protected_owner_and_system_only() {
    use std::os::unix::fs::PermissionsExt;
    let root = temp_root("mode");
    owner_only::restrict(&root).unwrap();
    assert_eq!(
        std::fs::metadata(&root).unwrap().permissions().mode() & 0o777,
        0o700
    );
}

#[test]
fn poisoned_lock_keeps_working() {
    let m = Arc::new(Mutex::new(1));
    let p = m.clone();
    let _ = std::thread::spawn(move || {
        let _g = p.lock().unwrap();
        panic!("잠금을 쥔 채 panic");
    })
    .join();
    assert!(m.is_poisoned());
    *sync::lock(&m) += 1;
    assert_eq!(*sync::lock(&m), 2);
}
