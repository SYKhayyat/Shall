use super::*;

/// Nesting can go several deep, and an overlap between two nested groups unions cleanly
/// (first occurrence wins the order) rather than tripping the grammar's "named twice".
#[test]
fn nesting_is_transitive_and_dedups() {
    let g = Groups::parse("a = apt\nb = a, dnf\nc = b, apt, cargo\n").unwrap();
    // c → b → a(apt) + dnf, then apt (dropped, already have it), then cargo.
    assert_eq!(
        g.expand("c"),
        Some(["apt".to_string(), "dnf".to_string(), "cargo".to_string()].as_slice())
    );
}
