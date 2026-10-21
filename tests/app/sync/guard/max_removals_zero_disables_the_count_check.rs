use super::*;

#[tokio::test]
async fn max_removals_zero_disables_the_count_check() {
    let reg = Arc::new(BackendRegistry::new());
    let many: Vec<String> = (0..500).map(|i| format!("pkg{}", i)).collect();
    let refs: Vec<&str> = many.iter().map(|s| s.as_str()).collect();
    assert!(inspect(&config_with(0), &reg, &pairs(&refs))
        .await
        .is_empty());
}
