use super::*;

#[test]
fn test_npm_object_and_pnpm_array_both_parse() {
    // npm: a single top-level object.
    let npm = r#"{"dependencies":{"cowsay":{"version":"1.6.0"},"typescript":{"version":"5.3.3"}}}"#;
    let r = parse_npm_style_json(npm, "npm").expect("npm's object form reads");
    assert_eq!(r.len(), 2);
    assert!(r
        .iter()
        .any(|p| p.name == "cowsay" && p.version.as_deref() == Some("1.6.0")));
    // pnpm: `pnpm ls -g --json` wraps the same shape in an ARRAY — must parse too.
    let pnpm = r#"[{"path":"/x","private":true,"dependencies":{"cowsay":{"from":"cowsay","version":"1.6.0"}}}]"#;
    let r2 = parse_npm_style_json(pnpm, "pnpm").expect("pnpm's array form reads");
    assert_eq!(
        r2.len(),
        1,
        "pnpm array form must parse (was empty before the fix)"
    );
    assert_eq!(r2[0].name, "cowsay");
    assert_eq!(r2[0].version.as_deref(), Some("1.6.0"));
    assert_eq!(r2[0].backend, "pnpm");
}
