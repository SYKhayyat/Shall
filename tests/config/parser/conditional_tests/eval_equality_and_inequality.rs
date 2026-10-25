use super::*;

#[test]
fn eval_equality_and_inequality() {
    let f = facts();
    assert!(eval_when("os == linux", &f).unwrap());
    assert!(!eval_when("os == macos", &f).unwrap());
    assert!(eval_when("os != windows", &f).unwrap());
    assert!(!eval_when("arch != x86_64", &f).unwrap());
    // case-insensitive value match
    assert!(eval_when("os == LINUX", &f).unwrap());
}
