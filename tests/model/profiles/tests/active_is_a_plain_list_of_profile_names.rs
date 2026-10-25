use super::*;

#[test]
fn active_is_a_plain_list_of_profile_names() {
    let out = parse_active(
        &PathBuf::from("active"),
        "# on now\nWork\nGaming\n",
        &facts(),
    )
    .unwrap();
    assert_eq!(out, ["Work", "Gaming"]);
}
