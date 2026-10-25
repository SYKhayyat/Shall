use super::*;

/// **The check-then-insert window, driven from sixteen threads at once.**
///
/// The four tests above ask for one policy in a loop, in one thread, which is why the race
/// survived all of them: a sequential caller always finds the client the last iteration left,
/// and the pool's *contents* look identical however many clients were built and thrown away.
/// So this one counts builds — the only observable that moves — and synchronises the callers
/// on a barrier so they arrive together rather than in a queue.
///
/// Sixteen is not decoration. Two threads can pass the `get` before either `insert`s often
/// enough to matter; the point of a barrier is that none of them has started yet.
#[test]
fn sixteen_threads_asking_at_once_build_one_client() {
    use std::sync::{Arc, Barrier};
    const THREADS: usize = 16;
    // A fresh user agent per round, so each round is its own key and cannot be answered by
    // a client an earlier round left behind.
    let agent = format!("shall-pool-race-{}", std::process::id());
    let barrier = Arc::new(Barrier::new(THREADS));

    std::thread::scope(|scope| {
        for _ in 0..THREADS {
            let barrier = Arc::clone(&barrier);
            let agent = agent.clone();
            scope.spawn(move || {
                barrier.wait();
                let _ = client(&agent, false, 15).unwrap();
            });
        }
    });

    let built = builds_for_test(&agent);
    assert_eq!(
        built, 1,
        "{THREADS} threads asking for one policy at the same instant built {built} clients. \
         The pool keeps one and discards the rest, and a discarded client is a discarded \
         connection pool — the cost lands on the first real request, not on the ask that \
         caused it. `client` must go through `POOL.entry` (`PLAN.md` #85)."
    );
    assert_eq!(
        mine(&agent),
        1,
        "and the pool should hold exactly one client for that policy"
    );
}
