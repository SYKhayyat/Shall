use super::*;

#[test]
fn eval_membership() {
    let f = facts();
    assert!(eval_when("os in [linux, macos]", &f).unwrap());
    assert!(!eval_when("os in [windows, macos]", &f).unwrap());
    assert!(eval_when("host in [laptop, desktop]", &f).unwrap());
}
