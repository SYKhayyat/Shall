use super::*;

#[test]
fn an_empty_schema_one_list_is_refused_too() {
    assert!(toml::from_str::<ExtrasLedger>("applied = []\n").is_err());
}
