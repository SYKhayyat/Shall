use super::*;

#[test]
fn active_holds_names_never_expressions() {
    // II.6: the set math lives inside profiles. `active` stays a list you can read at
    // a glance, because it is the one file you open to know what is on.
    assert!(read("(Work | Gaming)\n", "any").is_err());
}
