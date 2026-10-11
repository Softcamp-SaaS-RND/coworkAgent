use an_net::identity::{Identity, fingerprint};
use std::path::PathBuf;

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("an-net-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn created_then_same_fingerprint() {
    let path = temp_dir("idcreate").join("identity.key");
    let first = Identity::load_or_create(&path).unwrap();
    assert!(path.is_file());
    assert!(
        !path.with_extension("key.tmp").exists(),
        "임시 파일이 남았다"
    );
    let again = Identity::load_or_create(&path).unwrap();
    assert_eq!(first.spki(), again.spki());
    assert_eq!(first.fingerprint(), again.fingerprint());
    // 다른 설치는 다른 신원.
    let other = Identity::load_or_create(&temp_dir("idother").join("identity.key")).unwrap();
    assert_ne!(first.spki(), other.spki());
}

#[test]
fn corrupt_key_is_error_not_replaced() {
    let path = temp_dir("idcorrupt").join("identity.key");
    std::fs::write(&path, b"not a key").unwrap();
    let err = Identity::load_or_create(&path)
        .err()
        .expect("손상 키는 오류");
    assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
    assert_eq!(
        std::fs::read(&path).unwrap(),
        b"not a key",
        "손상 파일을 덮어썼다"
    );
}

#[test]
fn fingerprint_is_8_groups_of_4_hex() {
    let fp = fingerprint(b"spki");
    let groups: Vec<_> = fp.split(' ').collect();
    assert_eq!(groups.len(), 8, "{fp}");
    assert!(groups.iter().all(|g| {
        g.len() == 4
            && g.bytes()
                .all(|c| c.is_ascii_digit() || (b'A'..=b'F').contains(&c))
    }));
    // SHA-256("spki") 앞 16바이트(Python hashlib 로 구한 값) — 해시 · 절단 위치 회귀.
    assert_eq!(fp, "6FEE CC8C 16C5 551D 9FEB 3EB5 F77E 2DA7");
    assert_ne!(fp, fingerprint(b"spkj"));
}
