use super::*;

#[test]
fn eval_rejects_unknown_key_and_bad_syntax() {
    let f = facts();
    assert!(eval_when("kernel == 6.1", &f).is_err());
    assert!(eval_when("os linux", &f).is_err());
}
