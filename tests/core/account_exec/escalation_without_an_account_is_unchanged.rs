use super::*;

#[test]
fn escalation_without_an_account_is_unchanged() {
    let env = HashMap::new();
    assert_eq!(
        CommandExecutor::account_argv("apt-get", &["install", "-y", "nginx"], true, None, &env),
        (
            "sudo".to_string(),
            vec!["-n", "apt-get", "install", "-y", "nginx"]
                .into_iter()
                .map(String::from)
                .collect::<Vec<_>>()
        )
    );
    assert_eq!(
        CommandExecutor::account_argv("apt-get", &["install", "-y", "nginx"], false, None, &env),
        (
            "apt-get".to_string(),
            vec!["install", "-y", "nginx"]
                .into_iter()
                .map(String::from)
                .collect::<Vec<_>>()
        )
    );
}
