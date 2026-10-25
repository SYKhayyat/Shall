use super::*;

#[test]
fn active_ignores_a_repeat() {
    let out = parse_active(&PathBuf::from("active"), "Work\nWork\n", &facts()).unwrap();
    assert_eq!(out, ["Work"]);
}
