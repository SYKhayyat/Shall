use super::*;

#[test]
fn package_json_is_valid_and_scoped_to_node() {
    let pkgs = vec![
        p("yarn", "react", Some("18.2.0")),
        p("pnpm", "vite", None),
        p("apt", "curl", None),
    ];
    let out = Format::Npm.render(&pkgs).unwrap().unwrap();
    let doc: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(doc["dependencies"]["react"], "^18.2.0");
    assert_eq!(doc["dependencies"]["vite"], "*");
    assert!(doc["dependencies"].get("curl").is_none());
}
