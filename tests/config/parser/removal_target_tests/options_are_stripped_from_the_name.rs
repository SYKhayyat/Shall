use super::*;

#[test]
fn options_are_stripped_from_the_name() {
    assert_eq!(
        split_removal_target("npm:typescript@version=5", known),
        (Some("npm".to_string()), "typescript".to_string())
    );
}
