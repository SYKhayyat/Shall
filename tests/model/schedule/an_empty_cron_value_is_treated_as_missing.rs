use super::*;

#[test]
fn an_empty_cron_value_is_treated_as_missing() {
    let o = opts(&[("cron", "  "), ("run", "clean")]);
    assert!(schedule_config("t", &o, &origin(), &shipped()).is_err());
}
