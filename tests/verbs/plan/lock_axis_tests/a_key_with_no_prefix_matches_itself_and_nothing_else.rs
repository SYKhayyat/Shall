use super::*;

/// A bare name with no `:` at all — every entry in `locks/bare.HOST.toml` is one.
#[test]
fn a_key_with_no_prefix_matches_itself_and_nothing_else() {
    assert!(scoped_by("ripgrep", &names(&["ripgrep"])));
    assert!(!scoped_by("ripgrep", &names(&["rip"])));
}
