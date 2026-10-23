use super::*;

#[test]
fn scoop_missing_manifest_is_a_failure_despite_exit_zero() {
    let scoop = executor_for(exit_policy::scoop());
    let out = "Couldn't find manifest for 'shall-nonexistent-pkg'.\n";
    assert!(scoop.ensure_status("scoop", finished(0, out, "")).is_err());
    assert!(scoop
        .ensure_status(
            "scoop",
            finished(0, "'jq' (1.8.2) was installed successfully!\n", "")
        )
        .is_ok());
}
