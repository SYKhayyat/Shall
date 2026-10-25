use super::*;

#[test]
fn a_bare_name_is_left_for_the_prober() {
    // V.16: the bare name is the question, the lock is the answer. Resolution reads
    // the file; it does not probe.
    let f = fx(
        "Work\n",
        &[("Work", "use base\n")],
        &[("base.txt", "ripgrep\n")],
    );
    let d = resolve(&f).unwrap();
    assert_eq!(names(&d, BARE), ["ripgrep"]);
}
