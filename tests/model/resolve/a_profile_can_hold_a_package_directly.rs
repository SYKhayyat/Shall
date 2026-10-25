use super::*;

#[test]
fn a_profile_can_hold_a_package_directly() {
    let f = fx("Work\n", &[("Work", "apt:slack\n")], &[]);
    assert_eq!(names(&resolve(&f).unwrap(), "apt"), ["slack"]);
}
