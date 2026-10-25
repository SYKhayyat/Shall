use super::*;

#[test]
fn an_exact_filename_at_depth_is_found_and_a_near_miss_is_not() {
    let dir = tempdir().unwrap();
    let nested = dir.path().join("tool").join("v1");
    std::fs::create_dir_all(&nested).unwrap();
    std::fs::write(nested.join("fd-v10.2.0.tar.gz"), b"x").unwrap();
    // A near miss (prefix, different suffix) must not match — exact name only.
    std::fs::write(nested.join("fd-v10.2.0.tar.gz.part"), b"x").unwrap();

    let hits = find_cached(
        "fd-v10.2.0.tar.gz",
        std::slice::from_ref(&dir.path().to_path_buf()),
    );
    assert_eq!(hits.len(), 1);
    assert!(hits[0].ends_with("fd-v10.2.0.tar.gz"));
}
