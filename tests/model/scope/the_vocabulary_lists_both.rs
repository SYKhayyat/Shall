use super::*;

#[test]
fn the_vocabulary_lists_both() {
    let v = Scope::vocabulary();
    assert!(v.contains("user") && v.contains("system"), "{}", v);
}
