use super::*;

/// The refusal must say the two things a reader needs: that it will not start, and why the
/// missing snapshot provider is what stopped it.
#[test]
fn the_refusal_names_the_checks_and_the_reason() {
    let checks = vec![
        Check {
            subject: "apt:nginx".into(),
            probe: Probe::Port(80),
        },
        Check {
            subject: "preferences.toml".into(),
            probe: Probe::Command("true".into()),
        },
    ];
    let msg = cannot_revert_refusal(&checks);
    assert!(msg.contains("refusing to start"), "{}", msg);
    assert!(msg.contains("apt:nginx"), "{}", msg);
    assert!(msg.contains("preferences.toml"), "{}", msg);
    assert!(msg.contains("no snapshot provider"), "{}", msg);
}
