use super::*;

#[test]
fn hashing_is_stable_and_sensitive() {
    // Same bytes → same hash; one changed byte → different hash. The whole guard rests
    // on both halves of this.
    let a = hash_script("echo hello");
    assert_eq!(a, hash_script("echo hello"));
    assert_ne!(a, hash_script("echo hello "));
    // sha256 hex is 64 chars.
    assert_eq!(a.len(), 64);
}
