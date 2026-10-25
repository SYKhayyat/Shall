use super::*;

#[test]
fn btrfs_id_round_trips_to_its_creation_time() {
    let t = Local.with_ymd_and_hms(2026, 7, 17, 14, 30, 22).unwrap();
    let parsed = Snapshot::time_from_id(&btrfs_id(t)).expect("btrfs id carries a time");
    assert_eq!(parsed, t.with_timezone(&Utc));
}
