use super::*;

/// No document is `None`, which is what lets a caller tell "unreadable" from "empty".
/// Returning `Value::Null` here is the shape that made a format change look like a bare
/// machine, so it must not be spelled that way.
#[test]
fn output_with_no_document_is_none_and_not_null() {
    assert_eq!(json_document("error: could not connect\n"), None);
    assert_eq!(json_document(""), None);
    // An unterminated document is not a document.
    assert_eq!(json_document(r#"{"installed": ["#), None);
    // And an empty one is still one.
    assert_eq!(json_document("{}"), Some(serde_json::json!({})));
}
