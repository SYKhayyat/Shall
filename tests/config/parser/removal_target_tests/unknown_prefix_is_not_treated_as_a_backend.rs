use super::*;

#[test]
fn unknown_prefix_is_not_treated_as_a_backend() {
    // A colon in a name whose prefix isn't a real backend stays part of the name.
    assert_eq!(
        split_removal_target("some:weird-name", known),
        (None, "some:weird-name".to_string())
    );
}
