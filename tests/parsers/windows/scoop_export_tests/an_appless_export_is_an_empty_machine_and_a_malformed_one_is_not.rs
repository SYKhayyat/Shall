use super::*;

#[test]
fn an_appless_export_is_an_empty_machine_and_a_malformed_one_is_not() {
    assert!(parse_scoop_export(r#"{"apps":[]}"#)
        .expect("scoop with no apps exports an empty `apps` array")
        .is_empty());
    // `{"buckets":[]}` has no `apps` key at all. That is not scoop reporting no apps — it
    // is a document this reader does not recognise, and the two were the same answer.
    for broken in ["", "not json", r#"{"buckets":[]}"#] {
        let err = parse_scoop_export(broken)
            .expect_err("an export that could not be read is not an empty machine");
        assert_eq!(err.backend, "scoop", "{broken}");
    }
}
