use super::*;

#[test]
fn priority_names_a_backend_this_os_does_not_build() {
    // On Windows nothing registers `apt`, so without this the grammar calls `apt:curl`
    // an unrecognised line — and V.15's "apt isn't in your priority list" never fires.
    let cfg = Config::default();
    let reg = BackendRegistry::new();
    let priority = Priority::from_backends(vec!["apt".into()]);
    assert!(Vocab::new(&reg, &cfg, &priority).is_backend("apt"));
    assert!(!Vocab::new(&reg, &cfg, &priority).is_backend("nonsense"));
}
