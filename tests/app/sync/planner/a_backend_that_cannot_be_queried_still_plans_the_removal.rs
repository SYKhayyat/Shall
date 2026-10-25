use super::*;

/// A backend that cannot answer must not have its removals silently dropped. Unknown
/// means "assume it is there" — the behaviour that existed before the check — so a
/// backend having a bad day cannot quietly disable `absent:`.
#[tokio::test]
async fn a_backend_that_cannot_be_queried_still_plans_the_removal() {
    // The manager is **here** and did not answer. This used to be an empty registry,
    // which is a different machine entirely — see `registry_that_cannot_answer`.
    let changes = absent_removals(registry_that_cannot_answer("generic-test"), "libreoffice").await;
    assert_eq!(changes, 1, "not knowing must never mean not removing");
}
