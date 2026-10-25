use super::*;

#[test]
fn two_contradicting_declarations_are_an_error() {
    // Two lines that disagree are an error, not a last-one-wins.
    let a = decl("modules/a.txt", 1, &[("version", "1.6")]);
    let b = decl("modules/b.txt", 3, &[("version", "1.7")]);
    let err = reconcile("apt:jq", a, b, now()).unwrap_err();
    assert!(err.what.contains("two different ways"), "{}", err);
}
