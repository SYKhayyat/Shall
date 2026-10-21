use super::*;

/// `essential_names` returns what the backend said, qualified by backend.
///
/// Kills all three surviving mutants directly: `HashSet::new()` fails the emptiness check,
/// `[String::new()]` and `["xyzzy"]` fail the equality.
#[tokio::test]
async fn essential_names_reports_what_the_backend_says_is_essential() {
    let registry = registry_reporting("apt", &["tar", "sed", "grep"]);
    let backends: HashSet<String> = ["apt".to_string()].into_iter().collect();

    let answers = essential_names(&registry, &backends, 4, &mut EssentialsCache::default()).await;

    assert!(
        answers.unanswered.is_empty(),
        "a healthy backend is not unanswered: {:?}",
        answers.unanswered
    );
    let want: HashSet<String> = ["apt:tar", "apt:sed", "apt:grep"]
        .into_iter()
        .map(String::from)
        .collect();
    assert_eq!(
        answers.names, want,
        "the essential set must be exactly what the backend reported, qualified by backend"
    );
}
