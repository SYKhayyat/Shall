use super::*;

#[test]
fn a_rollback_targets_the_full_hash() {
    // The row shows the short hash, but a rollback must check out the full commit.
    let mut c = HistoryBrowser::new(vec![cv("a1b2c3d", "x", &[])]);
    c.next(); // stays on 0
    let sel = c.selected().unwrap();
    assert!(sel.full_hash.starts_with("a1b2c3d"));
    assert!(sel.full_hash.len() > 7);
}
