use super::*;

/// The other half: a manager that prints its summary *after* the document. `from_str`
/// rejects trailing bytes, so this failed for the same reason at the other end.
#[test]
fn a_note_below_the_document_does_not_hide_it_either() {
    let doc = json_document(concat!(
        r#"[{"name":"jq","version":"1.7"}]"#,
        "\ndone in 0.3s\n"
    ))
    .expect("the document above the note");
    assert_eq!(doc[0]["name"], "jq");
}
