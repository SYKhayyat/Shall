use super::*;

/// The ruling: a manager that could not answer has not said no, so the name still
/// falls through — but nothing is written down, and the next sync asks again.
#[tokio::test]
async fn a_manager_that_could_not_answer_leaves_no_lock() {
    let (backend, recorded) = settle(
        &bare_jq(),
        registry(DryRunOutput::faulted("E: package lists are empty"), found()),
    )
    .await
    .unwrap();
    assert_eq!(backend, "second");
    assert_eq!(recorded, None, "a guess must not be frozen");
}
