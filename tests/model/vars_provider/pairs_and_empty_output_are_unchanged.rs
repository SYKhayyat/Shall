use super::*;

/// **And the plain cases are untouched**, because a fix that changes what a provider's output
/// means is a different fix.
#[test]
fn pairs_and_empty_output_are_unchanged() {
    let origin = Origin::new("vars.js", 0);
    let (vars, _origins) = parse_output("A = 1\n# a comment\n\nB = two\n", &origin).unwrap();
    assert_eq!(vars.len(), 2);
    assert_eq!(vars["B"], Value::Str("two".into()));
    let (empty, _) = parse_output("   \n\n", &origin).unwrap();
    assert!(empty.is_empty(), "whitespace is not a variable");
}
