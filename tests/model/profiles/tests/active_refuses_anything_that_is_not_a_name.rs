use super::*;

#[test]
fn active_refuses_anything_that_is_not_a_name() {
    assert!(parse_active(&PathBuf::from("active"), "Work | Gaming\n", &facts()).is_err());
}
