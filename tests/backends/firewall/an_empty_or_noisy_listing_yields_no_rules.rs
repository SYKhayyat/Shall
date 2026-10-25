use super::*;

#[test]
fn an_empty_or_noisy_listing_yields_no_rules() {
    assert!(shipped("ufw").parse_rules("Status: inactive\n").is_empty());
    assert!(shipped("ufw").parse_rules("").is_empty());
}
