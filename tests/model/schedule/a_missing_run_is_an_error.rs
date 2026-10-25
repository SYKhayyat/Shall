use super::*;

#[test]
fn a_missing_run_is_an_error() {
    let o = opts(&[("cron", "0 2 * * *")]);
    let err = schedule_config("t", &o, &origin(), &shipped()).unwrap_err();
    assert!(err.what.contains("missing `run`"), "{}", err);
}
