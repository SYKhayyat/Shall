use super::*;

#[test]
fn bin_turns_the_guess_off_rather_than_falling_back_to_it() {
    let e = entries(&[("fd", true)]);
    let err = find_executable(&e, "sharkdp/fd", Some("nope")).unwrap_err();
    assert!(matches!(err, DiscoveryError::BinNotFound { .. }));
}
