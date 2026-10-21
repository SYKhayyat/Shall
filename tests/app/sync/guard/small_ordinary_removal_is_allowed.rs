use super::*;

#[tokio::test]
async fn small_ordinary_removal_is_allowed() {
    let reg = Arc::new(BackendRegistry::new());
    let report = inspect(&config_with(20), &reg, &pairs(&["jq", "htop"])).await;
    assert!(report.is_empty(), "{:?}", report.objections);
}
