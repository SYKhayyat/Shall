use super::*;

/// VIII.2's point: a lock that recorded a hash must not pass a re-download that carries
/// none. Skipping the comparison is how a hashing failure would substitute bytes
/// silently.
#[test]
fn a_locked_hash_with_no_new_hash_is_an_objection() {
    let l = lock("fd.tar.gz", Some("abc123"));
    let why = verify_against(&l, "fd.tar.gz", None).unwrap();
    assert!(why.contains("no checksum"), "{}", why);
    assert!(why.contains("abc123"), "the locked hash is named: {}", why);
}
