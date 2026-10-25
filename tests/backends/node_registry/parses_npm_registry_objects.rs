use super::*;

#[test]
fn parses_npm_registry_objects() {
    let json: serde_json::Value = serde_json::from_str(
        r#"{
        "objects": [
            {"package": {"name": "express", "version": "4.18.2", "description": "web framework"}},
            {"package": {"name": "@types/node", "version": "20.0.0"}}
        ],
        "total": 2
    }"#,
    )
    .unwrap();
    let pkgs = parse_npm_registry(&json, "npm");
    assert_eq!(pkgs.len(), 2);
    assert_eq!(pkgs[0].name, "express");
    assert_eq!(pkgs[0].version.as_deref(), Some("4.18.2"));
    assert_eq!(
        pkgs[0].properties.get("description").map(String::as_str),
        Some("web framework")
    );
    // scoped name preserved; missing description is fine
    assert_eq!(pkgs[1].name, "@types/node");
    assert!(!pkgs[1].properties.contains_key("description"));
}
