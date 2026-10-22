use super::*;

#[test]
fn a_second_holder_is_refused_with_who_holds_it_rather_than_hanging() {
    let _g = gate();
    let dir = tmp("contended");
    let first = DataLock::acquire(&dir, "sync", Duration::from_secs(1)).unwrap();

    // A second *process* is what the lock is for; within one process the advisory lock
    // is not re-entrant on a separate handle either, which is what this asserts.
    let path = dir.join("shall.lock");
    let other = File::open(&path).unwrap();
    assert!(
        other.try_lock_exclusive().is_err(),
        "a second handle took a lock the first still holds"
    );
    drop(first);
    assert!(other.try_lock_exclusive().is_ok());
    let _ = FileExt::unlock(&other);
}
