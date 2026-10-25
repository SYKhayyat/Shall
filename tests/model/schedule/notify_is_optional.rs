use super::*;

#[test]
fn notify_is_optional() {
    let o = opts(&[("cron", "0 2 * * *"), ("run", "clean")]);
    let cfg = schedule_config("t", &o, &origin(), &shipped()).unwrap();
    assert!(cfg.notification.is_none());
}
