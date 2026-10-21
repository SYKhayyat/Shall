use super::*;

/// And the protection those names buy: a removal of one is refused, by the OS-essential
/// rule rather than by a config rule.
///
/// This is the assertion the container measured on two binaries. It is here so that the
/// next person to delete `essential_names` finds out on their own machine in 30ms rather
/// than from apt on somebody's laptop.
#[tokio::test]
async fn a_package_the_os_calls_essential_is_never_removed() {
    let registry = registry_reporting("apt", &["tar", "sed", "grep", "gzip", "findutils"]);
    // No `protected_packages` rule covers these â€” the static config list is a different
    // mechanism and the container measurement turned specifically on the five it misses.
    let cfg = Config::default();
    let reaping = Reaping::new();

    let report = inspect_removals(
        &cfg,
        &registry,
        &pairs(&["tar", "sed", "grep", "gzip", "findutils"]),
        RemovalKind::Package,
        &reaping,
        &mut EssentialsCache::default(),
    )
    .await;

    assert!(
        !report.is_empty(),
        "five packages the OS reports as essential were planned for removal and the guard \
             said nothing"
    );
    let message = report.message(GuardScope::Sync, RemovalKind::Package);
    for name in ["tar", "sed", "grep", "gzip", "findutils"] {
        assert!(
            message.contains(name),
            "`{name}` is essential to the running system and the refusal does not name \
                 it:\n{message}"
        );
    }
}
