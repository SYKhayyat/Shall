use super::*;

#[test]
fn an_unset_flag_is_not_a_set_one() {
    assert!(!allows_http(&spec(&[])));
    assert!(!is_unverified(&spec(&[("unverified", "false")])));
}
