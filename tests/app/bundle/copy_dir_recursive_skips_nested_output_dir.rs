use super::*;

#[tokio::test]
async fn copy_dir_recursive_skips_nested_output_dir() {
    // src contains a file AND the destination dir (out) nested inside it. Without the skip,
    // copying src -> out/groups would recurse into out forever. With it, only the real file
    // is copied and the run terminates.
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("cfg");
    std::fs::create_dir_all(&src).unwrap();
    std::fs::write(src.join("local.txt"), "apt:curl\n").unwrap();
    let out = src.join("bundle"); // output dir lives INSIDE src
    std::fs::create_dir_all(&out).unwrap();

    let n = copy_dir_recursive(&src, &out.join("groups"), Some(&out), Writes::ToDisk)
        .await
        .unwrap();
    assert_eq!(
        n, 1,
        "only local.txt should be copied, never the nested out dir"
    );
}
