use super::*;

#[test]
fn the_bootstrap_pass_names_every_backend_and_evaluates_nothing() {
    // It runs before variables exist, so a predicate it cannot answer must not be an
    // error — and the superset it returns is a vocabulary for parsing `vars`, never an
    // order. `dnf` is inside a block that will not match; it is still named here.
    let p = Priority::every_backend(
        &PathBuf::from("priority"),
        "apt\nwhen $role == travel {\n  cargo\n}\nwhen os == plan9 {\n  dnf\n}\n",
    )
    .unwrap();
    assert_eq!(p.order(), ["apt", "cargo", "dnf"]);
}
