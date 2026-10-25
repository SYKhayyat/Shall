use super::*;

/// The two halves meet: for every cron Task Scheduler can express, the `when` derived from
/// the `/SC` arguments is one of the strings the XML reader produces. A canonical form only
/// one side can write is not a comparison, it is a permanent mismatch.
#[test]
fn the_rendered_when_and_the_read_when_use_one_vocabulary() {
    let pairs: &[(&str, &str)] = &[
        ("0 3 * * *", "daily at 03:00"),
        ("@reboot", "boot"),
        ("0 9 * * 1,3", "weekly on MON,WED at 09:00"),
        ("30 4 1 * *", "monthly on 1 at 04:30"),
        ("@yearly", "monthly in JAN on 1 at 00:00"),
        ("*/15 * * * *", "every 15m"),
        ("0 */6 * * *", "every 6h from 00:00"),
    ];
    for (cron, want) in pairs {
        let args = map_cron_to_schtasks(cron).unwrap();
        assert_eq!(
            schtasks_when(&args).as_deref(),
            Some(*want),
            "`{}` canonicalises wrong",
            cron
        );
    }
}
