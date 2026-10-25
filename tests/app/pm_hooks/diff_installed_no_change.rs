use super::*;

#[test]
fn diff_installed_no_change() {
    let list = s(&["a", "b"]);
    let (added, removed) = diff_installed(&list, &list);
    assert!(added.is_empty() && removed.is_empty());
}
