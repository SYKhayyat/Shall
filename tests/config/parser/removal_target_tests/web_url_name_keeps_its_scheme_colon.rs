use super::*;

#[test]
fn web_url_name_keeps_its_scheme_colon() {
    // web:https://x -> backend web, name "https://x" (only the first colon is the split).
    assert_eq!(
        split_removal_target("web:https://example.com/a.tar.gz", known),
        (
            Some("web".to_string()),
            "https://example.com/a.tar.gz".to_string()
        )
    );
}
