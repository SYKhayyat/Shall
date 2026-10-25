use super::*;

#[test]
fn the_schema_one_applied_list_is_refused_rather_than_migrated() {
    let body = "applied = [\"shim:rg\", \"dir:/home/u/state\"]\n";
    let err = toml::from_str::<ExtrasLedger>(body).expect_err("the old format is refused");
    let err = err.to_string();
    assert!(err.contains("schema-1"), "{err}");
    assert!(err.contains("applied"), "{err}");
    assert!(err.contains("shall sync -y"), "{err}");
}
