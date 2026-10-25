use super::*;

/// A URL that is not in the state file at all is not an error: `remove` is called with what
/// the plan asked to remove, and a resource already gone is the end state that was wanted.
#[tokio::test]
async fn removing_something_that_was_never_recorded_is_not_a_failure() {
    let (web, _mock, _tmp) = backend("absent");
    web.remove(
        &["https://example.invalid/never-installed.tar.gz".to_string()],
        false,
        reaped().await,
    )
    .await
    .expect("an already-absent resource is the end state that was asked for");
}
