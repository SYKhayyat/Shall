use super::*;

#[test]
fn deleting_the_entry_is_how_you_unfreeze() {
    // II.15's rule: the file is the switch. Nothing here does the unfreezing — the user's
    // editor does — so what this asserts is that a lock with no entry has no opinion.
    let lock = BareLock::new();
    assert_eq!(lock.get("ripgrep"), None);
}
