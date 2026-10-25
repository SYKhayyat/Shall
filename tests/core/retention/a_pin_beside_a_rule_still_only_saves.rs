use super::*;

#[test]
fn a_pin_beside_a_rule_still_only_saves() {
    // The other half: once a real rule is on, `keep` is a veto within it and nothing more.
    let p = RetentionPolicy {
        keep_last: 1,
        keep: vec!["known-good".into()],
        ..Default::default()
    };
    let items = vec![
        item("newest", 1),
        item("known-good", 300),
        item("ordinary", 400),
    ];
    assert_eq!(
        p.select_deletions(&items, now()),
        vec!["ordinary".to_string()]
    );
}
