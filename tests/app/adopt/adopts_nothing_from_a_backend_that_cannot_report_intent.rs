use super::*;

#[tokio::test]
async fn adopts_nothing_from_a_backend_that_cannot_report_intent() {
    // The safety backstop. `dpkg-query -W` is wired and would happily return an entire
    // dependency graph; because the backend admits it cannot tell user-chosen packages
    // from dependencies, adoption must skip it entirely rather than adopt all of it.
    let vfs: Arc<DashMap<PathBuf, String>> = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs));
    mock.set_response_that_must_not_be_used(
        "dpkg-query -W",
        Ok(DryRunOutput {
            stdout: b"apt 2.7.14\nlibperl5.38t64 5.38.2\npython3 3.12.3\n".to_vec(),
            stderr: vec![],
        }
        .into()),
    );

    let found = adopter(registry_with(ManualListing::Unsupported, mock.clone()))
        .audit()
        .await
        .unwrap();
    assert!(
        found.is_empty(),
        "expected no adoption candidates, got {:?}",
        found.iter().map(|p| &p.name).collect::<Vec<_>>()
    );
}
