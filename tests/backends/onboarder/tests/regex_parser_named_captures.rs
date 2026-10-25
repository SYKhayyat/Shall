use super::*;

#[test]
fn regex_parser_named_captures() {
    let spec = ParserSpec::Regex {
        pattern: r"^(\S+)\s+v(\d[\d.]*)$".to_string(),
        name_group: 1,
        version_group: Some(2),
    };
    let pkgs = spec
        .parse("exa v0.10.1\nripgrep v13.0.0\n", "c")
        .expect("this fixture parses");
    assert_eq!(pkgs.len(), 2);
    assert_eq!(pkgs[1].name, "ripgrep");
    assert_eq!(pkgs[1].version.as_deref(), Some("13.0.0"));
}
