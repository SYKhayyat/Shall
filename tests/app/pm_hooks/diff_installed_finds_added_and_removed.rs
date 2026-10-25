use super::*;

#[test]
fn diff_installed_finds_added_and_removed() {
    let before = s(&["curl", "nano", "htop"]);
    let after = s(&["curl", "htop", "ripgrep"]);
    let (added, removed) = diff_installed(&before, &after);
    assert_eq!(added, vec!["ripgrep"]);
    assert_eq!(removed, vec!["nano"]);
}
