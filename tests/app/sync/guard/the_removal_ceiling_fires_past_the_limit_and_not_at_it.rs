use super::*;

/// The ceiling triggers **above** `max_removals`, not at it.
///
/// The second finding in the same mutation run: three of the six survivors were
/// numeric-boundary mutations, and `too_many_changes`'s `>`â†’`>=` is the behavioural one â€”
/// it moves the refusal to fire *at* the limit instead of past it. It errs safe, which is
/// why it is a boundary test and not a bug report, and it says the same thing about the
/// other two: the thresholds were tested for "well over" and "well under" and never at the
/// limit itself, which is the only place a ceiling is interesting.
#[tokio::test]
async fn the_removal_ceiling_fires_past_the_limit_and_not_at_it() {
    let cfg = Config {
        guard: shall::config::GuardSettings {
            max_removals: 3,
            ..Default::default()
        },
        ..Default::default()
    };
    let registry = registry_reporting("apt", &[]);

    // Exactly at the limit: allowed.
    let reaping = Reaping::new();
    let at = inspect_removals(
        &cfg,
        &registry,
        &pairs(&["a", "b", "c"]),
        RemovalKind::Package,
        &reaping,
        &mut EssentialsCache::default(),
    )
    .await;
    assert!(
        at.objections.is_empty(),
        "three removals under a ceiling of three is at the limit, not over it: {:?}",
        at.objections
    );

    // One past it: refused.
    let reaping = Reaping::new();
    let over = inspect_removals(
        &cfg,
        &registry,
        &pairs(&["a", "b", "c", "d"]),
        RemovalKind::Package,
        &reaping,
        &mut EssentialsCache::default(),
    )
    .await;
    assert!(
        !over.objections.is_empty(),
        "four removals under a ceiling of three must be refused"
    );
}
