use super::*;

/// **And a line in front of the document does not either.**
///
/// A provider that prints a banner before its answer is as ordinary as one that does not, and
/// the old rule read the whole stream as `name = value` — so `{"HOME": "/home/x"}` behind a
/// `cd` notice produced a parse error naming the notice. This is the same shape
/// `parsers::json_document` exists for on the manager side.
#[test]
fn a_banner_before_the_document_does_not_either() {
    let origin = Origin::new("vars.js", 0);
    let stdout = "Changed current directory to /root\n{\"HOME\": \"/home/x\", \"N\": 3}\n";
    let (vars, _origins) =
        parse_output(stdout, &origin).expect("a document after a banner is still a document");
    assert_eq!(vars["HOME"], Value::Str("/home/x".into()));
}
