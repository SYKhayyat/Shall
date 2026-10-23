use super::*;

/// The bound is on silence, not on duration — the distinction the whole fix rests on. A
/// wall-clock cap set low enough to catch the hang would kill every real build.
///
/// **The bound is measured, not written down.** It is this host's interpreter start-up cost
/// plus three seconds, because the idle clock starts at spawn and start-up therefore counts
/// as silence — see [`interpreter_start_up_cost`]. A constant here is an assertion about the
/// machine: 3s failed under three concurrent `cargo test` jobs, 5s failed under one sharing
/// the box with a container build, and the next constant fails on the next busier day. The
/// property under test never changed: a child that runs *longer* than the idle bound while
/// still printing survives it.
///
/// **How long it talks is derived from the bound, not written down either** — and that is the
/// second calibration this test needed. A fixed twelve ticks was six seconds of talking
/// against a bound of *measured start-up + 3s*, which holds only while the fixture's own
/// start-up matches the one the calibration measured. Under a full `cargo test
/// --no-fail-fast` they diverged — the calibration caught a 4.6s start-up and the fixture a
/// 1.5s one — so the run finished **29ms under** its own bound and the self-test below
/// correctly said the test had proved nothing. Talking for the whole bound plus the margin is
/// true whatever either start-up turns out to be.
#[tokio::test]
async fn a_child_that_keeps_talking_outlives_the_bound() {
    let margin = std::time::Duration::from_secs(3);
    let bound = interpreter_start_up_cost().await + margin;
    let ticks = ((bound + margin).as_millis() / 500) as usize;

    let layer = RawExecutor::with_idle(ChildStdin::Closed, Some(bound));
    let (cmd, args) = chatty_for_a_while(ticks);
    let started = std::time::Instant::now();
    let out = layer
        .execute(cmd, &args, &HashMap::new())
        .await
        .unwrap_or_else(|e| {
            panic!("a command that is still printing has not hung (bound {bound:?}): {e}")
        });
    assert!(
        started.elapsed() > bound,
        "the fixture ran {:?} against a {bound:?} bound — it must outlive it or it proves \
         nothing",
        started.elapsed()
    );
    assert!(out.status.success());
    assert_eq!(
        String::from_utf8_lossy(&out.stdout).matches("tick").count(),
        ticks
    );
}
