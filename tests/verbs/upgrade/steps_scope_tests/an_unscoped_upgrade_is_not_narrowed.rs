use super::*;

/// And a bare `shall upgrade` is the whole machine, which is what the steps belong to.
/// The control: without it, `narrowed()` returning true unconditionally passes the test
/// above and silently removes the feature.
#[test]
fn an_unscoped_upgrade_is_not_narrowed() {
    assert!(!req(&[], None, false, None).narrowed());
}
