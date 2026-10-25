use super::*;

#[test]
fn secrets_in_template_finds_all_secret_references() {
    assert_eq!(secrets_in_template("no secrets here"), Vec::<String>::new());
    assert_eq!(
        secrets_in_template("password is ${secret:api_key}"),
        vec!["api_key".to_string()]
    );
    assert_eq!(
        secrets_in_template("${secret:a} and ${secret:b} and ${secret:a}"),
        vec!["a".to_string(), "b".to_string(), "a".to_string()]
    );
    assert_eq!(
        secrets_in_template("${secret:db-pass}"),
        vec!["db-pass".to_string()]
    );
    assert_eq!(
        secrets_in_template("prefix ${secret:x}${secret:y} suffix"),
        vec!["x".to_string(), "y".to_string()]
    );
    // Adjacent secrets without whitespace between
    assert_eq!(
        secrets_in_template("${secret:a}${secret:b}"),
        vec!["a".to_string(), "b".to_string()]
    );
    // Unrelated template syntax is ignored
    assert_eq!(
        secrets_in_template("{{ foo }} ${secret:bar} {% if %}"),
        vec!["bar".to_string()]
    );
    // Dollar-sign without the secret prefix is ignored
    assert_eq!(
        secrets_in_template("$secret:not_a_match ${secret:yes}"),
        vec!["yes".to_string()]
    );
}
