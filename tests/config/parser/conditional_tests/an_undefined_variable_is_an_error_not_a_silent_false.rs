use super::*;

#[test]
fn an_undefined_variable_is_an_error_not_a_silent_false() {
    // A typo'd `$rle` that read as false would be a block that never fires and never
    // complains, which is the failure IX.3 exists to delete.
    let f = with_role("travel");
    assert!(eval_when("$rle == travel", &f).is_err());
}
