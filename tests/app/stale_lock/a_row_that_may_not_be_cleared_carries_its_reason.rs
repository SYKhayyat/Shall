use super::*;

/// A row that may never be removed has to say why, in the row. The reason used to live in a
/// second table, which is the arrangement that lets someone extend one and not the other.
#[test]
fn a_row_that_may_not_be_cleared_carries_its_reason() {
    for lock in MANAGER_LOCKS {
        if let Some(why) = lock.never_remove_because {
            assert!(
                why.contains("flock"),
                "{} says it may not be cleared without saying what kind of lock it is",
                lock.holder()
            );
        }
    }
}
