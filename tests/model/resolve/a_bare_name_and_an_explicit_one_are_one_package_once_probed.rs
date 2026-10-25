use super::*;

#[test]
fn a_bare_name_and_an_explicit_one_are_one_package_once_probed() {
    // Probing has to happen BEFORE the merge. Keyed on `?`, `ripgrep` and `cargo:ripgrep`
    // never meet, and the run installs the same package twice — quietly, which is the
    // one thing this model exists to stop.
    let f = fx(
        "Work\n",
        &[("Work", "use a\nuse b\n")],
        &[("a.txt", "ripgrep\n"), ("b.txt", "cargo:ripgrep\n")],
    );
    let d = resolve_probed(&f, &[("ripgrep", "cargo")]).unwrap();
    assert_eq!(names(&d, "cargo"), ["ripgrep"], "one package, not two");
}
