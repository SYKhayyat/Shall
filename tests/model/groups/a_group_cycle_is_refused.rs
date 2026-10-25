use super::*;

/// A group that reaches itself is a cycle, refused like a `use` loop — it has no answer.
#[test]
fn a_group_cycle_is_refused() {
    assert!(Groups::parse("a = b\nb = a\n").is_err());
    assert!(Groups::parse("a = a\n").is_err());
    let err = Groups::parse("x = y\ny = z\nz = x\n").unwrap_err();
    assert!(err.contains("reaches itself"), "{}", err);
}
