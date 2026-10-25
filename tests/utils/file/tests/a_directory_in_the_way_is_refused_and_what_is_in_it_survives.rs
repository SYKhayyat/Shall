use super::*;

/// **The refusal, which is the half that a "just reuse `remove_by_kind`" fix would delete.**
/// `remove_by_kind` calls `remove_dir_all` on a real directory, so routing every existing path
/// through it would empty a directory that was merely *in the way* and then copy into the hole.
/// The copy is refused instead, and the refusal is checked by what survives rather than by the
/// error text — an assertion about the message passes just as happily when the tree is gone.
#[tokio::test]
async fn a_directory_in_the_way_is_refused_and_what_is_in_it_survives() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("src");
    std::fs::write(&src, b"the new contents").unwrap();
    let in_the_way = dir.path().join("in-the-way");
    std::fs::create_dir_all(&in_the_way).unwrap();
    std::fs::write(in_the_way.join("keep-me"), b"x").unwrap();

    copy_over(&src, &in_the_way)
        .await
        .expect_err("a directory in the way must be refused, not emptied");

    assert!(
        in_the_way.join("keep-me").exists(),
        "the refusal emptied the directory that was in the way: a mistake in a destination path \
     became data loss"
    );
}
