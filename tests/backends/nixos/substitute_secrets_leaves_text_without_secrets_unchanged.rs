use super::*;

#[test]
fn substitute_secrets_leaves_text_without_secrets_unchanged() {
    let secrets = shall::backends::link::SecretContext::new();
    let input = "plain text with no secrets";
    let result = substitute_secrets(input, &secrets).expect("no-op");
    assert_eq!(result, input);
}
