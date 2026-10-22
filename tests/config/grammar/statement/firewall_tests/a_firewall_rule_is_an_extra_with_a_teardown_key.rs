use super::*;

/// A firewall rule is a noun with a teardown — unlike `exec:`, it belongs in the extras
/// ledger so that deleting the line closes the port (N5).
#[test]
fn a_firewall_rule_is_an_extra_with_a_teardown_key() {
    let stmt = pv("firewall:22/tcp").unwrap();
    assert_eq!(
        shall::core::extra_key(&stmt)
            .unwrap()
            .map(|k| k.to_string())
            .as_deref(),
        Some("firewall:22/tcp")
    );
}
