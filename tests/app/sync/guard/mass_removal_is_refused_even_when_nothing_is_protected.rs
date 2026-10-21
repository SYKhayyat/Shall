use super::*;

#[tokio::test]
async fn mass_removal_is_refused_even_when_nothing_is_protected() {
    let reg = Arc::new(BackendRegistry::new());
    let many: Vec<String> = (0..30).map(|i| format!("pkg{}", i)).collect();
    let refs: Vec<&str> = many.iter().map(|s| s.as_str()).collect();
    let report = inspect(&config_with(20), &reg, &pairs(&refs)).await;
    assert!(matches!(
        report.objections.as_slice(),
        [Objection::TooMany {
            count: 30,
            limit: 20,
            setting: "max_removals"
        }]
    ));
}
