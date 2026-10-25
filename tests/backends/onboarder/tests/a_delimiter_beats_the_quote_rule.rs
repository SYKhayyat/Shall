use super::*;

/// `delimiter` already says where a column ends, so `quoted` beside it is a row that asked
/// for two answers to one question — and the explicit one wins.
#[test]
fn a_delimiter_beats_the_quote_rule() {
    let spec = ParserSpec::Columns {
        name_col: 0,
        version_col: Some(1),
        skip_header: 0,
        delimiter: Some("|".to_string()),
        skip_prefixes: vec![],
        quoted: true,
    };
    let pkgs = spec
        .parse("git|\"2.40 (x64)\"\n", "c")
        .expect("this fixture parses");
    assert_eq!(pkgs[0].version.as_deref(), Some("\"2.40 (x64)\""));
}
