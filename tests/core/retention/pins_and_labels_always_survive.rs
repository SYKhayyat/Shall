use super::*;

#[test]
fn pins_and_labels_always_survive() {
    let p = RetentionPolicy {
        keep_last: 1,
        keep: vec!["keep-me-id".into(), "golden".into()],
        ..Default::default()
    };
    let mut pinned = item("pinned-id", 500);
    pinned.pinned = true;
    let mut labeled = item("some-id", 600);
    labeled.label = "golden".into();
    let items = vec![
        item("newest", 1),
        item("keep-me-id", 700),
        pinned,
        labeled,
        item("doomed", 800),
    ];
    let del = p.select_deletions(&items, now());
    assert_eq!(del, vec!["doomed".to_string()]);
}
