use super::*;

#[tokio::test]
async fn an_os_essential_package_is_adopted_like_any_other() {
    // II.9, ruled 2026-08-05: adoption is a claim that Shall keeps the thing, and the
    // guard is what refuses to remove it. Commenting these lines out left Shall with no
    // opinion at all about the packages that matter most — nothing declared `bash`, so
    // nothing would put it back.
    let vfs: Arc<DashMap<PathBuf, String>> = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs));
    mock.set_response(
        "apt-mark showmanual",
        Ok(DryRunOutput {
            stdout: b"jq\nbash\n".to_vec(),
            stderr: vec![],
        }
        .into()),
    );
    // The OS's own answer, in dpkg's real format: bash is Essential, jq is not.
    // `Q47` ruled that adoption does not consult the guard, so this stub going unused *is*
    // the proof of that ruling — and nothing checked it until it was registered as one that
    // must not be used. Its content is dpkg's real format: bash is Essential, jq is not.
    mock.set_response_that_must_not_be_used(
        "dpkg-query -W -f=${Essential} ${Priority} ${Package}\\n",
        Ok(DryRunOutput {
            stdout: b"yes required bash\nno optional jq\n".to_vec(),
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
    let m = Adopter::new(
        over(reg),
        Arc::new(Mutex::new(StateRegistry::default())),
        &Config::default(),
    );
    let found = m.discover().await.unwrap();

    let names: Vec<&str> = found.adopt.iter().map(|p| p.name.as_str()).collect();
    assert_eq!(
        names,
        vec!["bash", "jq"],
        "the OS calling `bash` essential is a reason to declare it, not to skip it"
    );
    assert!(
        found.skipped.is_empty(),
        "nothing is left alone for being essential: {:?}",
        found.skipped
    );
}
