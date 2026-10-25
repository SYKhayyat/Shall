use super::*;

/// One subject with several checks is named once, not once per check.
#[test]
fn the_refusal_does_not_repeat_a_subject() {
    let checks = vec![
        Check {
            subject: "apt:nginx".into(),
            probe: Probe::Port(80),
        },
        Check {
            subject: "apt:nginx".into(),
            probe: Probe::Port(443),
        },
    ];
    let msg = cannot_revert_refusal(&checks);
    assert_eq!(msg.matches("apt:nginx").count(), 1, "{}", msg);
    // ...but the count is of checks, not subjects: two things are being checked.
    assert!(msg.contains("2 health check(s)"), "{}", msg);
}
