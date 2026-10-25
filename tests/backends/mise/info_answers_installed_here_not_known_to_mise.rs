use super::*;

/// The fail-silent bug the `tools` image caught: `info` asked mise's plugin CATALOGUE, so
/// it answered "yes" for anything mise had heard of. The planner read that as "already
/// installed" and `shall install mise:jq` reported *already up to date* while installing
/// nothing at all.
#[tokio::test]
async fn info_answers_installed_here_not_known_to_mise() {
    let (core, mock) = mise_with(REAL_LIST_JSON);
    let q = MiseQueryable { core };

    // `jq` is a tool mise knows and could install, but it is NOT installed.
    assert!(
        q.info("jq").await.unwrap().is_none(),
        "info claimed an uninstalled tool was present — the planner would skip its install"
    );
    // A tool mise has never heard of is likewise absent.
    assert!(q.info("nosuchtool").await.unwrap().is_none());
    // And one that really is installed is found, with its version.
    let found = q.info("node").await.unwrap().expect("node is installed");
    assert_eq!(found.version.as_deref(), Some("22.1.0"));

    // The catalogue is never consulted: it answers "could this be installed?", which is a
    // different question and the one that caused the bug.
    let calls = mock.get_calls().await;
    assert!(
        !calls.iter().any(|c| c.contains("plugins")),
        "info consulted the plugin catalogue: {:?}",
        calls
    );
}
