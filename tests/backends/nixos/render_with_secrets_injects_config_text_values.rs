use super::*;

#[test]
fn render_with_secrets_injects_config_text_values() {
    let mut secrets = shall::backends::link::SecretContext::new();
    secrets.insert("token".to_string(), "abc123".to_string());
    let module = Module {
        config_text: vec![(
            "services.nginx.extraConfig".to_string(),
            "auth_token = \"${secret:token}\";".to_string(),
        )],
        ..Module::default()
    };
    let out = render_with_secrets(&module, &secrets).expect("renders");
    assert!(
        out.contains("auth_token = \"abc123\";"),
        "secret substituted in rendered output: {out}"
    );
}
