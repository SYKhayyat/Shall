use super::*;

#[test]
fn two_assets_unpack_into_two_directories() {
    // Both archives can contain `bin/`, and one tree would lose one of them.
    assert_ne!(
        artifact_dir_name("fd-x86_64-musl.tar.gz"),
        artifact_dir_name("fd-x86_64-gnu.tar.gz")
    );
    assert!(!artifact_dir_name("../escape.tar.gz").contains('/'));
    assert!(!artifact_dir_name("..\\escape.tar.gz").contains('\\'));
}
