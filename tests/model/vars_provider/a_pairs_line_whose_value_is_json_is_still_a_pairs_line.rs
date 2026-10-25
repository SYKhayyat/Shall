use super::*;

/// **The regression this shape would otherwise cause, pinned.**
///
/// `labels = {"a": {"b": 1}}` is a legitimate `name = value` line whose value happens to be
/// JSON. `parsers::json_document` tries the first `{` *anywhere* in a stream, which is right
/// for a manager printing JSON behind a notice and wrong here: it would answer with the nested
/// object and report a variable called `a`. So the scan here is anchored to a line start, and
/// this test is what says so.
#[test]
fn a_pairs_line_whose_value_is_json_is_still_a_pairs_line() {
    let origin = Origin::new("vars.js", 0);
    let stdout = "HOME = /home/x\nlabels = {\"a\": {\"b\": 1}}\n";
    let (vars, _origins) = parse_output(stdout, &origin).expect("a pair line is a pair line");
    assert_eq!(vars["HOME"], Value::Str("/home/x".into()));
    assert_eq!(
        vars["labels"],
        Value::Str(r#"{"a": {"b": 1}}"#.into()),
        "the nested-brace line was swallowed and read as JSON instead"
    );
    assert!(
        !vars.contains_key("a"),
        "a variable called `a` was invented out of a pair line's value: {:?}",
        vars
    );
}
