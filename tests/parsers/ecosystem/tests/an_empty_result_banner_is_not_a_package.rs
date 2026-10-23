use super::*;

#[test]
fn an_empty_result_banner_is_not_a_package() {
    // `pixi global list` on a machine with nothing installed prints a sentence, and the
    // parser used to take its first token -- reporting a phantom `pixi:No` that `adopt`
    // would write to a manifest and `purge-undeclared` would try to delete.
    assert!(pixi_list(
        "No global environments found.
",
        "pixi"
    )
    .expect("this fixture parses")
    .is_empty());
    assert!(names_only(
        "No packages found.
",
        "spack"
    )
    .expect("this fixture parses")
    .is_empty());

    // A real listing that merely starts with a package beginning "no" still parses.
    let pkgs = names_only(
        "nodejs
nom
",
        "spack",
    )
    .expect("this fixture parses");
    assert_eq!(pkgs.len(), 2);
    assert_eq!(pkgs[0].name, "nodejs");
}
