use super::*;

/// Both bracket shapes, because the pattern accepts both and only one has a caller today.
#[test]
fn a_version_is_taken_from_either_bracket() {
    assert_eq!(
        extract_version_bracketed("Xcode (14.3.1)").as_deref(),
        Some("14.3.1")
    );
    assert_eq!(
        extract_version_bracketed("pkg [1.2.3]").as_deref(),
        Some("1.2.3")
    );
    assert_eq!(
        extract_version_bracketed("bundler (default: 4.0.10)").as_deref(),
        Some("default: 4.0.10")
    );
}
