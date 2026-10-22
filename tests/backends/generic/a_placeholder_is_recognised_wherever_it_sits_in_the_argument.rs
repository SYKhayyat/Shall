use super::*;

#[test]
fn a_placeholder_is_recognised_wherever_it_sits_in_the_argument() {
    assert_eq!(find_placeholder("{url}").as_deref(), Some("{url}"));
    assert_eq!(
        find_placeholder("sed -i '\\|{url}|d' /etc/apk/repositories").as_deref(),
        Some("{url}")
    );
    assert_eq!(find_placeholder("--name").as_deref(), None);
    // A brace that is not a placeholder must not become a refusal: shell brace expansion
    // and printf formats both use them.
    assert_eq!(find_placeholder("printf '%s\\n'").as_deref(), None);
    assert_eq!(find_placeholder("{}").as_deref(), None);
    assert_eq!(find_placeholder("a{b").as_deref(), None);
    // **Case-typo'd placeholders ARE caught now**: `{Url}` was a real defect that slipped
    // past the old lowercase-only filter and reached the manager literally.
    assert_eq!(find_placeholder("{NAME}").as_deref(), Some("{NAME}"));
    assert_eq!(find_placeholder("{Url}").as_deref(), Some("{Url}"));
}
