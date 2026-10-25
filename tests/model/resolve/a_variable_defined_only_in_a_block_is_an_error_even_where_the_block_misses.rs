use super::*;

#[test]
fn a_variable_defined_only_in_a_block_is_an_error_even_where_the_block_misses() {
    // IX.3 is a property of the FILE. If this were checked only against the blocks that
    // matched, the same repo would be valid on the laptop and broken on the desktop —
    // and the error would appear on whichever machine happened not to define it.
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
        "when os == plan9 {
  role = travel
}
",
    );
    let err = load_vars(&f).unwrap_err();
    assert!(
        err.what.contains("only defined inside a `when` block"),
        "{}",
        err
    );
}
