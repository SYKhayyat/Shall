use super::*;

/// **The two doors ask the same question, and both must answer it the same way.**
///
/// `try_acquire` stands down when the directory is busy; `acquire` waits for it. **Both read
/// the answer from one classification**, and the classification has two answers — so a door
/// that reads *every* failure as contention is a door that stands down (or waits) for a lock
/// nobody holds.
///
/// The gate that found this: `cargo mutants` reported four survivors in the guards themselves
/// — two at `try_acquire`, two at `acquire` — and each of them replaced the guard with `true`
/// or `false`. **The contended case is the one a test can reach**, and it is enough: a guard
/// that answers `true` for everything makes the contended door report contention *and* the
/// unrelated failure report contention, and the first is a change a test can see.
///
/// **The holder is a whole other process.** A wait has to end, and a test cannot release its own
/// guard from inside the wait; the re-entrant door refuses a second handle in one process
/// outright (that is `flock` waiting for the handle it is already inside), so the only honest
/// waiter is a process that will actually let go.
#[test]
fn both_doors_read_a_busy_lock_the_same_way() {
    let _g = gate();
    let dir = tmp("both-doors");

    // **The door that does not wait stands down** — and it stands down *quietly*, because
    // contention is the ordinary case for a hook fired mid-transaction, not a fault.
    let held = ChildHolder::on(&dir);
    let stood_down = DataLock::try_acquire(&dir, "hook");
    assert!(
        stood_down.is_ok(),
        "the non-waiting door failed on a lock that is merely held: {:?}",
        stood_down.err()
    );
    assert!(
        stood_down.unwrap().is_none(),
        "the non-waiting door took a lock somebody else holds"
    );

    // **And the door that waits says the same thing**, by waiting for the holder rather than
    // by refusing: the child releases well inside the timeout, so the wait ends in a lock.
    let waited = DataLock::acquire(&dir, "waiter", Duration::from_secs(30));
    assert!(
        waited.is_ok(),
        "the waiting door refused a lock that was merely held: {:?}",
        waited.err()
    );
    drop(held);

    // **And with the directory free, both doors take it** — the other half of the question,
    // and the half a guard mutated to `false` gets wrong.
    drop(waited);
    assert!(
        DataLock::try_acquire(&dir, "free").unwrap().is_some(),
        "a free directory handed out nothing"
    );
}
