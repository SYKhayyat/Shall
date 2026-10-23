use super::*;

/// A plugin that is ADDED and has nothing installed is not an installed package, and
/// `asdf list` prints its name either way. Captured from asdf v0.14.1 in the `tools`
/// image, 2026-07-29, on both sides of an `asdf uninstall jq`:
///
/// ```text
/// $ asdf list          $ asdf uninstall jq && asdf list
/// jq                   jq
///   1.8.2                No versions installed
/// ```
///
/// The sweep found it as `asdf: jq is gone from list (expected non-zero, got 0)`. Shall
/// removed the version correctly and went on reporting it as installed, which is permanent
/// phantom drift: `sync` would take it away and put it back forever.
#[test]
fn asdf_list_does_not_report_a_plugin_with_nothing_installed() {
    const INSTALLED: &str = include_str!("../../../../tests/fixtures/asdf/list-installed.txt");
    const EMPTY_PLUGIN: &str =
        include_str!("../../../../tests/fixtures/asdf/list-plugin-without-versions.txt");
    const NO_PLUGINS: &str = include_str!("../../../../tests/fixtures/asdf/list-no-plugins.txt");

    assert_eq!(
        asdf_list(INSTALLED, "asdf")
            .expect("this fixture parses")
            .iter()
            .map(|p| p.name.as_str())
            .collect::<Vec<_>>(),
        vec!["jq"],
        "the control: a plugin WITH a version is installed"
    );
    assert!(
        asdf_list(EMPTY_PLUGIN, "asdf")
            .expect("this fixture parses")
            .is_empty(),
        "a plugin with no versions was reported as an installed package"
    );
    assert!(
        asdf_list(NO_PLUGINS, "asdf")
            .expect("this fixture parses")
            .is_empty(),
        "asdf's own empty-state sentence was read as a package"
    );
}
