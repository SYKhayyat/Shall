use super::*;

/// **The stand-down door, from a directory nobody holds.** `try_for_one_step` is
/// `try_acquire` with the data directory resolved for the caller, and it is reached only
/// from `main`'s `acquire_data_lock`: `Ok(None)` there is `LockedRun::StandDown`, so a
/// mutant returning it always makes every manager hook do nothing and exit zero. The hook
/// tests assert that hooks *stand down* under contention and cannot see it â€” standing down
/// is what they want. Nothing asserted a hook proceeds when the lock is free.
#[test]
fn a_hook_with_nobody_holding_the_lock_proceeds_rather_than_standing_down() {
    let _g = gate();
    let dir = tmp("try-for-one-step");
    let _data_dir = DataDir::at(&dir);

    let taken = DataLock::try_for_one_step("hook-record").unwrap();
    assert!(
        taken.is_some(),
        "nobody holds this directory, so the hook had to take the lock and record; standing down here loses whatever the manager just did"
    );
    assert!(held());
    assert!(
        lock_file_is_in(&dir),
        "the hook took a lock somewhere other than the data directory it was pointed at"
    );
    drop(taken);
    assert!(!held());
}
