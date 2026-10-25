use super::*;

#[test]
fn an_older_id_parses_to_an_earlier_time_than_a_newer_one() {
    // The property retention actually depends on: order is preserved.
    let older = Local.with_ymd_and_hms(2026, 7, 10, 9, 0, 0).unwrap();
    let newer = Local.with_ymd_and_hms(2026, 7, 17, 9, 0, 0).unwrap();
    assert!(
        Snapshot::time_from_id(&btrfs_id(older)).unwrap()
            < Snapshot::time_from_id(&btrfs_id(newer)).unwrap()
    );
}
