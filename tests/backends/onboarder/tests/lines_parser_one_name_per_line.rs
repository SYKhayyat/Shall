use super::*;

#[test]
fn lines_parser_one_name_per_line() {
    let spec = ParserSpec::Lines {
        skip_prefixes: vec!["==".to_string()],
    };
    let pkgs = spec
        .parse("foo\n== legend\nbar\n\n", "c")
        .expect("this fixture parses");
    assert_eq!(pkgs.len(), 2);
    assert_eq!(pkgs[0].name, "foo");
    assert_eq!(pkgs[1].name, "bar");
}
