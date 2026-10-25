use super::*;

#[test]
fn a_directory_named_like_the_artifact_is_never_deleted() {
    let dir = tempdir().unwrap();
    std::fs::create_dir(dir.path().join("payload.bin")).unwrap();
    let hits = find_cached(
        "payload.bin",
        std::slice::from_ref(&dir.path().to_path_buf()),
    );
    assert!(
        hits.is_empty(),
        "a directory must never be a cache-clean target"
    );
}
