use super::*;

/// The comparison the planner runs. An unparseable declaration is not drift: the backend
/// refuses it by name, and a planner that called it a change would schedule that refusal on
/// every sync for ever.
#[test]
fn comparison_is_by_value_and_an_unreadable_declaration_is_not_drift() {
    assert!(same_size("10G", 10 * (1 << 30)));
    assert!(same_size("10240M", 10 * (1 << 30)));
    assert!(!same_size("10G", 20 * (1 << 30)));
    assert!(same_size("enormous", 42));
}
