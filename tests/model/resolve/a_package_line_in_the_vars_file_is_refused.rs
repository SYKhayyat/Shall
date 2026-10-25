use super::*;

#[test]
fn a_package_line_in_the_vars_file_is_refused() {
    let f = fx(
        "Work
",
        &[(
            "Work",
            "apt:curl
",
        )],
        &[],
    );
    with_vars(
        &f,
        "role = desktop
apt:nginx
",
    );
    let err = load_vars(&f).unwrap_err();
    assert!(err.what.contains("NAME = VALUE"), "{}", err);
}
