use super::*;

/// **The fail-open hole.** A backend whose essential query fails used to contribute
/// nothing â€” which to the guard reads exactly like "nothing here is essential" â€” so the
/// whole run lost its OS-essential protection, `purge-undeclared` included. Now the
/// failure is an answer too: removals through *that* backend are refused until it can
/// answer, while removals through backends that did answer are judged normally. The
/// refusal is protection-class: no mass flag clears it.
#[tokio::test]
async fn an_essential_query_failure_refuses_removals_through_that_backend_only() {
    let registry = registry_with(&[("apt", &["tar"], true), ("cargo", &[], false)]);
    let cfg = Config::default();
    let reaping = Reaping::new();

    let report = inspect_removals(
        &cfg,
        &registry,
        &[("apt".into(), "curl".into())],
        RemovalKind::Package,
        &reaping,
        &mut EssentialsCache::default(),
    )
    .await;
    assert!(
        report.objections.iter().any(|o| matches!(
            o,
            Objection::UnverifiedEssentials { key, .. } if key == "apt:curl"
        )),
        "a removal through a backend that cannot say what the OS needs must be refused: \
             {:?}",
        report.objections
    );

    // The sibling through a backend that answered: judged on its merits, not tarred by
    // apt's bad day.
    let other = inspect_removals(
        &cfg,
        &registry,
        &[("cargo".into(), "ripgrep".into())],
        RemovalKind::Package,
        &Reaping::new(),
        &mut EssentialsCache::default(),
    )
    .await;
    assert!(
        other.is_empty(),
        "a backend that answered must not inherit the refusal: {:?}",
        other.objections
    );

    // And the message says why, naming the manager that could not answer.
    let message = report.message(GuardScope::Sync, RemovalKind::Package);
    assert!(
        message.contains("`apt`") && message.contains("the OS needs"),
        "the refusal names the backend and the missing check:\n{message}"
    );
}
