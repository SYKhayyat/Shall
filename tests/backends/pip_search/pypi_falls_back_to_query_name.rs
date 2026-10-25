use super::*;

#[test]
fn pypi_falls_back_to_query_name() {
    let json: serde_json::Value = serde_json::from_str(r#"{"info": {}}"#).unwrap();
    let p = parse_pypi(&json, "somepkg", "pip");
    assert_eq!(p.name, "somepkg");
    assert!(p.version.is_none());
}
