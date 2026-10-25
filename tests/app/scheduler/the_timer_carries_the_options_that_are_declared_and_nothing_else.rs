use super::*;

/// Every new option reaches the unit systemd reads, and an option nobody wrote leaves the
/// timer exactly as it was before the option existed.
#[test]
fn the_timer_carries_the_options_that_are_declared_and_nothing_else() {
    let p = LinuxSystemdProvisioner;
    let plain = p.timer_unit(&cfg("0 2 * * *"));
    assert!(plain.contains("Persistent=true"), "{}", plain);
    assert!(!plain.contains("RandomizedDelaySec"), "{}", plain);

    let mut rich = cfg("0 2 * * *");
    rich.persistent = Some(false);
    rich.jitter = Some(1800);
    let unit = p.timer_unit(&rich);
    assert!(unit.contains("Persistent=false"), "{}", unit);
    assert!(unit.contains("RandomizedDelaySec=1800"), "{}", unit);
}
