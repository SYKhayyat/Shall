use super::*;

#[test]
fn columns_parser_skips_header_and_prefixes() {
    let spec = ParserSpec::Columns {
        name_col: 0,
        version_col: Some(1),
        skip_header: 1,
        delimiter: Some("|".to_string()),
        skip_prefixes: vec!["#".to_string()],
        quoted: false,
    };
    let pkgs = spec
        .parse("NAME|VER\ngit|2.40\n# comment|x\ncurl|8.1\n", "c")
        .expect("this fixture parses");
    assert_eq!(pkgs.len(), 2);
    assert_eq!(pkgs[0].name, "git");
    assert_eq!(pkgs[1].name, "curl");
}
