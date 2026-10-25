use super::*;

/// The other half of the same rule: when it IS there, `absent:` still removes it. A fix
/// that made `absent:` a no-op would pass the test above and destroy the feature.
#[tokio::test]
async fn an_absent_declaration_for_something_installed_is_still_a_removal() {
    let registry = registry_reporting("generic-test", &["libreoffice"]);
    assert_eq!(absent_removals(registry, "libreoffice").await, 1);
}
