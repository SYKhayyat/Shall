use super::*;

/// The refusal has to be actionable: it names the port, what would happen, and the two ways
/// out. A user reading it is, by construction, about to lose their connection.
///
/// **It used to take the prose as a `&str`, and this test handed it the prose directly** —
/// which is why nobody noticed that the only producer emitted `"an unattended watch tick"`
/// while the only consumer matched on `"watch"`. The test asserted the string it had just
/// written down. It now passes the scope, so the wording is the enum's answer and this
/// asserts the round trip rather than a literal.
#[test]
fn the_refusal_names_the_port_and_the_way_out() {
    use shall::app::sync::guard::GuardScope;

    let msg = lockout_refusal(22, GuardScope::Watch);
    assert!(msg.contains("port 22"), "{}", msg);
    assert!(msg.contains("firewall:22/tcp"), "{}", msg);
    assert!(msg.contains("console"), "{}", msg);
    assert!(
        msg.contains("an unattended watch tick"),
        "an unattended tick is the dangerous one (N7) and the refusal must say so: {}",
        msg
    );

    // And it must not say the same thing about every scope, which is what the label it
    // replaced did for nine of the twelve.
    let sync = lockout_refusal(22, GuardScope::Sync);
    assert!(
        !sync.contains("unattended"),
        "an attended `sync` was described as unattended: {}",
        sync
    );
    let purge = lockout_refusal(22, GuardScope::PurgeUndeclared);
    assert!(
        purge.contains("purge-undeclared"),
        "a purge was reported under another command's name: {}",
        purge
    );
}
