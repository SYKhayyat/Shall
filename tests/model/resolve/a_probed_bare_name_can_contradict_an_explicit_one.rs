use super::*;

#[test]
fn a_probed_bare_name_can_contradict_an_explicit_one() {
    // And having met, they obey rule 5 like any other pair.
    let f = fx(
        "Work\n",
        &[("Work", "use a\nuse b\n")],
        &[
            ("a.txt", "ripgrep@version=14.1.0\n"),
            ("b.txt", "cargo:ripgrep@version=13.0.0\n"),
        ],
    );
    let err = resolve_probed(&f, &[("ripgrep", "cargo")]).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("a.txt"), "{}", msg);
    assert!(msg.contains("b.txt"), "{}", msg);
}
