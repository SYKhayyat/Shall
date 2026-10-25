use super::*;

#[test]
fn a_block_that_does_not_match_leaves_the_default() {
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
when os == plan9 {
  role = travel
}
",
    );
    assert_eq!(load_vars(&f).unwrap()["role"].to_string(), "desktop");
}
