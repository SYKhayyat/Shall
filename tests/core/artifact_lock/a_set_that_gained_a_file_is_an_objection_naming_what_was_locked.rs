use super::*;

#[test]
fn a_set_that_gained_a_file_is_an_objection_naming_what_was_locked() {
    let locked = [lock("a.tar.gz", None)];
    let why = verify_set(
        &locked,
        &[("a.tar.gz".into(), None), ("b.tar.gz".into(), None)],
    )
    .unwrap();
    assert!(why.contains("a.tar.gz"), "{}", why);
    assert!(why.contains("b.tar.gz"), "{}", why);
}
