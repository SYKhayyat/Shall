use super::*;

/// **The door that reports contention has to open when there is none.** `try_acquire`
/// returning `Ok(None)` unconditionally means a `hook-*` subcommand never records anything
/// and never says why â€” it reads exactly like the ordinary contended case it was built for.
#[test]
fn a_free_directory_hands_out_the_lock_rather_than_reporting_contention() {
    let _g = gate();
    let dir = tmp("try-acquire-free");
    let taken = DataLock::try_acquire(&dir, "hook-install").unwrap();
    assert!(
        taken.is_some(),
        "nobody holds this directory and `try_acquire` reported contention"
    );
    assert!(
        held(),
        "the lock was handed out and the count did not go up"
    );
    drop(taken);
    assert!(!held());
}
