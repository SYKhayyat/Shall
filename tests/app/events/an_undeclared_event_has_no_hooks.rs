use super::*;

#[test]
fn an_undeclared_event_has_no_hooks() {
    let tmp = tempfile::tempdir().unwrap();
    let hooks = EventHooks::load(&config_at(tmp.path()));
    assert!(hooks.is_empty());
}
