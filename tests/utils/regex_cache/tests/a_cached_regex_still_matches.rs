use super::*;

#[test]
fn a_cached_regex_still_matches() {
    let re = compiled(r"^v(\d+)\.(\d+)$").unwrap();
    assert!(re.is_match("v1.2"));
    assert!(!re.is_match("1.2"));
}
