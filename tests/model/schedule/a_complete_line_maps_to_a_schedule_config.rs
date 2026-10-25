use super::*;

#[test]
fn a_complete_line_maps_to_a_schedule_config() {
    let o = opts(&[
        ("cron", "0 2 * * *"),
        ("run", "clean"),
        ("notify", "desktop"),
    ]);
    let cfg = schedule_config("nightly-tidy", &o, &origin(), &shipped()).unwrap();
    assert_eq!(cfg.name, "nightly-tidy");
    assert_eq!(cfg.cron, "0 2 * * *");
    assert_eq!(cfg.command, "clean");
    assert_eq!(cfg.notification.as_deref(), Some("desktop"));
    // Nothing declared is nothing decided: every option a line does not mention arrives as
    // `None`, which is what stops a scheduler refusing an option nobody wrote.
    assert_eq!(cfg.enabled, None);
    assert_eq!(cfg.persistent, None);
    assert_eq!(cfg.jitter, None);
    assert_eq!(cfg.elevated, None);
}
