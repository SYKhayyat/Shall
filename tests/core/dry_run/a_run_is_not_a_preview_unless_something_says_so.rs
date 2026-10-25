use super::*;

/// Runs alone. The flag is a process-global and production has one writer (`main` sets it
/// once, before anything else runs); the only parallel flipper was this test itself, whose
/// `set(true)` window could land inside another test's write. Nothing else calls `set`, so
/// nothing needs a cross-test lock — this test just stopped being its own race.
#[test]
fn a_run_is_not_a_preview_unless_something_says_so() {
    // The default matters more than it looks: a library caller that never sets the flag
    // must write for real, or `cargo test` and every embedding of this crate would
    // silently perform nothing.
    assert!(!active());
    set(true);
    assert!(active());
    set(false);
    assert!(!active());
}
