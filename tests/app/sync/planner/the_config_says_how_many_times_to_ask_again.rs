use super::*;

/// `[query] retries` is a number, and this is the number: one extra ask by default of nothing,
/// and enough of them to see through a manager that answers on the second try.
#[tokio::test]
async fn the_config_says_how_many_times_to_ask_again() {
    let mut config = Config::default();

    let once = Unreadable::new("brew", 1);
    let registry = registry_with_unreadable(&once, &[]);
    let changes = plan_with(registry, &config, &["brew"], &[("brew", "wget")]).await;
    assert_eq!(once.asks(), 1, "the default asks exactly once");
    assert_eq!(
        changes.total_install(),
        0,
        "a manager that could not answer is not evidence that the package is absent"
    );
    assert_eq!(
        changes
            .skipped
            .iter()
            .filter(|s| s.kind == shall::app::sync::planner::SkipKind::CouldNotAsk)
            .count(),
        1,
        "and the declaration is reported rather than dropped: {:?}",
        changes.skipped
    );

    config.query.retries = 1;
    let twice = Unreadable::new("brew", 1);
    let registry = registry_with_unreadable(&twice, &[]);
    let changes = plan_with(registry, &config, &["brew"], &[("brew", "wget")]).await;
    assert_eq!(twice.asks(), 2, "one retry is one more ask");
    assert_eq!(changes.total_install(), 1, "and the answer is believed");
    assert!(
        changes.skipped.is_empty(),
        "a manager that answers on the retry is not reported as unreadable: {:?}",
        changes.skipped
    );
}
