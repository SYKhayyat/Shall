use super::*;

/// The no-runtime case: a plain call, not a panic. Every unit test in this repo is one.
#[test]
fn without_a_runtime_it_simply_runs_the_work() {
    assert_eq!(on_the_terminal(|| 6 * 7), 42);
}
