use super::*;

/// A NON-empty bare array is a shape composer has never produced. Refuse it by name rather
/// than reading it — the same direction every other JSON reader protects against a renamed
/// schema, because reading it as installed packages is how a machine gets "install
/// everything declared" on the next sync.
#[test]
fn composer_nonempty_bare_array_is_refused() {
    let err = parse_composer_json(r#"[{"name": "x"}]"#).unwrap_err();
    assert!(err.to_string().contains("not empty"), "{}", err);
}
