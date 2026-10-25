use super::*;

#[test]
fn a_provider_makes_the_same_checks_fine() {
    assert_eq!(refusal_if_unrevertable(&one_check(), true, false), None);
}
