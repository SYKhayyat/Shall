use super::*;

/// **The reentrancy door, from outside.** Its sibling test proves `for_this_write` declines
/// when the caller already holds the lock. Nothing proved it *takes* one when the caller does
/// not, so `Ok(None)` always -- every deferred write racing every other -- survived.
#[test]
fn a_write_from_outside_the_lock_takes_one() {
    let _g = gate();
    let dir = tmp("for-this-write-outside");
    let _data_dir = DataDir::at(&dir);

    assert!(
        !held_on(&dir),
        "the gate above should have left this process unlocked"
    );
    let taken = DataLock::for_this_write("a ledger").unwrap();
    assert!(
        taken.is_some(),
        "no lock was held, so this write had to take one and it did not"
    );
    assert!(held_on(&dir));
    drop(taken);
}
