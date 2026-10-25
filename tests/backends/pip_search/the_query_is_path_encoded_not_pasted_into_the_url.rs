use super::*;

/// Path syntax in user text must not become a path: a `/` or space is escaped, ordinary
/// names pass through untouched.
#[test]
fn the_query_is_path_encoded_not_pasted_into_the_url() {
    assert_eq!(percent_encode_segment("scikit-learn"), "scikit-learn");
    assert_eq!(percent_encode_segment("a/b c?d"), "a%2Fb%20c%3Fd");
}
