use super::*;

#[test]
fn a_changed_hash_inside_a_set_is_still_an_objection() {
    let locked = [lock("a.tar.gz", Some("a1")), lock("b.tar.gz", Some("b2"))];
    let why = verify_set(
        &locked,
        &[
            ("a.tar.gz".into(), Some("a1".into())),
            ("b.tar.gz".into(), Some("ZZZ".into())),
        ],
    )
    .unwrap();
    assert!(why.contains("b.tar.gz"), "{}", why);
    assert!(why.contains("ZZZ"), "{}", why);
}
