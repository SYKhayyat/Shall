use super::*;

#[test]
fn test_composer_json_parsing() {
    let input = r#"{"installed": [{"name": "laravel/installer", "version": "v4.0.0"}]}"#;
    let res = parse_composer_json(input).expect("a composer listing reads");
    assert_eq!(res.len(), 1);
    assert_eq!(res[0].name, "laravel/installer");
    assert_eq!(res[0].version, Some("v4.0.0".into()));
}
