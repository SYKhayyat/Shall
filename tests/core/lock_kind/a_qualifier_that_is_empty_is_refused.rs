use super::*;

/// A trailing colon names no sub-category, which is a typo rather than a set.
#[test]
fn a_qualifier_that_is_empty_is_refused() {
    let err = LockSelection::parse("versions:", &[]).expect_err("empty qualifier");
    assert!(err.to_string().contains("no sub-category"), "{err}");
}
