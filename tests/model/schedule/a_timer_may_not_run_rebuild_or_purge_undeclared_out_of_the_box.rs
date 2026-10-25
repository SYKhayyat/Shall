use super::*;

#[test]
fn a_timer_may_not_run_rebuild_or_purge_undeclared_out_of_the_box() {
    // K13. Both shipped names, not just the one the ruling was asked about: a check that
    // covers `rebuild` alone is how `purge-undeclared` came to be refused by a constant
    // nobody could edit.
    for command in ["rebuild --all", "purge-undeclared"] {
        let o = opts(&[("cron", "0 2 * * *"), ("run", command)]);
        let err = schedule_config("nightly", &o, &origin(), &shipped()).unwrap_err();
        let head = command.split_whitespace().next().unwrap();
        assert!(
            err.what.contains(&format!("may not run `{}`", head)),
            "{}",
            err
        );
    }
}
