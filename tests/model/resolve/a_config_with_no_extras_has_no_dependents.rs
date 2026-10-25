use super::*;

#[test]
fn a_config_with_no_extras_has_no_dependents() {
    let f = fx("Work\n", &[("Work", "apt:curl\n")], &[]);
    assert!(!resolve(&f).unwrap().has_dependents());
}
