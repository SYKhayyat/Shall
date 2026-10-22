use super::*;

/// **The reentrancy door, from outside.** Its sibling test proves `for_this_write` declines
/// when the caller already holds the lock. Nothing proved it *takes* one when the caller
/// does not, so `Ok(None)` always â€” every deferred write racing every other â€” survived.
#[test]
fn a_write_from_outside_the_lock_takes_one() {
    let _g = gate();
    let dir = tmp("for-this-write-outside");
    std::fs::create_dir_all(&dir).unwrap();
    let previous = std::env::var_os("SHALL_DATA_DIR");
    std::env::set_var("SHALL_DATA_DIR", &dir);

    assert!(
        !held(),
        "the gate above should have left this process unlocked"
    );
    let taken = DataLock::for_this_write("a ledger").unwrap();
    assert!(
        taken.is_some(),
        "no lock was held, so this write had to take one and it did not"
    );
    assert!(held());
    drop(taken);

    match previous {
        Some(v) => std::env::set_var("SHALL_DATA_DIR", v),
        None => std::env::remove_var("SHALL_DATA_DIR"),
    }
}
