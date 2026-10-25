use super::*;

#[test]
fn keep_days_keeps_recent_and_the_floor() {
    let p = RetentionPolicy {
        keep_days: 30,
        ..Default::default()
    };
    let items = vec![item("recent", 10), item("old", 90), item("ancient", 400)];
    let del = p.select_deletions(&items, now());
    // 'recent' is within 30d; 'old'/'ancient' are older — but the single most-recent
    // ('recent') is the floor anyway. Both stale ones are deleted.
    assert_eq!(del, vec!["old".to_string(), "ancient".to_string()]);
}
