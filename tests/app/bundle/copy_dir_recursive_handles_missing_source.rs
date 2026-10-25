use super::*;

#[tokio::test]
async fn copy_dir_recursive_handles_missing_source() {
    let n = copy_dir_recursive(
        Path::new("/nonexistent/xyz"),
        Path::new("/tmp/whatever"),
        None,
        Writes::ToDisk,
    )
    .await
    .unwrap();
    assert_eq!(n, 0);
}
