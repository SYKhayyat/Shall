use super::*;

/// A restart is the one thing that stays imperative, and the options handed on carry the
/// transition alone — the enablement is the configuration's now, and asking for it twice is
/// the two-owners problem.
#[test]
fn only_a_restart_is_left_for_the_init_to_do() {
    assert!(imperative_remainder(&Options::default()).is_none());
    assert!(imperative_remainder(&opts(&[("enabled", "true")])).is_none());
    assert!(imperative_remainder(&opts(&[("status", "running")])).is_none());
    assert!(imperative_remainder(&opts(&[("status", "stopped")])).is_none());

    for restart in ["restarted", "restart"] {
        let left = imperative_remainder(&opts(&[("status", restart)]))
            .expect("a restart is still the init's");
        assert_eq!(left.one("status"), Some("restarted"));
        assert_eq!(left.one("enabled"), None);
    }
    // And with an enablement beside it, the enablement does NOT come along.
    let left = imperative_remainder(&opts(&[("enabled", "true"), ("status", "restarted")]))
        .expect("the restart half survives");
    assert_eq!(left.one("enabled"), None);
    assert_eq!(left.len(), 1);
}
