use super::*;

#[test]
fn extras_are_carried_in_declaration_order() {
    let f = fx(
        "Work\n",
        &[("Work", "use py\n")],
        &[("py.txt", "repo:apt:ppa:deadsnakes/ppa\napt:python3.12\n")],
    );
    let d = resolve(&f).unwrap();
    assert!(matches!(d.extras[0].0, Statement::Repo { .. }));
    assert_eq!(names(&d, "apt"), ["python3.12"]);
}
