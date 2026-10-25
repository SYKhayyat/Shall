use super::*;

/// The instrument, self-tested: this fake really does answer, so the zero above is the
/// planner not asking rather than the fake not knowing.
#[tokio::test]
async fn the_fake_manager_really_does_answer_a_dependency_query() {
    use shall::core::manager::MetadataProvider;
    let backend = DepAnswering::new(&[("nginx", &["libfoo", "libbar"])]);
    assert_eq!(
        backend.get_dependencies("nginx").await.unwrap(),
        vec!["libfoo".to_string(), "libbar".to_string()]
    );
    assert_eq!(backend.asked.load(std::sync::atomic::Ordering::SeqCst), 1);
}
