use super::*;

/// The message has to say why *now* matters, or a reader postpones it until after the
/// reboot — which is the one time it cannot be fixed from the machine itself.
#[test]
fn the_failure_says_why_before_the_reboot_matters() {
    let msg = failed_to_build(
        &[Module {
            name: "nvidia".into(),
            version: "550.78".into(),
            kernel: "6.8.0-35-generic".into(),
            state: "added".into(),
        }],
        "6.8.0-35-generic",
    );
    assert!(msg.contains("nvidia/550.78"), "{}", msg);
    assert!(msg.contains("6.8.0-35-generic"), "{}", msg);
    assert!(msg.contains("reboot"), "{}", msg);
    assert!(msg.contains("make.log"), "{}", msg);
}
