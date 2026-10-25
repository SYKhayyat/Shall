use super::*;

#[test]
fn an_explicit_subtraction_leaves_the_other_backends_line_alone() {
    // You said which one, so only that one goes.
    let f = fx(
        "Work\n",
        &[("Work", "use editors\n-apt:ripgrep\n")],
        &[("editors.txt", "cargo:ripgrep\napt:ripgrep\n")],
    );
    let d = resolve(&f).unwrap();
    assert_eq!(names(&d, "cargo"), ["ripgrep"]);
    assert_eq!(names(&d, "apt"), [] as [String; 0]);
}
