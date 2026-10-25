use super::*;

/// D14: the reason names which of the three rules won, in the order they win — pattern,
/// then the user's formats, then the detected default.
#[test]
fn the_selection_reason_names_the_winning_rule() {
    assert_eq!(selection_reason(true, true), "your @asset= pattern");
    assert_eq!(selection_reason(true, false), "your @asset= pattern");
    assert_eq!(selection_reason(false, true), "your @formats= line");
    assert_eq!(
        selection_reason(false, false),
        "the built-in default order for this machine"
    );
}
