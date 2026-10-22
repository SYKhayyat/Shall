use super::*;

#[test]
fn a_backend_not_in_priority_says_so() {
    // V.15: not listed means Shall does not use it at all, and saying so catches typos.
    let err = parse(&o(), "flatpak:gimp", &known).unwrap_err();
    assert!(err.what.contains("flatpak"), "{}", err.what);
}
