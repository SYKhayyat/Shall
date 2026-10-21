use super::*;

#[tokio::test]
async fn protected_package_is_refused_even_when_alone() {
    // The count limit cannot catch this one: it is a single removal.
    let reg = Arc::new(BackendRegistry::new());
    let report = inspect(&config_with(20), &reg, &pairs(&["python3"])).await;
    assert!(matches!(
        report.objections.as_slice(),
        [Objection::Protected { .. }]
    ));
}
