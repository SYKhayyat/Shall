use super::*;

#[test]
fn a_label_is_matched_the_same_way_an_id_is() {
    let p = RetentionPolicy {
        keep_last: 1,
        keep: vec!["golden".into()],
        ..Default::default()
    };
    let items = vec![
        item("newest", 1),
        item("some-id", 300).labelled("golden"),
        item("ordinary", 400),
    ];
    assert_eq!(
        p.select_deletions(&items, now()),
        vec!["ordinary".to_string()]
    );
}
