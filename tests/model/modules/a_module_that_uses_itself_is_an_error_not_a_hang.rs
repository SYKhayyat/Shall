use super::*;

#[test]
fn a_module_that_uses_itself_is_an_error_not_a_hang() {
    // II.7: the error names every file and line in the loop, in order, and stops.
    let f = fixture(&[("a.txt", "use b\n"), ("b.txt", "use a\n")]);
    let err = expand_module(&f, "a").unwrap_err();
    assert!(
        err.what.contains("modules use each other in a loop"),
        "{}",
        err
    );
    assert!(err.what.contains("a.txt:1  use b"), "{}", err);
    assert!(err.what.contains("b.txt:1  use a"), "{}", err);
    assert!(err.what.trim_end().ends_with("^ back to a"), "{}", err);
}
