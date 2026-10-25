use super::*;

#[test]
fn render_with_secrets_fails_on_missing_secret() {
    let secrets = shall::backends::link::SecretContext::new();
    let module = Module {
        config_text: vec![(
            "services.app.key".to_string(),
            "\"${secret:gone}\"".to_string(),
        )],
        ..Module::default()
    };
    let err = render_with_secrets(&module, &secrets).expect_err("missing secret");
    assert!(
        err.to_string().contains("gone"),
        "error names the missing secret: {err}"
    );
}
