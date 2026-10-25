use super::*;

#[test]
fn only_what_this_machine_has_is_listed() {
    // "Detected, never configured" (V.41). Listing apt on a machine with no apt is
    // homework, and `priority` is not a wish list.
    let out = starter_order(&["cargo".into()]);
    assert_eq!(out, ["cargo"]);
}
