use super::*;

/// The siblings. Every one of these went through the same two lines that dropped the
/// padding, and each produced either a hard error or — worse — a schedule that fires more
/// often than it was told to.
#[test]
fn schtasks_maps_every_shape_a_cron_can_take() {
    let cases: &[(&str, &[&str])] = &[
        // `@daily` split on whitespace to ONE field, so the minute came out as the literal
        // string "@daily" and the time was `02:@daily`.
        ("@daily", &["/SC", "DAILY", "/ST", "00:00"]),
        ("@midnight", &["/SC", "DAILY", "/ST", "00:00"]),
        ("@hourly", &["/SC", "HOURLY", "/ST", "00:00"]),
        ("@reboot", &["/SC", "ONSTART"]),
        ("@weekly", &["/SC", "WEEKLY", "/D", "MON", "/ST", "00:00"]),
        ("@monthly", &["/SC", "MONTHLY", "/D", "1", "/ST", "00:00"]),
        (
            "@yearly",
            &["/SC", "MONTHLY", "/M", "JAN", "/D", "1", "/ST", "00:00"],
        ),
        (
            "@annually",
            &["/SC", "MONTHLY", "/M", "JAN", "/D", "1", "/ST", "00:00"],
        ),
        // A step in the minute field became the time `*:*/15`.
        (
            "*/15 * * * *",
            &["/SC", "MINUTE", "/MO", "15", "/ST", "00:00"],
        ),
        ("* * * * *", &["/SC", "MINUTE", "/ST", "00:00"]),
        // A step in the hour field became `*/6:0`.
        (
            "0 */6 * * *",
            &["/SC", "HOURLY", "/MO", "6", "/ST", "00:00"],
        ),
        ("0 * * * *", &["/SC", "HOURLY", "/ST", "00:00"]),
        // Day-of-week was ignored entirely: this ran EVERY day, seven times as often as
        // it was declared to, and reported success while doing it.
        ("0 3 * * 1", &["/SC", "WEEKLY", "/D", "MON", "/ST", "03:00"]),
        (
            "30 4 * * 0",
            &["/SC", "WEEKLY", "/D", "SUN", "/ST", "04:30"],
        ),
        (
            "0 9 * * 1-5",
            &["/SC", "WEEKLY", "/D", "MON,TUE,WED,THU,FRI", "/ST", "09:00"],
        ),
        (
            "0 9 * * 1,3",
            &["/SC", "WEEKLY", "/D", "MON,WED", "/ST", "09:00"],
        ),
        // Day-of-month was ignored too: monthly became daily.
        ("30 4 1 * *", &["/SC", "MONTHLY", "/D", "1", "/ST", "04:30"]),
    ];
    for (cron, want) in cases {
        assert_eq!(
            map_cron_to_schtasks(cron).unwrap_or_else(|e| panic!("{cron}: {e}")),
            *want,
            "wrong schtasks args for `{cron}`"
        );
    }
}
