use super::*;

#[test]
fn empty_history_has_no_selection() {
    let c = HistoryBrowser::new(vec![]);
    assert!(c.selected().is_none());
}
