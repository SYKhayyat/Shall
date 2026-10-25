use super::*;

#[test]
fn columns_parser_extracts_name_and_version() {
    let spec = ParserSpec::Columns {
        name_col: 0,
        version_col: Some(1),
        skip_header: 0,
        delimiter: None,
        skip_prefixes: vec![],
        quoted: false,
    };
    let pkgs = spec
        .parse("ripgrep 13.0.0\nbat 0.24.0\n", "custom")
        .expect("this fixture parses");
    assert_eq!(pkgs.len(), 2);
    assert_eq!(pkgs[0].name, "ripgrep");
    assert_eq!(pkgs[0].version.as_deref(), Some("13.0.0"));
    assert_eq!(pkgs[0].backend, "custom");
}
