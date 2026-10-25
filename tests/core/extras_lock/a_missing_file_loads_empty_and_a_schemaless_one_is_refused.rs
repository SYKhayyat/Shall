use super::*;

#[test]
fn a_missing_file_loads_empty_and_a_schemaless_one_is_refused() {
    let tmp = tempfile::tempdir().unwrap();
    assert!(ExtrasLedger::load(&tmp.path().join("absent.toml"))
        .unwrap()
        .is_empty());

    let path = tmp.path().join("extras.toml");
    std::fs::write(&path, "records = []\n").unwrap();
    let err = ExtrasLedger::load(&path).expect_err("a file with no schema is refused");
    let err = err.to_string();
    assert!(err.contains("no `schema` field"), "{err}");
    assert!(err.contains("shall sync -y"), "{err}");
}
