use super::*;

/// A cron Task Scheduler genuinely cannot express is refused by name — it does not quietly
/// become DAILY. Running more often than declared is the failure mode this whole fix is
/// about, and it must not survive as the error path.
#[test]
fn a_cron_windows_cannot_express_is_refused_rather_than_widened() {
    // Every 15 minutes, but only during hour 9. `/SC MINUTE /MO 15` runs all day.
    let err = map_cron_to_schtasks("*/15 9 * * *").unwrap_err();
    assert!(
        err.contains("*/15 9 * * *"),
        "does not quote the cron: {err}"
    );
    assert!(
        err.to_lowercase().contains("task scheduler"),
        "does not say which scheduler cannot do it: {err}"
    );
}
