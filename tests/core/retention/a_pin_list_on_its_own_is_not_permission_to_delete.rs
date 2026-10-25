use super::*;

#[test]
fn a_pin_list_on_its_own_is_not_permission_to_delete() {
    // Two predicates disagreed about this policy: `prunes()` said it deletes nothing, and
    // an `is_active()` beside it counted a non-empty `keep` as a rule — under which every
    // snapshot except the pin and the newest was reaped, on the strength of a line whose
    // whole meaning is *keep*.
    let p = RetentionPolicy {
        keep: vec!["known-good".into()],
        ..Default::default()
    };
    let items = vec![
        item("newest", 1),
        item("known-good", 300),
        item("ordinary", 400),
    ];
    assert!(!p.prunes());
    assert!(
        p.select_deletions(&items, now()).is_empty(),
        "a policy that only pins deleted something"
    );
}
