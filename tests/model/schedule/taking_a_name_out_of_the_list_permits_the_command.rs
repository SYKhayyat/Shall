use super::*;

/// Taking a name out of the list is how a machine permits that command — the whole point
/// of the ruling that replaced the constant.
#[test]
fn taking_a_name_out_of_the_list_permits_the_command() {
    let permitting = vec!["purge-undeclared".to_string()];
    let o = opts(&[("cron", "0 2 * * *"), ("run", "rebuild --all")]);
    assert!(schedule_config("nightly", &o, &origin(), &permitting).is_ok());

    // And the name still in the list is still refused, so the edit is per-command.
    let o = opts(&[("cron", "0 2 * * *"), ("run", "purge-undeclared")]);
    assert!(schedule_config("nightly", &o, &origin(), &permitting).is_err());
}
