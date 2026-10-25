use super::*;

/// A version with a space in it is one column, which is the whole of what `quoted` buys.
#[test]
fn a_quoted_column_survives_the_space_inside_it() {
    let bare = ParserSpec::Columns {
        name_col: 0,
        version_col: Some(1),
        skip_header: 0,
        delimiter: None,
        skip_prefixes: vec![],
        quoted: false,
    };
    let quoted = ParserSpec::Columns {
        name_col: 0,
        version_col: Some(1),
        skip_header: 0,
        delimiter: None,
        skip_prefixes: vec![],
        quoted: true,
    };
    let line = "Microsoft.PowerShell \"7.3.4 (x64)\" installed\n";

    // Without it the architecture is torn off and left as a column of its own.
    let torn = bare.parse(line, "c").expect("this fixture parses");
    assert_eq!(torn[0].version.as_deref(), Some("\"7.3.4"));

    let whole = quoted.parse(line, "c").expect("this fixture parses");
    assert_eq!(whole[0].name, "Microsoft.PowerShell");
    assert_eq!(whole[0].version.as_deref(), Some("7.3.4 (x64)"));
}
