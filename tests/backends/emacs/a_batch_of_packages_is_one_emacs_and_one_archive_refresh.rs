use super::*;

/// `Q46`. Each install spawned a whole Emacs **and** a `package-refresh-contents`, which is
/// a network fetch of the archive. Ten packages meant ten startups and ten refreshes of an
/// archive that had not changed. One `--batch` with a `dolist` does all of it; verified
/// against GNU Emacs 29.3 in a container before this was written.
#[tokio::test]
async fn a_batch_of_packages_is_one_emacs_and_one_archive_refresh() {
    let (core, mock) = wired();
    let specs = vec![
        shall::core::PackageSpec {
            name: "csv-mode".into(),
            backend: "emacs".into(),
            ..Default::default()
        },
        shall::core::PackageSpec {
            name: "rainbow-mode".into(),
            backend: "emacs".into(),
            ..Default::default()
        },
    ];
    EmacsInstallable { core: core.clone() }
        .install(&specs, false)
        .await
        .unwrap();

    let calls = mock.get_calls().await;
    assert_eq!(calls.len(), 1, "one Emacs for the batch, got {:?}", calls);
    assert_eq!(
        calls[0].matches("package-refresh-contents").count(),
        1,
        "the archive is fetched once, not once per package: {:?}",
        calls
    );
    assert!(
        calls[0].contains("csv-mode") && calls[0].contains("rainbow-mode"),
        "{:?}",
        calls
    );
}
