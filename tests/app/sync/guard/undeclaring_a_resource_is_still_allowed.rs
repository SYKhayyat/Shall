use super::*;

/// The other half of the same rule: `sync` undoing a `service:` line the user deleted is a
/// teardown (`RemovalKind::Extra`) and must still go through. A guard that refuses both
/// directions has made the declaration unremovable, which is the bug `plan.rs` already hit
/// once when it ran the package test over resource keys.
#[tokio::test]
async fn undeclaring_a_resource_is_still_allowed() {
    let registry = Arc::new(BackendRegistry::default());
    let report = inspect_removals(
        &Config::default(),
        &registry,
        &[("service".to_string(), "sshd".to_string())],
        RemovalKind::Extra,
        &Reaping::new(),
        &mut EssentialsCache::default(),
    )
    .await;
    assert!(
        report.is_empty(),
        "deleting the line is how a service is torn down: {:?}",
        report.objections
    );
}
