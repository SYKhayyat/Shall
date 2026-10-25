use super::*;

#[test]
fn a_different_asset_is_an_objection_that_says_how_to_accept_it() {
    let l = lock("fd-gnu.tar.gz", None);
    let why = verify_against(&l, "fd-musl.tar.gz", None).unwrap();
    assert!(why.contains("fd-gnu.tar.gz"), "{}", why);
    assert!(why.contains("fd-musl.tar.gz"), "{}", why);
    assert!(why.contains("shall lock"), "{}", why);
}
