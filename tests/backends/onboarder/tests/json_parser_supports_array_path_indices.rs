use super::*;

#[test]
fn json_parser_supports_array_path_indices() {
    let spec = ParserSpec::Json {
        array_path: Some("results.1.packages".to_string()),
        name_key: "name".to_string(),
        version_key: Some("version".to_string()),
    };
    let out = r#"{"results":[{"packages":[]},{"packages":[{"name":"jq","version":3}]}]}"#;
    let pkgs = spec.parse(out, "c").expect("numeric array path parses");
    assert_eq!(pkgs[0].name, "jq");
    assert_eq!(pkgs[0].version.as_deref(), Some("3"));
}
