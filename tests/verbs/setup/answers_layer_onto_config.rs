use super::*;

#[test]
fn answers_layer_onto_config() {
    let base = shall::config::Config::default();
    let answers = InitAnswers {
        snapshot_count: Some(42),
        starter_packages: vec![],
    };
    let cfg = apply_init_answers(base, &answers);
    assert_eq!(cfg.retention.snapshots.keep_last, 42);
}
