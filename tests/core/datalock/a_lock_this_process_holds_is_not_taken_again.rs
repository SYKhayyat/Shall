use super::*;

/// **The re-entrant door refuses at once when the lock is this process's own.**
///
/// `flock` is per open file description, so the second `try_lock` reports contention and the
/// deadline loop then naps for `WAIT_SECS` -- 120 seconds of silence for a lock whose release
/// is the code that is standing there. The mutation gate measured exactly that: three mutants
/// of the old process-wide count made every e2e command wait the full two minutes, and the
/// shard timed out before it could report which mutants survived.
///
/// The refusal is what makes that cost unreachable rather than unlikely: whatever sends a
/// caller down this path, it comes back in milliseconds with the reason. A *direct* acquire
/// still waits, because another thread of this process may be about to release -- the property
/// `watch` relies on and the reason this refusal belongs to one door rather than to the lock.
#[test]
fn a_lock_this_process_holds_is_not_taken_again() {
    let _g = gate();
    let dir = tmp("self-contended");
    let _first = DataLock::acquire(&dir, "sync", Duration::from_secs(1)).unwrap();

    let started = Instant::now();
    let outcome = DataLock::acquire_reentrant(&dir, "again", Duration::from_secs(5));
    let waited = started.elapsed();

    let Err(err) = outcome else {
        panic!("a second handle in this process took the lock the first one holds")
    };
    assert!(
        waited < Duration::from_secs(1),
        "refusing a lock this process holds took {waited:?} -- that is the wait, not the refusal"
    );
    assert!(
        err.to_string().contains("already holds"),
        "the refusal does not say the lock is this process's own: {err}"
    );
}
