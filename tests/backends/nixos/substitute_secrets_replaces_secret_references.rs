use super::*;

#[test]
fn substitute_secrets_replaces_secret_references() {
    let mut secrets = shall::backends::link::SecretContext::new();
    secrets.insert("api_key".to_string(), "s3cret".to_string());
    secrets.insert("db_pass".to_string(), "hunter2".to_string());

    let result =
        substitute_secrets("password = ${secret:api_key};", &secrets).expect("substitutes");
    assert_eq!(result, "password = s3cret;");

    let result =
        substitute_secrets("${secret:api_key}:${secret:db_pass}", &secrets).expect("substitutes");
    assert_eq!(result, "s3cret:hunter2");
}
