use super::*;

#[test]
fn declaring_the_same_thing_twice_is_not_a_disagreement() {
    let a = decl("modules/a.txt", 1, &[("version", "1.6")]);
    let b = decl("modules/b.txt", 3, &[("version", "1.6")]);
    assert!(reconcile("apt:jq", a, b, now()).is_ok());
}
