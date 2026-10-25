use super::*;

/// `info` asked `marketplace.visualstudio.com` and returned `Some` for anything that
/// *exists* there, at the marketplace's *latest* version. Three silent consequences:
/// `shall install vscode:x` reported success and installed nothing, a `@version=` pin
/// re-installed for ever the moment upstream published, and every plan made one
/// rate-limited HTTPS POST per declared extension.
///
/// The same bug's obituary is in `mise.rs`, written 2026-07-24. The assertion that the
/// catalogue is not consulted is the one that file already makes; this is its sibling.
#[tokio::test]
async fn info_answers_installed_here_not_published_to_the_marketplace() {
    let (core, mock) = vscode_with("ms-python.python@2024.2.1\nrust-lang.rust-analyzer@0.3.1\n");
    let q = VscodeQueryable { core };

    // An extension the marketplace certainly carries, and this machine does not.
    assert!(
        q.info("ms-vscode.cpptools").await.unwrap().is_none(),
        "info claimed an uninstalled extension was present — the planner skips its install"
    );
    // One that really is installed, at the version the machine has and not the latest one.
    let found = q
        .info("ms-python.python")
        .await
        .unwrap()
        .expect("installed");
    assert_eq!(found.version.as_deref(), Some("2024.2.1"));
    assert_eq!(
        found.properties.get("publisher").map(String::as_str),
        Some("ms-python")
    );

    // Extension ids are case-insensitive on the marketplace, so a manifest that spells the
    // publisher's name differently still has to resolve to the installed row.
    assert!(q.info("MS-Python.Python").await.unwrap().is_some());

    // The network is not the machine, and nothing here asked it.
    let calls = mock.get_calls().await;
    assert!(
        calls.iter().all(|c| c.starts_with("code ")),
        "info reached past the `code` CLI: {:?}",
        calls
    );
}
