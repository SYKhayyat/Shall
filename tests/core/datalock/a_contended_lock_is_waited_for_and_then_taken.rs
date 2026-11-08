use super::*;

/// **A wait that is given time spends it, rather than refusing at once.**
///
/// Found by the mutation gate: `acquire` computes its deadline as `now + timeout` and leaves
/// the loop when `now >= deadline`, and BOTH of those survived being inverted -- to `now -
/// timeout` and to `now < deadline`. Either mutant puts the deadline in the past on the first
/// iteration, so a contended lock returns the timeout error immediately instead of waiting.
///
/// Nothing noticed, because every test above contends and then asserts on the *refusal*.
/// Refusing is what `acquire` does at the END of the wait; not one test made it wait. So the
/// whole point of the parameter -- that a run started by a `DPkg::Post-Invoke` hook stands
/// behind an ordinary `apt install` instead of failing under it -- was unmeasured.
///
/// **The holder is a second process**, because that is the only contention `acquire` waits for:
/// a handle this process already holds is refused at once (that is what `for_this_write` exists
/// to avoid), and a wait pinned against a thread of this same process would pin a case the
/// program no longer has.
#[test]
fn a_contended_lock_is_waited_for_and_then_taken() {
    let _g = gate();
    let dir = tmp("wait-succeeds");
    let _holder = ChildHolder::on(&dir);

    let started = Instant::now();
    let taken = DataLock::acquire(&dir, "waiter", Duration::from_secs(30));
    let waited = started.elapsed();

    assert!(
        taken.is_ok(),
        "the holder released well inside the timeout and the wait still failed: {:?}",
        taken.err()
    );
    // The half that kills the arithmetic mutants: a deadline in the past would have come
    // back at once with the error above, and a wait that returns instantly is not a wait.
    assert!(
        waited >= Duration::from_millis(400),
        "took the lock after {waited:?}, which is sooner than the holder released it -- the \
         wait did not happen"
    );
}
