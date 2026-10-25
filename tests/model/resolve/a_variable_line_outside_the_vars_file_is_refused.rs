use super::*;

#[test]
fn a_variable_line_outside_the_vars_file_is_refused() {
    let f = fx(
        "Work
",
        &[(
            "Work", "use m
",
        )],
        &[(
            "m.txt",
            "role = travel
apt:curl
",
        )],
    );
    let err = resolve(&f).unwrap_err();
    assert!(err.what.contains("is not in the `vars` file"), "{}", err);
}
