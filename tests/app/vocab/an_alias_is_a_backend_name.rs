use super::*;

#[test]
fn an_alias_is_a_backend_name() {
    let mut cfg = Config::default();
    cfg.aliases.insert("pkg".to_string(), "apt".into());
    let reg = BackendRegistry::new();
    assert!(Vocab::without_priority(&reg, &cfg).is_backend("pkg"));
}
