use super::*;

#[test]
fn a_dotted_version_is_not_mistaken_for_an_extension() {
    // Cutting at the first `.` named the installed binary `ripgrep-14`.
    assert_eq!(
        strip_archive_suffixes("ripgrep-14.1.0-x86_64.tar.gz"),
        "ripgrep-14.1.0-x86_64"
    );
    assert_eq!(
        strip_archive_suffixes("fd-v10.2.0-x86_64-unknown-linux-gnu.tar.gz"),
        "fd-v10.2.0-x86_64-unknown-linux-gnu"
    );
}
