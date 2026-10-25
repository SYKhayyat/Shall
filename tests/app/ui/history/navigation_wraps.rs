use super::*;

#[test]
fn navigation_wraps() {
    let mut c = HistoryBrowser::new(vec![
        cv("c3", "", &[]),
        cv("c2", "", &[]),
        cv("c1", "", &[]),
    ]);
    assert_eq!(c.selected().unwrap().short, "c3");
    c.previous(); // from 0 wraps to last
    assert_eq!(c.selected().unwrap().short, "c1");
    c.next(); // wraps back to 0
    assert_eq!(c.selected().unwrap().short, "c3");
}
