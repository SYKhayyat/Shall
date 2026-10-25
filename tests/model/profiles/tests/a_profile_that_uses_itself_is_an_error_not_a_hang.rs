use super::*;

#[test]
fn a_profile_that_uses_itself_is_an_error_not_a_hang() {
    // II.7: the error names every file and line in the loop, in order, and stops.
    let f = fixture(&[("A", "use B\n"), ("B", "use A\n")], &[]);
    let err = resolve(&f, "A").unwrap_err();
    assert!(
        err.what.contains("profiles reference each other in a loop"),
        "{}",
        err
    );
    assert!(err.what.contains("A:1  use B"), "{}", err);
    assert!(err.what.contains("B:1  use A"), "{}", err);
    assert!(err.what.trim_end().ends_with("^ back to A"), "{}", err);
}
