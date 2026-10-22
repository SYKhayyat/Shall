use super::*;

/// And a wait that runs out runs out *after* the time it was given, not before.
///
/// The other side of the same two mutants, and the one that pins the number rather than the
/// behaviour: without it, `acquire` could satisfy the test above by waiting a fixed instant
/// and ignoring `timeout` entirely.
#[test]
fn a_wait_that_runs_out_first_spends_the_time_it_was_given() {
    let _g = gate();
    let dir = tmp("wait-expires");
    let _held = DataLock::acquire(&dir, "holder", Duration::from_secs(5)).unwrap();

    let started = Instant::now();
    // Matched rather than `expect_err`, which would want `DataLock: Debug` â€” a derive on a
    // production type to satisfy a test is the test choosing what the type looks like.
    let outcome = DataLock::acquire(&dir, "waiter", Duration::from_millis(700));
    let waited = started.elapsed();
    let Err(err) = outcome else {
        panic!("the holder never let go, so the wait must fail â€” it succeeded after {waited:?}")
    };

    assert!(
        waited >= Duration::from_millis(450),
        "gave up after {waited:?} on a 700ms timeout"
    );
    // And it still says who, because a timeout that names nobody is the sentence this file
    // exists to print going missing at the one moment it is read.
    assert!(
        err.to_string().contains("holder"),
        "the timeout does not name the holder: {err}"
    );
}
