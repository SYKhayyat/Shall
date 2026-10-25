use super::*;

#[test]
fn a_set_that_lost_a_file_is_an_objection_naming_it() {
    let locked = [lock("a.tar.gz", None), lock("b.tar.gz", None)];
    let why = verify_set(&locked, &[("a.tar.gz".into(), None)]).unwrap();
    assert!(why.contains("b.tar.gz"), "{}", why);
}
