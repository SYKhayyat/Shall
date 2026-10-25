use super::*;

#[test]
fn no_vars_file_is_no_variables_and_not_an_error() {
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
    assert!(load_vars(&f).unwrap().is_empty());
}
