use super::*;

/// A name with no record is a skip, not a failure: `remove` is handed what the plan asked to
/// remove, and something already gone is the end state that was wanted.
#[tokio::test]
async fn removing_something_that_was_never_recorded_is_not_a_failure() {
    let (app, _tmp) = backend("absent");
    app.remove(&["never-installed".to_string()], false, reaped().await)
        .await
        .expect("an already-absent AppImage is the end state that was asked for");
}
