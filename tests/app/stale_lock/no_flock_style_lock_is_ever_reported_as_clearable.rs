use super::*;

/// **The apt/dpkg family may never be cleared, and the scan proves it.** Those files exist
/// permanently and are locked with `flock(2)`; removing one deletes what the next `apt`
/// expects to lock, and the kernel had already released the lock when the holder died.
///
/// The rule is checked through `find` rather than by reading the table, because the table is
/// what a future edit changes: a row that lost its reason would still be spotted here.
#[test]
fn no_flock_style_lock_is_ever_reported_as_clearable() {
    let flock: Vec<&str> = MANAGER_LOCKS
        .iter()
        .filter(|l| !l.removable())
        .flat_map(|l| l.paths.iter().copied())
        .collect();
    assert!(!flock.is_empty(), "the apt/dpkg family left the table");
    for path in flock {
        let found = find(
            &Fake {
                alive: vec![],
                running: vec![],
            },
            &reader(&[(path, "")]),
        );
        assert!(found.stale.is_empty(), "{path} was reported as clearable");
        // Nor as left-alone: these are on every Debian machine, and four lines of "still
        // there" on every heal is noise that trains a reader to skip the report.
        assert!(
            found.left.is_empty(),
            "{path} was reported at all: {found:?}"
        );
    }
}
