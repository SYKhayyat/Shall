use super::*;

#[test]
fn substitute_secrets_returns_error_for_missing_secret() {
    let secrets = shall::backends::link::SecretContext::new();
    let err = substitute_secrets("val = ${secret:missing};", &secrets)
        .expect_err("missing secret is an error");
    assert!(
        err.to_string().contains("missing"),
        "error names the missing secret: {err}"
    );
}
