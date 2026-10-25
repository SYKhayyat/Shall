use super::*;

#[test]
fn no_conflict_for_single_backend() {
    let specs = vec![spec("apt", "curl", None), spec("cargo", "ripgrep", None)];
    assert!(detect_conflicts(&specs).is_empty());
}
