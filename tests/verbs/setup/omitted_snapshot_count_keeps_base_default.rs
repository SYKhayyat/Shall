use super::*;

#[test]
fn omitted_snapshot_count_keeps_base_default() {
    let base = shall::config::Config::default();
    let base_count = base.retention.snapshots.keep_last;
    let answers = InitAnswers {
        snapshot_count: None,
        ..Default::default()
    };
    let cfg = apply_init_answers(base, &answers);
    assert_eq!(cfg.retention.snapshots.keep_last, base_count);
}
