use super::*;

#[test]
fn deactivating_a_name_that_is_not_there_changes_nothing_and_says_so() {
    let e = deactivate("Work\n", "laptop", &["Gaming"]);
    assert_eq!(e.absent, ["Gaming"]);
    assert!(!e.changed());
}
