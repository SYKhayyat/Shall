use super::*;

/// A backend that reports nothing essential contributes nothing â€” the control that makes the
/// test above a measurement rather than a coincidence.
#[tokio::test]
async fn a_backend_with_no_essential_concept_contributes_nothing() {
    let registry = registry_reporting("cargo", &[]);
    let backends: HashSet<String> = ["cargo".to_string()].into_iter().collect();
    let answers = essential_names(&registry, &backends, 4, &mut EssentialsCache::default()).await;
    assert!(answers.names.is_empty());
    assert!(answers.unanswered.is_empty());
}
