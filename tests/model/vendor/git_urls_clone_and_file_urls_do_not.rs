use super::*;

/// A git remote must not be read as a single file to download — it clones.
#[test]
fn git_urls_clone_and_file_urls_do_not() {
    assert!(matches!(
        Source::classify("https://gitlab.com/x/y.git"),
        Some(Source::Git(_))
    ));
    assert!(matches!(
        Source::classify("git@github.com:x/y.git"),
        Some(Source::Git(_))
    ));
    assert!(matches!(
        Source::classify("https://example.com/mod.txt"),
        Some(Source::File(_))
    ));
}
