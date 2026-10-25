use super::*;

#[test]
fn the_provider_schema_parses() {
    let toml = r#"
[[secret]]
name = "onepassword"
decrypt = ["op", "read", "{ref}"]
stdout_only = true
"#;
    let file: SecretProviderFile = toml::from_str(toml).unwrap();
    assert_eq!(file.secret.len(), 1);
    assert!(file.secret[0].unusable().is_none());
}
