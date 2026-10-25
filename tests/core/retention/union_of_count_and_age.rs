use super::*;

#[test]
fn union_of_count_and_age() {
    // keep_last=1 alone would drop everything but the newest; keep_days=30 rescues the
    // one that is old-by-count but young-by-age.
    let p = RetentionPolicy {
        keep_last: 1,
        keep_days: 30,
        ..Default::default()
    };
    let items = vec![item("newest", 1), item("young", 10), item("stale", 90)];
    assert_eq!(p.select_deletions(&items, now()), vec!["stale".to_string()]);
}
