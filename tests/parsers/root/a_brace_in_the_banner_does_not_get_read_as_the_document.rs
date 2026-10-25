use super::*;

/// A banner that itself contains a brace. The first bracket byte fails, and the retry is
/// anchored to a line start rather than to the next brace along â€” because the next brace
/// along is inside the document, and parsing from there would answer confidently with one
/// sub-object of the tree.
#[test]
fn a_brace_in_the_banner_does_not_get_read_as_the_document() {
    let out = concat!(
        "Changed current directory to /root/{conf}\n",
        r#"{"installed":[{"name":"psr/log","version":"1.1.4"}]}"#,
        "\n"
    );
    let doc = json_document(out).expect("the real document, not the brace in the path");
    assert_eq!(doc["installed"][0]["name"], "psr/log");
}
