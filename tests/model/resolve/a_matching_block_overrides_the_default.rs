use super::*;

#[test]
fn a_matching_block_overrides_the_default() {
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
when os == linux {
  role = travel
}
",
    );
    assert_eq!(load_vars(&f).unwrap()["role"].to_string(), "travel");
}
