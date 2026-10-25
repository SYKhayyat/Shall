use super::*;

#[tokio::test]
async fn adopts_only_the_packages_the_backend_reports_as_user_chosen() {
    // The real fix: ask `apt-mark showmanual`, not the installed listing. The
    // dependency present in dpkg-query's output must not be adopted.
    let vfs: Arc<DashMap<PathBuf, String>> = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs));
    mock.set_response(
        "apt-mark showmanual",
        Ok(DryRunOutput {
            stdout: b"apt\njq\n".to_vec(),
            stderr: vec![],
        }
        .into()),
    );
    mock.set_response_that_must_not_be_used(
        "dpkg-query -W",
        Ok(DryRunOutput {
            stdout: b"apt 2.7.14\njq 1.7.1\nlibperl5.38t64 5.38.2\n".to_vec(),
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
        mock.clone(),
    );
    let names: Vec<String> = adopter(reg)
        .audit()
        .await
        .unwrap()
        .into_iter()
        .map(|p| p.name)
        .collect();

    assert_eq!(names, vec!["apt", "jq"]);
    assert!(
        !names.contains(&"libperl5.38t64".to_string()),
        "a pure dependency was adopted — this is the bug that purged the container"
    );
}
