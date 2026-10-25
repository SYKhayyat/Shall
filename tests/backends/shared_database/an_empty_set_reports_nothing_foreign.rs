use super::*;

/// A machine with no shared-database pair asks nothing and reports nothing foreign, which
/// is what keeps this off the path of every listing on every other distribution.
#[test]
fn an_empty_set_reports_nothing_foreign() {
    let none = ForeignSets::default();
    assert!(!none.is_foreign("pacman", "anything"));
    assert!(!ForeignSets::of("pacman", &["x"]).is_foreign("apt", "x"));
}
