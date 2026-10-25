use super::*;

#[test]
fn hold_matches_qualified_and_bare_names() {
    let mut r = StateRegistry::new(PathBuf::from("/tmp/x"));
    assert!(r.hold("cargo:ripgrep"));
    assert!(!r.hold("cargo:ripgrep"), "no duplicate");
    assert!(r.hold("curl"));

    assert!(r.is_held("cargo", "ripgrep"));
    assert!(r.is_held("apt", "curl")); // bare-name match across any backend
    assert!(!r.is_held("cargo", "bat"));

    assert!(r.unhold("cargo:ripgrep"));
    assert!(!r.is_held("cargo", "ripgrep"));
    assert!(!r.unhold("cargo:ripgrep"), "already gone");
    assert_eq!(r.list_held(), &["curl".to_string()]);
}
