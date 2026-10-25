use super::*;

#[test]
fn a_signature_git_will_not_vouch_for_is_not_a_good_one() {
    // 'U' is a good signature by an untrusted key, 'X' an expired one. Both are signed,
    // neither is verified — collapsing them into `Good` is how a signature becomes
    // decoration (V.32).
    for code in ["U", "X", "Y", "R"] {
        let sig = Signature::from_git(code, "someone");
        assert!(!sig.is_verified(), "{} must not verify", code);
        assert!(matches!(sig, Signature::Unverified { .. }));
    }
    assert_eq!(Signature::from_git("B", "someone"), Signature::Bad);
    assert_eq!(Signature::from_git("E", ""), Signature::Bad);
    assert!(Signature::from_git("G", "someone").is_verified());
}
