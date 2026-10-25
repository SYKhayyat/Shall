use super::*;

#[test]
fn rewrite_backend_prefix_changes_only_the_prefix() {
    // bare name gains a prefix; options and comment are kept
    assert_eq!(rewrite_backend_prefix("ripgrep", "apt"), "apt:ripgrep");
    assert_eq!(
        rewrite_backend_prefix("  cargo:ripgrep@version=14  # fast", "apt"),
        "  apt:ripgrep@version=14 # fast"
    );
    assert_eq!(
        rewrite_backend_prefix("ripgrep@version=14", "apt"),
        "apt:ripgrep@version=14"
    );
}
