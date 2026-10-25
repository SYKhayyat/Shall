use super::*;

/// Every entry says which manager it belongs to and where, because the report names both
/// and a row with an empty field is a report with a hole in it.
#[test]
fn every_row_names_a_manager_and_at_least_one_path() {
    for lock in MANAGER_LOCKS {
        assert!(!lock.holders.is_empty());
        assert!(!lock.holder().is_empty());
        assert!(!lock.paths.is_empty(), "{} has no path", lock.holder());
        assert!(
            !lock.backends.is_empty(),
            "{} explains no backend's failures, so nothing can ever consult it",
            lock.holder()
        );
        assert!(
            !lock.taken_markers.is_empty(),
            "{} has no phrasing for 'someone else holds it', so the wait never triggers",
            lock.holder()
        );
        assert!(
            lock.paths.iter().all(|p| p.starts_with('/')),
            "{} has a relative lock path",
            lock.holder()
        );
    }
}
