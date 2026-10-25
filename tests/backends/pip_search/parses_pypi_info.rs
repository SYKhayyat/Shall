use super::*;

#[test]
fn parses_pypi_info() {
    let json: serde_json::Value = serde_json::from_str(
        r#"{
        "info": {"name": "requests", "version": "2.31.0", "summary": "Python HTTP for Humans."}
    }"#,
    )
    .unwrap();
    let p = parse_pypi(&json, "requests", "pip");
    assert_eq!(p.name, "requests");
    assert_eq!(p.backend, "pip");
    assert_eq!(p.version.as_deref(), Some("2.31.0"));
    assert_eq!(
        p.properties.get("description").map(String::as_str),
        Some("Python HTTP for Humans.")
    );
}
