use super::*;

/// The version is the contract. A consumer reads it before anything else, so it must be
/// present and first — and it must not move without someone meaning to move it.
#[test]
fn the_document_is_versioned() {
    let doc = Evaluation::of(&shall::model::DesiredState::default(), Path::new("/repo"));
    assert_eq!(doc.schema, SCHEMA);
    let json = serde_json::to_string(&doc).unwrap();
    assert!(json.starts_with("{\"schema\":1"), "{}", json);
}
