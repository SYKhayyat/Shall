use super::*;

/// A machine with no DKMS modules has nothing to rebuild and nothing to report.
#[test]
fn no_modules_is_not_a_failure() {
    assert!(not_installed("").is_empty());
}
