use super::*;

#[test]
fn the_same_asset_and_hash_is_no_objection() {
    let l = lock("fd.tar.gz", Some("abc123"));
    assert!(verify_against(&l, "fd.tar.gz", Some("ABC123")).is_none());
}
