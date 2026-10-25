use super::*;

#[test]
fn same_backend_twice_is_not_a_conflict() {
    let specs = vec![
        spec("apt", "python3", Some("3.11")),
        spec("apt", "python", Some("3.11")),
    ];
    // Both from apt → one provider → no cross-backend conflict.
    assert!(detect_conflicts(&specs).is_empty());
}
