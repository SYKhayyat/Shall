use super::*;

/// An empty mise reports `{}`, which must be no packages rather than a parse error.
#[tokio::test]
async fn an_empty_mise_lists_nothing() {
    let (core, _) = mise_with("{}");
    assert!(MiseQueryable { core }
        .list_installed()
        .await
        .unwrap()
        .is_empty());
}
