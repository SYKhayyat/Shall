use super::*;

/// A run of failures that were *all* passing ones is itself a passing failure — the whole
/// reason the class is carried at all. If folding could only ever make things worse, the
/// summary would answer `permanent` to every partial run and the class would carry no
/// information.
#[test]
fn a_run_of_passing_failures_stays_passing() {
    let all_transient = [Retryability::Transient; 3]
        .into_iter()
        .fold(Retryability::Transient, Retryability::and_also);
    assert_eq!(all_transient, Retryability::Transient);
}
