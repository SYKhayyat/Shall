use super::*;

#[test]
fn snapshot_retention_reads_the_one_config_dialect() {
    // `[retention.snapshots]` is the only surface now (legacy `[snapshots]` count/age keys
    // deleted). The default preserves the old out-of-box behaviour: keep 10 / 30 days.
    let p = Config::default().snapshot_retention();
    assert_eq!(
        (p.keep_last, p.keep_days),
        (10, 30),
        "default should keep 10 / 30 days"
    );

    // And an explicit policy is read straight through.
    let mut cfg = Config::default();
    cfg.retention.snapshots = shall::core::RetentionPolicy {
        keep_last: 5,
        keep_days: 14,
        keep: Vec::new(),
    };
    let q = cfg.snapshot_retention();
    assert_eq!((q.keep_last, q.keep_days), (5, 14));
}
