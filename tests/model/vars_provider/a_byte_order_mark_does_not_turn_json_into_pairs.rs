use super::*;

/// **A byte-order mark in front of a valid document is not a `name = value` line.**
///
/// `\u{feff}` is not whitespace, so the old `trim()` left it there and the first `{` of the
/// document never split on `=`. The provider's own output came back as an error about a line
/// nobody wrote — which is a worse answer than a wrong one, because it points at the user.
/// Windows shells and editors emit the mark on their own; a provider inheriting one is
/// ordinary.
#[test]
fn a_byte_order_mark_does_not_turn_json_into_pairs() {
    let origin = Origin::new("vars.js", 0);
    let (vars, _origins) = parse_output("\u{feff}{\"HOME\": \"/home/x\", \"N\": 3}", &origin)
        .expect("a JSON document behind a BOM is still a JSON document");
    assert_eq!(vars["HOME"], Value::Str("/home/x".into()));
    assert_eq!(vars["N"], Value::Num(3.0));
}
