use super::*;

#[test]
fn a_changed_hash_on_the_same_asset_is_an_objection() {
    let l = lock("fd.tar.gz", Some("abc123"));
    let why = verify_against(&l, "fd.tar.gz", Some("def456")).unwrap();
    assert!(why.contains("abc123"), "{}", why);
    assert!(why.contains("def456"), "{}", why);
}
