/// Which runs are reconciliations, asserted as the two exceptions rather than as ten rules.
#[test]
fn only_a_reconciling_run_may_leave_a_removal_in_place() {
    use shall::app::sync::guard::GuardScope as S;
    for scope in [
        S::Apply,
        S::RemoveOrphans,
        S::PurgeUndeclared,
        S::Sync,
        S::Watch,
        S::Upgrade,
        S::Canary,
        S::ShellExit,
        S::ExpirySweep,
        S::Heal,
    ] {
        assert!(
            scope.reconciles(),
            "{} removes what nothing declares",
            scope.as_str()
        );
    }
    // A rebuild's removal phase is the first half of a reinstall of DECLARED packages,
    // split into two transactions so the Remove and the Install cannot race in one graph.
    // Leaving one of those removals in place is a machine missing software it declares.
    assert!(!S::Rebuild.reconciles());
    // And an uninstall was typed by a person, not derived from a manifest.
    assert!(!S::Remove.reconciles());
}
