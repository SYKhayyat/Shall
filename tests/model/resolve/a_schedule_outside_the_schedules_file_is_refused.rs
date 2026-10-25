use super::*;

#[test]
fn a_schedule_outside_the_schedules_file_is_refused() {
    // S21: a `schedule:` line belongs in the `schedules` file, and only there. One in a
    // module is an error naming the file, not a silently-parsed extra.
    let f = fx(
        "Work\n",
        &[("Work", "use m\n")],
        &[("m.txt", "schedule:nightly@cron=0 2 * * *,run=clean\n")],
    );
    let err = resolve(&f).unwrap_err();
    assert!(
        err.what.contains("is not in the `schedules` file"),
        "{}",
        err
    );
}
