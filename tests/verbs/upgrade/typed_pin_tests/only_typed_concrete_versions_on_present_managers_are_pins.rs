use super::*;

/// The whole point: a typed pin on a manager that is here is named; the same line on an
/// absent manager is not this machine's business, and a versionless line pins nothing.
#[test]
fn only_typed_concrete_versions_on_present_managers_are_pins() {
    let here = registry(true);
    let found = typed_version_pins(
        &here,
        &desired(vec![
            spec("jq", Some("1.7"), true),
            spec("curl", None, true),
            spec("gone", Some("2.0"), false),
            spec("float", Some("latest"), true),
            spec("star", Some("*"), true),
        ]),
    );
    assert_eq!(found, vec!["apt:jq@version=1.7".to_string()]);

    let absent = registry(false);
    assert!(typed_version_pins(&absent, &desired(vec![spec("jq", Some("1.7"), true)])).is_empty());
}
