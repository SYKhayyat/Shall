use super::*;

#[test]
fn json_parser_array_of_objects_with_path() {
    let spec = ParserSpec::Json {
        array_path: Some("results".to_string()),
        name_key: "name".to_string(),
        version_key: Some("version".to_string()),
    };
    let out = r#"{"results":[{"name":"httpie","version":"3.2"},{"name":"jq","version":"1.7"}]}"#;
    let pkgs = spec.parse(out, "c").expect("this fixture parses");
    assert_eq!(pkgs.len(), 2);
    assert_eq!(pkgs[0].name, "httpie");
    assert_eq!(pkgs[0].version.as_deref(), Some("3.2"));
}
