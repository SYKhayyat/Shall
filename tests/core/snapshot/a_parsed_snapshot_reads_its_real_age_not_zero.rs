use super::*;

#[test]
fn a_parsed_snapshot_reads_its_real_age_not_zero() {
    // The bug in one assertion: a snapshot created a week ago must NOT read as ~now.
    let a_week_ago = Local::now() - chrono::Duration::days(7);
    let snap = Snapshot {
        id: btrfs_id(a_week_ago),
        timestamp: Snapshot::timestamp_from_id(&btrfs_id(a_week_ago)).unwrap(),
        description: "test".into(),
        backend: "btrfs".into(),
    };
    let age = Utc::now() - snap.parse_time().unwrap();
    assert!(age.num_days() >= 6, "age should be ~7 days, was {:?}", age);
}
