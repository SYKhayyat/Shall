use super::*;

/// **The bug:** `absent:` scheduled a removal whether or not the package was there, so a
/// machine that had never had it failed every single sync — the package manager refusing
/// to remove something it does not have, forever, with no way to converge.
#[tokio::test]
async fn an_absent_declaration_for_something_not_installed_is_not_a_removal() {
    let registry = registry_reporting("generic-test", &["something-else"]);
    assert_eq!(
        absent_removals(registry, "libreoffice").await,
        0,
        "removing what is not there is not a change, it is a command that always fails"
    );
}
