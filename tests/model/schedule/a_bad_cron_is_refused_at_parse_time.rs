use super::*;

/// A bad cron is refused where the line is read, so the error can name the file and the
/// line rather than surfacing when the OS scheduler is handed the job.
#[test]
fn a_bad_cron_is_refused_at_parse_time() {
    let o = opts(&[("cron", "not a cron"), ("run", "sync")]);
    let err = schedule_config("nightly", &o, &origin(), &shipped()).unwrap_err();
    assert!(err.to_string().contains("schedules:3"), "{}", err);
}
