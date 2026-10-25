use super::*;

#[test]
fn a_variable_block_in_active_is_read_against_this_runs_variables() {
    // W8. The editing verbs used to read `active` against detected facts only, so this
    // file was not "a block that does not apply" — it was an unknown key, and every one
    // of them refused a correct file.
    let body = "when $role == travel {\n  Trip\n}\nWork\n";
    let on = parse_active(
        &PathBuf::from("active"),
        body,
        &facts_with(&[("role", "travel")]),
    )
    .unwrap();
    assert_eq!(on, ["Trip", "Work"]);

    let off = parse_active(
        &PathBuf::from("active"),
        body,
        &facts_with(&[("role", "desktop")]),
    )
    .unwrap();
    assert_eq!(off, ["Work"]);
}
