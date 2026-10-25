use super::*;

#[test]
fn a_bare_subtraction_takes_the_package_out_whatever_backend_holds_it() {
    let f = fx(
        "Work\n",
        &[("Work", "use editors\n-ripgrep\n")],
        &[("editors.txt", "cargo:ripgrep\napt:vim\n")],
    );
    let d = resolve(&f).unwrap();
    assert_eq!(names(&d, "cargo"), [] as [String; 0]);
    assert_eq!(names(&d, "apt"), ["vim"]);
}
