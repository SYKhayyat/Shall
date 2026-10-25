use super::*;

/// **Not "could not ask" is not "is not installed".** A manager that fails must not schedule
/// an install for a package it may well already have: that is a transaction node for
/// nothing, and one later failure rolls back across the whole set.
#[tokio::test]
async fn a_manager_that_cannot_answer_schedules_no_install() {
    let config = Config::default();
    let unreadable = Unreadable::new("brew", usize::MAX);
    let registry = registry_with_unreadable(&unreadable, &[]);
    let changes = plan_with(registry, &config, &["brew"], &[("brew", "wget")]).await;
    assert_eq!(
        changes.total_install(),
        0,
        "Shall cannot see whether wget is installed, so it must not install it"
    );
}
