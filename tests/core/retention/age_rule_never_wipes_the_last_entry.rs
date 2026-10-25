use super::*;

#[test]
fn age_rule_never_wipes_the_last_entry() {
    // Everything is older than the window, but the newest must still survive.
    let p = RetentionPolicy {
        keep_days: 7,
        ..Default::default()
    };
    let items = vec![item("a", 100), item("b", 200)];
    assert_eq!(p.select_deletions(&items, now()), vec!["b".to_string()]);
}
