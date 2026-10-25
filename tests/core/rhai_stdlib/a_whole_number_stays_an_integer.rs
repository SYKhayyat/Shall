use super::*;

/// **An integer must not arrive as a float.** `json_to_dynamic` tries `as_i64` first and
/// falls back to `as_f64`; if that order ever inverts, `1` becomes `1.0` and every script
/// comparing a count or an index against an integer silently stops matching.
#[test]
fn a_whole_number_stays_an_integer() {
    let engine = engine("test");
    let same: bool = engine
        .eval(r#"parse_json("{\"n\":3}")["n"] == 3"#)
        .expect("the body did not parse");
    assert!(same, "a whole number did not compare equal to an integer");
}
