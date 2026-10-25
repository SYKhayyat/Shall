use super::*;

/// mise keeps a tool in `list --json` after `mise uninstall`, flagged `installed: false`,
/// because the declaration in its config survives. Reporting those as installed told Shall
/// a package was present when it was not.
#[tokio::test]
async fn a_tool_mise_says_is_not_installed_is_not_listed() {
    let (core, _) = mise_with(REAL_LIST_JSON);
    let listed = MiseQueryable { core }.list_installed().await.unwrap();
    let names: Vec<&str> = listed.iter().map(|p| p.name.as_str()).collect();
    assert_eq!(
        names,
        vec!["node"],
        "an uninstalled tool was reported as installed"
    );
}
