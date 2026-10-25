use super::*;

#[tokio::test]
async fn the_manifest_warns_that_it_is_an_estimate_and_names_its_source() {
    // The manifest asks the user to trust a guess. These three facts are the whole
    // reason it is safe to do that, so they are pinned rather than left to drift.
    let vfs: Arc<DashMap<PathBuf, String>> = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs));
    mock.set_response(
        "apt-mark showmanual",
        Ok(DryRunOutput {
            stdout: b"jq\npython3\n".to_vec(),
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
    // No `protected_packages` here: protection has nothing to say about the manifest
    // (E7/II.9). What is commented out is what the OS calls essential.
    let m = Adopter::new(
        over(reg),
        Arc::new(Mutex::new(StateRegistry::default())),
        &Config::default(),
    );
    let text = m.render_manifest(&m.discover().await.unwrap(), &m.adoption_options());

    assert!(text.contains("THIS IS AN ESTIMATE"), "{}", text);
    assert!(text.contains("UNINSTALLED"), "{}", text);
    assert!(text.contains("shall unmanage"), "{}", text);
    // The source of the estimate, so a reader can reproduce it.
    assert!(text.contains("apt-mark showmanual"), "{}", text);
    // Every manual package is a live line — a PROTECTED one too, if it were listed:
    // protection stops the removal, not the adoption (V.26). Here both are just manual.
    assert!(text.contains("\napt:jq\n"), "{}", text);
    assert!(text.contains("\napt:python3\n"), "{}", text);
}
