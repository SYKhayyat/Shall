use super::*;

#[test]
fn present_and_absent_contradict() {
    let a = decl("modules/a.txt", 1, &[]);
    let b = absent("modules/bloat.txt", 2);
    assert!(reconcile("apt:libreoffice", a, b, now()).is_err());
}
