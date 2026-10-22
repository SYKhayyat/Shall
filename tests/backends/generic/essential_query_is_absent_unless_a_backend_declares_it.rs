use super::*;

#[tokio::test]
async fn essential_query_is_absent_unless_a_backend_declares_it() {
    let vfs = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs.clone()));
    let q = queryable_with(ManualListing::AllInstalled, mock, vfs);
    // apt_like_core sets essential_args: None → no OS essential list, and no crash.
    assert!(q.essential().await.unwrap().is_empty());
}
