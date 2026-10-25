use super::*;

#[test]
fn nix_search_empty_is_ok() {
    assert!(parse_nix_search("{}").unwrap().is_empty());
    assert!(parse_nix_search("").unwrap().is_empty());
}
