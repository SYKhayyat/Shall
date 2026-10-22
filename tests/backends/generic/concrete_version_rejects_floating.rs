use super::*;

#[test]
fn concrete_version_rejects_floating() {
    assert!(is_concrete_version("1.2.3"));
    assert!(!is_concrete_version("latest"));
    assert!(!is_concrete_version("*"));
    assert!(!is_concrete_version(""));
}
