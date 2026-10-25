use super::*;

#[tokio::test]
async fn audit_and_adopt_cannot_disagree() {
    // They were two loops that had already drifted apart on two points. `audit` is now
    // literally `discover().adopt`, so this test would have to be deleted to break it.
    let vfs: Arc<DashMap<PathBuf, String>> = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs));
    mock.set_response(
        "apt-mark showmanual",
        Ok(DryRunOutput {
            stdout: b"jq\nhtop\n".to_vec(),
            stderr: vec![],
        }
        .into()),
    );
    let reg = registry_with(
        ManualListing::Command {
            binary: Some("apt-mark".into()),
            args: vec!["showmanual".into()],
            format: ManualFormat::BareNames,
        },
        mock,
    );
    let m = adopter(reg);
    let audited: Vec<String> = m
        .audit()
        .await
        .unwrap()
        .into_iter()
        .map(|p| p.name)
        .collect();
    let discovered: Vec<String> = m
        .discover()
        .await
        .unwrap()
        .adopt
        .into_iter()
        .map(|p| p.name)
        .collect();
    assert_eq!(audited, discovered);
    // Sorted, because the manifest is a file people diff.
    assert_eq!(audited, vec!["htop", "jq"]);
}
