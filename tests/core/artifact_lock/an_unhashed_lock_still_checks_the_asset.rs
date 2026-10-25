use super::*;

#[test]
fn an_unhashed_lock_still_checks_the_asset() {
    // Older entries and download-only artifacts may have no hash; the asset name is
    // still an identity worth holding to.
    let l = lock("fd.tar.gz", None);
    assert!(verify_against(&l, "fd.tar.gz", Some("abc123")).is_none());
    assert!(verify_against(&l, "other.tar.gz", None).is_some());
}
