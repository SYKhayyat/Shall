use super::*;

#[test]
fn bare_name_has_no_backend() {
    assert_eq!(
        split_removal_target("ripgrep", known),
        (None, "ripgrep".to_string())
    );
}
