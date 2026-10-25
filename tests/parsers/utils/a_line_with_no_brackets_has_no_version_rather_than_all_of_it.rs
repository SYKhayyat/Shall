use super::*;

/// The whole reason the two callers extract instead of trimming: a trim on a line with no
/// brackets hands the line back, and the caller writes it down as a version.
#[test]
fn a_line_with_no_brackets_has_no_version_rather_than_all_of_it() {
    assert_eq!(extract_version_bracketed("Xcode 14.3.1"), None);
    assert_eq!(extract_version_bracketed(""), None);
    // An opening bracket alone is not a bracketed run.
    assert_eq!(extract_version_bracketed("pkg (14.3.1"), None);
}
