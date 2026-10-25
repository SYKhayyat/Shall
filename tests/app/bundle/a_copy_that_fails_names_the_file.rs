use super::*;

/// An I/O failure that names no file is a failure nobody can act on. This is the error
/// that cost a CI round: `Permission denied (os error 13)`, on one of several hundred
/// copied paths, with nothing to say which.
#[tokio::test]
async fn a_copy_that_fails_names_the_file() {
    let tmp = std::env::temp_dir().join(format!("shall-copy-names-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();
    let missing = tmp.join("no-such-source");
    let err = copy_over(&missing, &tmp.join("dest"))
        .await
        .expect_err("copying a file that is not there must fail");
    let msg = err.to_string();
    assert!(
        msg.contains("no-such-source"),
        "the error must name the path it could not copy, got: {msg}"
    );
    let _ = std::fs::remove_dir_all(&tmp);
}
