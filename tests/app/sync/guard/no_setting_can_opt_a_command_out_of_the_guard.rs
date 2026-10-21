use super::*;

#[tokio::test]
async fn no_setting_can_opt_a_command_out_of_the_guard() {
    // `[guard.enforce_on]` used to do exactly this: a config key that switched the
    // guard off per command, so `enforce_on.sync = false` â€” copied from a dotfiles repo
    // â€” made a routine sync remove python3. II.10 lists ten refusals and that was not
    // one of them; V.21 says no setting anyone can flip makes sync dangerous.
    let reg = Arc::new(BackendRegistry::new());
    let cfg = config_with(20);
    for scope in [
        GuardScope::Apply,
        GuardScope::Sync,
        GuardScope::RemoveOrphans,
        GuardScope::PurgeUndeclared,
        GuardScope::Watch,
        GuardScope::Upgrade,
        GuardScope::Canary,
        GuardScope::Remove,
        GuardScope::ShellExit,
        GuardScope::ExpirySweep,
        GuardScope::Heal,
        GuardScope::Rebuild,
    ] {
        assert!(
            enforce(&cfg, &reg, &pairs(&["python3"]), &Reaping::new(), scope)
                .await
                .is_err(),
            "{:?} must be guarded, and nothing may turn that off",
            scope
        );
    }
}
