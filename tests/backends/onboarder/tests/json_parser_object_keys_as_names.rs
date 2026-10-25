use super::*;

#[test]
fn json_parser_object_keys_as_names() {
    let spec = ParserSpec::Json {
        array_path: None,
        name_key: default_name_key(),
        version_key: None,
    };
    let pkgs = spec
        .parse(r#"{"numpy":[],"pandas":[]}"#, "c")
        .expect("this fixture parses");
    assert_eq!(pkgs.len(), 2);
    assert!(pkgs.iter().any(|p| p.name == "numpy"));
}
