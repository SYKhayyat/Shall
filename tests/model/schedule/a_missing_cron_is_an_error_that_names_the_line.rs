use super::*;

#[test]
fn a_missing_cron_is_an_error_that_names_the_line() {
    let o = opts(&[("run", "clean")]);
    let err = schedule_config("t", &o, &origin(), &shipped()).unwrap_err();
    assert!(err.what.contains("missing `cron`"), "{}", err);
    assert!(err.to_string().contains("schedules:3"), "{}", err);
}
