use super::*;

#[test]
fn cyclonedx_has_expected_shape() {
    let pkgs = vec![
        p("cargo", "ripgrep", Some("13.0.0")),
        p("winget", "Foo", None),
    ];
    let doc = build_cyclonedx(&pkgs);
    assert_eq!(doc["bomFormat"], "CycloneDX");
    assert_eq!(doc["specVersion"], "1.5");
    let comps = doc["components"].as_array().unwrap();
    assert_eq!(comps.len(), 2);
    assert_eq!(comps[0]["name"], "ripgrep");
    assert_eq!(comps[0]["version"], "13.0.0");
    assert_eq!(comps[0]["purl"], "pkg:cargo/ripgrep@13.0.0");
    // winget has no purl mapping and no version → those keys absent.
    assert!(comps[1].get("purl").is_none());
    assert!(comps[1].get("version").is_none());
    assert_eq!(comps[1]["properties"][0]["value"], "winget");
}
