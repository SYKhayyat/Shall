use super::*;

#[test]
fn keep_last_keeps_newest_n_and_deletes_the_rest() {
    let p = RetentionPolicy {
        keep_last: 2,
        ..Default::default()
    };
    // g5 newest ... g1 oldest.
    let items = vec![
        item("g1", 5),
        item("g5", 1),
        item("g3", 3),
        item("g2", 4),
        item("g4", 2),
    ];
    let mut del = p.select_deletions(&items, now());
    del.sort();
    assert_eq!(del, vec!["g1", "g2", "g3"]);
}
