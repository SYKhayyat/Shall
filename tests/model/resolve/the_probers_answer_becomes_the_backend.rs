use super::*;

#[test]
fn the_probers_answer_becomes_the_backend() {
    let f = fx(
        "Work\n",
        &[("Work", "use base\n")],
        &[("base.txt", "ripgrep\n")],
    );
    let d = resolve_probed(&f, &[("ripgrep", "cargo")]).unwrap();
    assert_eq!(names(&d, "cargo"), ["ripgrep"]);
    assert_eq!(names(&d, BARE), [] as [String; 0]);
}
