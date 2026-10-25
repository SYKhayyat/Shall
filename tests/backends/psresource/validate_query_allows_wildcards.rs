use super::*;

#[test]
fn validate_query_allows_wildcards() {
    assert!(validate_query("Az*").is_ok());
    assert!(validate_query("PS?eadLine").is_ok());
    assert!(validate_query("bad;name").is_err());
}
