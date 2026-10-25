use super::*;

#[test]
fn conda_search_error_payload_is_empty() {
    let input = r#"{"error": "PackagesNotFoundError", "exception_name": "PackagesNotFoundError"}"#;
    assert!(parse_conda_search(input).is_empty());
}
