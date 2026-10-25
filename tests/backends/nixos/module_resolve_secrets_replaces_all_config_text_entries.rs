use super::*;

#[test]
fn module_resolve_secrets_replaces_all_config_text_entries() {
    let mut secrets = shall::backends::link::SecretContext::new();
    secrets.insert("user".to_string(), "admin".to_string());
    secrets.insert("pass".to_string(), "s3cret".to_string());
    let mut module = Module {
        config_text: vec![
            (
                "services.app.settings.USER".to_string(),
                "\"${secret:user}\"".to_string(),
            ),
            (
                "services.app.settings.PASS".to_string(),
                "\"${secret:pass}\"".to_string(),
            ),
        ],
        ..Module::default()
    };
    module.resolve_secrets(&secrets).expect("resolves");
    assert_eq!(module.config_text[0].1, "\"admin\"", "first entry resolved");
    assert_eq!(
        module.config_text[1].1, "\"s3cret\"",
        "second entry resolved"
    );
}
