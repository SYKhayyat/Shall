use super::*;

/// The bound watches the child's **exit**, and the read of its output sits outside it.
///
/// A manager that backgrounds its work and returns leaves Shall reading a pipe the orphan
/// still holds open: `child.wait()` has already returned, so the loop that could abort the
/// readers is over, and `out_task.await` has no clock on it at all. There is no child left
/// to kill and nothing in the tree that can end the wait.
///
/// Found by capturing a wedged sweep instead of killing it: `shall -y install
/// nimble:nimjson` at zero CPU with **no children**, while three orphaned `nim.exe` ran
/// outside its process tree. Then reproduced to a number — a 20s bound and a child that
/// detached for 60s took 64s and reported SUCCESS.
#[tokio::test]
async fn a_detached_grandchild_cannot_hold_the_read_open_past_the_bound() {
    const HOLD: usize = 10;
    let (cmd, args) = detaches_holding_stdout(HOLD);

    // **The fixture is checked before it is trusted.** The first version of this test used a
    // command Windows rejected for bad quoting, so it returned in 0.06s and passed while
    // proving nothing — the defect class this whole file exists to catch. An unbounded layer
    // must sit here for the orphan's whole life; if it does not, the fixture is not holding
    // the pipe and the assertion below would be meaningless.
    let control = RawExecutor::with_idle(ChildStdin::Closed, None);
    let t0 = std::time::Instant::now();
    let _ = control.execute(&cmd, &args, &HashMap::new()).await;
    let held = t0.elapsed();
    assert!(
        held >= std::time::Duration::from_secs(7),
        "the fixture returned after {:?}, so no orphan is holding the pipe and this test \
         cannot fail — fix the fixture before reading the result",
        held
    );

    let layer = RawExecutor::with_idle(ChildStdin::Closed, Some(std::time::Duration::from_secs(2)));
    let started = std::time::Instant::now();
    let outcome = layer.execute(&cmd, &args, &HashMap::new()).await;
    let waited = started.elapsed();

    // Above the bound and below the orphan's lifetime: this asserts that *a* bound applied,
    // not how long it took to apply.
    assert!(
        waited < std::time::Duration::from_secs(7),
        "waited {:?} for a child that had already exited, on a 2s bound — the bound covers \
         the wait but not the read, so an orphan holding the pipe sets the duration",
        waited
    );
    // The separable half, and the worse one: the child exits 0, so before the bound reached
    // the readers this returned SUCCESS after waiting out the orphan. A command Shall
    // stopped waiting on did not do what it was asked (Q28).
    let err = outcome.expect_err("a command Shall gave up on must not report success");
    let said = err.to_string();
    assert!(
        said.contains(&cmd) && said.contains("holds its output open"),
        "the failure must name the command and why Shall stopped: {}",
        said
    );
    assert_eq!(
        err.retryability(),
        Retryability::Permanent,
        "a second attempt spends another bound reproducing this"
    );
}
