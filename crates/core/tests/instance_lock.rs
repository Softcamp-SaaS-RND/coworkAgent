use an_core::instance::{Acquired, InstanceLock};
use std::path::PathBuf;

fn lock_path(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("an-core-lock-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir.join("instance.lock")
}

#[test]
fn first_is_primary() {
    let path = lock_path("first");
    assert!(matches!(
        InstanceLock::acquire(&path).unwrap(),
        Acquired::Primary(_)
    ));
}

#[test]
fn second_while_held_is_secondary() {
    let path = lock_path("second");
    let _held = InstanceLock::acquire(&path).unwrap();
    assert!(matches!(
        InstanceLock::acquire(&path).unwrap(),
        Acquired::Secondary
    ));
}

#[test]
fn reacquire_after_release() {
    let path = lock_path("again");
    let held = InstanceLock::acquire(&path).unwrap();
    assert!(matches!(held, Acquired::Primary(_)));
    drop(held);
    assert!(matches!(
        InstanceLock::acquire(&path).unwrap(),
        Acquired::Primary(_)
    ));
}
